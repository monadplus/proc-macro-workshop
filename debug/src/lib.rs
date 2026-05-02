use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, spanned::Spanned};

#[proc_macro_derive(CustomDebug)]
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

    let debug_struct_fields = fields.named.iter().map(|f| {
        let f_ident = &f.ident.as_ref().expect("Named fields should have an ident");
        quote_spanned!(f.span() =>
           .field(stringify!(#f_ident), &self.#f_ident)
        )
    });

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
