//! Re-exports `insta` and `rstest` crates.
//! These are re-exported from a helper crate because `proc-macro` crates cannot re-export items
//! that aren't marked with procedural macro attributes.

pub use insta;
pub use rstest;
