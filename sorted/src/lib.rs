use proc_macro2::Span;
use quote::quote;
use syn::visit_mut::VisitMut;

#[proc_macro_attribute]
pub fn sorted(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
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

    let variants = item_enum
        .variants
        .into_iter()
        .map(|v| v.ident)
        .collect::<Vec<_>>();

    find_out_of_order(variants)
}

#[proc_macro_attribute]
pub fn check(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut fn_item = syn::parse_macro_input!(input as syn::ItemFn);

    let mut visitor = MatchSorted::default();
    visitor.visit_item_fn_mut(&mut fn_item);
    let mut output = proc_macro::TokenStream::from(quote! {#fn_item});

    for error in visitor.errors {
        output.extend(proc_macro::TokenStream::from(error.into_compile_error()));
    }

    // eprintln!("{}", output);
    // panic!("{}", output);
    output
}

#[derive(Default)]
struct MatchSorted {
    pub errors: Vec<syn::Error>,
}

impl VisitMut for MatchSorted {
    fn visit_expr_match_mut(&mut self, node: &mut syn::ExprMatch) {
        if let Some(idx) = node
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident("sorted"))
        {
            // Strip to avoid a compilation error
            node.attrs.remove(idx);

            let variants = node
                .arms
                .iter()
                .filter_map(|arm| match &arm.pat {
                    syn::Pat::Struct(v) => v.path.segments.last().map(|p| p.ident.clone()),
                    syn::Pat::TupleStruct(v) => v.path.segments.last().map(|p| p.ident.clone()),
                    otherwise => {
                        let err = syn::Error::new_spanned(otherwise, r#"unsupported by #[sorted]"#);
                        self.errors.push(err);
                        None
                    }
                })
                .collect::<Vec<_>>();

            if let Some(err) = find_out_of_order(variants) {
                self.errors.push(err);
            }
        }
    }
}

fn find_out_of_order<T>(vs: Vec<T>) -> Option<syn::Error>
where
    T: Ord + syn::spanned::Spanned + std::fmt::Display,
{
    for i in 1..vs.len() {
        let current = &vs[i];
        let prev = &vs[i - 1];

        if current < prev {
            let before = vs[..i].binary_search_by(|v| v.cmp(&current)).unwrap_err();
            let before = &vs[before];

            return Some(syn::Error::new(
                current.span(),
                format!("{} should sort before {}", current, before),
            ));
        }
    }

    None
}
