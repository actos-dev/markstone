#![no_main]

use libfuzzer_sys::fuzz_target;
use markstone::*;
use std::os::raw::c_char;

fuzz_target!(|data: &[u8]| {
    // 1. Rust direct API: ensure neither generic nor actos pipelines panic
    let _ = markstone_core::to_html_bytes(data);
    let _ = markstone_core::to_ast_bytes(data);
    let _ = markstone_actos::to_html_bytes(data);
    let _ = markstone_actos::to_ast_bytes(data);

    // 2. C ABI functions: ensure no panic across FFI boundary
    let fns = [
        markstone_to_html,
        markstone_to_ast,
        markstone_actos_to_html,
        markstone_actos_to_ast,
    ];

    for f in fns {
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let status = f(
            data.as_ptr() as *const c_char,
            data.len(),
            &mut out,
            &mut out_len,
        );

        if status == MarkstoneStatus::Ok {
            assert!(!out.is_null());
            // Verify trailing NUL byte
            unsafe {
                assert_eq!(*out.add(out_len), 0);
            }
            markstone_free(out, out_len);
        } else {
            // On error, out and out_len must remain null / untouched
            assert!(out.is_null());
            assert_eq!(out_len, 0);
        }
    }
});
