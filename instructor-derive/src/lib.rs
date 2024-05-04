use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    instructor_derive_internals::derive_deserialize(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}