use proc_macro2::{Group, Literal, TokenStream, TokenTree};
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
    content: TokenStream,
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
            content: content.parse()?,
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

fn derive(input: Seq) -> syn::Result<TokenStream> {
    let start = input.start.base10_parse::<usize>()?;
    let end = input.end.base10_parse::<usize>()?;

    Ok(
        (start..end).fold(TokenStream::new(), |mut output, i: usize| {
            output.extend(replace_ident(input.content.clone(), &input.ident, i));
            output
        }),
    )
}

fn replace_ident(tokens: TokenStream, ident: &Ident, value: usize) -> TokenStream {
    tokens
        .into_iter()
        .map(|token| match token {
            TokenTree::Ident(token_ident) if token_ident == *ident => {
                let mut lit = Literal::usize_unsuffixed(value);
                lit.set_span(token_ident.span());
                TokenTree::from(lit)
            }
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let span = group.span();
                let stream = replace_ident(group.stream(), ident, value);
                let mut group = Group::new(delimiter, stream);
                group.set_span(span);
                TokenTree::from(group)
            }
            other => other,
        })
        .collect()
}
