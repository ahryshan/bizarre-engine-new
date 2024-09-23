use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    braced, bracketed, parenthesized,
    parse::Parse,
    parse_macro_input,
    token::{Brace, Colon, Comma, FatArrow},
    Attribute, LitInt, Token, Visibility,
};

// define_keys {
//      EnumName {
//          (win: number, linux: number, mac: number) => ident
//          number => ident
//          ...
//      }
// }
struct DefineKeysInput {
    visibility: Visibility,
    attributes: Vec<Attribute>,
    enum_name: Ident,
    idents: Vec<Ident>,
    lin: HashMap<Ident, usize>,
    mac: HashMap<Ident, usize>,
    win: HashMap<Ident, usize>,
}

impl Parse for DefineKeysInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut lin_map = HashMap::new();
        let mut mac_map = HashMap::new();
        let mut win_map = HashMap::new();
        let mut idents = Vec::new();

        let attributes = input.call(Attribute::parse_outer)?;

        let visibility = input.parse::<Visibility>().unwrap_or(Visibility::Inherited);

        let enum_name = input.parse::<Ident>()?;

        let key_defs;

        braced!(key_defs in input);

        while !key_defs.is_empty() {
            let mut mac = None;
            let mut win = None;
            let mut lin = None;
            let ident;

            if let Ok(val) = key_defs.parse::<LitInt>() {
                let val: usize = val.base10_parse()?;

                key_defs.parse::<FatArrow>()?;

                ident = key_defs.parse::<Ident>()?;

                mac = Some(val);
                win = Some(val);
                lin = Some(val);
            } else {
                let platforms;
                parenthesized!(platforms in key_defs);

                while let Ok(platform) = platforms.parse::<Ident>() {
                    platforms.parse::<Colon>()?;
                    let val: usize = platforms.parse::<LitInt>()?.base10_parse()?;

                    match platform.to_string().as_str() {
                        "linux" => lin = Some(val),
                        "win" => win = Some(val),
                        "mac" => mac = Some(val),
                        platform => panic!("Unknown platform: {platform}"),
                    }

                    if platforms.is_empty() {
                        break;
                    } else {
                        platforms.parse::<Comma>()?;
                    }
                }

                key_defs.parse::<FatArrow>()?;
                ident = key_defs.parse::<Ident>()?;
            }

            idents.push(ident.clone());

            if let Some(keycode) = lin {
                lin_map.insert(ident.clone(), keycode);
            }

            if let Some(keycode) = win {
                win_map.insert(ident.clone(), keycode);
            }

            if let Some(keycode) = mac {
                mac_map.insert(ident, keycode);
            }

            if key_defs.is_empty() {
                break;
            } else {
                key_defs.parse::<Comma>()?;
            }
        }

        Ok(Self {
            visibility,
            attributes,
            enum_name,
            idents,
            lin: lin_map,
            mac: mac_map,
            win: win_map,
        })
    }
}

pub fn define_keys_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DefineKeysInput {
        visibility,
        attributes,
        enum_name,
        lin,
        mac,
        win,
        idents,
    } = parse_macro_input!(input as DefineKeysInput);

    let attributes = attributes
        .into_iter()
        .map(|attr| attr.into_token_stream())
        .collect::<Vec<_>>();

    let linux_from_raw = impl_from_raw(&lin);
    let linux_as_usize = impl_as_usize(&lin);

    let win_from_raw = impl_from_raw(&win);
    let win_as_usize = impl_as_usize(&win);

    let mac_from_raw = impl_from_raw(&mac);
    let mac_as_usize = impl_as_usize(&mac);

    quote! {
        #(#attributes)*
        #visibility enum #enum_name {
            #(#idents,)*
            Unknown(usize),
        }

        #[cfg(target_os = "linux")]
        impl #enum_name {
            #linux_from_raw

            #linux_as_usize
        }

        #[cfg(target_os = "macos")]
        impl enum_name {
            #mac_from_raw

            #mac_as_usize
        }

        #[cfg(target_os = "windows")]
        impl enum_name {
            #win_from_raw

            #win_as_usize
        }
    }
    .into()
}

fn impl_as_usize(map: &HashMap<Ident, usize>) -> TokenStream {
    let arms = map.iter().map(|(ident, raw)| {
        quote! {
            Self::#ident => #raw
        }
    });

    quote! {
        pub fn as_usize(&self) -> usize {
            match self {
                #(#arms,)*
                Self::Unknown(val) => *val
            }
        }
    }
}

fn impl_from_raw(map: &HashMap<Ident, usize>) -> TokenStream {
    let arms = map
        .iter()
        .map(|(ident, raw)| {
            quote! {
                #raw => Self::#ident
            }
        })
        .collect::<Vec<_>>();

    quote! {
        pub fn from_raw(value: usize) -> Self {
            match value {
                #(#arms,)*
                _ => Self::Unknown(value)
            }
        }
    }
}
