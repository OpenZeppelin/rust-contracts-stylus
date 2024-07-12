use std::{mem, str::FromStr};

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    FnArg, ImplItem, Index, ItemImpl, Lit, LitStr, Pat, PatType, Result,
    ReturnType, Token, Type,
};

pub fn r#virtual(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemImpl);
    let mut output = quote! {};

    let mut inherits = vec![];
    for attr in mem::take(&mut input.attrs) {
        if !attr.path().is_ident("inherit") {
            input.attrs.push(attr);
            continue;
        }
        let contents: InheritsAttr = match attr.parse_args() {
            Ok(contents) => contents,
            Err(err) => {
                return proc_macro::TokenStream::from(err.to_compile_error())
            }
        };
        for ty in contents.types {
            inherits.push(ty);
        }
    }
    output.extend(quote! {
        type Override = inherit!(
            #(#inherits)*
        );
    });
    output.into()
}

struct InheritsAttr {
    types: Punctuated<Type, Token![,]>,
}

impl Parse for InheritsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let types = Punctuated::parse_separated_nonempty(input)?;
        Ok(Self { types })
    }
}
