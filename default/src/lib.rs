use std::{collections::HashSet, fmt::Display};

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, DeriveInput};

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
        mut generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

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

        let variant_ident = default_variant.ident.clone();
        let default_variant_constr = match default_variant.fields.clone() {
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

        add_trait_bounds(&mut generics, &default_variant);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let output = quote! {
            impl #impl_generics std::default::Default for #struct_ident #ty_generics #where_clause {
                fn default() -> Self {
                    #default_variant_constr
                }
            }
        };
        // eprintln!("{}", output);
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

fn add_trait_bounds(generics: &mut syn::Generics, variant: &syn::Variant) {
    let used_types: HashSet<syn::Ident> = variant
        .fields
        .iter()
        .filter_map(|field| type_ident(&field.ty))
        .cloned()
        .collect();

    for type_param in generics.type_params_mut() {
        if used_types.contains(&type_param.ident) {
            type_param
                .bounds
                .push(parse_quote!(::std::default::Default));
        }
    }
}

fn type_ident(ty: &syn::Type) -> Option<&syn::Ident> {
    if let &syn::Type::Path(syn::TypePath {
        qself: None,
        ref path,
    }) = ty
    {
        if path.segments.len() == 1 {
            return Some(&path.segments.first()?.ident);
        }
    }
    None
}
