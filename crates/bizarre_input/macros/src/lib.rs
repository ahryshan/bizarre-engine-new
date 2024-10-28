use define_keys::define_keys_impl;
use proc_macro::TokenStream;

mod define_keys;

/// # Examples
///
/// ```
/// define_keys! {
///     SomeKeyset: u8 {
///         // This will map `SomeKey` to 0 keycode on Windows, 1 on Linux and 2 on Mac
///         (win: 0, linux: 1, mac: 2) => SomeKey,          
///         // This will map `SomeOtherKey` to keycode 3 on all platforms
///         3 => SomeOtherKey
///     }
/// }
///
/// ```
#[proc_macro]
pub fn define_keys(input: TokenStream) -> TokenStream {
    define_keys_impl(input)
}
