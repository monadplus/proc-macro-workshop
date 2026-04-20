use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    let fields = match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => fields,
            _ => unimplemented!("Builder macro requires named fields structs"),
        },
        other => unimplemented!("Builder macro does not support {}", other),
    };

    let builder_fields = fields.named.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        quote_spanned! {field.span()=>
            #name : Option<#ty>
        }
    });

    let builder_name = format_ident!("{}Builder", struct_name);
    let builder_struct = quote! {
        pub struct #builder_name {
            #(#builder_fields),*
        }
    };

    let default_builder_fields = fields.named.iter().map(|field| {
        let field_name = &field.ident;
        quote_spanned! {field.span()=>
            #field_name : None
        }
    });

    let output = quote! {
        #builder_struct

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#default_builder_fields),*
                }
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
