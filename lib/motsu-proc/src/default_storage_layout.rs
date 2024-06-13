//! Defines the `#[derive(motsu::DefaultStorageLayout)]` procedural macro.
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, PathArguments, Type};

pub fn impl_default_storage_layout(ast: &DeriveInput) -> TokenStream {
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
        let field_ty = &field.ty;

        let Type::Path(type_path) = field_ty else {
            error!(
                field_ty,
                "unsupported field type. Only path types are supported"
            );
        };

        // Types when using `sol_storage!` look like this:
        // `stylus_sdk::storage::type<generic arguments>`
        // (e.g. uint256 is stylus_sdk::storage::StorageUint<256,4>).
        // So we must first get the last argument, which is the main
        // type.
        let segments = &type_path.path.segments;
        let last_segment = if segments.len() >= 3 {
            segments.last().unwrap()
        } else {
            error!(type_path, "unexpected type path");
        };
        let main_type = &last_segment.ident;

        // If the type has generic arguments form the token stream that
        // we latter append to access `new` and `SLOT_BYTES`
        let generic_args = match &last_segment.arguments {
            PathArguments::AngleBracketed(args) => quote! { ::#args },
            _ => quote! {},
        };

        // Reconstruct the type with the correct formatting
        let type_ident =
            quote! { stylus_sdk::storage:: #main_type #generic_args };

        let field_init = quote! {
            {
                if offset + #type_ident::SLOT_BYTES as u8 > 32 {
                    next_slot += 32;
                    offset = 0;
                }
                let instance = unsafe { #type_ident::new(alloy_primitives::U256::from(next_slot), offset) };
                offset += #type_ident::SLOT_BYTES as u8;
                instance
            }
        };

        field_initializations.push(quote! {
            #field_name: #field_init
        });
    }

    quote! {
        use stylus_sdk::prelude::StorageType;
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
