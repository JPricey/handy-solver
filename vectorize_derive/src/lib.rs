use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::{Data, DataStruct, Fields};

#[proc_macro_derive(Vectorize)]
pub fn vectorize_derive(stream: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(stream).unwrap();
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields.iter().map(|field| &field.ident);

    let gen = quote! {
        impl Vectorize for #name {
            fn vectorize_append(&self, vec: &mut Vec<f32>) {
                #(
                    self.#field_name.vectorize_append(vec);
                )*
            }

            fn vectorize(&self) -> Vec<f32> {
                let mut result = Vec::new();
                self.vectorize_append(&mut result);
                return result;
            }
        }
    };
    gen.into()
}
