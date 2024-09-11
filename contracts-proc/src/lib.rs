// TODO#q: add crate documentation.

extern crate proc_macro;
use proc_macro::TokenStream;

/// Shorthand to print nice errors.
macro_rules! error {
    ($tokens:expr, $($msg:expr),+ $(,)?) => {{
        let error = syn::Error::new(syn::spanned::Spanned::span(&$tokens), format!($($msg),+));
        return error.to_compile_error().into();
    }};
    (@ $tokens:expr, $($msg:expr),+ $(,)?) => {{
        return Err(syn::Error::new(syn::spanned::Spanned::span(&$tokens), format!($($msg),+)))
    }};
}

mod interface;

/// Computes interface id as an associated constant for the trait.
#[proc_macro_attribute]
pub fn interface(attr: TokenStream, input: TokenStream) -> TokenStream {
    interface::interface(attr, input)
}
