use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput, Fields, FieldsNamed, Ident, spanned::Spanned, FieldsUnnamed, punctuated::Punctuated, Variant};

// TODO:
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
        syn::Data::Enum(e) => derive_enum(name, e.variants),
        _ => syn::Error::new_spanned(name, "`FromPrompt` cannot be derived for unions.")
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

// TODO: 
// - [ ] newtype enum
// - [ ] named enums
// - [ ] unnamed enums
fn derive_enum<Sep>(name: Ident, variants: Punctuated<Variant, Sep>) -> proc_macro2::TokenStream {
    if variants.is_empty() {
        return syn::Error::new_spanned(name, "`FromPrompt` is not supported for empty Enum") .to_compile_error();
    }

    let options = variants.iter().map(|variant| {
        format!("{}", variant.ident)
    });

    let cases = variants.iter().map(|variant| build_enum_case(name.clone(), &variant));

    let tokens = quote! {
        impl derive_prompt::Prompt for #name {
            fn prompt(_name: String, _help: Option<String>) -> derive_prompt::InquireResult<Self> {
                let options = vec!(#(#options),*);
                let __selected_variant = derive_prompt::Select::new("Select the variant:", options).prompt()?;
                #(#cases)*
                panic!("The user selected an enum variant that is not valid")
            }
        }
    };
    eprintln!("{}", tokens);
    tokens
}

fn build_enum_case(enum_ident: Ident, variant: &Variant) -> proc_macro2::TokenStream {
    match &variant.fields {
        Fields::Unnamed(FieldsUnnamed { unnamed,.. }) if unnamed.len() == 1 => {
            let variant_ident = &variant.ident;
            let variant_ident_str: String = format!("{}", &variant.ident);
            let prompt_str: String = format!("{}::{}", &enum_ident, &variant.ident);
            let field_type = &unnamed[0].ty;
            quote! {
                if (__selected_variant == #variant_ident_str) {
                    let field_instance = <#field_type as derive_prompt::Prompt>::prompt(#prompt_str.to_string(), None)?;
                    return Ok(#enum_ident::#variant_ident(field_instance))
                }
            }
        }
        Fields::Unnamed(_) => todo!(),
        Fields::Named(_) => todo!(),
        Fields::Unit => todo!(),
    }
}

























