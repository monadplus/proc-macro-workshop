use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, Fields, GenericArgument, PathArguments, Type,
    TypePath,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        vis: _,
        ident: struct_ident,
        generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let bound_attr = match get_bound_attr(&attrs) {
        Ok(bound_attr) => bound_attr,
        Err(err) => return err.into_compile_error().into(),
    };

    let fields = match data {
        syn::Data::Struct(strct) => {
            if let Fields::Named(fields) = strct.fields {
                fields.named
            } else {
                unimplemented!("CustomDebug only supports named fields")
            }
        }
        other => unimplemented!("CustomDebug is not supported for {:?}", other),
    };

    let generic_idents = generics
        .type_params()
        .map(|t| t.ident.clone())
        .collect::<Vec<_>>();

    let phantom_types: Vec<&Ident> = fields
        .iter()
        .filter_map(|field| get_phantom_type_ident(&field.ty, &generic_idents))
        .collect::<Vec<_>>();

    let associated_types: Vec<&TypePath> = fields
        .iter()
        .filter_map(|field| get_associated_field_type(&field.ty, &generic_idents))
        .collect::<Vec<_>>();

    let generics = add_trait_bounds(generics, phantom_types, associated_types, bound_attr);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_names = fields.iter().map(|field| {
        field
            .ident
            .as_ref()
            .expect("Named fields should have an ident")
    });

    let field_formats = fields
        .iter()
        .map(|field| get_debug_attr(field).unwrap_or_else(|| String::from("{:?}")));

    let output = quote! {
        impl #impl_generics ::std::fmt::Debug for #struct_ident #ty_generics #where_clause {
          fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
              f.debug_struct(stringify!(#struct_ident))
                  #(.field(stringify!(#field_names), &format_args!(#field_formats, &self.#field_names)))*
                  .finish()
          }
        }
    };

    // eprintln!("{}", output);

    proc_macro::TokenStream::from(output)
}

fn get_debug_attr(field: &syn::Field) -> Option<String> {
    let attr = &field.attrs.get(0)?;
    match attr.parse_meta() {
        Ok(syn::Meta::NameValue(name_value)) => {
            if !name_value.path.is_ident("debug") {
                return None;
            }
            match &name_value.lit {
                syn::Lit::Str(lit_str) => Some(lit_str.value()),
                _ => unimplemented!(r#"Only #[debug = ""] is supported"#),
            }
        }
        _ => None,
    }
}

fn get_bound_attr(attrs: &[syn::Attribute]) -> Result<Option<syn::WherePredicate>, syn::Error> {
    fn attr_error<T: quote::ToTokens>(tokens: T) -> syn::Error {
        syn::Error::new_spanned(tokens, r#"expected `debug(bound = "..")`"#)
    }

    match attrs.get(0) {
        Some(attr) => match attr.parse_meta() {
            Ok(syn::Meta::List(meta_list)) => {
                if !meta_list.path.is_ident("debug") {
                    return Ok(None);
                }

                if meta_list.nested.len() != 1 {
                    return Err(attr_error(meta_list.nested));
                }

                match &meta_list.nested[0] {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                        if !name_value.path.is_ident("bound") {
                            return Err(attr_error(&name_value.path));
                        }

                        match &name_value.lit {
                            syn::Lit::Str(lit_str) => {
                                let bound_str = lit_str.value();
                                match syn::parse_str::<syn::WherePredicate>(&bound_str) {
                                    Ok(where_clause) => Ok(Some(where_clause)),
                                    Err(err) => Err(syn::Error::new_spanned(&name_value.lit, err)),
                                }
                            }
                            _ => Err(attr_error(&name_value.lit)),
                        }
                    }
                    _ => Err(attr_error(&meta_list.nested[0])),
                }
            }
            _ => Ok(None),
        },

        None => Ok(None),
    }
}

fn add_trait_bounds(
    mut generics: syn::Generics,
    phantom_types: Vec<&Ident>,
    associated_types: Vec<&TypePath>,
    bound_attr: Option<syn::WherePredicate>,
) -> syn::Generics {
    if let Some(where_clause) = bound_attr {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#where_clause));
    } else {
        let associated_types_ident: Vec<&Ident> = associated_types
            .iter()
            .map(|type_path| &type_path.path.segments[0].ident)
            .collect();

        // impl <T: Debug, T2: Debug> Debug for Foo { ... }
        for type_param in generics.type_params_mut() {
            // Skip bound for phantom types.
            if phantom_types.contains(&&type_param.ident) {
                continue;
            }

            // Skip bound for associated types.
            if associated_types_ident.contains(&&type_param.ident) {
                continue;
            }

            type_param.bounds.push(parse_quote!(::std::fmt::Debug));
        }

        // impl <T: Debug, T2: Debug> Debug for Foo { ... } where <T3::Value>: Debug
        let where_clause = generics.make_where_clause();
        for associated_type in associated_types {
            // Notice, here we need to pass the whole type and not only the ident.
            where_clause.predicates.push(parse_quote! {
                #associated_type : ::std::fmt::Debug
            })
        }
    }
    generics
}

/// Retrieves the type parameter `T` of a phantom type `Phantom<T>`.
fn get_phantom_type_ident<'a>(ty: &'a Type, generic_idents: &[Ident]) -> Option<&'a syn::Ident> {
    if let syn::Type::Path(type_path) = inner_type(ty, Some("PhantomData"))? {
        let type_ident = &type_path.path.segments.first()?.ident;
        if generic_idents.contains(type_ident) {
            return Some(type_ident);
        }
    }
    None
}

/// Retrieves the inner most associated type e.g. `Box<Option<Vec<T::Value>> -> T::Value`.
fn get_associated_field_type<'a>(
    ty: &'a Type,
    generic_idents: &[Ident],
) -> Option<&'a syn::TypePath> {
    let mut ty: &Type = ty;
    while let Some(inner_type) = inner_type(ty, None) {
        ty = inner_type;
    }

    if let Type::Path(type_path @ TypePath { qself: None, path }) = ty {
        if path.segments.len() > 1 {
            let type_ident = &type_path.path.segments.first()?.ident;
            if generic_idents.contains(type_ident) {
                return Some(type_path);
            }
        }
    }
    None
}

/// Returns the type parameter of a type constructor e.g. `PhantomData<T> -> T`.
fn inner_type<'a>(
    ty: &'a Type,
    wrapping_ty_ident: Option<&str>, /* None = ignore */
) -> Option<&'a syn::Type> {
    // Tip: eprintln! on the Type.
    // NB. Make sure the test compiles to be able to emit the debug to the sdterr.
    // eprintln!("{ty:#?}");

    if let Type::Path(syn::TypePath {
        qself: None,
        ref path,
    }) = ty
    {
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
