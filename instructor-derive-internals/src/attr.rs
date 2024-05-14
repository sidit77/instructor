use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{parenthesized, Attribute, LitInt, LitStr, Token};

#[derive(Debug)]
pub enum Endian {
    Little,
    Big,
    Generic
}

impl ToTokens for Endian {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = match self {
            Endian::Little => quote! { instructor::LittleEndian },
            Endian::Big => quote! { instructor::BigEndian },
            Endian::Generic => quote! { E }
        };
        ident.to_tokens(tokens);
    }
}

pub fn parse_top_level_attributes(attrs: &Vec<Attribute>) -> syn::Result<Endian> {
    let mut endian = Endian::Generic;
    for attr in attrs {
        if attr.path().is_ident("instructor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("endian") {
                    let lit = meta.value()?.parse::<LitStr>()?;
                    return match lit.value().as_str() {
                        "little" => Ok(Endian::Little),
                        "big" => Ok(Endian::Big),
                        _ => Err(meta.error("endian can either be \"little\" or \"big\""))
                    }
                    .map(|e| endian = e);
                }
                Err(meta.error("unknown attribute"))
            })?;
        }
    }
    Ok(endian)
}

pub fn get_repr(attrs: &Vec<Attribute>) -> syn::Result<Option<Ident>> {
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

pub fn is_default(attrs: &Vec<Attribute>) -> syn::Result<bool> {
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

#[allow(clippy::type_complexity)]
pub fn get_bitfield_start(attrs: &Vec<Attribute>) -> syn::Result<(Option<Ident>, Option<(u32, u32)>)> {
    let mut bitfield = None;
    let mut bitrange = None;
    for attr in attrs {
        if attr.path().is_ident("instructor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("bits") {
                    let content;
                    parenthesized!(content in meta.input);
                    let start: u32 = content.parse::<LitInt>()?.base10_parse()?;
                    content.parse::<Token![..]>()?;
                    let end: u32 = content.parse::<LitInt>()?.base10_parse()?;
                    bitrange = Some((start, end));
                    return Ok(());
                }
                if meta.path.is_ident("bitfield") {
                    let content;
                    parenthesized!(content in meta.input);
                    let ident: Ident = content.parse()?;
                    bitfield = Some(ident);
                    return Ok(());
                }
                Err(meta.error("unknown attribute"))
            })
            .unwrap();
        }
    }
    Ok((bitfield, bitrange))
}
