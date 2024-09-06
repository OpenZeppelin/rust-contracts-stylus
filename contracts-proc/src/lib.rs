extern crate proc_macro;
use proc_macro::TokenStream;
use syn::parse::Parse;

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

#[proc_macro_attribute]
pub fn interface(attr: TokenStream, input: TokenStream) -> TokenStream {
    interface::interface(attr, input)
}
