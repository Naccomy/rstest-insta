use proc_macro_crate::FoundCrate;
use proc_macro2::Span;
use syn::{Ident, Path, parse_quote};

pub(crate) fn crate_name(crate_name: &str) -> Path {
    let found_crate =
        proc_macro_crate::crate_name(crate_name).expect("missing crate in `Cargo.toml`");
    resolve_crate_name(crate_name, found_crate)
}

fn resolve_crate_name(crate_name: &str, found_crate: FoundCrate) -> Path {
    match found_crate {
        FoundCrate::Itself => {
            let ident = Ident::new(crate_name, Span::call_site());
            parse_quote!(::#ident)
        }
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            parse_quote!(::#ident)
        }
    }
}

#[cfg(test)]
mod resolve_crate_name {
    use proc_macro_crate::FoundCrate;
    use proc_macro2::Span;
    use rstest::rstest;
    use syn::{Ident, Path, parse_quote};

    #[rstest]
    #[case("rstest")]
    #[case("rstest_insta")]
    #[case("rstest_insta_macros")]
    fn when_found_crate_is_itself(#[case] crate_name: &str) {
        // Given
        let found_crate = FoundCrate::Itself;

        // When
        let path = super::resolve_crate_name(crate_name, found_crate);

        // Then
        let ident = Ident::new(crate_name, Span::call_site());
        let expected: Path = parse_quote!(::#ident);
        assert_eq!(path, expected);
    }

    #[rstest]
    #[case("rstest")]
    #[case("rstest_insta")]
    #[case("rstest_insta_macros")]
    fn when_found_crate_is_name(#[case] crate_name: &str) {
        // Given
        let found_crate = FoundCrate::Name(crate_name.to_string());

        // When
        let path = super::resolve_crate_name("orig_crate_name", found_crate);
        let ident = Ident::new(crate_name, Span::call_site());
        let expected: Path = parse_quote!(::#ident);
        assert_eq!(path, expected);
    }
}
