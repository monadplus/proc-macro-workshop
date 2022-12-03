use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
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

// ```rust, ignore
// #[derive(Builder)]
// pub struct Command {
//     #[builder(each = "env")]
//     env: Vec<String>,
// }
//
// get_attr(&field, "builder") // Some(attr)
// get_attr(&field, "foo") // None
// ```
fn get_attr<'a, 'b>(field: &'a syn::Field, attr_name: &'b str) -> Option<&'a syn::Attribute> {
    let attrs = &field.attrs;
    if attrs.len() == 1 {
        let attr = &attrs[0];
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == attr_name {
            return Some(attr);
        }
    }
    None
}

fn get_builder_attr<'a>(field: &'a syn::Field) -> Option<&'a syn::Attribute> {
    get_attr(field, "builder")
}

// ```rust, ignore
// #[derive(Builder)]
// pub struct Command {
//     #[builder(each = "env")]
//     env: Vec<String>,
// }
//
// let attr = get_attr(&field, "builder")?;
// get_attr_value(&attr) // Some("env")
// ```
fn get_attr_value(attr: &syn::Attribute) -> Option<String> {
    match attr.parse_meta() {
        Ok(syn::Meta::List(meta_list)) => {
            if meta_list.nested.len() != 1 {
                return None;
            }
            match &meta_list.nested[0] {
                syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                    if !name_value.path.is_ident("each") {
                        return None;
                    }

                    match &name_value.lit {
                        syn::Lit::Str(lit_str) => Some(lit_str.value()),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn mk_attr_error<T: ToTokens>(tokens: T) -> Option<proc_macro2::TokenStream> {
    Some(
        syn::Error::new_spanned(tokens, r##"expecting #[builder(each = "...")]"##)
            .to_compile_error(),
    )
}

#[proc_macro_derive(Builder, attributes(builder))]
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
        // Do not wrap neither Option nor Vec
        if inner_type("Option", &ty).is_some() || get_builder_attr(field).is_some() {
            quote!(#name: #ty)
        } else {
            quote!(#name: ::std::option::Option<#ty>)
        }
    });

    let default_builder_fields = fields.iter().map(|field| {
        let name = &field.ident;
        if get_builder_attr(field).is_some() {
            quote!(#name: ::std::vec::Vec::new())
        } else {
            quote!(#name: ::std::option::Option::None)
        }
    });

    let setter_fns = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let ty = inner_type("Option", &field.ty).unwrap_or_else(|| &field.ty);

        match get_builder_attr(&field) {
            None => Some(quote! {
                fn #field_name(&mut self, #field_name: #ty) -> &mut Self {
                    self.#field_name = ::std::option::Option::Some(#field_name);
                    self
                }
            }),

            Some(attr) => match get_attr_value(&attr) {
                Some(value) if value == format!("{}", field_name) => None,
                _ => Some(quote! {
                    fn #field_name(&mut self, #field_name: #ty) -> &mut Self {
                        self.#field_name = #field_name;
                        self
                    }
                }),
            },
        }
    });

    let vec_setter_fns = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let attr = get_builder_attr(&field)?;

        match attr.parse_meta() {
            Ok(syn::Meta::List(meta_list)) => {
                if meta_list.nested.len() != 1 {
                    return mk_attr_error(meta_list.nested);
                }

                match &meta_list.nested[0] {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                        if !name_value.path.is_ident("each") {
                            return mk_attr_error(&name_value.path);
                        }

                        match &name_value.lit {
                            syn::Lit::Str(lit_str) => {
                                let method_name =
                                    syn::Ident::new(&lit_str.value()[..], lit_str.span());
                                let inner_ty = inner_type("Vec", ty).unwrap();
                                Some(quote! {
                                    fn #method_name(&mut self, elem: #inner_ty) -> &mut Self {
                                        self.#field_name.push(elem);
                                        self
                                    }
                                })
                            }
                            _ => mk_attr_error(name_value),
                        }
                    }
                    other => mk_attr_error(other),
                }
            }
            _ => mk_attr_error(attr),
        }
    });

    let build_fields_assignments = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        if inner_type("Option", &ty).is_some() | get_builder_attr(&field).is_some() {
            quote!(#field_name: self.#field_name.clone())
        } else {
            quote!(#field_name : self.#field_name.clone().ok_or(concat!(stringify!(#field_name), " is not set"))?)
        }
    });

    let output = quote! {
        #vis struct #builder_name #generics {
            #(#builder_fields),*
        }

        impl #builder_name {
            pub fn build(&mut self) -> std::result::Result<#struct_name, std::boxed::Box<dyn std::error::Error>> {
                std::result::Result::Ok(#struct_name {
                  #(#build_fields_assignments),*
                })
            }
            #(#setter_fns)*
            #(#vec_setter_fns)*
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
