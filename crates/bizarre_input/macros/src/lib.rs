use define_keys::define_keys_impl;
use proc_macro::TokenStream;

mod define_keys;

#[proc_macro]
pub fn define_keys(input: TokenStream) -> TokenStream {
    define_keys_impl(input)
}
