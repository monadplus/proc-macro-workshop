use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as proc_macro2::TokenStream);
    let input = parse_macro_input!(input as syn::Item);

    match __sorted(args, input) {
        Ok(output) => proc_macro::TokenStream::from(output),
        Err(err) => err.into_compile_error().into(),
    }
}

fn __sorted(
    args: proc_macro2::TokenStream,
    input: syn::Item,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ienum = match input {
        syn::Item::Enum(ienum) => ienum,
        _ => {
            return Err(syn::Error::new_spanned(
                args,
                "expected enum or match expression",
            ))
        }
    };

    let output: proc_macro2::TokenStream = ienum.to_token_stream();

    Ok(output)
}
