use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, LitStr, Result};

#[derive(Debug)]
enum Endian {
    Little,
    Big,
    Generic,
}

impl ToTokens for Endian {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = match self {
            Endian::Little => quote! { instructor::LittleEndian },
            Endian::Big => quote! { instructor::BigEndian },
            Endian::Generic => quote! { E },
        };
        ident.to_tokens(tokens);
    }
}

fn parse_top_level_attributes(attrs: &Vec<Attribute>) -> Result<Endian> {
    let mut endian = Endian::Generic;
    for attr in attrs {
        if attr.path().is_ident("instructor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("endian") {
                    let lit = meta.value()?.parse::<LitStr>()?;
                    return match lit.value().as_str() {
                        "little" => Ok(Endian::Little),
                        "big" => Ok(Endian::Big),
                        _ => Err(meta.error("endian can either be \"little\" or \"big\"")),
                    }.map(|e| endian = e);
                }
                Err(meta.error("unknown attribute"))
            })?;
        }
    }
    Ok(endian)
}

fn get_repr(attrs: &Vec<Attribute>) -> Result<Option<Ident>> {
    let mut repr = None;
    for attr in attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                repr = meta.path.get_ident().cloned();
                Ok(())
            })?;
        }
    }
    Ok(repr)
}

fn is_default(attrs: &Vec<Attribute>) -> Result<bool> {
    let mut default = false;
    for attr in attrs {
        if attr.path().is_ident("instructor") {
            attr.parse_nested_meta(|meta| {
                default |= meta.path.is_ident("default");
                Ok(())
            })?;
        }
    }
    Ok(default)
}

pub fn derive_unpack(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, attrs, ..} = input;

    let endian = parse_top_level_attributes(&attrs)?;


    match data {
        Data::Struct(data) => generate_struct_impl(endian, ident, data),
        Data::Enum(data) => match get_repr(&attrs)? {
            Some(repr) => generate_enum_impl(endian, repr, ident, data),
            None => Err(syn::Error::new_spanned(ident, "enums must have a repr attribute")),
        }
        Data::Union(_) => Err(syn::Error::new_spanned(ident, "unions are not supported")),
    }
}

fn generate_struct_impl(endian: Endian, ident: Ident, data: DataStruct) -> Result<TokenStream> {
    let mut fields = Vec::new();
    let mut statements = Vec::new();
    for field in data.fields.iter() {
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| format_ident!("field_{}", fields.len()));
        let ty = &field.ty;
        statements.push(quote! {
                    let #ident: #ty = instructor::Unpack::<#endian>::unpack(buffer)?;
                });
        fields.push(ident);
    }
    let ret = match data.fields {
        Fields::Named(_) => quote! {
                    Self {
                        #(#fields),*
                    }
                },
        Fields::Unnamed(_) => quote! {
                    Self (#(#fields),*)
                },
        Fields::Unit => quote! {
                    Self
                }
    };
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {},
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Unpack<#endian> for #ident {
            fn unpack<B: instructor::Buffer + ?Sized>(buffer: &mut B) -> Result<Self, instructor::Error> {
                #(#statements)*
                Ok(#ret)
            }
        }
    };
    Ok(output)
}

fn generate_enum_impl(endian: Endian, repr: Ident, ident: Ident, data: DataEnum) -> Result<TokenStream> {
    let mut default = None;
    let mut variants = Vec::new();
    for variant in data.variants.iter() {
        let ident = &variant.ident;
        let discr = match &variant.discriminant {
            Some((_, expr)) => expr,
            None => return Err(syn::Error::new_spanned(ident, "every variant must have a discriminant")),
        };
        if is_default(&variant.attrs)? {
            if default.is_some() {
                return Err(syn::Error::new_spanned(ident, "only one variant can be marked as default"));
            }
            default = Some(ident.clone());
        }
        variants.push(quote! {
            #discr => Ok(Self::#ident)
        });
    }
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {},
    };
    let default = match default {
        Some(ident) => quote! { _ => Ok(Self::#ident) },
        None => quote! { _ => Err(instructor::Error::InvalidValue) },
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Unpack<#endian> for #ident {
            fn unpack<B: instructor::Buffer + ?Sized>(buffer: &mut B) -> Result<Self, instructor::Error> {
                let value: #repr = instructor::Unpack::<#endian>::unpack(buffer)?;
                match value {
                    #(#variants,)*
                    #default,
                }
            }
        }
    };
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_deserialize() {
        let input = syn::parse_quote! {
            #[instructor(endian = "little")]
            #[repr(align(4))]
            struct Header(u32);
        };

        //let input = syn::parse_quote! {
        //    struct Header {
        //        #[field]
        //        sdf: u32,
        //    }
        //};
        let output = derive_unpack(input).unwrap();
        let formatted = prettyplease::unparse(&syn::parse2(output).unwrap());
        print!("{}", formatted);
    }

    #[test]
    fn print_enum() {
        let input = syn::parse_quote! {
            #[repr(u8)]
            enum Data {
                A = 0x01,
                B = 0x02
            }
        };

        //let input = syn::parse_quote! {
        //    struct Header {
        //        #[field]
        //        sdf: u32,
        //    }
        //};
        let output = derive_unpack(input).unwrap();
        let formatted = prettyplease::unparse(&syn::parse2(output).unwrap());
        print!("{}", formatted);
    }
}
