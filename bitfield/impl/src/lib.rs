use std::{collections::HashSet, fmt::Display};

use either::Either;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Expr, Fields, Ident, Lit, Variant};

#[allow(dead_code)]
fn error<U: Display, T: ToTokens>(message: U, tokens: T) -> proc_macro::TokenStream {
    syn::Error::new_spanned(tokens, message)
        .into_compile_error()
        .into()
}

#[proc_macro_attribute]
pub fn bitfield(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis,
        ident: struct_ident,
        generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let fields = match data {
        syn::Data::Struct(strct) => {
            if let Fields::Named(fields) = strct.fields {
                fields.named
            } else {
                unimplemented!("#[bitfield] must be used on a named struct");
            }
        }
        _ => unimplemented!("#[bitfield] must be used on a struct"),
    };

    let getters_and_setters = fields.clone().into_iter().scan(quote!(0), |offset, field| {
        let ident = field.ident.unwrap();
        let get_ident = quote::format_ident!("get_{}", ident);
        let set_ident = quote::format_ident!("set_{}", ident);

        let ty = field.ty;
        let ty_span = ty.span();

        let output = quote_spanned! {ty_span=>
            pub fn #get_ident(&self) -> <#ty as Specifier>::TypeRepr {
                <#ty as Specifier>::get(&self.data[..], #offset)
            }
            pub fn #set_ident(&mut self, value: <#ty as Specifier>::TypeRepr) {
                <#ty as Specifier>::set(&mut self.data[..], #offset, value)
            }
        };
        *offset = quote!((#offset + <#ty as Specifier>::BITS));
        Some(output)
    });

    let tys = fields.into_iter().map(|field| field.ty);

    let assert_mod8 = {
        let ty_mod = tys.clone().fold(
            quote!(ZeroMod8),
            |acc_mod, ty| quote!(AddMod8<#acc_mod, <#ty as Specifier>::Mod8>),
        );
        quote_spanned! {struct_ident.span() =>
            struct _AssertMod8 where #ty_mod: ::bitfield::TotalSizeIsMultipleOfEightBits;
        }
    };

    let size = quote!((#(<#tys as Specifier>::BITS)+*) / 8);

    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    let output = quote! {
        #assert_mod8

        #[repr(C)]
        #vis struct #struct_ident #ty_generics {
            data: [u8; #size],
        }

        impl #impl_generics #struct_ident #ty_generics {
            pub fn new() -> Self {
                Self {
                    data: [0u8; #size],
                }
            }
            #(#getters_and_setters)*
        }
    };

    proc_macro::TokenStream::from(output)
}

fn get_int_repr(value: usize) -> TokenStream {
    match value {
        0..=8 => quote!(u8),
        9..=16 => quote!(u16),
        17..=32 => quote!(u32),
        33..=64 => quote!(u64),
        _ => unreachable!(),
    }
}

fn get_type_mod8(value: usize) -> TokenStream {
    match value {
        i if i % 8 == 0 => quote!(ZeroMod8),
        i if i % 8 == 1 => quote!(OneMod8),
        i if i % 8 == 2 => quote!(TwoMod8),
        i if i % 8 == 3 => quote!(ThreeMod8),
        i if i % 8 == 4 => quote!(FourMod8),
        i if i % 8 == 5 => quote!(FiveMod8),
        i if i % 8 == 6 => quote!(SixMod8),
        i if i % 8 == 7 => quote!(SevenMod8),
        _ => unreachable!(),
    }
}

#[proc_macro]
pub fn generate_specifiers(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = TokenStream::new();

    let specifiers = (1..=64).map(|index: usize| {
        let s_ident = Ident::new(&format!("B{}", index), Span::call_site());
        let int_repr = get_int_repr(index);
        let type_mod8 = get_type_mod8(index);
        quote! {
            pub enum #s_ident {}

            impl Specifier for #s_ident {
                const BITS: usize = #index;
                type TypeRepr = #int_repr;
                type IntRepr = #int_repr;
                type Mod8 = #type_mod8;

                fn to_type_repr(x: Self::IntRepr) -> Self::TypeRepr {
                    x
                }
            }
        }
    });

    let last_byte_impls = [8, 16, 32, 64, 128].into_iter().map(|size| {
        let ident = Ident::new(&format!("u{}", size), Span::call_site());
        quote! {
            impl LastByte for #ident {
                fn last_byte(self) -> u8 {
                    self as u8
                }
            }
        }
    });

    output.extend(specifiers);
    output.extend(last_byte_impls);

    proc_macro::TokenStream::from(output)
}

#[proc_macro]
pub fn generate_mod8_impls(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = TokenStream::new();

    let values = [
        Ident::new("ZeroMod8", Span::call_site()),
        Ident::new("OneMod8", Span::call_site()),
        Ident::new("TwoMod8", Span::call_site()),
        Ident::new("ThreeMod8", Span::call_site()),
        Ident::new("FourMod8", Span::call_site()),
        Ident::new("FiveMod8", Span::call_site()),
        Ident::new("SixMod8", Span::call_site()),
        Ident::new("SevenMod8", Span::call_site()),
    ];

    let impls: Vec<TokenStream> = values
        .clone()
        .into_iter()
        .enumerate()
        .flat_map(|(i, lhs)| {
            let values = &values;
            values.clone().into_iter().enumerate().map(move |(j, rhs)| {
                let output = values[(i + j) % 8].clone();
                quote! {
                    impl CAddMod8<#rhs> for #lhs { type Output = #output; }
                }
            })
        })
        .collect();
    output.extend(impls);

    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(BitfieldSpecifier)]
pub fn derive_bitfield_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident: enum_ident,
        generics: _,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let variants = match data {
        syn::Data::Enum(e) => e.variants,
        other => unimplemented!("BitfieldSpecifier is not supported for {:?}", other),
    };
    let n_variants = variants.len();

    if n_variants == 0 {
        return syn::Error::new_spanned(enum_ident, r#"Empty enums are not allowed"#)
            .into_compile_error()
            .into();
    }

    if !n_variants.is_power_of_two() {
        // syn::Error::new(Span::call_site(), "...")
        return syn::Error::new_spanned(enum_ident, r#"Number of variants must be power of two"#)
            .into_compile_error()
            .into();
    }

    let bits = n_variants.ilog2() as usize;
    let int_repr = get_int_repr(bits);
    let type_mod8 = get_type_mod8(bits);

    let mut discriminants: HashSet<usize> = HashSet::with_capacity(n_variants);
    for variant in variants.iter() {
        let err = |msg: &str| {
            let error = syn::Error::new_spanned(variant, msg).into_compile_error();
            proc_macro::TokenStream::from(error)
        };
        // We only validate explicit discriminants.
        // We can assume that rust discriminants are always valid.
        // TODO: validate const variables discriminants
        if let Some(Either::Left(discriminant)) = get_discriminant(variant) {
            let not_in_range_err = err(&format!(r#"Valid range [0-{}]"#, n_variants - 1));

            if discriminant < 0 {
                return not_in_range_err;
            }

            let discriminant = discriminant as usize;
            if discriminant >= n_variants {
                return not_in_range_err;
            }

            // Discriminants cannot be repeated, otherwise you won't cover the whole range.
            if !discriminants.insert(discriminant) {
                return err("Repeated discriminant");
            }
        }
    }

    let type_repr_cases = variants.into_iter().map(|variant| {
        let ident = variant.ident;
        let case_ident = { Ident::new(&ident.to_string().to_lowercase(), ident.span()) };
        quote!( #case_ident if #case_ident == Self::#ident as Self::IntRepr => Self::#ident )
    });

    let output = quote! {
        impl From<#enum_ident> for #int_repr {
            fn from(type_repr: #enum_ident) -> #int_repr {
                // Safety: the macro guarantees the correct size of the discriminants
                type_repr as #int_repr
            }
        }

        impl Specifier for #enum_ident {
            const BITS: usize = #bits;
            type TypeRepr = Self;
            type IntRepr = #int_repr;
            type Mod8 = #type_mod8;

            fn to_type_repr(int_repr: Self::IntRepr) -> Self::TypeRepr {
                match int_repr {
                    #(#type_repr_cases,)*
                    _ => unreachable!("`IntRepr` range should have been covered."),
                }
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

fn get_discriminant(variant: &Variant) -> Option<Either<isize, &Ident>> {
    variant
        .discriminant
        .as_ref()
        .and_then(|(_, expr)| match expr {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                Lit::Int(lit) => {
                    let value = lit
                        .base10_parse()
                        .expect("Enum discriminants should be `isize`");
                    Some(Either::Left(value))
                }
                _ => None,
            },
            Expr::Path(expr_path) => expr_path.path.get_ident().map(Either::Right),
            _ => None,
        })
}
