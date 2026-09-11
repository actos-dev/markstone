//! `markstone-abi` provides the C ABI surface (cdylib, staticlib, rlib) for markstone.
//!
//! # Memory Allocation & Deallocation Strategy
//!
//! Markstone follows a strict and explicit memory ownership model across the C ABI boundary:
//!
//! 1. Output buffers (`out` and `out_len`) are allocated in Rust using the global allocator
//!    via `std::alloc::alloc` with exact layout `Layout::array::<u8>(out_len + 1)`.
//! 2. The returned buffer always includes a trailing NUL (`\0`) terminator at offset `out_len`.
//!    `out_len` represents the length in bytes *excluding* the NUL terminator.
//! 3. Ownership of the allocated buffer is transferred to the C caller.
//! 4. The caller MUST release the buffer by calling `markstone_free(ptr, len)`, where `ptr`
//!    is the pointer handed back in `*out` and `len` is the exact value returned in `*out_len`.
//! 5. `markstone_free` reconstructs the matching `Layout::array::<u8>(len + 1)` and deallocates
//!    via `std::alloc::dealloc`.
//! 6. If `ptr` is NULL, `markstone_free` is a no-op.
//! 7. On any error, `*out` and `*out_len` are left strictly untouched, and no memory is allocated.
//! 8. All public FFI boundaries are guarded with `std::panic::catch_unwind` to prevent panics
//!    from crossing the ABI boundary.

use std::alloc::Layout;
use std::os::raw::c_char;

pub use markstone_actos as actos;
pub use markstone_core as core;

/// Status codes returned by markstone C ABI functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkstoneStatus {
    Ok = 0,
    ErrNullArgument = 1,
    ErrInvalidUtf8 = 2,
    ErrInputTooLarge = 3,
    ErrDepthExceeded = 4,
    ErrInternal = 5,
}

impl From<markstone_core::error::MarkstoneError> for MarkstoneStatus {
    fn from(err: markstone_core::error::MarkstoneError) -> Self {
        match err {
            markstone_core::error::MarkstoneError::InputTooLarge => {
                MarkstoneStatus::ErrInputTooLarge
            }
            markstone_core::error::MarkstoneError::DepthExceeded => {
                MarkstoneStatus::ErrDepthExceeded
            }
            markstone_core::error::MarkstoneError::InvalidUtf8 => MarkstoneStatus::ErrInvalidUtf8,
            markstone_core::error::MarkstoneError::Internal(_) => MarkstoneStatus::ErrInternal,
        }
    }
}

/// Static semver string, NUL-terminated, never freed.
static VERSION_CSTR: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();

/// Static semver string, never freed.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_version() -> *const c_char {
    let res = std::panic::catch_unwind(|| VERSION_CSTR.as_ptr() as *const c_char);
    res.unwrap_or(std::ptr::null())
}

/// AST JSON schema version. Increments when the schema breaks.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_ast_schema_version() -> u32 {
    let res = std::panic::catch_unwind(|| markstone_core::AST_SCHEMA_VERSION);
    res.unwrap_or(0)
}

/// Internal helper to release an allocated buffer with layout matching `len + 1`.
fn free_internal(ptr: *mut c_char, len: usize) {
    if ptr.is_null() {
        return;
    }

    let total_len = match len.checked_add(1) {
        Some(l) => l,
        None => return,
    };

    let layout = match Layout::array::<u8>(total_len) {
        Ok(l) => l,
        Err(_) => return,
    };

    // SAFETY:
    // 1. `ptr` is non-null, checked above.
    // 2. Caller guarantees `ptr` was allocated by one of the `markstone_to_*` or
    //    `markstone_actos_to_*` functions, and `len` exactly equals the `out_len`
    //    returned by that call.
    // 3. The allocation was performed with `Layout::array::<u8>(len + 1)` in `convert_internal`.
    //    Thus, `layout` precisely matches the size and alignment of the original allocation.
    unsafe {
        std::alloc::dealloc(ptr as *mut u8, layout);
    }
}

/// Releases a buffer returned by markstone_to_*. Does nothing if ptr is NULL.
/// len must be exactly the out_len that call handed back.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_free(ptr: *mut c_char, len: usize) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        free_internal(ptr, len);
    }));
}

