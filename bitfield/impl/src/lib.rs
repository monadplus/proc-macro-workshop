use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    unimplemented!()
}

#[proc_macro]
pub fn generate_specifiers(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    let specifiers = (1..=64).map(|index: usize| {
        let s_ident = syn::Ident::new(&format!("B{}", index), Span::call_site());
        quote! {
            pub enum #s_ident {}

            impl Specifier for #s_ident {
                const BITS: usize = #index;
            }
        }
    });
    output.extend(specifiers);

    output.into()
}
