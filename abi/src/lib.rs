//! `markstone-abi` provides the C ABI surface (cdylib, staticlib) for markstone.

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

/// Static semver string, never freed.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_version() -> *const c_char {
    c"0.1.0".as_ptr()
}

/// AST JSON schema version. Increments when the schema breaks.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_ast_schema_version() -> u32 {
    core::AST_SCHEMA_VERSION
}

/// Releases a buffer returned by markstone_to_*. Does nothing if ptr is NULL.
/// len must be exactly the out_len that call handed back.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_free(_ptr: *mut c_char, _len: usize) {
    // Phase 0 stub: full memory management implemented in Phase 4.
}

/// Generic markdown to HTML.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_to_html(
    _input: *const c_char,
    _input_len: usize,
    _out: *mut *mut c_char,
    _out_len: *mut usize,
) -> MarkstoneStatus {
    // Phase 0 stub: full implementation in Phase 4.
    MarkstoneStatus::ErrInternal
}

/// Generic markdown to AST JSON.
#[unsafe(no_mangle)]
pub extern "C" fn markstone_to_ast(
    _input: *const c_char,
    _input_len: usize,
    _out: *mut *mut c_char,
    _out_len: *mut usize,
) -> MarkstoneStatus {
    // Phase 0 stub: full implementation in Phase 4.
    MarkstoneStatus::ErrInternal
}

/// Actos markdown to HTML (generic pipeline plus mention/tag pass).
#[unsafe(no_mangle)]
pub extern "C" fn markstone_actos_to_html(
    _input: *const c_char,
    _input_len: usize,
    _out: *mut *mut c_char,
    _out_len: *mut usize,
) -> MarkstoneStatus {
    // Phase 0 stub: full implementation in Phase 4.
    MarkstoneStatus::ErrInternal
}

/// Actos markdown to AST JSON (generic pipeline plus mention/tag pass).
#[unsafe(no_mangle)]
pub extern "C" fn markstone_actos_to_ast(
    _input: *const c_char,
    _input_len: usize,
    _out: *mut *mut c_char,
    _out_len: *mut usize,
) -> MarkstoneStatus {
    // Phase 0 stub: full implementation in Phase 4.
    MarkstoneStatus::ErrInternal
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
