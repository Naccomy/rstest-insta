# rstest-insta

## Issues

In order to use the `rstest-insta` macro we still need to import individually the following dependencies:
- `rstest = "0.25"`
- `insta = "1.43"`

This cannot be re-exported by some crate `rstest-insta-macros-helpers` since `rstest` needs to be part of the caller's
`Cargo.toml` file to be used. So instead of having `rstest-insta-macros-helpers` just export `insta` we drop the entirety of
this helper crate.
