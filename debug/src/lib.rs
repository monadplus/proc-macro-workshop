use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as DeriveInput);
    let output = TokenStream::new();
    proc_macro::TokenStream::from(output)
}
