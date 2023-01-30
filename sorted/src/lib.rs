use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let item = parse_macro_input!(input as syn::Item);

    let output = item.to_token_stream();

    proc_macro::TokenStream::from(output)
}
