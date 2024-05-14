use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields};
use crate::attr::{Endian, get_bitfield_start, get_repr, is_default, parse_top_level_attributes};

pub fn derive_unpack(input: DeriveInput) -> syn::Result<TokenStream> {
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

fn generate_struct_impl(endian: Endian, ident: Ident, data: DataStruct) -> syn::Result<TokenStream> {
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
                        let #ident: #ty = instructor::Exstruct::<instructor::BigEndian>::read_from_buffer(&mut #bitfield_ident)?;
                    });
                }
                None => return Err(syn::Error::new_spanned(field, "bitfield range without bitfield")),
            }
        } else {
            bitfield_ident = None;
            statements.push(quote! {
                let #ident: #ty = instructor::Exstruct::<#endian>::read_from_buffer(buffer)?;
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
        impl #generic instructor::Exstruct<#endian> for #ident {
            #[inline]
            fn read_from_buffer<B: instructor::Buffer>(buffer: &mut B) -> core::result::Result<Self, instructor::Error> {
                #(#statements)*
                Ok(#ret)
            }
        }
    };
    Ok(output)
}

fn generate_enum_impl(endian: Endian, repr: Ident, ident: Ident, data: DataEnum) -> syn::Result<TokenStream> {
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
        impl #generic instructor::Exstruct<#endian> for #ident {
            fn read_from_buffer<B: instructor::Buffer>(buffer: &mut B) -> core::result::Result<Self, instructor::Error> {
                let value: #repr = instructor::Exstruct::<#endian>::read_from_buffer(buffer)?;
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
            struct Header(u32, u8);
        };

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
