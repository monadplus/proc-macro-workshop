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

    // Look for a variant with #[default]
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
        let default_variant_constr = match default_variant.fields {
            syn::Fields::Unit => {
                let variant_ident = default_variant.ident;
                quote! {
                    Self::#variant_ident
                }
            }
            syn::Fields::Named(_) => unimplemented!(),
            syn::Fields::Unnamed(_) => unimplemented!(),
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
