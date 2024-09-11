use std::mem;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, ItemTrait, LitStr, Result, Token, TraitItem,
};

/// Computes interface id as an associated constant for the trait.
pub(crate) fn interface(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemTrait);
    let mut output = quote! {};

    let mut selectors = Vec::new();
    for item in &mut input.items {
        let TraitItem::Fn(func) = item else {
            continue;
        };

        let mut override_name = None;
        for attr in mem::take(&mut func.attrs) {
            let Some(ident) = attr.path().get_ident() else {
                func.attrs.push(attr);
                continue;
            };
            if *ident == "selector" {
                if override_name.is_some() {
                    error!(attr.path(), "more than one selector attribute");
                }
                let args: SelectorArgs = match attr.parse_args() {
                    Ok(args) => args,
                    Err(error) => error!(ident, "{}", error),
                };
                override_name = Some(args.name);
                continue;
            }
            func.attrs.push(attr);
        }

        let sol_name = override_name.unwrap_or_else(|| {
            func.sig.ident.clone().to_string().to_case(Case::Camel)
        });

        let args = func.sig.inputs.iter();
        let arg_types: Vec<_> = args
            .filter_map(|arg| match arg {
                FnArg::Typed(t) => Some(t.ty.clone()),
                _ => None,
            })
            .collect();

        let selector = quote! { u32::from_be_bytes(stylus_sdk::function_selector!(#sol_name #(, #arg_types )*)) };
        selectors.push(selector);
    }

    let name = input.ident.clone();
    let vis = input.vis.clone();
    let attrs = input.attrs.clone();
    let trait_items = input.items.clone();
    let (_impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    output.extend(quote! {
        #(#attrs)*
        #vis trait #name #ty_generics #where_clause {
            #(#trait_items)*

            /// Solidity interface id associated with current trait.
            const INTERFACE_ID: u32 = {
                #(#selectors)^*
            };
        }
    });

    output.into()
}

struct SelectorArgs {
    name: String,
}

impl Parse for SelectorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;

        if input.is_empty() {
            error!(@input.span(), "missing id or text argument");
        }

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            match ident.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    if name.is_some() {
                        error!(@lit, r#"only one "name" is allowed"#);
                    }
                    name = Some(lit.value());
                }
                _ => error!(@ident, "Unknown selector attribute"),
            }

            // allow a comma
            let _: Result<Token![,]> = input.parse();
        }

        if let Some(name) = name {
            Ok(Self { name })
        } else {
            error!(@input.span(), r#""name" is required"#);
        }
    }
}
