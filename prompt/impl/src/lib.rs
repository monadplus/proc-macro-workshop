use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput, Fields, FieldsNamed, Ident, spanned::Spanned, FieldsUnnamed};

// TODO:
// - [ ] Newtype (single unnamed field) should default to underlying type
// - [ ] Unnamed fields not allowed
// - [ ] Enum as Select, then recurse.
// - [ ] attribute to rely on the FromString+ToString instance

#[proc_macro_derive(FromPrompt, attributes(newtype))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident: name,
        generics: _,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let output = match data {
        syn::Data::Struct(s) => derive_struct(name, s.fields),
        _ => syn::Error::new_spanned(name, "`Prompt` cannot be derived for unions.")
            .to_compile_error(),
    };

    proc_macro::TokenStream::from(output)
}

fn derive_struct(name: Ident, fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let fields_stmts = named.iter().map(|field| {
                let field_name = &field.ident.as_ref().unwrap() /* Named field*/;
                let field_name_str = format!("{}.{}", name, field_name);
                let ty = &field.ty;
                quote_spanned!(ty.span() => let #field_name = <#ty as derive_prompt::Prompt>::prompt(#field_name_str.to_string(), None)?;)
            });
            let fields_name = named.iter().map(|field| &field.ident);
            let new_instance_msg = format!("New instance of {}", name);
            let tokens = quote! {
                impl derive_prompt::Prompt for #name {
                    fn prompt(_name: String, _help: Option<String>) -> derive_prompt::InquireResult<Self> {
                        println!(#new_instance_msg);

                        #(#fields_stmts)*

                        Ok(#name {
                          #(#fields_name),*
                        })
                    }
                }
            };
            // eprintln!("{}", tokens);
            tokens
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, ..}) => {
            let tuple_fields = unnamed.iter().enumerate().map(|(i, field)| {
                let ty = &field.ty;
                let field_name = format!("{}.{}", name, i);
                quote_spanned!(ty.span() => <#ty as derive_prompt::Prompt>::prompt(#field_name.to_string(), None)?)
            });
            let new_instance_msg = format!("New instance of {}", name);
            let tokens = quote! {
                impl derive_prompt::Prompt for #name {
                    fn prompt(_name: String, _help: Option<String>) -> derive_prompt::InquireResult<Self> {
                        println!(#new_instance_msg);
                        Ok(#name(#(#tuple_fields),*))
                    }
                }
            };
            // eprintln!("{}", tokens);
            tokens
        }
        _ => syn::Error::new_spanned(name, "`FromPrompt` is not supported for unit types") .to_compile_error(),
    }
}
