//! Harp core library
//!
//! Generates random names by combining adjectives and nouns.

mod error;
pub mod indexed;
pub mod words;

pub use error::{Error, Result};
pub use indexed::{
    generate_name_with_options_and_seed, generate_name_with_seed, IndexError, NameSpace,
    Permutation, MAX_COMPONENTS, MIN_COMPONENTS, SCHEME_VERSION,
};
pub use words::{ADJECTIVES, NOUNS};

/// Options for generating names. Every function taking them validates them
/// strictly and refuses, never clamps: see [`IndexError`].
#[derive(Debug, Clone)]
pub struct NameOptions {
    /// Number of components, 2-4 (N-1 adjectives and a noun). Default: 3
    pub components: u8,
    /// Maximum length per element. None means no limit. A limit that leaves
    /// no adjective or no noun is refused.
    pub max_element_length: Option<usize>,
    /// Separator between components. Default: "-". It must be non-empty and
    /// hold no ASCII lowercase letter, so a name splits back into its words
    /// one way only.
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
    generate_name_with_options(&NameOptions::default()).expect("the default options are valid")
}

/// Generates a random name with custom options, uniformly over the space
/// they describe. Refuses components outside 2..=4, a length filter that
/// empties a word list, and an ambiguous separator.
///
/// # Example
/// ```
/// use harp_core::{generate_name_with_options, IndexError, NameOptions};
///
/// let opts = NameOptions {
///     components: 2,
///     max_element_length: Some(5),
///     separator: "_".to_string(),
/// };
/// let name = generate_name_with_options(&opts)?;
/// assert!(name.contains('_'));
///
/// let five = NameOptions { components: 5, ..Default::default() };
/// assert_eq!(
///     generate_name_with_options(&five),
///     Err(IndexError::InvalidComponents(5))
/// );
/// # Ok::<(), IndexError>(())
/// ```
pub fn generate_name_with_options(opts: &NameOptions) -> std::result::Result<String, IndexError> {
    Ok(NameSpace::new(opts)?.random_name(&mut rand::rng()))
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
        assert!(
            words::NOUNS.contains(&parts[2]),
            "third word should be noun"
        );
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
        let name = generate_name_with_options(&opts).unwrap();
        assert_eq!(name.matches('-').count(), 1);
    }

    #[test]
    fn test_four_components() {
        let opts = NameOptions {
            components: 4,
            ..Default::default()
        };
        let name = generate_name_with_options(&opts).unwrap();
        assert_eq!(name.matches('-').count(), 3);
    }

    #[test]
    fn test_components_past_four_are_refused_not_clamped() {
        for components in [5, 16, 20] {
            let opts = NameOptions {
                components,
                ..Default::default()
            };
            assert_eq!(
                generate_name_with_options(&opts).unwrap_err(),
                IndexError::InvalidComponents(components)
            );
        }
    }

    #[test]
    fn test_max_element_length() {
        let opts = NameOptions {
            max_element_length: Some(4),
            ..Default::default()
        };
        let name = generate_name_with_options(&opts).unwrap();
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
        let name = generate_name_with_options(&opts).unwrap();
        assert!(name.contains('_'));
        assert!(!name.contains('-'));
    }
}
