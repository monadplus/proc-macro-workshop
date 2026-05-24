use proc_macro2::Span;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let mut output = input.clone();
    let enum_item = syn::parse_macro_input!(input as syn::Item);

    if let Some(error) = sorted_enum(enum_item) {
        let error = error.into_compile_error();
        output.extend(proc_macro::TokenStream::from(error));
    }
    // eprintln!("{}", output);
    // panic!("{}", output);

    proc_macro::TokenStream::from(output)
}

fn sorted_enum(item: syn::Item) -> Option<syn::Error> {
    let syn::Item::Enum(item_enum) = item else {
        return Some(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    let variants = item_enum.variants.iter().collect::<Vec<_>>();
    for i in 1..variants.len() {
        let current = &variants[i].ident;
        let prev = &variants[i - 1].ident;

        if current < prev {
            let before = variants[..i]
                .binary_search_by(|variant| variant.ident.cmp(&current))
                .unwrap_err();
            let before = &variants[before].ident;

            return Some(syn::Error::new(
                variants[i].ident.span(),
                format!("{} should sort before {}", current, before),
            ));
        }
    }

    None
}
