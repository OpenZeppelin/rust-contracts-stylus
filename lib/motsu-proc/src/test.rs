//! Defines the `#[motsu::test]` procedural macro.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg};

/// Defines a unit test that provides access to Stylus' execution context.
///
/// For more information see [`crate::test`].
pub(crate) fn test(_attr: &TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as syn::ItemFn);
    let attrs = &item_fn.attrs;
    let sig = &item_fn.sig;
    let fn_name = &sig.ident;
    let fn_return_type = &sig.output;
    let fn_block = &item_fn.block;
    let fn_args = &sig.inputs;

    // Currently, more than one contract per unit test is not supported.
    if fn_args.len() > 1 {
        error!(fn_args, "expected at most one contract in test signature");
    }

    // Whether 1 or none contracts will be declared.
    let contract_declarations = fn_args.into_iter().map(|arg| {
        let FnArg::Typed(arg) = arg else {
            error!(arg, "unexpected receiver argument in test signature");
        };
        let contract_arg_binding = &arg.pat;
        let contract_ty = &arg.ty;

        // Test case assumes, that contract's variable has `&mut` reference
        // to contract's type.
        quote! {
            let mut #contract_arg_binding = <#contract_ty>::default();
            let #contract_arg_binding = &mut #contract_arg_binding;
        }
    });

    // Output full testcase function.
    // Declare contract.
    // And in the end, reset storage for test context.
    quote! {
        #( #attrs )*
        #[test]
        fn #fn_name() #fn_return_type {
            use ::motsu::prelude::DefaultStorage;
            #( #contract_declarations )*
            let res = #fn_block;
            ::motsu::prelude::Context::current().reset_storage();
            res
        }
    }
    .into()
}
