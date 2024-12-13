//! Defines the `#[motsu::test]` procedural macro.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg};

/// Defines a unit test that provides access to Stylus execution context.
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

    // Whether 1 or none contracts will be declared.
    let arg_binding_and_ty = match fn_args
        .into_iter()
        .map(|arg| {
            let FnArg::Typed(arg) = arg else {
                error!(@arg, "unexpected receiver argument in test signature");
            };
            let contract_arg_binding = &arg.pat;
            let contract_ty = &arg.ty;
            Ok((contract_arg_binding, contract_ty))
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(res) => res,
        Err(err) => return err.to_compile_error().into(),
    };

    let contract_arg_defs =
        arg_binding_and_ty.iter().map(|(arg_binding, contract_ty)| {
            // Test case assumes, that contract's variable has `&mut` reference
            // to contract's type.
            quote! {
                #arg_binding: &mut #contract_ty
            }
        });

    let contract_args =
        arg_binding_and_ty.iter().map(|(_arg_binding, contract_ty)| {
            // Pass mutable reference to the contract.
            quote! {
                &mut <#contract_ty>::default()
            }
        });

    // Declare test case closure.
    // Pass mut ref to the test closure and call it.
    // Reset storage for the test context and return test's output.
    quote! {
        #( #attrs )*
        #[test]
        fn #fn_name() #fn_return_type {
            use ::motsu::prelude::DefaultStorage;
            let test = | #( #contract_arg_defs ),* | #fn_block;
            let res = test( #( #contract_args ),* );
            ::motsu::prelude::Context::current().reset_storage();
            res
        }
    }
    .into()
}
