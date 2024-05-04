use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Result};

pub fn derive_unpack(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, ..} = input;

    let tokens = match data {
        Data::Struct(data) => {
            let mut fields = Vec::new();
            let mut statements = Vec::new();
            for field in data.fields.iter() {
                let ident = field
                    .ident
                    .clone()
                    .unwrap_or_else(|| format_ident!("field_{}", fields.len()));
                let ty = &field.ty;
                statements.push(quote! {
                    let #ident: #ty = instructor::Unpack::<E>::unpack(buffer)?;
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
            quote! {
                impl<E: instructor::Endian> instructor::Unpack<E> for #ident {
                    fn unpack<B: instructor::Buffer + ?Sized>(buffer: &mut B) -> Result<Self, instructor::Error> {
                        #(#statements)*
                        Ok(#ret)
                    }
                }
            }
        }
        _ => return Err(syn::Error::new_spanned(ident, "expected struct")),
    };

    Ok(tokens)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_deserialize() {
        let input = syn::parse_quote! {
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
}
