//! C FFI bindings for harp-core
//!
//! This crate exposes harp-core functionality via C ABI for use by Go and other
//! languages that can call C functions.

use std::ffi::{c_char, c_uchar, c_uint, CStr, CString};

/// Generate a random name. Caller must free the returned string with `harp_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn harp_generate_name() -> *mut c_char {
    let name = harp_core::generate_name();
    CString::new(name).unwrap().into_raw()
}

/// Generate a random name with options.
///
/// - components: Number of components (2-4)
/// - max_element_length: Maximum length per element (0 means no limit)
/// - separator: Separator string (must be valid UTF-8)
///
/// Caller must free the returned string with `harp_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn harp_generate_name_with_options(
    components: c_uchar,
    max_element_length: c_uint,
    separator: *const c_char,
) -> *mut c_char {
    let separator_str = if separator.is_null() {
        "-"
    } else {
        unsafe { CStr::from_ptr(separator) }.to_str().unwrap_or("-")
    };

    let opts = harp_core::NameOptions {
        components,
        max_element_length: if max_element_length == 0 {
            None
        } else {
            Some(max_element_length as usize)
        },
        separator: separator_str.to_string(),
    };

    let name = harp_core::generate_name_with_options(&opts);
    CString::new(name).unwrap().into_raw()
}

/// Free a string allocated by this library.
/// # Safety
/// The pointer must have been allocated by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn harp_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}

/// Get the library version.
#[unsafe(no_mangle)]
pub extern "C" fn harp_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}
