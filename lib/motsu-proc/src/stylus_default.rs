extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, PathArguments, Type};

pub fn impl_stylus_default(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = match &ast.data {
        syn::Data::Struct(data) => {
            let fields = match &data.fields {
                syn::Fields::Named(fields) => &fields.named,
                syn::Fields::Unnamed(fields) => &fields.unnamed,
                syn::Fields::Unit => return TokenStream::new(),
            };

            let mut field_initializations = Vec::new();

            for field in fields {
                let field_name = &field.ident;
                let field_type = &field.ty;
                let type_path: &syn::TypePath = match field_type {
                    Type::Path(type_path) => type_path,
                    _ => {
                        let field_type_str =
                            field_type.to_token_stream().to_string();
                        return TokenStream::from(quote! {
                            compile_error!(concat!("Unsupported field type: ", #field_type_str, ". Only path types are supported."));
                        });
                    }
                };

                // Types when using `sol_storage!` look like this: `stylus_sdk::storage::type<generic arguments>`
                // (e.g. uint256 is stylus_sdk::storage::StorageUint<256,4>).
                // So we must first get the third argument, which is the main type.
                let segments = &type_path.path.segments;
                let main_type = if segments.len() >= 3 {
                    &segments[2].ident
                } else {
                    let type_path_str = type_path.to_token_stream().to_string();
                    return TokenStream::from(quote! {
                        compile_error!(concat!("Unexpected type path: ", #type_path_str));
                    });
                };

                // If the type has generic arguments form the token stream that
                // we latter append to access `new` and `SLOT_BYTES`
                let generic_args = match &segments[2].arguments {
                    PathArguments::AngleBracketed(args) => {
                        let args_tokens = args.to_token_stream();
                        quote! { ::#args_tokens }
                    }
                    _ => quote! {},
                };

                // Reconstruct the type with the correct formatting
                let type_ident =
                    quote! { stylus_sdk::storage:: #main_type #generic_args };

                let field_init = quote! {
                    {
                        // Usually we would include an import of `alloy_primitives::U256`, but this causes conflicts
                        // if it is already imported in the file that is using this macro. Instead we use the full
                        // here to avoid this issue.
                        let instance = unsafe { #type_ident::new(alloy_primitives::U256::from(next_slot), offset) };
                        offset += #type_ident::SLOT_BYTES as u8;
                        if offset >= 32 {
                            next_slot += 32;
                            offset = 0;
                        }
                        instance
                    }
                };

                field_initializations.push(quote! {
                    #field_name: #field_init
                });
            }

            let combined_initializations = quote! {
                #(#field_initializations),*
            };

            quote! {
                use stylus_sdk::prelude::StorageType;
                impl Default for #name {
                    fn default() -> Self {
                        let mut next_slot: i32 = 0;
                        let mut offset: u8 = 0;
                        #name {
                            #combined_initializations
                        }
                    }
                }
            }
        }
        _ => {
            return TokenStream::from(quote! {
                compile_error!("StylusDefault can only be derived for structs.");
            })
        }
    };
    gen.into()
}
