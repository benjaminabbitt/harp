//! WebAssembly bindings for harp-core using wasm-bindgen

use wasm_bindgen::prelude::*;

/// Generate a random name from two adjectives and a noun.
#[wasm_bindgen]
pub fn generate_name() -> String {
    harp_core::generate_name()
}

/// Generate a random name with options. Throws when the options are refused
/// (components outside 2-4, a length that leaves no word, or an ambiguous
/// separator).
#[wasm_bindgen]
pub fn generate_name_with_options(
    components: u8,
    max_element_length: Option<usize>,
    separator: &str,
) -> Result<String, JsError> {
    with_options(components, max_element_length, separator)
        .map_err(|e| JsError::new(&e.to_string()))
}

fn with_options(
    components: u8,
    max_element_length: Option<usize>,
    separator: &str,
) -> Result<String, harp_core::IndexError> {
    harp_core::generate_name_with_options(&harp_core::NameOptions {
        components,
        max_element_length,
        separator: separator.to_string(),
    })
}

/// Get the library version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_generate_name_format() {
        let name = generate_name();
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 3);
    }

    #[wasm_bindgen_test]
    fn test_generate_name_with_options() {
        let name = with_options(2, Some(5), "_").unwrap();
        assert!(name.contains('_'));
    }

    // A plain test: the helper runs natively, where wasm_bindgen_test does not.
    #[test]
    fn test_generate_name_with_options_refuses_five_components() {
        assert_eq!(
            with_options(5, None, "-").unwrap_err(),
            harp_core::IndexError::InvalidComponents(5)
        );
    }
}
