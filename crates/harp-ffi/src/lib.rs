//! C FFI bindings for harp-core
//!
//! This crate exposes harp-core functionality via C ABI for use by Go and other
//! languages that can call C functions.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

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
/// - separator: Separator string: non-empty, valid UTF-8, no a-z
///
/// Returns NULL when the options are refused (components outside 2-4, a
/// length that leaves no word, or a null, invalid or ambiguous separator).
/// Otherwise the caller must free the returned string with `harp_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn harp_generate_name_with_options(
    components: c_uchar,
    max_element_length: c_uint,
    separator: *const c_char,
) -> *mut c_char {
    if separator.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(separator_str) = unsafe { CStr::from_ptr(separator) }.to_str() else {
        return std::ptr::null_mut();
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

    match harp_core::generate_name_with_options(&opts) {
        Ok(name) => CString::new(name).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_call_returns_null() {
        let dash = CString::new("-").unwrap();
        assert!(harp_generate_name_with_options(5, 0, dash.as_ptr()).is_null());
        assert!(harp_generate_name_with_options(2, 0, std::ptr::null()).is_null());
        let empty = CString::new("").unwrap();
        assert!(harp_generate_name_with_options(2, 0, empty.as_ptr()).is_null());
    }

    #[test]
    fn a_valid_call_returns_a_name() {
        let dash = CString::new("_").unwrap();
        let p = harp_generate_name_with_options(4, 0, dash.as_ptr());
        assert!(!p.is_null());
        let name = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string();
        unsafe { harp_free_string(p) };
        assert_eq!(name.split('_').count(), 4, "{name}");
    }
}
