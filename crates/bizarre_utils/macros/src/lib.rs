use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parse,
    token::{Comma, Minus},
    Ident, LitInt,
};

struct MassImplScaffoldInput {
    macro_ident: Ident,
    len: usize,
    ident: Ident,
}

impl Parse for MassImplScaffoldInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let len = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let ident = input.parse::<Ident>()?;

        Ok(MassImplScaffoldInput {
            macro_ident,
            len,
            ident,
        })
    }
}

#[proc_macro]
pub fn mass_impl(input: TokenStream) -> TokenStream {
    let MassImplScaffoldInput {
        macro_ident,
        len,
        ident,
    } = syn::parse_macro_input!(input as MassImplScaffoldInput);

    let idents = (0..len)
        .map(|i| format_ident!("{ident}{i}"))
        .collect::<Vec<_>>();

    let invocations = (0..len).map(|i| {
        let idents = &idents[..=i];
        quote! {
            #macro_ident!(#(#idents),*);
        }
    });

    quote! {
        #(#invocations)*
    }
    .into()
}
