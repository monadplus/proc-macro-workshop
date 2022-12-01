use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    match input.data {
        syn::Data::Struct(struk) => match struk.fields {
            syn::Fields::Named(named) => {
                for field in named.named {
                    println!("{:?}", field.ty);
                }
            }
            syn::Fields::Unnamed(_) => todo!(),
            syn::Fields::Unit => todo!(),
        },

        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
    TokenStream::new()
}
