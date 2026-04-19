use syn::DeriveInput;

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _input = syn::parse_macro_input!(input as DeriveInput);
    let output = quote::quote! {
        const {}
    };
    proc_macro::TokenStream::from(output)
}
