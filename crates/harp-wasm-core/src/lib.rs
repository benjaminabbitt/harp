//! WASI-compatible WASM module for harp.
//!
//! Exports simple C-style functions for use with wazero, wasmer, wasmtime, etc.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// Allocate memory. Returns pointer to allocated block.
#[no_mangle]
pub extern "C" fn harp_alloc(size: u32) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { alloc(layout) }
}

/// Free memory allocated by harp_alloc.
#[no_mangle]
pub extern "C" fn harp_free(ptr: *mut u8, size: u32) {
    if ptr.is_null() || size == 0 {
        return;
    }
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { dealloc(ptr, layout) }
}

/// Generate a name and write it to the provided buffer.
/// Returns the actual length written, or 0 on error.
/// If buffer is too small, returns required size (call again with larger buffer).
#[no_mangle]
pub extern "C" fn harp_generate_name(buf: *mut u8, buf_len: u32) -> u32 {
    let name = harp_core::generate_name();
    let bytes = name.as_bytes();
    let required_len = bytes.len() as u32;

    if buf.is_null() || buf_len < required_len {
        return required_len;
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
    }
    required_len
}

/// Generate a name with options and write to buffer.
/// separator must be a null-terminated C string.
/// Returns actual length written, or required size if buffer too small.
#[no_mangle]
pub extern "C" fn harp_generate_name_with_options(
    components: u8,
    max_element_length: u32,
    separator: *const u8,
    separator_len: u32,
    buf: *mut u8,
    buf_len: u32,
) -> u32 {
    let sep = if separator.is_null() || separator_len == 0 {
        "-".to_string()
    } else {
        let sep_slice = unsafe { std::slice::from_raw_parts(separator, separator_len as usize) };
        String::from_utf8_lossy(sep_slice).into_owned()
    };

    let opts = harp_core::NameOptions {
        components,
        max_element_length: if max_element_length == 0 {
            None
        } else {
            Some(max_element_length as usize)
        },
        separator: sep,
    };

    let name = harp_core::generate_name_with_options(&opts);
    let bytes = name.as_bytes();
    let required_len = bytes.len() as u32;

    if buf.is_null() || buf_len < required_len {
        return required_len;
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
    }
    required_len
}

/// Get version string. Returns pointer to static null-terminated string.
#[no_mangle]
pub extern "C" fn harp_version() -> *const u8 {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr()
}

/// Get version string length (excluding null terminator).
#[no_mangle]
pub extern "C" fn harp_version_len() -> u32 {
    env!("CARGO_PKG_VERSION").len() as u32
}
