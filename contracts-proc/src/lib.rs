extern crate proc_macro;

mod derive_virtual;
mod r#virtual;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, ItemFn};

use crate::derive_virtual::derive_virtual;

const ERC721_CALL_TRAITS: &[(&str, &str)] = &[
    ("Update", "ERC721UpdateVirtual"),
    ("SafeTransfer", "ERC721SafeTransferVirtual"),
    ("Approve", "ERC721ApproveVirtual"),
];

#[proc_macro_derive(ERC721Virtual, attributes(set))]
pub fn erc721_derive_virtual(input: TokenStream) -> TokenStream {
    derive_virtual(input, ERC721_CALL_TRAITS)
}

#[proc_macro_attribute]
pub fn r#override(attr: TokenStream, input: TokenStream) -> TokenStream {
    r#virtual::r#override(attr, input)
}

#[proc_macro_attribute]
pub fn r#virtual(attr: TokenStream, input: TokenStream) -> TokenStream {
    r#virtual::r#virtual(attr, input)
}

#[proc_macro]
pub fn inherit(input: TokenStream) -> TokenStream {
    let override_types = parse_macro_input!(input as OverrideTypes);
    create_complex_type_rec(&override_types.0).into()
}

fn create_complex_type_rec(
    override_types: &[syn::Type],
) -> proc_macro2::TokenStream {
    if override_types.len() == 1 {
        let base_override = &override_types[0];
        quote! { #base_override }
    } else {
        let child = &override_types[0];
        let parent = create_complex_type_rec(&override_types[1..]);
        quote! {
            #child < #parent >
        }
    }
}

struct OverrideTypes(Vec<syn::Type>);

impl Parse for OverrideTypes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args: Punctuated<syn::Type, syn::Token![,]> =
            Punctuated::parse_terminated(input)?;
        Ok(OverrideTypes(args.into_iter().collect()))
    }
}
