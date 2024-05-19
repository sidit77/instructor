use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, Index};

use crate::attr::{get_bitfield_start, get_repr, parse_top_level_attributes, Endian};

pub fn derive_pack(input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput { ident, data, attrs, .. } = input;

    let (endian, bitflags) = parse_top_level_attributes(&attrs)?;
    if bitflags {
        return generate_bitflags_impl(endian, ident);
    }
    match data {
        Data::Struct(data) => generate_struct_impl(endian, ident, data),
        Data::Enum(data) => match get_repr(&attrs)? {
            Some(repr) => generate_int_enum_impl(endian, repr, ident, data),
            None => generate_data_enum_impl(endian, ident, data)
        },
        Data::Union(_) => Err(syn::Error::new_spanned(ident, "unions are not supported"))
    }
}

fn generate_bitflags_impl(endian: Endian, ident: Ident) -> syn::Result<TokenStream> {
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {}
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Instruct<#endian> for #ident {
            #[inline]
            fn write_to_buffer<B: instructor::BufferMut>(&self, buffer: &mut B) {
                instructor::Instruct::<#endian>::write_to_buffer(&self.bits(), buffer)
            }
        }
    };
    Ok(output)
}

fn generate_struct_impl(endian: Endian, ident: Ident, data: DataStruct) -> syn::Result<TokenStream> {
    let mut bitfield_ident = None;
    let mut statements = Vec::new();
    for (i, field) in data.fields.iter().enumerate() {
        let ident = field
            .ident
            .as_ref()
            .map(|i| i.to_token_stream())
            .unwrap_or_else(|| Index::from(i).to_token_stream());
        let (bitfield, bitrange) = get_bitfield_start(&field.attrs)?;
        if let Some(bitfield) = bitfield {
            if let Some(bitfield) = bitfield_ident.take() {
                statements.push(quote! {
                    instructor::Instruct::<#endian>::write_to_buffer(&#bitfield, buffer);
                });
            }
            let ident = quote! { ___instructor_bitfield };
            statements.push(quote! {
                let mut #ident = instructor::BitBuffer::<#bitfield>::empty();
            });
            bitfield_ident = Some(ident);
        }
        if let Some((start, end)) = bitrange {
            match bitfield_ident.as_ref() {
                Some(bitfield_ident) => {
                    statements.push(quote! {
                        #bitfield_ident.set_range(#start, #end);
                        instructor::Instruct::<instructor::BigEndian>::write_to_buffer(&self.#ident, &mut #bitfield_ident);
                    });
                }
                None => return Err(syn::Error::new_spanned(field, "bitfield range without bitfield"))
            }
        } else {
            if let Some(bitfield) = bitfield_ident.take() {
                statements.push(quote! {
                    instructor::Instruct::<#endian>::write_to_buffer(&#bitfield, buffer);
                });
            }
            statements.push(quote! {
                instructor::Instruct::<#endian>::write_to_buffer(&self.#ident, buffer);
            });
        }
    }
    if let Some(bitfield) = bitfield_ident.take() {
        statements.push(quote! {
            instructor::Instruct::<#endian>::write_to_buffer(&#bitfield, buffer);
        });
    }
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {}
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Instruct<#endian> for #ident {
            #[inline]
            fn write_to_buffer<B: instructor::BufferMut>(&self, buffer: &mut B) {
                #(#statements)*
            }
        }
    };
    Ok(output)
}

fn generate_int_enum_impl(endian: Endian, repr: Ident, ident: Ident, data: DataEnum) -> syn::Result<TokenStream> {
    for variant in data.variants.iter() {
        if variant.discriminant.is_none() {
            return Err(syn::Error::new_spanned(&variant.ident, "every variant must have a discriminant"));
        }
    }
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {}
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Instruct<#endian> for #ident {
            #[inline]
            fn write_to_buffer<B: instructor::BufferMut>(&self, buffer: &mut B) {
                let discriminant: #repr = unsafe { core::mem::transmute_copy(self) };
                instructor::Instruct::<#endian>::write_to_buffer(&discriminant, buffer)
            }
        }
    };
    Ok(output)
}

fn generate_data_enum_impl(endian: Endian, ident: Ident, data: DataEnum) -> syn::Result<TokenStream> {
    let mut matches = Vec::new();
    for variant in data.variants.iter() {
        if variant.discriminant.is_some() {
            return Err(syn::Error::new_spanned(&variant.ident, "disciminants are not supported for data enums"));
        }
        let ident = &variant.ident;
        let fields = &variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.ident.clone().unwrap_or_else(|| format_ident!("arg{}", i)))
            .collect::<Vec<_>>();

        matches.push(match &variant.fields {
            Fields::Named(_) => quote! {
                Self::#ident { #(#fields),* } => {
                    #(instructor::Instruct::<#endian>::write_to_buffer(#fields, buffer);)*
                }
            },
            Fields::Unnamed(_) => quote! {
                Self::#ident(#(#fields),*) => {
                    #(instructor::Instruct::<#endian>::write_to_buffer(#fields, buffer);)*
                }
            },
            Fields::Unit => quote! { Self::#ident => {} }
        });
    }
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {}
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Instruct<#endian> for #ident {
            #[inline]
            fn write_to_buffer<B: instructor::BufferMut>(&self, buffer: &mut B) {
                match self {
                    #(#matches)*
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
            struct Header(u32, u32);
        };

        let output = derive_pack(input).unwrap();
        let formatted = prettyplease::unparse(&syn::parse2(output).unwrap());
        print!("{}", formatted);
    }

    #[test]
    fn print_deserialize2() {
        let input = syn::parse_quote! {
            #[instructor(endian = "little")]
            #[repr(align(4))]
            struct Header {
                field1: u32,
                bar: i8
            }
        };

        let output = derive_pack(input).unwrap();
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

        let output = derive_pack(input).unwrap();
        let formatted = prettyplease::unparse(&syn::parse2(output).unwrap());
        print!("{}", formatted);
    }

    #[test]
    fn print_enum() {
        let input = syn::parse_quote! {
            #[repr(u32)]
            enum Data {
                A = 0x01,
                B = 0x02
            }
        };

        let output = derive_pack(input).unwrap();
        let formatted = prettyplease::unparse(&syn::parse2(output).unwrap());
        print!("{}", formatted);
    }

    #[test]
    fn print_enum_2() {
        let input = syn::parse_quote! {
            enum Data {
                A {
                    a: u8,
                    b: u16
                },
                B(i32),
                C
            }
        };

        let output = derive_pack(input).unwrap();
        let formatted = prettyplease::unparse(&syn::parse2(output).unwrap());
        print!("{}", formatted);
    }
}
