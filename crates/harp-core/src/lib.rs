//! Harp core library
//!
//! Generates random names by combining adjectives and nouns.

mod error;
pub mod words;

pub use error::{Error, Result};
pub use words::{ADJECTIVES, NOUNS};

use rand::seq::IndexedRandom;

/// Options for generating names.
#[derive(Debug, Clone)]
pub struct NameOptions {
    /// Number of components (2-16). Default: 3
    pub components: u8,
    /// Maximum length per element. None means no limit.
    pub max_element_length: Option<usize>,
    /// Separator between components. Default: "-"
    pub separator: String,
}

impl Default for NameOptions {
    fn default() -> Self {
        Self {
            components: 3,
            max_element_length: None,
            separator: "-".to_string(),
        }
    }
}

/// Generates a random name from two adjectives and a noun, separated by dashes.
///
/// # Example
/// ```
/// let name = harp_core::generate_name();
/// assert!(name.contains('-'));
/// assert_eq!(name.matches('-').count(), 2);
/// ```
pub fn generate_name() -> String {
    generate_name_with_options(&NameOptions::default())
}

/// Generates a random name with custom options.
///
/// # Example
/// ```
/// use harp_core::{generate_name_with_options, NameOptions};
///
/// let opts = NameOptions {
///     components: 2,
///     max_element_length: Some(5),
///     separator: "_".to_string(),
/// };
/// let name = generate_name_with_options(&opts);
/// assert!(name.contains('_'));
/// ```
pub fn generate_name_with_options(opts: &NameOptions) -> String {
    let mut rng = rand::rng();

    let adjectives: Vec<&str> = match opts.max_element_length {
        Some(len) => ADJECTIVES.iter().filter(|w| w.len() <= len).copied().collect(),
        None => ADJECTIVES.iter().copied().collect(),
    };
    let nouns: Vec<&str> = match opts.max_element_length {
        Some(len) => NOUNS.iter().filter(|w| w.len() <= len).copied().collect(),
        None => NOUNS.iter().copied().collect(),
    };

    let components = opts.components.clamp(2, 16);
    let mut parts: Vec<&str> = Vec::with_capacity(components as usize);

    // Add adjectives (components - 1)
    for _ in 0..(components - 1) {
        if let Some(adj) = adjectives.choose(&mut rng) {
            parts.push(adj);
        }
    }

    // Add noun
    if let Some(noun) = nouns.choose(&mut rng) {
        parts.push(noun);
    }

    parts.join(&opts.separator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_name_format() {
        let name = generate_name();
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 3, "name should have 3 parts: {name}");
    }

    #[test]
    fn test_generate_name_uses_valid_words() {
        let name = generate_name();
        let parts: Vec<&str> = name.split('-').collect();

        assert!(
            words::ADJECTIVES.contains(&parts[0]),
            "first word should be adjective"
        );
        assert!(
            words::ADJECTIVES.contains(&parts[1]),
            "second word should be adjective"
        );
        assert!(words::NOUNS.contains(&parts[2]), "third word should be noun");
    }

    #[test]
    fn test_generate_name_produces_different_results() {
        let names: Vec<String> = (0..10).map(|_| generate_name()).collect();
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert!(unique.len() > 1, "should produce varied names");
    }

    #[test]
    fn test_two_components() {
        let opts = NameOptions {
            components: 2,
            ..Default::default()
        };
        let name = generate_name_with_options(&opts);
        assert_eq!(name.matches('-').count(), 1);
    }

    #[test]
    fn test_four_components() {
        let opts = NameOptions {
            components: 4,
            ..Default::default()
        };
        let name = generate_name_with_options(&opts);
        assert_eq!(name.matches('-').count(), 3);
    }

    #[test]
    fn test_sixteen_components() {
        let opts = NameOptions {
            components: 16,
            ..Default::default()
        };
        let name = generate_name_with_options(&opts);
        assert_eq!(name.matches('-').count(), 15);
    }

    #[test]
    fn test_components_clamped_to_max() {
        let opts = NameOptions {
            components: 20,
            ..Default::default()
        };
        let name = generate_name_with_options(&opts);
        assert_eq!(name.matches('-').count(), 15); // clamped to 16
    }

    #[test]
    fn test_max_element_length() {
        let opts = NameOptions {
            max_element_length: Some(4),
            ..Default::default()
        };
        let name = generate_name_with_options(&opts);
        for part in name.split('-') {
            assert!(part.len() <= 4, "part too long: {part}");
        }
    }

    #[test]
    fn test_custom_separator() {
        let opts = NameOptions {
            separator: "_".to_string(),
            ..Default::default()
        };
        let name = generate_name_with_options(&opts);
        assert!(name.contains('_'));
        assert!(!name.contains('-'));
    }
}
