use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident, LitInt, Token, braced,
    parse::{Parse, ParseStream},
    token,
};

#[allow(unused)]
struct Seq {
    ident: Ident,
    in_token: Token![in],
    start: LitInt,
    range_token: Token![..],
    end: LitInt,
    brace_token: token::Brace,
}

impl Parse for Seq {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Seq {
            ident: input.parse()?,
            in_token: input.parse()?,
            start: input.parse()?,
            range_token: input.parse()?,
            end: input.parse()?,
            brace_token: braced!(content in input),
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as Seq);
    let output = derive(input).unwrap_or_else(syn::Error::into_compile_error);
    // eprintln!("{}", output);
    // panic!("{}", output);
    proc_macro::TokenStream::from(output)
}

fn derive(_input: Seq) -> syn::Result<TokenStream> {
    let output = quote! {};
    Ok(output)
}
