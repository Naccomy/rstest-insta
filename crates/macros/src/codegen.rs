use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Block, FnArg, Ident, ItemFn, Pat, parse_quote, spanned::Spanned};

use crate::utils::crate_name;

/// Default variable name for the `rstest::Context` binding in the generated code.
const DEFAULT_CTX_BINDING: &str = "__rstest_insta__ctx";

pub fn expand(input_fn: ItemFn) -> TokenStream {
    let input_fn = expand_attribute(input_fn);
    let (input_fn, ctx_binding) = context_binding(input_fn);
    let input_fn = expand_body(input_fn, ctx_binding);
    input_fn.into_token_stream()
}

fn expand_attribute(mut input_fn: ItemFn) -> ItemFn {
    let rstest_crate = crate_name("rstest");
    let rstest_attr: syn::Attribute = parse_quote! {
        #[#rstest_crate::rstest]
    };
    input_fn.attrs.insert(0, rstest_attr);
    input_fn
}

/// Retrieve the function argument annotated with the `context` attribute.
/// If not argument with the `context` attribute is present, it is added to the function signature.
fn context_binding(mut input_fn: ItemFn) -> (ItemFn, Ident) {
    match context_var(&input_fn.sig.inputs) {
        Some(ctx_ident) => (input_fn, ctx_ident.clone()),
        None => {
            let default_ctx_binding = Ident::new(DEFAULT_CTX_BINDING, input_fn.span());

            let ctx_arg: FnArg = parse_quote! {
                #[context] #default_ctx_binding: Context
            };
            input_fn.sig.inputs.insert(0, ctx_arg);
            (input_fn, default_ctx_binding)
        }
    }
}

fn context_var<'a, I>(args: I) -> Option<Ident>
where
    I: IntoIterator<Item = &'a FnArg>,
{
    args.into_iter()
        .filter_map(|arg| match &arg {
            FnArg::Typed(pat_type) => pat_type
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("context"))
                .then_some(pat_type),
            FnArg::Receiver(_receiver) => None,
        })
        .filter_map(|pat_type| match &*pat_type.pat {
            Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
            _ => None,
        })
        .next()
}

fn expand_body(mut input_fn: ItemFn, ctx_binding: Ident) -> ItemFn {
    const RSTEST_INSTA_SUFFIX: &str = "__rstest_insta__suffix";
    let rstest_insta_suffix = Ident::new(RSTEST_INSTA_SUFFIX, input_fn.span());
    let insta_crate = crate_name("insta");

    let fn_block = input_fn.block;

    let body: Block = parse_quote! {
        {
            let #rstest_insta_suffix = #ctx_binding.description.map(|s| s.to_string()).unwrap_or(#ctx_binding.case.unwrap_or(0).to_string());
            #insta_crate::with_settings!({snapshot_suffix => #rstest_insta_suffix}, {
                #fn_block
            })
        }
    };
    input_fn.block = Box::new(body);

    input_fn
}

#[cfg(test)]
mod tests {

    use crate::test_helpers;

    use super::*;

    #[test]
    fn expand_attribute_when_fn_contains_no_attributes() {
        // Given
        let input_fn: syn::ItemFn = parse_quote! {
            fn my_test() {
                println!("Hello, World!")
            }
        };

        // When
        let expanded = expand_attribute(input_fn);

        // Then
        insta::assert_snapshot!(test_helpers::pretty(expanded));
    }

    #[test]
    fn context_binding_when_fn_contains_no_context_arg() {
        // Given
        let input_fn: syn::ItemFn = parse_quote! {
            fn my_test() {
                println!("Hello, World!")
            }
        };

        // When
        let (expanded, _binding) = context_binding(input_fn);

        // Then
        insta::assert_snapshot!(test_helpers::pretty(expanded));
    }

    #[test]
    fn context_binding_when_fn_contains_context_arg() {
        // Given
        let input_fn: syn::ItemFn = parse_quote! {
            fn my_test(#[context] my_context: Context) {
                println!("Hello, World!")
            }
        };

        // When
        let (expanded, _binding) = context_binding(input_fn);

        // Then
        insta::assert_snapshot!(test_helpers::pretty(expanded));
    }

    #[test]
    fn expand_when_fn_contains_no_context_arg() {
        // Given
        let input_fn: syn::ItemFn = parse_quote! {
            fn my_test(a: usize) {
                println!("Hello, World!")
            }
        };

        // When
        let expanded = expand(input_fn);

        // Then
        insta::assert_snapshot!(test_helpers::pretty(expanded));
    }
}
