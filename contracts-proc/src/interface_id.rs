//! Defines the `#[interface_id]` procedural macro.

use std::mem;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, ItemTrait, LitStr, Result, Token, TraitItem,
};

/// Computes an interface id as an associated constant for the trait.
pub(crate) fn interface_id(
    _attr: &TokenStream,
    input: TokenStream,
) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemTrait);

    let mut selectors = Vec::new();
    for item in &mut input.items {
        let TraitItem::Fn(func) = item else {
            continue;
        };

        let mut override_fn_name = None;
        for attr in mem::take(&mut func.attrs) {
            if attr.path().is_ident("selector") {
                if override_fn_name.is_some() {
                    error!(attr.path(), "more than one selector attribute");
                }
                let args: SelectorArgs = match attr.parse_args() {
                    Ok(args) => args,
                    Err(error) => error!(attr.path(), "{}", error),
                };
                override_fn_name = Some(args.name);
            } else {
                // Put back any other attributes.
                func.attrs.push(attr);
            }
        }

        let solidity_fn_name = override_fn_name.unwrap_or_else(|| {
            let rust_fn_name = func.sig.ident.to_string();
            rust_fn_name.to_case(Case::Camel)
        });

        let arg_types = func.sig.inputs.iter().filter_map(|arg| match arg {
            FnArg::Typed(t) => Some(t.ty.clone()),
            // Opt out any `self` arguments.
            FnArg::Receiver(_) => None,
        });

        // Store selector expression from every function in the trait.
        selectors.push(
            quote! { alloy_primitives::FixedBytes::<4>::new(stylus_sdk::function_selector!(#solidity_fn_name #(, #arg_types )*)) }
        );
    }

    let name = input.ident;
    let vis = input.vis;
    let attrs = input.attrs;
    let trait_items = input.items;
    let (_impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    let supertrait_tokens = if input.supertraits.is_empty() {
        quote! {}
    } else {
        let supertraits = &input.supertraits;
        quote! { : #supertraits }
    };

    // Keep the same trait with an additional associated constant
    // `INTERFACE_ID`.
    quote! {
        #(#attrs)*
        #vis trait #name #ty_generics #supertrait_tokens #where_clause {
            #(#trait_items)*

            #[doc = concat!("Solidity interface id associated with ", stringify!(#name), " trait.")]
            #[doc = "Computed as a XOR of selectors for each function in the trait."]
            fn interface_id() -> alloy_primitives::FixedBytes<4>
            where
                Self: Sized,
            {
                #(#selectors)^*
            }
        }
    }
    .into()
}

/// Contains arguments of the `#[selector(..)]` attribute.
struct SelectorArgs {
    name: String,
}

impl Parse for SelectorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;

        if ident == "name" {
            let _: Token![=] = input.parse()?;
            let lit: LitStr = input.parse()?;
            Ok(SelectorArgs { name: lit.value() })
        } else {
            error!(@ident, "expected identifier 'name'")
        }
    }
}
