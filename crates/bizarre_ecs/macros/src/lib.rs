use component::derive_component_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod component;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_component_impl(parse_macro_input!(input as DeriveInput)).into()
}
