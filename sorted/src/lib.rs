use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Item;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let input = syn::parse_macro_input!(input as syn::Item);

    let output = sorted_enum(input).unwrap_or_else(syn::Error::into_compile_error);
    // eprintln!("{}", output);
    // panic!("{}", output);

    proc_macro::TokenStream::from(output)
}

fn sorted_enum(item: Item) -> syn::Result<TokenStream> {
    let Item::Enum(item_enum) = item else {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    Ok(item_enum.into_token_stream())
}
