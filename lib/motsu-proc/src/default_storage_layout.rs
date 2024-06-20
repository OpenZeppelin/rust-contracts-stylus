//! Defines the `#[derive(motsu::DefaultStorageLayout)]` procedural macro.
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

const STORAGE_WORD_BYTES: u8 = 32;

pub(crate) fn impl_default_storage_layout(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let Data::Struct(ref data_struct) = ast.data else {
        error!(ast, "DefaultStorageLayout can only be derived for structs");
    };

    let fields = match &data_struct.fields {
        syn::Fields::Named(fields) => &fields.named,
        syn::Fields::Unnamed(fields) => &fields.unnamed,
        syn::Fields::Unit => return TokenStream::new(),
    };

    let mut field_initializations = Vec::new();
    for field in fields {
        let field_name = &field.ident;
        let field_type = &field.ty;

        let ty = quote! { <#field_type as stylus_sdk::storage::StorageType> };
        let field_init = quote! {
            {
                if offset + #ty::SLOT_BYTES as u8 > #STORAGE_WORD_BYTES {
                    next_slot += 1;
                    offset = 0;
                }
                let instance = unsafe { #ty::new(alloy_primitives::U256::from(next_slot), offset) };
                offset += #ty::SLOT_BYTES as u8;
                instance
            }
        };

        field_initializations.push(quote! {
            #field_name: #field_init
        });
    }

    quote! {
        impl Default for #name {
            fn default() -> Self {
                let mut next_slot: i32 = 0;
                let mut offset: u8 = 0;
                #name {
                    #(#field_initializations),*
                }
            }
        }
    }
    .into()
}
