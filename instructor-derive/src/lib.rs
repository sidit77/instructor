use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Exstruct, attributes(instructor))]
pub fn derive_unpack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    instructor_derive_internals::derive_unpack(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Instruct, attributes(instructor))]
pub fn derive_pack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    instructor_derive_internals::derive_pack(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
