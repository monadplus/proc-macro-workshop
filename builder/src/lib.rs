use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, DeriveInput, Field, GenericArgument, Ident, LitStr,
    PathArguments, Type, TypePath,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let output = derive(input).unwrap_or_else(syn::Error::into_compile_error);
    // eprintln!("{}", output);
    proc_macro::TokenStream::from(output)
}

fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    let fields = match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => fields,
            _ => unimplemented!("Only named fields are supported"),
        },
        other => unimplemented!("Builder macro does not support {:?}", other),
    };

    let builder_fields = fields.named.iter().map(|field| {
        let name = &field.ident;

        if inner_type(&field.ty, "Vec").is_some() {
            let ty = &field.ty;
            return quote! {
                #name : #ty
            };
        }

        let ty = inner_type(&field.ty, "Option").unwrap_or(&field.ty);
        quote! {
            #name : ::std::option::Option<#ty>
        }
    });

    let default_builder_fields = fields.named.iter().map(|field| {
        let name = &field.ident;

        if inner_type(&field.ty, "Vec").is_some() {
            return quote! {
                #name : ::std::vec::Vec::new()
            };
        }

        quote! {
            #name : ::std::option::Option::None
        }
    });

    let setter_fns = fields
        .named
        .iter()
        .map(|field| {
            let name = &field.ident;
            let outer_ty = &field.ty;

            if let Some(inner_ty) = inner_type(&field.ty, "Vec") {
                let mut all_in_one_setter = quote! {
                    pub fn #name (&mut self, #name: #outer_ty) -> &mut Self {
                        self.#name = #name;
                        self
                    }
                };

                let each_setter = builder_attr(field)?.map(|setter_name| {
                    let setter_name = Ident::new(&setter_name, Span::call_site());
                    // If the new one-at-a-time builder method is given the same name as the field,
                    // avoid generating an all-at-once builder method for that field because the
                    // names would conflict.
                    if name.as_ref().is_some_and(|name| name == &setter_name) {
                        all_in_one_setter = quote!();
                    }

                    quote! {
                        pub fn #setter_name (&mut self, #name: #inner_ty) -> &mut Self {
                            self.#name.push(#name);
                            self
                        }
                    }
                });

                return Ok(quote! {
                    #all_in_one_setter

                    #each_setter
                });
            }

            let ty = inner_type(&field.ty, "Option").unwrap_or(outer_ty);
            Ok(quote! {
                pub fn #name (&mut self, #name: #ty) -> &mut Self {
                    self.#name = ::std::option::Option::Some(#name);
                    self
                }
            })
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let build_fields_assignments = fields.named.iter().map(|field| {
        let name = &field.ident;
        if inner_type(&field.ty, "Option").is_some() {
            quote! {
                #name: self.#name.take()
            }
        } else if inner_type(&field.ty, "Vec").is_some() {
            quote! {
                #name: self.#name.drain(..).collect::<Vec<_>>()
            }
        } else {
            quote! {
                #name: self.#name.take().ok_or_else(|| format!("{} not set", stringify!(#name)))?
            }
        }
    });

    let output = quote! {
        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            #(#setter_fns)*

            pub fn build(&mut self) -> ::std::result::Result<#struct_name, ::std::boxed::Box<dyn ::std::error::Error>> {
                Ok(#struct_name {
                    #(#build_fields_assignments),*
                })
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#default_builder_fields),*
                }
            }
        }
    };

    Ok(output)
}

fn builder_attr(field: &Field) -> Result<Option<String>, syn::Error> {
    let mut each_value = None;

    for attr in &field.attrs {
        if !attr.path().is_ident("builder") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("each") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                each_value = Some(lit.value());
                return Ok(());
            }

            Err(meta.error(r#"expected `builder(each = "...")`"#))
        })?
    }

    Ok(each_value)
}

fn inner_type<'a>(ty: &'a Type, expected: &str) -> Option<&'a Type> {
    if let Type::Path(TypePath { qself: _, path }) = ty {
        if path.segments.len() != 1 || path.segments[0].ident != expected {
            return None;
        }

        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: _,
            lt_token: _,
            ref args,
            gt_token: _,
        }) = path.segments[0].arguments
            && args.len() == 1
            && let GenericArgument::Type(ref inner_type) = args[0]
        {
            return Some(inner_type);
        }
    }

    None
}
