use markstone::*;
use std::ffi::CStr;
use std::os::raw::c_char;

#[test]
fn test_version_and_ast_schema_version() {
    let ptr = markstone_version();
    assert!(!ptr.is_null());
    // SAFETY: markstone_version() returns a static null-terminated C string.
    let c_str = unsafe { CStr::from_ptr(ptr) };
    assert_eq!(c_str.to_str().unwrap(), "0.1.0");

    let schema_ver = markstone_ast_schema_version();
    assert_eq!(schema_ver, 1);
}

#[test]
fn test_normal_conversions_html_and_ast() {
    let input = "# Hello ABI\n\nThis is **bold** text and [link](https://example.com).\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n@alice and #rust\n";
    let input_bytes = input.as_bytes();

    // 1. Generic HTML: mentions and tags are NOT linked
    let mut out: *mut c_char = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let status = markstone_to_html(
        input_bytes.as_ptr() as *const c_char,
        input_bytes.len(),
        &mut out,
        &mut out_len,
    );
    assert_eq!(status, MarkstoneStatus::Ok);
    assert!(!out.is_null());
    assert!(out_len > 0);

    // SAFETY: out points to out_len bytes plus terminating NUL
    unsafe {
        assert_eq!(*out.add(out_len), 0);
        let s = std::str::from_utf8(std::slice::from_raw_parts(out as *const u8, out_len)).unwrap();
        assert!(s.contains("<h1>Hello ABI</h1>"));
        assert!(s.contains("<strong>bold</strong>"));
        assert!(s.contains(r#"<a href="https://example.com" rel="nofollow noopener noreferrer">link</a>"#));
        assert!(s.contains("<table>"));
        assert!(s.contains("@alice and #rust"));
        assert!(!s.contains(r#"href="/u/alice""#));
        assert!(!s.contains(r#"href="/t/rust""#));
    }
    markstone_free(out, out_len);

    // 2. Actos HTML: mentions and tags ARE linked
    let mut out: *mut c_char = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let status = markstone_actos_to_html(
        input_bytes.as_ptr() as *const c_char,
        input_bytes.len(),
        &mut out,
        &mut out_len,
    );
    assert_eq!(status, MarkstoneStatus::Ok);
    assert!(!out.is_null());
    assert!(out_len > 0);

    // SAFETY: out points to out_len bytes plus terminating NUL
    unsafe {
        assert_eq!(*out.add(out_len), 0);
        let s = std::str::from_utf8(std::slice::from_raw_parts(out as *const u8, out_len)).unwrap();
        assert!(s.contains("<h1>Hello ABI</h1>"));
        assert!(s.contains(r#"<a href="/u/alice" class="mention">@alice</a>"#));
        assert!(s.contains(r#"<a href="/t/rust" class="tag">#rust</a>"#));
    }
    markstone_free(out, out_len);

    // 3. Generic AST: pure commonmark+gfm AST
    let mut out: *mut c_char = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let status = markstone_to_ast(
        input_bytes.as_ptr() as *const c_char,
        input_bytes.len(),
        &mut out,
        &mut out_len,
    );
    assert_eq!(status, MarkstoneStatus::Ok);
    assert!(!out.is_null());
    assert!(out_len > 0);

    // SAFETY: out points to out_len bytes plus terminating NUL
    unsafe {
        assert_eq!(*out.add(out_len), 0);
        let s = std::str::from_utf8(std::slice::from_raw_parts(out as *const u8, out_len)).unwrap();
        let val: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(val["schema"], 1);
        assert_eq!(val["root"]["type"], "document");
        assert!(!s.contains(r#""type":"mention""#));
        assert!(!s.contains(r#""type":"tag""#));
    }
    markstone_free(out, out_len);

    // 4. Actos AST: includes mention and tag nodes
    let mut out: *mut c_char = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let status = markstone_actos_to_ast(
        input_bytes.as_ptr() as *const c_char,
        input_bytes.len(),
        &mut out,
        &mut out_len,
    );
    assert_eq!(status, MarkstoneStatus::Ok);
    assert!(!out.is_null());
    assert!(out_len > 0);

    // SAFETY: out points to out_len bytes plus terminating NUL
    unsafe {
        assert_eq!(*out.add(out_len), 0);
        let s = std::str::from_utf8(std::slice::from_raw_parts(out as *const u8, out_len)).unwrap();
        let val: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(val["schema"], 1);
        assert_eq!(val["root"]["type"], "document");
        assert!(s.contains(r#""type":"mention""#));
        assert!(s.contains(r#""username":"alice""#));
        assert!(s.contains(r#""type":"tag""#));
        assert!(s.contains(r#""name":"rust""#));
    }
    markstone_free(out, out_len);
}

#[test]
fn test_empty_and_whitespace_inputs() {
    let fns = [
        markstone_to_html,
        markstone_to_ast,
        markstone_actos_to_html,
        markstone_actos_to_ast,
    ];

    for f in fns {
        // Case A: empty byte slice ""
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = usize::MAX;
        let status = f(b"".as_ptr() as *const c_char, 0, &mut out, &mut out_len);
        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        // SAFETY: out is valid for out_len + 1 bytes
        unsafe {
            assert_eq!(*out.add(out_len), 0);
        }
        markstone_free(out, out_len);

        // Case B: NULL pointer with length 0
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = usize::MAX;
        let status = f(std::ptr::null(), 0, &mut out, &mut out_len);
        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        // SAFETY: out is valid for out_len + 1 bytes
        unsafe {
            assert_eq!(*out.add(out_len), 0);
        }
        markstone_free(out, out_len);

        // Case C: Whitespace-only input
        let ws = "   \n\t  \r\n   ";
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = usize::MAX;
        let status = f(
            ws.as_ptr() as *const c_char,
            ws.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        // SAFETY: out is valid for out_len + 1 bytes
        unsafe {
            assert_eq!(*out.add(out_len), 0);
        }
        markstone_free(out, out_len);
    }
}

#[test]
fn test_embedded_nul_characters() {
    let input = b"Hello\0World @alice #rust\0end";
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
            input.as_ptr() as *const c_char,
            input.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        assert!(out_len > 0);
        // SAFETY: out is valid for out_len + 1 bytes, and terminating NUL is present
        unsafe {
            assert_eq!(*out.add(out_len), 0);
        }
        markstone_free(out, out_len);
    }
}

#[test]
fn test_invalid_utf8_input() {
    let invalid_sequences: &[&[u8]] = &[
        &[0xFF, 0xFE],
        &[0xC0, 0x80],
        b"valid prefix\x80invalid suffix",
        &[0xF0, 0x90, 0x80], // truncated 4-byte sequence
    ];

    let fns = [
        markstone_to_html,
        markstone_to_ast,
        markstone_actos_to_html,
        markstone_actos_to_ast,
    ];

    for f in fns {
        for &invalid in invalid_sequences {
            let sentinel_ptr = 0x12345678 as *mut c_char;
            let sentinel_len = 0xdeadbeef;
            let mut out = sentinel_ptr;
            let mut out_len = sentinel_len;

            let status = f(
                invalid.as_ptr() as *const c_char,
                invalid.len(),
                &mut out,
                &mut out_len,
            );

            assert_eq!(status, MarkstoneStatus::ErrInvalidUtf8);
            // On error: *out and *out_len are left strictly UNTOUCHED
            assert_eq!(out, sentinel_ptr);
            assert_eq!(out_len, sentinel_len);
        }
    }
}

#[test]
fn test_null_argument_handling() {
    let valid_input = b"# Hello";
    let fns = [
        markstone_to_html,
        markstone_to_ast,
        markstone_actos_to_html,
        markstone_actos_to_ast,
    ];

    for f in fns {
        // 1. out is NULL
        let mut out_len: usize = 42;
        let status = f(
            valid_input.as_ptr() as *const c_char,
            valid_input.len(),
            std::ptr::null_mut(),
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::ErrNullArgument);
        assert_eq!(out_len, 42); // untouched

        // 2. out_len is NULL
        let sentinel_ptr = 0x1234 as *mut c_char;
        let mut out = sentinel_ptr;
        let status = f(
            valid_input.as_ptr() as *const c_char,
            valid_input.len(),
            &mut out,
            std::ptr::null_mut(),
        );
        assert_eq!(status, MarkstoneStatus::ErrNullArgument);
        assert_eq!(out, sentinel_ptr); // untouched

        // 3. both out and out_len are NULL
        let status = f(
            valid_input.as_ptr() as *const c_char,
            valid_input.len(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert_eq!(status, MarkstoneStatus::ErrNullArgument);

        // 4. input is NULL but input_len > 0
        let mut out = sentinel_ptr;
        let mut out_len: usize = 999;
        let status = f(std::ptr::null(), 10, &mut out, &mut out_len);
        assert_eq!(status, MarkstoneStatus::ErrNullArgument);
        assert_eq!(out, sentinel_ptr); // untouched
        assert_eq!(out_len, 999); // untouched
    }
}

#[test]
fn test_error_conditions_untouched_pointers() {
    let fns = [
        markstone_to_html,
        markstone_to_ast,
        markstone_actos_to_html,
        markstone_actos_to_ast,
    ];

    // 1. Input too large (> 4 MiB)
    let large_input = vec![b'a'; markstone_core::MAX_INPUT_SIZE + 1];
    for f in fns {
        let sentinel_ptr = 0xbeef as *mut c_char;
        let sentinel_len = 0x1337;
        let mut out = sentinel_ptr;
        let mut out_len = sentinel_len;

        let status = f(
            large_input.as_ptr() as *const c_char,
            large_input.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::ErrInputTooLarge);
        assert_eq!(out, sentinel_ptr);
        assert_eq!(out_len, sentinel_len);
    }

    // 2. Depth exceeded (65 block quotes)
    let deep_input = "> ".repeat(64) + "exceeded depth limit";
    let deep_bytes = deep_input.as_bytes();
    for f in fns {
        let sentinel_ptr = 0xfeed as *mut c_char;
        let sentinel_len = 0x7777;
        let mut out = sentinel_ptr;
        let mut out_len = sentinel_len;

        let status = f(
            deep_bytes.as_ptr() as *const c_char,
            deep_bytes.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::ErrDepthExceeded);
        assert_eq!(out, sentinel_ptr);
        assert_eq!(out_len, sentinel_len);
    }
}

#[test]
fn test_markstone_free_variants() {
    // Freeing NULL pointer should be a no-op and never crash
    markstone_free(std::ptr::null_mut(), 0);
    markstone_free(std::ptr::null_mut(), 1024);
    markstone_free(std::ptr::null_mut(), usize::MAX);

    // Freeing buffers of varying sizes
    for size in [1, 10, 100, 1000, 10_000] {
        let input = "a".repeat(size);
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let status = markstone_to_html(
            input.as_ptr() as *const c_char,
            input.len(),
            &mut out,
            &mut out_len,
        );
        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        assert_eq!(out_len, format!("<p>{}</p>\n", input).len());
        markstone_free(out, out_len);
    }
}
