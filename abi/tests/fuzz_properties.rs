use markstone::*;
use std::os::raw::c_char;

// Simple deterministic PRNG for reproducible test vectors
struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0xdeadbeefc0ffeeba } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next_u64();
            let bytes = val.to_ne_bytes();
            for (i, b) in chunk.iter_mut().enumerate() {
                *b = bytes[i];
            }
        }
    }
}

#[test]
fn test_fuzz_property_no_panic_random_bytes() {
    let mut rng = XorShift64::new(123456789);

    for len in [0, 1, 2, 5, 16, 64, 256, 1024, 4096] {
        for _ in 0..20 {
            let mut buf = vec![0u8; len];
            rng.fill_bytes(&mut buf);

            // 1. Rust direct API: ensure no panic
            let _ = markstone_core::to_html_bytes(&buf);
            let _ = markstone_core::to_ast_bytes(&buf);
            let _ = markstone_actos::to_html_bytes(&buf);
            let _ = markstone_actos::to_ast_bytes(&buf);

            // 2. C ABI functions: ensure no panic and proper error handling
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
                    buf.as_ptr() as *const c_char,
                    buf.len(),
                    &mut out,
                    &mut out_len,
                );

                match status {
                    MarkstoneStatus::Ok => {
                        assert!(!out.is_null());
                        // SAFETY: verify terminating NUL
                        unsafe {
                            assert_eq!(*out.add(out_len), 0);
                        }
                        markstone_free(out, out_len);
                    }
                    MarkstoneStatus::ErrInvalidUtf8
                    | MarkstoneStatus::ErrInputTooLarge
                    | MarkstoneStatus::ErrDepthExceeded
                    | MarkstoneStatus::ErrInternal => {
                        assert!(out.is_null());
                        assert_eq!(out_len, 0);
                    }
                    MarkstoneStatus::ErrNullArgument => {
                        panic!("unexpected null argument error");
                    }
                }
            }
        }
    }
}

#[test]
fn test_fuzz_property_limits_hold() {
    // 1. Input size limit holds: inputs > 4 MiB MUST be rejected
    let too_large = vec![b'a'; markstone_core::MAX_INPUT_SIZE + 1];
    let mut out: *mut c_char = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let status = markstone_to_html(
        too_large.as_ptr() as *const c_char,
        too_large.len(),
        &mut out,
        &mut out_len,
    );
    assert_eq!(status, MarkstoneStatus::ErrInputTooLarge);
    assert!(out.is_null());

    // 2. Depth limit holds: nesting > 64 MUST be rejected with ErrDepthExceeded
    for depth in [65, 70, 100, 500] {
        let deep = "> ".repeat(depth) + "text";
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let status = markstone_to_html(
            deep.as_ptr() as *const c_char,
            deep.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::ErrDepthExceeded);
        assert!(out.is_null());
    }
}

#[test]
fn test_fuzz_property_valid_utf8_output() {
    let snippets = [
        "Plain text snippet",
        "## Heading with [url](https://example.com) and `code`",
        "@valid_user and #valid-tag alongside @Invalid! and #",
        "```python\ndef foo(): return 42\n```",
        "| A | B |\n|---|---|\n| 1 | 2 |",
        "- [x] task 1\n- [ ] task 2",
        "Text with \u{200B} invisible \u{202E} bidi characters",
        "Hello \0 embedded nul byte",
    ];

    for snippet in snippets {
        let bytes = snippet.as_bytes();

        // Check HTML output
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let status = markstone_to_html(
            bytes.as_ptr() as *const c_char,
            bytes.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        // SAFETY: verify valid UTF-8
        unsafe {
            assert_eq!(*out.add(out_len), 0);
            let s = std::slice::from_raw_parts(out as *const u8, out_len);
            assert!(std::str::from_utf8(s).is_ok());
        }
        markstone_free(out, out_len);

        // Check AST output
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let status = markstone_actos_to_ast(
            bytes.as_ptr() as *const c_char,
            bytes.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        // SAFETY: verify valid UTF-8 and valid JSON
        unsafe {
            assert_eq!(*out.add(out_len), 0);
            let s = std::slice::from_raw_parts(out as *const u8, out_len);
            let json_str = std::str::from_utf8(s).expect("AST output must be valid UTF-8");
            let parsed: serde_json::Value =
                serde_json::from_str(json_str).expect("AST output must be valid JSON");
            assert_eq!(parsed["schema"], 1);
            assert_eq!(parsed["root"]["type"], "document");
        }
        markstone_free(out, out_len);
    }
}
