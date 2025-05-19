mod codegen;
mod utils;

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn rstest_insta(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let expanded = codegen::expand(input_fn);
    TokenStream::from(expanded)
}

#[cfg(test)]
pub mod test_helpers {
    pub fn pretty(stream: impl quote::ToTokens) -> String {
        let content = stream.into_token_stream().to_string();
        let file = syn::parse_file(&content).expect("parse file");
        prettyplease::unparse(&file)
    }
}
