use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, token::Comma, Ident, LitInt};

struct MassImplScaffoldInput {
    macro_ident: Ident,
    len: usize,
    idents: Vec<Ident>,
}

impl Parse for MassImplScaffoldInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let len = input.parse::<LitInt>()?.base10_parse()?;

        let mut idents = vec![];

        while input.parse::<Comma>().is_ok() {
            idents.push(input.parse::<Ident>()?);
        }

        Ok(MassImplScaffoldInput {
            macro_ident,
            len,
            idents,
        })
    }
}

#[proc_macro]
pub fn mass_impl(input: TokenStream) -> TokenStream {
    let MassImplScaffoldInput {
        macro_ident,
        len,
        idents,
    } = syn::parse_macro_input!(input as MassImplScaffoldInput);

    let input_idents = idents;

    let mut tuples = vec![];

    for i in 0..=len {
        let idents = input_idents.iter().map(|ident| format_ident!("{ident}{i}"));

        if input_idents.len() == 1 {
            tuples.push(quote! {
                #(#idents)*
            })
        } else {
            tuples.push(quote! {
                (#(#idents),*)
            })
        }
    }

    let invocations = (1..=len).map(|i| {
        let tuples = &tuples[..i];

        quote! {
            #macro_ident!(#(#tuples),*);
        }
    });

    quote! {
        #(#invocations)*
    }
    .into()
}
