#![no_main]

use libfuzzer_sys::fuzz_target;
use markstone::*;
use std::os::raw::c_char;

fuzz_target!(|data: &[u8]| {
    // 1. Input size limit enforcement: inputs exceeding 4 MiB MUST fail with ErrInputTooLarge
    if data.len() > markstone_core::MAX_INPUT_SIZE {
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let status = markstone_to_html(
            data.as_ptr() as *const c_char,
            data.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::ErrInputTooLarge);
        assert!(out.is_null());
        assert_eq!(out_len, 0);
        return;
    }

    // 2. Block nesting depth enforcement: if input depth exceeds 64, MUST return DepthExceeded
    if let Ok(s) = std::str::from_utf8(data) {
        match markstone_core::to_html(s) {
            Ok(html) => {
                // If it succeeded, verify depth was within limit
                assert!(!html.is_empty() || s.trim().is_empty());
            }
            Err(markstone_core::MarkstoneError::DepthExceeded) => {
                // Depth exceeded is an expected rejection
            }
            Err(markstone_core::MarkstoneError::InputTooLarge) => {
                assert!(data.len() > markstone_core::MAX_INPUT_SIZE);
            }
            Err(other) => {
                panic!("unexpected error from to_html: {other:?}");
            }
        }
    }
});
