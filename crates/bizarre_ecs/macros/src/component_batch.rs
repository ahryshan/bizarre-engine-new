use std::collections::{BTreeMap, BTreeSet, HashSet};

use proc_macro::Diagnostic;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, DeriveInput};

pub fn derive_component_batch_impl(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let data = match data {
        syn::Data::Struct(data_struct) => data_struct,
        _ => {
            Diagnostic::spanned(
                ident.span().unwrap(),
                proc_macro::Level::Error,
                "`ComponentBatch` can be derived only for structs",
            )
            .emit();

            return TokenStream::new();
        }
    };

    if let Some(field) = data.fields.iter().next() {
        let is_tuple = field.ident.is_none();
        if is_tuple {
            Diagnostic::spanned(
                ident.span().unwrap(),
                proc_macro::Level::Error,
                "`ComponentBatch` cannot be derived for tuple structs",
            )
            .emit();

            return TokenStream::new();
        }
    }

    let mut visited_types = HashSet::<syn::Type>::new();

    let (types, idents): (Vec<_>, Vec<_>) = data
        .fields
        .iter()
        .cloned()
        .map(|field| {
            let type_repr = field.ty.to_token_stream().to_string();

            if let Some(first_ty) = visited_types.get(&field.ty) {
                let error_msg = format!("A `ComponentBatch` cannot have more than one component of the same type. Found multiple `{type_repr}` components");
                let note_msg = format!("First `{type_repr}` is here");
                Diagnostic::spanned(vec![field.ty.span().unwrap()], proc_macro::Level::Error, error_msg).span_note(vec![first_ty.span().unwrap()], note_msg).emit();
            } else {
                visited_types.insert(field.ty.clone());
            }

            let ident = field.ident.unwrap();

            (
                field.ty,
                ident,
            )
        })
        .unzip();

    quote! {
        #[automatically_derived]
        impl #impl_generics ComponentBatch for #ident #type_generics #where_clause {
            fn register(registry: &mut ComponentRegistry) {
                #(registry.register::<#types>();)*
            }

            fn insert(self, registry: &mut ComponentRegistry, entity: Entity) {
                let Self {#(#idents,)*} = self;
                #(registry.insert(entity, #idents);)*
            }

            fn remove(registry: &mut ComponentRegistry, entity: Entity) {
                #(registry.remove::<#types>(entity);)*
            }
        }
    }
}
