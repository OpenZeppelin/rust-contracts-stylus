//! Procedural macros for `openzeppelin-stylus-testing`.
use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, GenericArgument, ItemFn, Pat, PathArguments, Type};

/// Defines a unit test that runs inside a fresh test environment.
///
/// Every parameter of the function is created from the environment before
/// the body runs:
///
/// * `Contract<T>` deploys `T` at an address derived from the parameter name.
///   If `T` has a `#[public]` impl block, other contracts can call it.
/// * `Address` creates an account whose address is derived from the parameter
///   name.
/// * `Env` gives access to the environment itself.
///
/// The macro expands to a regular `#[test]` function; other attributes such
/// as `#[should_panic]` are kept. The crate must be imported as `testing`.
///
/// # Examples
///
/// ```rust,ignore
/// #[testing::test]
/// fn transfers(contract: Contract<Erc20>, alice: Address, bob: Address) {
///     contract.sender(alice)._mint(alice, U256::ONE).unwrap();
///     contract.sender(alice).transfer(bob, U256::ONE).unwrap();
///     assert_eq!(contract.sender(alice).balance_of(bob), U256::ONE);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "`#[testing::test]` takes no arguments",
        )
        .to_compile_error()
        .into();
    }
    let item_fn = syn::parse_macro_input!(input as ItemFn);
    let attrs = &item_fn.attrs;
    let vis = &item_fn.vis;
    let sig = &item_fn.sig;
    let name = &sig.ident;
    let output = &sig.output;
    let stmts = &item_fn.block.stmts;

    let mut bindings = Vec::new();
    for (index, arg) in sig.inputs.iter().enumerate() {
        let FnArg::Typed(arg) = arg else {
            return syn::Error::new_spanned(
                arg,
                "test functions cannot take `self`",
            )
            .to_compile_error()
            .into();
        };
        let pat = &arg.pat;
        let ty = &arg.ty;
        let label = match &**pat {
            Pat::Ident(ident) => ident.ident.to_string(),
            _ => format!("arg{index}"),
        };
        let binding = match contract_type_argument(ty) {
            Some(inner) => quote! {
                let #pat: #ty = {
                    #[allow(unused_imports)]
                    use ::testing::__private::{DeployAny, DeployRoutable};
                    (&::testing::__private::Deployer::<#inner>::new())
                        .deploy(&__env, #label)
                };
            },
            None => quote! {
                let #pat: #ty = ::testing::FromEnv::from_env(&__env, #label);
            },
        };
        bindings.push(binding);
    }

    quote! {
        #( #attrs )*
        #[test]
        #vis fn #name() #output {
            let __env = ::testing::Env::new();
            #( #bindings )*
            #( #stmts )*
        }
    }
    .into()
}

/// Returns `T` if `ty` is written as `Contract<T>` (with any path prefix).
fn contract_type_argument(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Contract" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}
