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
    let trait_path = input.trait_.unwrap().1;
    let self_ty = input.self_ty.clone();
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

    let mut items = Vec::new();
    for item in input.items.iter_mut() {
        let ImplItem::Fn(func) = item else {
            items.push(quote! {
                #item
            });
            continue;
        };
        let generics = func.sig.generics.params.clone();
        if !generics.is_empty() { 
            items.push(quote! {
                #func
            });
            continue;
        }
        let name = func.sig.ident.clone();
        let return_ty = func.sig.output.clone();
        let args = func.sig.inputs.clone();
        let block = func.block.clone();
        items.push(quote! {
            fn #name <This: #trait_path> ( #args ) #return_ty
            #block
        });
    }

    output.extend(quote! {
        impl<Super: #trait_path> #trait_path for #self_ty<Super> {
            type Base = Super;

            #(#items)*
        }

        type Override = #override_ty;
        
        pub struct #self_ty<Super: #trait_path>(Super);
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
