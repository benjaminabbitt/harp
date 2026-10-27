//! WASI-compatible WASM module for harp.
//!
//! Exports simple C-style functions for use with wazero, wasmer, wasmtime, etc.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

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

/// Set in `harp_generate_name_with_options`'s result when the options are
/// refused: the low bits are then the length of the error message.
pub const HARP_ERROR_FLAG: u32 = 1 << 31;

/// Generate a name with options and write to buffer.
/// separator is `separator_len` UTF-8 bytes: non-empty, without a-z.
/// Returns actual length written, or required size if buffer too small.
/// When the options are refused (components outside 2-4, a length that
/// leaves no word, or an empty, invalid or ambiguous separator), the result
/// has `HARP_ERROR_FLAG` set and the buffer holds the error message instead,
/// under the same length rule.
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
        Ok(String::new())
    } else {
        let sep_slice = unsafe { std::slice::from_raw_parts(separator, separator_len as usize) };
        String::from_utf8(sep_slice.to_vec())
    };
    let Ok(sep) = sep else {
        return write_out("separator is not valid UTF-8", buf, buf_len) | HARP_ERROR_FLAG;
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

    match harp_core::generate_name_with_options(&opts) {
        Ok(name) => write_out(&name, buf, buf_len),
        Err(e) => write_out(&e.to_string(), buf, buf_len) | HARP_ERROR_FLAG,
    }
}

/// Copy `text` into `buf` when it fits; return its length either way.
fn write_out(text: &str, buf: *mut u8, buf_len: u32) -> u32 {
    let bytes = text.as_bytes();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn call(components: u8, max: u32, sep: &[u8]) -> (u32, String) {
        let mut buf = vec![0u8; 256];
        let n = harp_generate_name_with_options(
            components,
            max,
            sep.as_ptr(),
            sep.len() as u32,
            buf.as_mut_ptr(),
            buf.len() as u32,
        );
        let len = (n & !HARP_ERROR_FLAG) as usize;
        (n, String::from_utf8(buf[..len].to_vec()).unwrap())
    }

    #[test]
    fn a_valid_call_writes_the_name_without_the_flag() {
        let (n, name) = call(2, 0, b"_");
        assert_eq!(n & HARP_ERROR_FLAG, 0);
        assert_eq!(name.split('_').count(), 2, "{name}");
    }

    #[test]
    fn a_refusal_sets_the_flag_and_writes_the_reason() {
        let (n, msg) = call(5, 0, b"-");
        assert_ne!(n & HARP_ERROR_FLAG, 0);
        assert_eq!(msg, "components must be 2..=4, got 5");
        let (n, msg) = call(2, 0, b"");
        assert_ne!(n & HARP_ERROR_FLAG, 0);
        assert_eq!(msg, "separator \"\" cannot be split unambiguously");
        let (n, msg) = call(2, 0, &[0xff]);
        assert_ne!(n & HARP_ERROR_FLAG, 0);
        assert_eq!(msg, "separator is not valid UTF-8");
    }

    #[test]
    fn a_short_buffer_gets_the_required_length() {
        let sep = b"-";
        let n = harp_generate_name_with_options(5, 0, sep.as_ptr(), 1, ptr::null_mut(), 0);
        assert_eq!(
            n,
            "components must be 2..=4, got 5".len() as u32 | HARP_ERROR_FLAG
        );
    }
}
