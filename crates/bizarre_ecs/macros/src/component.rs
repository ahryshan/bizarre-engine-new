use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_component_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    let DeriveInput {
        ident,
        generics,
        attrs,
        ..
    } = input;

    let mut insert_fn = None;
    let mut remove_fn = None;

    for attr in attrs {
        let Some(ident) = attr.path().get_ident() else {
            continue;
        };
        let ident_string = ident.to_string();

        match ident_string.as_str() {
            "on_insert_fn" => insert_fn = attr.parse_args::<syn::Path>().ok(),
            "on_remove_fn" => remove_fn = attr.parse_args::<syn::Path>().ok(),
            _ => {}
        }
    }

    let on_insert_impl = if let Some(insert_fn) = insert_fn {
        quote! {
            fn on_insert(&mut self, world: &mut World) {
                #insert_fn(self, world);
            }
        }
    } else {
        TokenStream::new()
    };

    let on_remove_impl = if let Some(remove_fn) = remove_fn {
        quote! {
            fn on_remove(&mut self, world: &mut World) {
                #remove_fn(self, world);
            }
        }
    } else {
        TokenStream::new()
    };

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics Resource for #ident #type_generics #where_clause {}

        #[automatically_derived]
        impl #impl_generics Component for #ident #type_generics #where_clause {
            #on_insert_impl
            #on_remove_impl
        }
    }
}
