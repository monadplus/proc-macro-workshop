use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Fields};

// Extract the simple inner type of an outer type from a field
//
// ```
// Option<String> -> Some(String)
// String => None
// ```
fn inner_type<'a>(outer_type: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        qself: None,
        ref path,
    }) = ty
    {
        if path.segments.len() != 1 || path.segments[0].ident != outer_type {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(ref inner_type) = path.segments[0].arguments {
            if inner_type.args.len() != 1 {
                return None;
            }

            if let syn::GenericArgument::Type(ref ty) = inner_type.args[0] {
                return Some(ty);
            }
        }
    }
    None
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs: _,
        vis,
        ident: struct_name,
        generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let builder_name = format_ident!("{}Builder", struct_name);

    let fields = match data {
        syn::Data::Struct(strct) => {
            if let Fields::Named(fields) = strct.fields {
                fields.named
            } else {
                unimplemented!("Builder only supports named fields")
            }
        }
        other => unimplemented!("Builder is not supported for {:?}", other),
    };

    let builder_fields = fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        if inner_type("Option", &ty).is_some() {
            quote!(#name: #ty)
        } else {
            quote!(#name: ::std::option::Option<#ty>)
        }
    });

    let default_builder_fields = fields.iter().map(|field| {
        let name = &field.ident;
        quote!(#name : ::std::option::Option::None)
    });

    let output = quote! {
        #vis struct #builder_name #generics {
            #(#builder_fields),*
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#default_builder_fields),*
                }
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
