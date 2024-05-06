use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Index};
use crate::attr::{Endian, get_bitfield_start, get_repr, parse_top_level_attributes};

pub fn derive_pack(input: DeriveInput) -> syn::Result<TokenStream> {
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
    let mut statements = Vec::new();
    for (i, field) in data.fields.iter().enumerate() {
        let ident = field
            .ident
            .as_ref()
            .map(|i| i.to_token_stream())
            .unwrap_or_else(|| Index::from(i).to_token_stream());
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
                        let #ident: #ty = instructor::Unpack::<instructor::BigEndian>::unpack(&mut #bitfield_ident);
                    });
                }
                None => return Err(syn::Error::new_spanned(field, "bitfield range without bitfield")),
            }
        } else {
            bitfield_ident = None;
            statements.push(quote! {
                instructor::Pack::<#endian>::pack(&self.#ident, buffer);
            });
        }

    }
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {},
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Pack<#endian> for #ident {
            #[inline]
            fn pack<B: BufferMut + ?Sized>(&self, buffer: &mut B) {
                #(#statements)*
            }
        }
    };
    Ok(output)
}

fn generate_enum_impl(endian: Endian, repr: Ident, ident: Ident, data: DataEnum) -> syn::Result<TokenStream> {
    for variant in data.variants.iter() {
        if variant.discriminant.is_none() {
            return Err(syn::Error::new_spanned(&variant.ident, "every variant must have a discriminant"));
        }
    }
    let generic = match endian {
        Endian::Generic => quote! { <E: instructor::Endian> },
        _ => quote! {},
    };
    let output = quote! {
        #[automatically_derived]
        impl #generic instructor::Pack<#endian> for #ident {
            #[inline]
            fn pack<B: BufferMut + ?Sized>(&self, buffer: &mut B) {
                let discriminant: #repr = unsafe { core::mem::transmute_copy(self) };
                instructor::Pack::<#endian>::pack(&discriminant, buffer)
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
}
