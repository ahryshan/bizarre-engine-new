use quote::quote;
use syn::DeriveInput;

pub fn derive_component_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = input;

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics Resource for #ident #type_generics #where_clause {}

        #[automatically_derived]
        impl #impl_generics Component for #ident #type_generics #where_clause {}
    }
}
