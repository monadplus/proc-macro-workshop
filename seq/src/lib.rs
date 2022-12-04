use proc_macro::TokenStream;
use syn::{braced, parse::Parse, parse_macro_input, Token};

struct Sequence {
    var_name: syn::Ident,
    start: syn::LitInt,
    end: syn::LitInt,
    loop_expr: proc_macro2::TokenStream,
}

// seq!(N in 0..8 {
//   ...
// });
impl Parse for Sequence {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let var_name: syn::Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: syn::LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        let end: syn::LitInt = input.parse()?;
        let content;
        braced!(content in input);
        let loop_expr: proc_macro2::TokenStream = content.parse()?;
        Ok(Self {
            var_name,
            start,
            end,
            loop_expr,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let Sequence {
        var_name,
        start,
        end,
        loop_expr,
    } = parse_macro_input!(input as Sequence);

    let output = proc_macro2::TokenStream::new();

    TokenStream::from(output)
}
