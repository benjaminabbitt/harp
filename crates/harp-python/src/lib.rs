//! Python bindings for harp-core using PyO3

use pyo3::prelude::*;

/// Generate a random name from two adjectives and a noun.
#[pyfunction]
fn generate_name() -> String {
    harp_core::generate_name()
}

/// Generate a random name with options.
///
/// Args:
///     components: Number of components (2-4). Default: 3
///     max_element_length: Maximum length per element. None means no limit.
///     separator: Separator between components, non-empty and without a-z.
///         Default: "-"
///
/// Raises:
///     ValueError: the options are refused (components outside 2-4, a
///         length that leaves no word, or an ambiguous separator).
#[pyfunction]
#[pyo3(signature = (components=3, max_element_length=None, separator="-"))]
fn generate_name_with_options(
    components: u8,
    max_element_length: Option<usize>,
    separator: &str,
) -> PyResult<String> {
    let opts = harp_core::NameOptions {
        components,
        max_element_length,
        separator: separator.to_string(),
    };
    harp_core::generate_name_with_options(&opts)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Get the library version.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Harp Python module
#[pymodule]
fn harp(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_name, m)?)?;
    m.add_function(wrap_pyfunction!(generate_name_with_options, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
