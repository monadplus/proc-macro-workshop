use proc_macro2::{TokenStream, TokenTree};
use quote::format_ident;
use syn::{braced, parse::Parse, parse_macro_input, LitInt, Token};

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut seq = parse_macro_input!(input as Sequence);
    seq.expand().into()
}

struct Sequence {
    var_name: syn::Ident,
    start: u64,
    end: u64,
    body: proc_macro2::TokenStream,
}

// ```rust, ignore
// seq!(N in 0..8 {
//   ...
// });
// ```
impl Parse for Sequence {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let var_name: syn::Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: u64 = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Token![..]>()?;
        let inclusive = input.parse::<Option<Token![=]>>()?.is_some();
        let mut end: u64 = input.parse::<LitInt>()?.base10_parse()?;
        if inclusive {
            end += 1;
        }
        let content;
        braced!(content in input);
        let body: proc_macro2::TokenStream = content.parse()?;
        Ok(Self {
            var_name,
            start,
            end,
            body,
        })
    }
}

impl Sequence {
    fn expand(&mut self) -> TokenStream {
        let (tokens, found) = self.replace_repetition_section(self.body.clone());
        if found {
            return tokens;
        }
        (self.start..self.end).fold(TokenStream::new(), |mut ts, i| {
            let ts_aux = self.replace_number(self.body.clone(), i);
            ts.extend(ts_aux);
            ts
        })
    }

    /// #[derive(Copy, Clone, PartialEq, Debug)]
    /// enum Interrupt {
    ///   #(Irq~N,)*
    /// }
    fn replace_repetition_section(&self, body: TokenStream) -> (TokenStream, bool) {
        let mut output_stream = TokenStream::new();
        let mut repetition_found = false;
        let mut token_iter = body.into_iter();
        while let Some(token) = token_iter.next() {
            let output_token: TokenTree = match token {
                // A possible repeated section '#'
                TokenTree::Punct(ref punct) if punct.as_char() == '#' => {
                    match look_ahead2(&token_iter) {
                        (Some(TokenTree::Group(group)), Some(TokenTree::Punct(punct)))
                            if group.delimiter() == proc_macro2::Delimiter::Parenthesis
                                && punct.as_char() == '*' =>
                        {
                            token_iter.next(); // (...)
                            token_iter.next(); // '*'
                            repetition_found = true;
                            let stream =
                                (self.start..self.end).fold(TokenStream::new(), |mut ts, i| {
                                    let ts_aux = self.replace_number(group.stream().clone(), i);
                                    ts.extend(ts_aux);
                                    ts
                                });
                            let mut group =
                                proc_macro2::Group::new(proc_macro2::Delimiter::None, stream);
                            group.set_span(token.span());
                            TokenTree::from(group)
                        }
                        _ => token,
                    }
                }
                // Expand content of (), {}, []
                TokenTree::Group(ref group) => {
                    let del = group.delimiter();
                    let (stream, found) = self.replace_repetition_section(group.stream());
                    repetition_found |= found;
                    let mut group = proc_macro2::Group::new(del, stream);
                    group.set_span(token.span());
                    TokenTree::from(group)
                }
                _ => token,
            };
            output_stream.extend(TokenStream::from(output_token));
        }
        (output_stream, repetition_found)
    }

    /// fn f~N () -> u64 {
    ///     N * 2
    /// }
    fn replace_number(&self, body: TokenStream, val: u64) -> TokenStream {
        let mut output_stream = TokenStream::new();
        let mut token_iter = body.into_iter();

        while let Some(token) = token_iter.next() {
            let output_token = match token {
                // N
                TokenTree::Ident(ref ident) if ident == &self.var_name => {
                    let mut lit = proc_macro2::Literal::u64_unsuffixed(val);
                    lit.set_span(token.span());
                    TokenTree::from(lit)
                }
                // <prefix>~N
                TokenTree::Ident(ref prefix) => {
                    match look_ahead2(&token_iter) {
                        (Some(TokenTree::Punct(punct)), Some(TokenTree::Ident(ref ident)))
                            if punct.as_char() == '~' && ident == &self.var_name =>
                        {
                            token_iter.next(); // Consume '~'
                            token_iter.next(); // Consume ident
                            let mut ident = format_ident!("{}{}", prefix, val);
                            ident.set_span(token.span());
                            TokenTree::from(ident)
                        }
                        _ => token,
                    }
                }
                // Expand content of (), {}, []
                TokenTree::Group(ref group) => {
                    let del = group.delimiter();
                    let stream = self.replace_number(group.stream(), val);
                    let mut group = proc_macro2::Group::new(del, stream);
                    group.set_span(token.span());
                    TokenTree::from(group)
                }
                _ => token,
            };
            output_stream.extend(TokenStream::from(output_token));
        }

        output_stream
    }
}

fn look_ahead2(
    token_iter: &proc_macro2::token_stream::IntoIter,
) -> (Option<TokenTree>, Option<TokenTree>) {
    let mut peek = token_iter.clone();
    (peek.next(), peek.next())
}
