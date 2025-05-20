use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Block, FnArg, Ident, ItemFn, Pat, parse_quote, spanned::Spanned};

use crate::utils::crate_name;

/// Expands the given function so that the generated snapshot has consistent naming.
///
/// Applies the following modifications:
/// - Expands attributes by adding the `[::rstest::rstest]` attribute macro.
/// - Adds `[context]` variable to the function if not already present.
/// - Expands the body by calling `::insta::with_suffix` to ensure consistent snapshot suffixes.
pub fn expand(input_fn: ItemFn) -> TokenStream {
    let input_fn = expand_attribute(input_fn);
    let (input_fn, ctx_binding) = context_binding(input_fn);
    let input_fn = expand_body(input_fn, ctx_binding);
    input_fn.into_token_stream()
}

/// Expand function's attributes.
///
/// Adds the `[::rstest::rstest]` attribute to the given function.
fn expand_attribute(mut input_fn: ItemFn) -> ItemFn {
    let rstest_crate = crate_name("rstest");
    let rstest_attr: syn::Attribute = parse_quote! {
        #[#rstest_crate::rstest]
    };
    input_fn.attrs.insert(0, rstest_attr);
    input_fn
}

/// Get context binding.
///
/// Retrieve function's argument annotated with the `[context]` attribute.
/// If no argument annotated with `[context]` attribute is present, one is added as the first argument to the function signature.
///
/// # Returns
///
/// The function returns a tuple with the first value being the function with the context binding.
/// If the input's function already contained a `context` variable, the function is left untouched.
/// The second value is the identifier of the variable annotated with the context attribute.
fn context_binding(mut input_fn: ItemFn) -> (ItemFn, Ident) {
    match context_var(&input_fn.sig.inputs) {
        Some(ctx_ident) => (input_fn, ctx_ident.clone()),
        None => {
            const DEFAULT_CTX_BINDING: &str = "__rstest_insta__ctx";
            let default_ctx_binding = Ident::new(DEFAULT_CTX_BINDING, input_fn.span());

            // We use the `Context` type instead of the fully-qualified `::rstest::Context` because
            // `rstest` needs it to be imported. This forces the user to import `rstest::Context`.
            let ctx_arg: FnArg = parse_quote! {
                #[context] #default_ctx_binding: Context
            };
            input_fn.sig.inputs.insert(0, ctx_arg);
            (input_fn, default_ctx_binding)
        }
    }
}

/// Get variable annotated with the `[context]` attribute.
///
/// # Returns
///
/// Returns the first variable annotated with the `[context]` attribute.
/// If no variable such variable is found `None` is returned.
///
/// # Panics
///
/// The function panics if one the given variable is the `self` argument.
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
            FnArg::Receiver(_receiver) => panic!("self type not supported"),
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
    use super::*;
    use crate::test_helpers;

    mod expand_attribute {
        use super::*;

        #[test]
        fn when_fn_does_not_contain_attr() {
            // Given
            let input_fn: ItemFn = parse_quote! {
                fn my_test() {
                    println!("Hello, World!")
                }
            };

            // When
            let expanded = expand_attribute(input_fn);

            // Then
            insta::assert_snapshot!(test_helpers::pretty(expanded))
        }

        #[test]
        fn when_fn_does_contain_attr() {
            // Given
            let input_fn: ItemFn = parse_quote! {
                #[attr1]
                #[attr2]
                fn my_test() {
                    println!("Hello, World!")
                }
            };

            // When
            let expanded = expand_attribute(input_fn);

            // Then
            insta::assert_snapshot!(test_helpers::pretty(expanded))
        }
    }

    mod context_binding {
        use super::*;

        #[test]
        fn when_fn_does_not_contain_arg() {
            // Given
            let input_fn: ItemFn = parse_quote! {
                fn my_test() {
                    println!("Hello, World!")
                }
            };

            // When
            let (expanded, binding) = context_binding(input_fn);

            // Then
            assert_eq!(binding.to_string(), "__rstest_insta__ctx");
            insta::assert_snapshot!(test_helpers::pretty(expanded));
        }

        #[test]
        fn when_fn_does_not_contain_ctx_arg() {
            // Given
            let input_fn: ItemFn = parse_quote! {
                fn my_test(a: usize, b: String) {
                    println!("Hello, World!")
                }
            };

            // When
            let (expanded, binding) = context_binding(input_fn);

            // Then
            assert_eq!(binding.to_string(), "__rstest_insta__ctx");
            insta::assert_snapshot!(test_helpers::pretty(expanded));
        }

        #[test]
        fn when_fn_contains_context_arg() {
            // Given
            let input_fn: ItemFn = parse_quote! {
                fn my_test(#[context] my_context: Context) {
                    println!("Hello, World!")
                }
            };

            // When
            let (expanded, binding) = context_binding(input_fn);

            // Then
            assert_eq!(binding.to_string(), "my_context");
            insta::assert_snapshot!(test_helpers::pretty(expanded));
        }

        #[test]
        #[should_panic]
        fn when_fn_contains_self_arg() {
            // Given
            let input_fn: ItemFn = parse_quote! {
                fn my_test(self, a: usize) {
                    println!("Hello, World!")
                }
            };

            // When
            let (_expanded, _binding) = context_binding(input_fn);

            // Then
            // `context_binding` should panic with self arguments.
        }

        mod expand {
            use super::*;

            #[test]
            fn when_fn_does_not_contain_arg() {
                // Given
                let input_fn: ItemFn = parse_quote! {
                    fn my_test() {
                        println!("Hello, World!")
                    }
                };

                // When
                let expanded = expand(input_fn);

                // Then
                insta::assert_snapshot!(test_helpers::pretty(expanded));
            }

            #[test]
            fn when_fn_contains_ctx_arg() {
                // Given
                let input_fn: ItemFn = parse_quote! {
                    fn my_test(a: usize, #[context] toto: Context) {
                        println!("Hello, World!")
                    }
                };

                // When
                let expanded = expand(input_fn);

                // Then
                insta::assert_snapshot!(test_helpers::pretty(expanded));
            }
        }
    }
}
