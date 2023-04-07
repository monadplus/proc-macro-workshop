use std::fmt::Display;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

fn compile_err<T: ToTokens, M: Display>(tokens: T, message: M) -> TokenStream {
    let err = syn::Error::new_spanned(tokens, message);
    return err.into_compile_error().into();
}

#[proc_macro_derive(Default, attributes(default))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident: struct_ident,
        generics,
        data,
    } = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let variants = match data {
        syn::Data::Enum(enm) => enm.variants,
        other => unimplemented!("Default is not supported for {:?}", other),
    };

    if variants.is_empty() {
        unimplemented!("Default is not supported for empty enums");
    }

    let mut default_variants = variants
        .into_iter()
        .filter(|variant| has_default_attr(&variant).unwrap_or_default());

    if let Some(default_variant) = default_variants.next() {
        if let Some(another_default_variant) = default_variants.next() {
            return compile_err(
                another_default_variant,
                "#[default] is defined more than once",
            );
        }
        let variant_ident = default_variant.ident;
        let default_variant_constr = match default_variant.fields {
            syn::Fields::Unit => {
                quote! {
                    Self::#variant_ident
                }
            }
            syn::Fields::Unnamed(unnamed) => {
                let fields_constr = unnamed.unnamed.into_iter().map(|field| {
                    let ty = field.ty;
                    quote!(#ty::default())
                });

                quote! {
                    Self::#variant_ident(#(#fields_constr),*)
                }
            }
            syn::Fields::Named(named) => {
                let fields_constr = named.named.into_iter().map(|field| {
                    let field_name = field.ident.expect("named fields should contain an ident");
                    let ty = &field.ty;
                    quote!(#field_name : #ty::default())
                });

                quote! {
                    Self::#variant_ident{#(#fields_constr),*}
                }
            }
        };

        let output = quote! {
            impl #impl_generics std::default::Default for #struct_ident #ty_generics #where_clause {
                fn default() -> Self {
                    #default_variant_constr
                }
            }
        };
        eprintln!("{}", output);
        proc_macro::TokenStream::from(output)
    } else {
        compile_err(struct_ident, "expected one variant with #[default]")
    }
}

fn has_default_attr(variant: &syn::Variant) -> Option<bool> {
    let attr = variant.attrs.get(0)?;
    let is_default = attr.path().is_ident("default");
    Some(is_default)
}
