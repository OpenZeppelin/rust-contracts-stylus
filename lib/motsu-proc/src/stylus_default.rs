extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Type};

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
                let type_path = match field_type {
                    Type::Path(type_path) => type_path,
                    _ => panic!("Unsupported field type: {:?}. Only path types are supported.", field_type),
                };

                let type_ident = &type_path.path;

                let field_init = quote! {
                    {
                        let instance = unsafe { #type_ident::new(U256::from(next_slot), offset) };
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
        _ => panic!("StylusDefault can only be derived for structs."),
    };
    gen.into()
}

pub fn view_type_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    // Assume we want to view the type of the first field
    let type_ident = match &ast.data {
        syn::Data::Struct(data) => {
            if let Some(field) = data.fields.iter().next() {
                match &field.ty {
                    Type::Path(type_path) => {
                        let ident = &type_path.path;
                        quote! { #ident }
                    }
                    _ => quote! { UnsupportedType },
                }
            } else {
                quote! { NoField }
            }
        }
        _ => quote! { NotAStruct },
    };

    let output = quote! {
        compile_error!(concat!("Type ident is: ", stringify!(#type_ident)));
    };

    output.into()
}