/// Helper function implementing argument checks, UTF-8 validation, conversion execution,
/// buffer allocation, and output pointer setting.
fn convert_internal(
    input: *const c_char,
    input_len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
    transform: fn(&str) -> Result<String, markstone_core::error::MarkstoneError>,
) -> MarkstoneStatus {
    // 1. NULL argument checks
    if out.is_null() || out_len.is_null() {
        return MarkstoneStatus::ErrNullArgument;
    }
    if input.is_null() && input_len > 0 {
        return MarkstoneStatus::ErrNullArgument;
    }

    // 2. Input size limit check (4 MiB)
    if input_len > markstone_core::MAX_INPUT_SIZE {
        return MarkstoneStatus::ErrInputTooLarge;
    }

    // 3. Construct input byte slice safely
    // SAFETY: When `input_len == 0`, we use an empty slice `&[]` without dereferencing `input`.
    // When `input_len > 0`, `input` is verified non-null. The caller guarantees that `input`
    // points to at least `input_len` valid, initialized bytes and memory is not mutated concurrently.
    let bytes = if input_len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(input as *const u8, input_len) }
    };

    // 4. Validate UTF-8 encoding
    let input_str = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return MarkstoneStatus::ErrInvalidUtf8,
    };

    // 5. Execute conversion transformation
    let rendered = match transform(input_str) {
        Ok(s) => s,
        Err(err) => return MarkstoneStatus::from(err),
    };

    let len = rendered.len();

    // 6. Allocate output buffer: `len` content bytes + 1 terminating NUL byte
    let total_len = match len.checked_add(1) {
        Some(l) => l,
        None => return MarkstoneStatus::ErrInternal,
    };

    let layout = match Layout::array::<u8>(total_len) {
        Ok(l) => l,
        Err(_) => return MarkstoneStatus::ErrInternal,
    };

    // SAFETY: total_len >= 1 (since len >= 0 and checked_add(1)), ensuring non-zero allocation size.
    let ptr = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() {
        return MarkstoneStatus::ErrInternal;
    }

    // SAFETY:
    // 1. `ptr` points to freshly allocated heap memory of `total_len = len + 1` bytes.
    // 2. `rendered.as_ptr()` points to `len` valid, initialized UTF-8 bytes.
    // 3. Source and destination buffers do not overlap because `ptr` was just allocated.
    // 4. `ptr.add(len)` is within bounds of the allocated buffer (offset `len` of `len + 1`),
    //    where we write the terminating NUL byte `0`.
    unsafe {
        std::ptr::copy_nonoverlapping(rendered.as_ptr(), ptr, len);
        *ptr.add(len) = 0;
    }

    // 7. Write output pointers
    // SAFETY:
    // `out` and `out_len` were verified to be non-null at the start of this function.
    // The caller guarantees they point to valid, properly aligned, writable memory.
    // We only write to `*out` and `*out_len` upon successful conversion and allocation.
    unsafe {
        *out = ptr as *mut c_char;
        *out_len = len;
    }

    MarkstoneStatus::Ok
}

/// Generic markdown to HTML.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_to_html(
    input: *const c_char,
    input_len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> MarkstoneStatus {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        convert_internal(input, input_len, out, out_len, markstone_core::to_html)
    })) {
        Ok(status) => status,
        Err(_) => MarkstoneStatus::ErrInternal,
    }
}

/// Generic markdown to AST JSON.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_to_ast(
    input: *const c_char,
    input_len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> MarkstoneStatus {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        convert_internal(input, input_len, out, out_len, markstone_core::to_ast)
    })) {
        Ok(status) => status,
        Err(_) => MarkstoneStatus::ErrInternal,
    }
}

/// Actos markdown to HTML (generic pipeline plus mention/tag pass).
#[unsafe(no_mangle)]
pub extern "C" fn markstone_actos_to_html(
    input: *const c_char,
    input_len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> MarkstoneStatus {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        convert_internal(input, input_len, out, out_len, markstone_actos::to_html)
    })) {
        Ok(status) => status,
        Err(_) => MarkstoneStatus::ErrInternal,
    }
}

/// Actos markdown to AST JSON (generic pipeline plus mention/tag pass).
#[unsafe(no_mangle)]
pub extern "C" fn markstone_actos_to_ast(
    input: *const c_char,
    input_len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> MarkstoneStatus {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        convert_internal(input, input_len, out, out_len, markstone_actos::to_ast)
    })) {
        Ok(status) => status,
        Err(_) => MarkstoneStatus::ErrInternal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_version() {
        let ptr = markstone_version();
        assert!(!ptr.is_null());
        // SAFETY: ptr points to a static C string literal
        let c_str = unsafe { CStr::from_ptr(ptr) };
        assert_eq!(c_str.to_str().unwrap(), "0.1.0");
    }

    #[test]
    fn test_ast_schema_version() {
        assert_eq!(markstone_ast_schema_version(), 1);
    }

    #[test]
    fn test_dependencies() {
        assert_eq!(core::VERSION, "0.1.0");
        assert_eq!(actos::VERSION, "0.1.0");
    }
}
