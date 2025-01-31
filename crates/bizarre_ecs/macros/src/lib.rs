#![feature(proc_macro_diagnostic)]

use component::derive_component_impl;
use component_batch::derive_component_batch_impl;
use proc_macro::TokenStream;
use resource::derive_resource_impl;
use syn::{parse_macro_input, DeriveInput};

mod component;
mod component_batch;
mod resource;

#[proc_macro_derive(Component, attributes(on_insert_fn, on_remove_fn))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_component_impl(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    derive_resource_impl(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(ComponentBatch)]
pub fn derive_component_batch(input: TokenStream) -> TokenStream {
    derive_component_batch_impl(parse_macro_input!(input as DeriveInput)).into()
}
