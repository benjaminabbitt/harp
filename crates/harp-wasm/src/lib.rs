//! WebAssembly bindings for harp-core using wasm-bindgen

use wasm_bindgen::prelude::*;

/// Generate a random name from two adjectives and a noun.
#[wasm_bindgen]
pub fn generate_name() -> String {
    harp_core::generate_name()
}

/// Generate a random name with options.
#[wasm_bindgen]
pub fn generate_name_with_options(
    components: u8,
    max_element_length: Option<usize>,
    separator: &str,
) -> String {
    let opts = harp_core::NameOptions {
        components,
        max_element_length,
        separator: separator.to_string(),
    };
    harp_core::generate_name_with_options(&opts)
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
        let name = generate_name_with_options(2, Some(5), "_");
        assert!(name.contains('_'));
    }
}
