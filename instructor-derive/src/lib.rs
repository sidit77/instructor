use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Unpack, attributes(instructor))]
pub fn derive_unpack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    instructor_derive_internals::derive_unpack(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Pack, attributes(instructor))]
pub fn derive_pack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    instructor_derive_internals::derive_pack(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}