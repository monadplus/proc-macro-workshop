use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Fields, Ident};

#[allow(dead_code)]
fn error<U: Display, T: ToTokens>(message: U, tokens: T) -> proc_macro::TokenStream {
    syn::Error::new_spanned(tokens, message)
        .into_compile_error()
        .into()
}

#[proc_macro_attribute]
pub fn bitfield(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis,
        ident: struct_ident,
        generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let fields = match data {
        syn::Data::Struct(strct) => {
            if let Fields::Named(fields) = strct.fields {
                fields.named
            } else {
                unimplemented!("#[bitfield] must be used on a named struct");
            }
        }
        _ => unimplemented!("#[bitfield] must be used on a struct"),
    };

    let tys = fields.into_iter().map(|field| field.ty);

    let size = quote!((#(<#tys as Specifier>::BITS)+*) / 8);

    let output = quote! {
        #[repr(C)]
        #vis struct #struct_ident #generics {
            data: [u8; #size],
        }
    };

    proc_macro::TokenStream::from(output)
}

#[proc_macro]
pub fn generate_specifiers(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = TokenStream::new();

    let specifiers = (1..=64).map(|index: usize| {
        let s_ident = Ident::new(&format!("B{}", index), Span::call_site());
        quote! {
            pub enum #s_ident {}

            impl Specifier for #s_ident {
                const BITS: usize = #index;
            }
        }
    });
    output.extend(specifiers);

    proc_macro::TokenStream::from(output)
}
