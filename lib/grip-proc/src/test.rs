//! Defines the `#[grip::test]` procedural macro.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg};

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

/// Defines a unit test that provides access to Stylus' execution context.
///
/// For more information see [`crate::test`].
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as syn::ItemFn);
    let attrs = &item_fn.attrs;
    let sig = &item_fn.sig;
    let fn_name = &sig.ident;
    let fn_return_type = &sig.output;
    let fn_block = &item_fn.block;
    let fn_args = &sig.inputs;

    // If the test function has no params, then it doesn't need access to the
    // contract, so it is just a regular test.
    if fn_args.is_empty() {
        let vis = &item_fn.vis;
        return quote! {
            #( #attrs )*
            #[test]
            #vis #sig #fn_block
        }
        .into();
    }

    // We can unwrap because we handle the empty case above. We don't support
    // more than one parameter for now, so we skip them.
    let arg = fn_args.first().unwrap();
    let FnArg::Typed(arg) = arg else {
        error!(arg, "unexpected receiver argument in test signature");
    };
    let contract_arg_binding = &arg.pat;
    let contract_ty = &arg.ty;
    quote! {
        #( #attrs )*
        #[test]
        fn #fn_name() #fn_return_type {
            ::grip::prelude::with_context::<#contract_ty>(| #contract_arg_binding |
                #fn_block
            )
        }
    }
    .into()
}
