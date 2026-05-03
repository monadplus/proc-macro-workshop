use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    DeriveInput, Field, FieldsNamed, GenericArgument, GenericParam, Generics, PathArguments, Type,
    parse_quote, spanned::Spanned,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let output = derive(input).unwrap_or_else(syn::Error::into_compile_error);
    // eprintln!("{}", output);
    // panic!("{}", output);
    proc_macro::TokenStream::from(output)
}

fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => fields,
            _ => unimplemented!("Only named fields are supported"),
        },
        other => unimplemented!("Builder macro does not support {:?}", other),
    };

    let debug_struct_fields = fields
        .named
        .iter()
        .map(|f| {
            let f_ident = &f.ident.as_ref().expect("Named fields should have an ident");
            let fmt_str = debug_attr(f)?.unwrap_or_else(|| "{:?}".to_string());
            Ok(quote_spanned!(f.span() =>
               .field(stringify!(#f_ident), &format_args!(#fmt_str, self.#f_ident))
            ))
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let generics = add_trait_bounds(input.generics, &fields);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::std::fmt::Debug for #struct_ident #ty_generics #where_clause {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                fmt.debug_struct(stringify!(#struct_ident))
                   #(#debug_struct_fields)*
                   .finish()
            }
        }
    };

    Ok(output)
}

// Find attribute `#[debug = "..."]`
fn debug_attr<'a>(field: &Field) -> Result<Option<String>, syn::Error> {
    for attr in &field.attrs {
        if !attr.path().is_ident("debug") {
            continue;
        }

        match &attr.meta {
            syn::Meta::NameValue(meta) => match &meta.value {
                syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                    syn::Lit::Str(s) => return Ok(Some(s.value())),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &expr_lit.lit,
                            "expected string literal",
                        ));
                    }
                },
                other => return Err(syn::Error::new_spanned(other, "expected string literal")),
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    attr,
                    r#"expected #[debug = "..."]"#,
                ));
            }
        }
    }

    Ok(None)
}

// Add a bound `T: Debug` to every type parameter T.
fn add_trait_bounds(mut generics: Generics, fields: &FieldsNamed) -> Generics {
    let phantom_idents = fields
        .named
        .iter()
        .filter_map(|f| get_phantom_type_ident(&f.ty))
        .collect::<HashSet<_>>();

    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            // impl<T: ?Sized> Debug for PhantomData<T> {...}
            if phantom_idents.contains(&type_param.ident) {
                continue;
            }
            type_param.bounds.push(parse_quote!(::std::fmt::Debug));
        }
    }
    generics
}

/// Retrieves the type parameter `T` of a phantom type `Phantom<T>`.
fn get_phantom_type_ident<'a>(ty: &'a Type) -> Option<&'a syn::Ident> {
    if let syn::Type::Path(type_path) = inner_type(ty, Some("PhantomData"))? {
        let type_ident = &type_path.path.segments.first()?.ident;
        return Some(type_ident);
    }

    None
}

/// Returns the type parameter of a type constructor e.g. `PhantomData<T> -> T`.
fn inner_type<'a>(ty: &'a Type, wrapping_ty_ident: Option<&str>) -> Option<&'a syn::Type> {
    if let Type::Path(syn::TypePath { qself: None, path }) = ty {
        if path.segments.len() != 1 {
            return None;
        }

        if let Some(ty_ident) = wrapping_ty_ident {
            if path.segments[0].ident != ty_ident {
                return None;
            }
        }

        if let PathArguments::AngleBracketed(ref inner_type) = path.segments[0].arguments {
            if inner_type.args.len() != 1 {
                return None;
            }

            if let GenericArgument::Type(ref ty) = inner_type.args[0] {
                return Some(ty);
            }
        }
    }
    None
}
