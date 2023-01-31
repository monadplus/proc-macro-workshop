use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as proc_macro2::TokenStream);
    let mut output = input.clone();

    let input = parse_macro_input!(input as syn::Item);

    if let Err(err) = __sorted(args, input) {
        let err = err.into_compile_error().into();
        output.extend::<TokenStream>(err);
    }
    output
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

    for (i, v1) in ienum.variants.iter().enumerate() {
        for v2 in ienum.variants.iter().skip(i + 1) {
            if v1.ident > v2.ident {
                return Err(syn::Error::new_spanned(
                    &v2.ident,
                    format!("{} should sort before {}", v2.ident, v1.ident),
                ));
            }
        }
    }

    let output: proc_macro2::TokenStream = ienum.to_token_stream();

    Ok(output)
}
