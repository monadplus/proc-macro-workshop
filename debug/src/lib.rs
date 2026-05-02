use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(CustomDebug)]
pub fn derive_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let output = derive(input).unwrap_or_else(syn::Error::into_compile_error);
    // eprintln!("{}", output);
    proc_macro::TokenStream::from(output)
}

fn derive(_input: DeriveInput) -> syn::Result<TokenStream> {
    let output = quote! {};

    Ok(output)
}
