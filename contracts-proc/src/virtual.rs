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
use crate::create_complex_type_rec;

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
            Err(err) => return err.to_compile_error().into(),
        };
        for ty in contents.types {
            inherits.push(ty);
        }
    }
    let override_ty = create_complex_type_rec(&inherits);

    output.extend(quote! {
        #input

        type Override = #override_ty;
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
