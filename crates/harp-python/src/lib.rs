//! Python bindings for harp-core using PyO3

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyInt};

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
    harp_core::generate_name_with_options(&opts).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// A seed as harp-core takes it: an int in 0..2**64, refused rather than
/// wrapped or truncated. A bool is refused although Python counts it an int.
fn seed_from(seed: &Bound<'_, PyAny>) -> PyResult<u64> {
    if seed.is_instance_of::<PyBool>() || !seed.is_instance_of::<PyInt>() {
        return Err(PyTypeError::new_err(format!(
            "seed must be an int, got {}",
            seed.get_type().name()?
        )));
    }
    seed.extract::<u64>().map_err(|_| {
        PyValueError::new_err(format!("seed must be an integer in 0..2**64, got {seed}"))
    })
}

/// A deterministic name for `seed` with the default options: position 0 of
/// the seed's scheme-v1 permutation, so it equals the first name a store
/// with this seed issues.
///
/// Raises:
///     TypeError: the seed is not an int (a bool is refused).
///     ValueError: the seed is outside 0..2**64.
#[pyfunction]
#[pyo3(signature = (seed))]
fn generate_name_with_seed(seed: &Bound<'_, PyAny>) -> PyResult<String> {
    Ok(harp_core::generate_name_with_seed(seed_from(seed)?))
}

/// A deterministic name for `seed` with options: position 0 of the seed's
/// scheme-v1 permutation of that space.
///
/// Args:
///     seed: An int in 0..2**64.
///     components, max_element_length, separator: As for
///         generate_name_with_options.
///
/// Raises:
///     TypeError: the seed is not an int (a bool is refused).
///     ValueError: the seed is outside 0..2**64, or the options are refused
///         as generate_name_with_options refuses them.
#[pyfunction]
#[pyo3(signature = (seed, components=3, max_element_length=None, separator="-"))]
fn generate_name_with_options_and_seed(
    seed: &Bound<'_, PyAny>,
    components: u8,
    max_element_length: Option<usize>,
    separator: &str,
) -> PyResult<String> {
    let seed = seed_from(seed)?;
    let opts = harp_core::NameOptions {
        components,
        max_element_length,
        separator: separator.to_string(),
    };
    harp_core::generate_name_with_options_and_seed(&opts, seed)
        .map_err(|e| PyValueError::new_err(e.to_string()))
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
    m.add_function(wrap_pyfunction!(generate_name_with_seed, m)?)?;
    m.add_function(wrap_pyfunction!(generate_name_with_options_and_seed, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("SCHEME_VERSION", harp_core::indexed::SCHEME_VERSION)?;
    Ok(())
}
