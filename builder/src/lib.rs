use quote::{format_ident, quote, quote_spanned};
use syn::{
    AngleBracketedGenericArguments, DeriveInput, GenericArgument, PathArguments, Type, TypePath,
    spanned::Spanned,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

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
        let ty = inner_type(&field.ty, "Option").unwrap_or(&field.ty);
        quote_spanned! {field.span()=>
            #name : ::std::option::Option<#ty>
        }
    });

    let default_builder_fields = fields.named.iter().map(|field| {
        let name = &field.ident;
        quote! {
            #name : ::std::option::Option::None
        }
    });

    let setter_fns = fields.named.iter().map(|field| {
        let name = &field.ident;
        let ty = inner_type(&field.ty, "Option").unwrap_or(&field.ty);
        quote! {
            pub fn #name (&mut self, #name: #ty) -> &mut Self {
                self.#name = ::std::option::Option::Some(#name);
                self
            }
        }
    });

    let build_fields_assignments = fields.named.iter().map(|field| {
        let name = &field.ident;
        if inner_type(&field.ty, "Option").is_some() {
            quote! {
                #name: self.#name.take()
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

    // eprintln!("{}", output);

    proc_macro::TokenStream::from(output)
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
