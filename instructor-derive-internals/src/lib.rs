use proc_macro2::{TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, Fields, LitStr, Result};

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

pub fn derive_unpack(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, attrs, ..} = input;

    let endian = parse_top_level_attributes(&attrs)?;

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
            quote! {
                impl #generic instructor::Unpack<#endian> for #ident {
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
            #[instructor(endian = "little")]
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
