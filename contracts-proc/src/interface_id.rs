//! Defines the `#[interface_id]` procedural macro.

use std::{collections::HashMap, mem};

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, ItemTrait, LitStr, Result, Token, TraitItem,
};

/// Computes an interface id as an associated function for the trait.
pub(crate) fn interface_id(
    _attr: &TokenStream,
    input: TokenStream,
) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemTrait);

    let unsafety = input.unsafety;
    let mut selectors_map =
        HashMap::<String, (String, proc_macro2::TokenStream)>::new();

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

        let rust_fn_name = func.sig.ident.to_string();

        let solidity_fn_name =
            override_fn_name.unwrap_or(rust_fn_name.to_case(Case::Camel));

        let arg_types = func.sig.inputs.iter().filter_map(|arg| match arg {
            FnArg::Typed(t) => Some(t.ty.clone()),
            // Opt out any `self` arguments.
            FnArg::Receiver(_) => None,
        });

        // Build the function signature string for selector computation.
        let type_strings: Vec<String> =
            arg_types.clone().map(|ty| quote!(#ty).to_string()).collect();
        let signature =
            format!("{}({})", solidity_fn_name, type_strings.join(","));

        let selector = quote! { alloy_primitives::aliases::B32::new(stylus_sdk::function_selector!(#solidity_fn_name #(, #arg_types )*)) };

        // Store selector expression from every function in the trait.
        match selectors_map.get(&signature) {
            Some((existing_rust_fn_name, _)) => {
                error!(
                    existing_rust_fn_name,
                    "selector collision detected: function '{}' has the same selector as function '{}': {}",
                    rust_fn_name,
                    existing_rust_fn_name,
                    signature,
                );
            }
            None => selectors_map.insert(signature, (rust_fn_name, selector)),
        };
    }

    let name = input.ident;
    let vis = input.vis;
    let attrs = input.attrs;
    let trait_items = input.items;
    let generics = input.generics.clone();
    let where_clause = &generics.where_clause;

    let supertrait_tokens = if input.supertraits.is_empty() {
        quote! {}
    } else {
        let supertraits = &input.supertraits;
        quote! { : #supertraits }
    };

    let selectors = selectors_map.values().map(|(_, tokens)| tokens);

    // Keep the same trait with an additional associated function
    // `interface_id`.
    quote! {
        #(#attrs)*
        #vis #unsafety trait #name #generics #supertrait_tokens #where_clause {
            #(#trait_items)*

            #[doc = concat!("Solidity interface id associated with ", stringify!(#name), " trait.")]
            #[doc = "Computed as a XOR of selectors for each function in the trait."]
            fn interface_id() -> alloy_primitives::aliases::B32
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
