use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::Parse,
    parse_macro_input,
    token::{Colon, Comma, FatArrow},
    Attribute, LitInt, Visibility,
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
    repr: Ident,
    lin: Vec<(Ident, LitInt)>,
    mac: Vec<(Ident, LitInt)>,
    win: Vec<(Ident, LitInt)>,
}

impl Parse for DefineKeysInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut lin_map = Vec::new();
        let mut mac_map = Vec::new();
        let mut win_map = Vec::new();

        let attributes = input.call(Attribute::parse_outer)?;

        let visibility = input.parse::<Visibility>().unwrap_or(Visibility::Inherited);

        let enum_name = input.parse::<Ident>()?;

        input.parse::<Colon>()?;

        let repr = input.parse::<Ident>()?;

        let key_defs;

        braced!(key_defs in input);

        while !key_defs.is_empty() {
            let mut mac = None;
            let mut win = None;
            let mut lin = None;
            let ident;

            if let Ok(val) = key_defs.parse::<LitInt>() {
                key_defs.parse::<FatArrow>()?;

                ident = key_defs.parse::<Ident>()?;

                mac = Some(val.clone());
                win = Some(val.clone());
                lin = Some(val);
            } else {
                let platforms;
                parenthesized!(platforms in key_defs);

                while let Ok(platform) = platforms.parse::<Ident>() {
                    platforms.parse::<Colon>()?;
                    let val = platforms.parse::<LitInt>()?;

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

            if let Some(keycode) = lin {
                lin_map.push((ident.clone(), keycode));
            }

            if let Some(keycode) = win {
                win_map.push((ident.clone(), keycode));
            }

            if let Some(keycode) = mac {
                mac_map.push((ident.clone(), keycode));
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
            repr,
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
        repr,
        lin,
        mac,
        win,
    } = parse_macro_input!(input as DefineKeysInput);

    let attributes = attributes
        .into_iter()
        .map(|attr| attr.into_token_stream())
        .collect::<Vec<_>>();

    let lin_members = gen_members(&repr, &lin);
    let win_members = gen_members(&repr, &win);
    let mac_members = gen_members(&repr, &mac);

    let lin_converts = gen_converts(&repr, &lin);
    let win_converts = gen_converts(&repr, &win);
    let mac_converts = gen_converts(&repr, &mac);

    quote! {
        #[cfg(target_os = "linux")]
        #(#attributes)*
        #[repr(#repr)]
        #visibility enum #enum_name {
            #lin_members
        }

        #[cfg(target_os = "macos")]
        #(#attributes)*
        #[repr(#repr)]
        #visibility enum #enum_name {
            #mac_members
        }

        #[cfg(target_os = "windows")]
        #(#attributes)*
        #[repr(#repr)]
        #visibility enum #enum_name {
            #win_members
        }

        #[cfg(target_os = "linux")]
        impl #enum_name {
            #lin_converts
        }

        #[cfg(target_os = "macos")]
        impl enum_name {
            #mac_converts
        }

        #[cfg(target_os = "windows")]
        impl enum_name {
            #win_converts
        }
    }
    .into()
}

fn gen_members(repr: &Ident, map: &Vec<(Ident, LitInt)>) -> TokenStream {
    let members = map.iter().map(|(ident, _)| {
        quote! {
            #ident
        }
    });

    quote! {
        #(#members,)*
        Unknown(#repr),
    }
}

fn gen_converts(repr: &Ident, map: &Vec<(Ident, LitInt)>) -> TokenStream {
    let from_raw_arms = map.iter().map(|(ident, lit)| {
        quote! {
            #lit => Self::#ident
        }
    });

    let into_raw_arms = map.iter().map(|(ident, lit)| {
        quote! {
            Self::#ident => #lit
        }
    });

    quote! {
        pub fn from_raw(val: #repr) -> Self {
            match val {
                #(#from_raw_arms,)*
                _ => Self::Unknown(val),
            }
        }

        pub fn into_raw(self) -> #repr {
            match self {
                #(#into_raw_arms,)*
                Self::Unknown(val) => val,
            }
        }

        pub fn as_usize(self) -> usize {
            self.into_raw() as usize
        }
    }
}
