use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, Field, spanned::Spanned};

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
            let fmt_str = debug_attr(&f)?.unwrap_or_else(|| "{:?}".to_string());
            Ok(quote_spanned!(f.span() =>
               .field(stringify!(#f_ident), &format_args!(#fmt_str, self.#f_ident))
            ))
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let output = quote! {
        impl ::std::fmt::Debug for #struct_ident {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                fmt.debug_struct(stringify!(#struct_ident))
                   #(#debug_struct_fields)*
                   .finish()
            }
        }
    };

    Ok(output)
}

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
