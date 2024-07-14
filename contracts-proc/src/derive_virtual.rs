use std::mem;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, Data, DataStruct,
    DeriveInput, Fields,
};

// TODO#q: remove this macro
pub fn derive_virtual(
    input: TokenStream,
    call_traits: &[(&str, &str)],
) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    if input.generics.params.len() > 1 {
        panic!("override type should have no more then a single generic param");
    }
    let is_base = input.generics.params.is_empty();
    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();
    let Data::Struct(DataStruct {
        fields: Fields::Unnamed(_) | Fields::Unit,
        ..
    }) = input.data
    else {
        panic!("override type should be a tuple struct")
    };

    let mut set_attrs = vec![];
    for attr in mem::take(&mut input.attrs) {
        if !attr.path().is_ident("set") {
            input.attrs.push(attr);
            continue;
        }

        let upd_type: SetAttr = match attr.parse_args() {
            Ok(contents) => contents,
            Err(err) => {
                return err.to_compile_error().into();
            }
        };
        set_attrs.push(upd_type);
    }

    let call_overrides: Vec<_> = call_traits
        .iter()
        .map(|&(call_name, trait_name)| {
            let matched_set_attr = set_attrs
                .iter()
                .find(|&attr| attr.call_path.is_ident(call_name));
            let call_name: proc_macro2::TokenStream =
                call_name.parse().unwrap();

            match matched_set_attr {
                Some(SetAttr { call_path, override_path }) if is_base => {
                    quote! {
                        type #call_name = #override_path;
                    }
                }
                _ if is_base => panic!("base override should `set` every call"),
                Some(SetAttr { call_path, override_path }) => {
                    quote! {
                        type #call_name = #override_path<#ty_generics::#call_name>;
                    }
                }
                None => {
                    quote! {
                        type #call_name = #ty_generics::#call_name;
                    }
                }
            }
        })
        .collect();

    let struct_overrides: Vec<_> = set_attrs
        .iter()
        .filter_map(|SetAttr { call_path, override_path }| {
            let matched_call_trait = call_traits
                .iter()
                .find(|&(call_name, trait_name)| call_path.is_ident(call_name));
            let &(_call_name, trait_name) = matched_call_trait?;
            let struct_declaration = if is_base {
                quote! {
                    pub struct #override_path;
                }
            } else {
                let trait_name: proc_macro2::TokenStream =
                    trait_name.parse().unwrap();
                quote! {
                    pub struct #override_path<Base: #trait_name>(Base);
                }
            };
            Some(struct_declaration)
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics ERC721Virtual for #name #ty_generics #where_clause {
            #(#call_overrides)*
        }

        #(#struct_overrides)*
    };

    expanded.into()
}

struct SetAttr {
    call_path: syn::Path,
    override_path: syn::Path,
}

impl Parse for SetAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args: Punctuated<syn::Path, syn::Token![=]> =
            Punctuated::parse_separated_nonempty(input)?;
        let mut iter = args.into_iter();
        let call_path = iter.next().expect(
            "function associated type name is required for `set` attribute",
        );
        let override_path = iter
            .next()
            .expect("overriding type name is required for `set` attribute");
        if iter.next().is_none() {
            Ok(Self { call_path, override_path })
        } else {
            panic!(
                "`set` attribute accept just two parameters delimited with `=`"
            )
        }
    }
}
