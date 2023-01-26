use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident: struct_ident,
        generics: _,
        data,
    } = parse_macro_input!(input as DeriveInput);

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

    let field_names = fields.iter().map(|field| {
        field
            .ident
            .as_ref()
            .expect("Named fields should have an ident")
    });

    let field_formats = fields
        .iter()
        .map(|field| get_debug_attr(&field).unwrap_or_else(|| String::from("{:?}")));

    let output = quote! {
        impl ::std::fmt::Debug for #struct_ident {
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
