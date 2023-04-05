use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::visit_mut::VisitMut;

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

fn __sorted(args: proc_macro2::TokenStream, input: syn::Item) -> Result<(), syn::Error> {
    match input {
        syn::Item::Enum(ienum) => {
            let variants = ienum.variants.iter().map(|v| v.ident.clone()).collect();

            check_order(variants, |ident: &syn::Ident| ident.to_string())?;

            Ok(())
        }
        _ => Err(syn::Error::new_spanned(
            args,
            "expected enum or match expression",
        )),
    }
}

fn check_order<T1, T2, F>(v: Vec<T1>, f: F) -> Result<(), syn::Error>
where
    F: Fn(&T1) -> T2,
    T1: quote::ToTokens,
    T2: Ord + std::fmt::Display,
{
    for (i, v1) in v.iter().enumerate() {
        for v2 in v.iter().skip(i + 1) {
            let fv1 = f(v1);
            let fv2 = f(v2);
            if fv1 > fv2 {
                return Err(syn::Error::new_spanned(
                    v2,
                    format!("{} should sort before {}", fv2, fv1),
                ));
            }
        }
    }

    Ok(())
}

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(input as syn::ItemFn);

    let mut visitor = CheckVisitor { errors: vec![] };
    visitor.visit_item_fn_mut(&mut item_fn);

    let mut output = TokenStream::from(quote!(#item_fn));

    for err in visitor.errors {
        let err: TokenStream = err.into_compile_error().into();
        output.extend(err);
    }

    output
}

struct CheckVisitor {
    errors: Vec<syn::Error>,
}

impl VisitMut for CheckVisitor {
    fn visit_expr_match_mut(&mut self, expr_match: &mut syn::ExprMatch) {
        // Check if there's the attribute #[sorted] attr and remove it
        if let Some(idx) = expr_match
            .attrs
            .iter()
            .position(|attr| attr.path.is_ident("sorted"))
        {
            // Remove attribute to make code compile
            expr_match.attrs.remove(idx);

            // Check underscore is last
            if let Some(idx) = expr_match
                .arms
                .iter()
                .position(|arm| matches!(arm.pat, syn::Pat::Wild(_)))
            {
                if idx != expr_match.arms.len() - 1 {
                    let wildcard_pat = expr_match.arms[idx].clone();
                    let err = syn::Error::new_spanned(
                        &wildcard_pat.pat,
                        r#"wildcards must be placed last"#,
                    );
                    self.errors.push(err);
                }
            }

            let arms_path: Option<Vec<syn::Path>> = expr_match
                .arms
                .iter()
                .filter(|arm| !matches!(arm.pat, syn::Pat::Wild(_)))
                .map(|arm| match &arm.pat {
                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone().into()),
                    syn::Pat::Path(pat_path) => Some(pat_path.path.clone()),
                    syn::Pat::Struct(pat_struct) => Some(pat_struct.path.clone()),
                    syn::Pat::TupleStruct(pat_tuple_struct) => Some(pat_tuple_struct.path.clone()),
                    otherwise => {
                        eprintln!("{:#?}", &otherwise);
                        let err = syn::Error::new_spanned(otherwise, r#"unsupported by #[sorted]"#);
                        self.errors.push(err);
                        None
                    }
                })
                .collect();

            if let Some(arms_path) = arms_path {
                if let Err(err) = check_order(arms_path, |path: &syn::Path| {
                    path.segments
                        .iter()
                        .map(|segment| quote!(#segment).to_string())
                        .collect::<Vec<_>>()
                        .join("::")
                }) {
                    self.errors.push(err);
                }
            }
        }
    }
}
