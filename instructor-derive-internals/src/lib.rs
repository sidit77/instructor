use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, LitInt, LitStr, parenthesized, Result, Token};

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

fn get_bitfield_start(attrs: &Vec<Attribute>) -> Result<(Option<Ident>, Option<(usize, usize)>)> {
    let mut bitfield = None;
    let mut bitrange = None;
    for attr in attrs {
        if attr.path().is_ident("instructor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("bits") {
                    let content;
                    parenthesized!(content in meta.input);
                    let start: usize = content.parse::<LitInt>()?.base10_parse()?;
                    content.parse::<Token![..]>()?;
                    let end: usize = content.parse::<LitInt>()?.base10_parse()?;
                    bitrange = Some((start, end));
                    return Ok(())
                }
                if meta.path.is_ident("bitfield") {
                    let content;
                    parenthesized!(content in meta.input);
                    let ident: Ident = content.parse()?;
                    bitfield = Some(ident);
                    return Ok(());
                }
                Err(meta.error("unknown attribute"))
            }).unwrap();
        }
    }
    Ok((bitfield, bitrange))
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
    let mut bitfield_ident = None;
    let mut fields = Vec::new();
    let mut statements = Vec::new();
    for field in data.fields.iter() {
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| format_ident!("field_{}", fields.len()));
        let ty = &field.ty;
        let (bitfield, bitrange) = get_bitfield_start(&field.attrs)?;
        if let Some(bitfield) = bitfield {
            let ident = quote! { ___instructor_bitfield };
            statements.push(quote! {
                let mut #ident = instructor::BitBuffer::<#bitfield>::new::<#endian, B>(buffer)?;
            });
            bitfield_ident = Some(ident);
        }
        if let Some((start, end)) = bitrange {
            match bitfield_ident.as_ref() {
                Some(bitfield_ident) => {
                    statements.push(quote! {
                        #bitfield_ident.set_range(#start, #end);
                        let #ident: #ty = instructor::Unpack::<instructor::BigEndian>::unpack(&mut #bitfield_ident)?;
                    });
                }
                None => return Err(syn::Error::new_spanned(field, "bitfield range without bitfield")),
            }
        } else {
            bitfield_ident = None;
            statements.push(quote! {
                let #ident: #ty = instructor::Unpack::<#endian>::unpack(buffer)?;
            });
        }

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
    fn print_bitfield() {
        let input = syn::parse_quote! {
            struct Header {
                #[instructor(bitfield(u16))]
                //#[instructor(bitfield(u16))]
                #[instructor(bits(0..4))]
                a: u8,
                #[instructor(bits(4..8))]
                b: u8
            }
        };

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

        let output = derive_unpack(input).unwrap();
        let formatted = prettyplease::unparse(&syn::parse2(output).unwrap());
        print!("{}", formatted);
    }
}
