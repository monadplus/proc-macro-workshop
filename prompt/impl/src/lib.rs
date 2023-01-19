use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, DeriveInput, Fields, FieldsNamed,
    FieldsUnnamed, Ident, Variant,
};

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
    // eprintln!("{}", output);

    proc_macro::TokenStream::from(output)
}

fn derive_struct(name: Ident, fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let constr = quote!(#name);
            let NamedInstance {
                let_fields_decl,
                struct_decl,
            } = named_instance(constr, fields);
            let new_instance_msg = format!("New instance of {}", name);
            quote! {
                impl derive_prompt::Prompt for #name {
                    fn prompt(_name: String, _help: Option<String>) -> derive_prompt::InquireResult<Self> {
                        println!(#new_instance_msg);
                        #(#let_fields_decl)*
                        Ok(#struct_decl)
                    }
                }
            }
        }
        Fields::Unnamed(fields) => {
            let constr = quote!(#name);
            let unnamed_instance = unnamed_instance(constr, fields);
            let new_instance_msg = format!("New instance of {}", name);
            quote! {
                impl derive_prompt::Prompt for #name {
                    fn prompt(_name: String, _help: Option<String>) -> derive_prompt::InquireResult<Self> {
                        println!(#new_instance_msg);
                        Ok(#unnamed_instance)
                    }
                }
            }
        }
        _ => syn::Error::new_spanned(name, "`FromPrompt` is not supported for unit types")
            .to_compile_error(),
    }
}

fn derive_enum<Sep>(name: Ident, variants: Punctuated<Variant, Sep>) -> proc_macro2::TokenStream {
    if variants.is_empty() {
        return syn::Error::new_spanned(name, "`FromPrompt` is not supported for empty Enum")
            .to_compile_error();
    }

    let options = variants
        .iter()
        .map(|variant| variant.ident.to_string())
        .collect::<Vec<_>>();

    let cases = variants
        .into_iter()
        .map(|variant| enum_case_decl(name.clone(), variant));

    let new_instance_msg = format!("New instance of {}", name);

    quote! {
        impl derive_prompt::Prompt for #name {
            fn prompt(_name: String, _help: Option<String>) -> derive_prompt::InquireResult<Self> {
                println!(#new_instance_msg);
                let options = vec!(#(#options),*);
                let __selected_variant = derive_prompt::Select::new("Select the variant:", options).prompt()?;
                #(#cases)*
                panic!("The user selected an enum variant that is not valid")
            }
        }
    }
}

fn enum_case_decl(enum_ident: Ident, variant: Variant) -> proc_macro2::TokenStream {
    let variant_ident = variant.ident;

    let enum_instance_stmt = match variant.fields {
        Fields::Unnamed(fields) => {
            let constr = quote!(#enum_ident::#variant_ident);
            let unnamed_instance = unnamed_instance(constr, fields);
            quote! {
                return Ok(#unnamed_instance)
            }
        }
        Fields::Named(fields) => {
            let constr = quote!(#enum_ident::#variant_ident);
            let NamedInstance {
                let_fields_decl,
                struct_decl,
            } = named_instance(constr, fields);
            quote! {
                #(#let_fields_decl)*
                return Ok(#struct_decl)
            }
        }
        Fields::Unit => {
            return syn::Error::new_spanned(
                enum_ident,
                "`FromPrompt` is not supported for empty enums",
            )
            .to_compile_error()
        }
    };

    let variant_ident_str = variant_ident.to_string();
    quote! {
        if (__selected_variant == #variant_ident_str) {
            #enum_instance_stmt
        }
    }
}

struct NamedInstance {
    let_fields_decl: Vec<proc_macro2::TokenStream>,
    struct_decl: proc_macro2::TokenStream,
}

/// - `constr`: usually an ident inside a quote e.g. quote!(#constr_name)
///             or an enum constr e.g. quote!(#enum_ident::#enum_variant)
fn named_instance(constr: proc_macro2::TokenStream, fields: FieldsNamed) -> NamedInstance {
    let FieldsNamed { named, .. } = fields;

    let let_fields_name = named
        .iter()
        .map(|field| field.ident.clone())
        .collect::<Vec<_>>();
    let struct_decl = quote!(#constr { #(#let_fields_name),* });

    let let_fields_decl = named.into_iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap() /* Named field is always Some*/;
        let ty = &field.ty;
        let field_name_str = format!("<{}>.{}", constr, field_name);
        quote_spanned!{ty.span() =>
            let #field_name = <#ty as derive_prompt::Prompt>::prompt(#field_name_str.to_string(), None)?;
        }
    }).collect::<Vec<_>>();

    NamedInstance {
        let_fields_decl,
        struct_decl,
    }
}

/// - `constr`: usually an ident inside a quote e.g. quote!(#constr_name)
///             or an enum constr e.g. quote!(#enum_ident::#enum_variant)
fn unnamed_instance(
    constr: proc_macro2::TokenStream,
    fields: FieldsUnnamed,
) -> proc_macro2::TokenStream {
    let FieldsUnnamed { unnamed, .. } = fields;

    let tuple_fields = unnamed.iter().enumerate().map(|(i, field)| {
        let ty = &field.ty;
        let field_name_str = format!("{}.{}", constr, i);
        quote_spanned! {ty.span() =>
            <#ty as derive_prompt::Prompt>::prompt(#field_name_str.to_string(), None)?
        }
    });

    quote! {
        #constr(#(#tuple_fields),*)
    }
}
