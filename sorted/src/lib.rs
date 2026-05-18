use proc_macro2::TokenStream;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let input = syn::parse_macro_input!(input as syn::Item);

    let output = __sorted(input).unwrap_or_else(syn::Error::into_compile_error);
    // eprintln!("{}", output);
    // panic!("{}", output);

    proc_macro::TokenStream::from(output)
}

fn __sorted(item: syn::Item) -> syn::Result<TokenStream> {
    Ok(item.into_token_stream())
}
