#![no_main]

use libfuzzer_sys::fuzz_target;
use markstone::*;
use std::os::raw::c_char;

fuzz_target!(|data: &[u8]| {
    let mut out: *mut c_char = std::ptr::null_mut();
    let mut out_len: usize = 0;

    // 1. Check markstone_to_html output validity
    let status = markstone_to_html(
        data.as_ptr() as *const c_char,
        data.len(),
        &mut out,
        &mut out_len,
    );
    if status == MarkstoneStatus::Ok {
        assert!(!out.is_null());
        // SAFETY: verify valid UTF-8 and trailing NUL
        unsafe {
            assert_eq!(*out.add(out_len), 0);
            let bytes = std::slice::from_raw_parts(out as *const u8, out_len);
            assert!(std::str::from_utf8(bytes).is_ok());
        }
        markstone_free(out, out_len);
    }

    // 2. Check markstone_to_ast output validity
    out = std::ptr::null_mut();
    out_len = 0;
    let status = markstone_to_ast(
        data.as_ptr() as *const c_char,
        data.len(),
        &mut out,
        &mut out_len,
    );
    if status == MarkstoneStatus::Ok {
        assert!(!out.is_null());
        unsafe {
            assert_eq!(*out.add(out_len), 0);
            let bytes = std::slice::from_raw_parts(out as *const u8, out_len);
            let s = std::str::from_utf8(bytes).expect("AST JSON must be valid UTF-8");
            let json: serde_json::Value =
                serde_json::from_str(s).expect("AST output must be valid JSON");
            assert_eq!(json["schema"], 1);
            assert_eq!(json["root"]["type"], "document");
        }
        markstone_free(out, out_len);
    }

    // 3. Check markstone_actos_to_html output validity
    out = std::ptr::null_mut();
    out_len = 0;
    let status = markstone_actos_to_html(
        data.as_ptr() as *const c_char,
        data.len(),
        &mut out,
        &mut out_len,
    );
    if status == MarkstoneStatus::Ok {
        assert!(!out.is_null());
        unsafe {
            assert_eq!(*out.add(out_len), 0);
            let bytes = std::slice::from_raw_parts(out as *const u8, out_len);
            assert!(std::str::from_utf8(bytes).is_ok());
        }
        markstone_free(out, out_len);
    }

    // 4. Check markstone_actos_to_ast output validity
    out = std::ptr::null_mut();
    out_len = 0;
    let status = markstone_actos_to_ast(
        data.as_ptr() as *const c_char,
        data.len(),
        &mut out,
        &mut out_len,
    );
    if status == MarkstoneStatus::Ok {
        assert!(!out.is_null());
        unsafe {
            assert_eq!(*out.add(out_len), 0);
            let bytes = std::slice::from_raw_parts(out as *const u8, out_len);
            let s = std::str::from_utf8(bytes).expect("Actos AST JSON must be valid UTF-8");
            let json: serde_json::Value =
                serde_json::from_str(s).expect("Actos AST output must be valid JSON");
            assert_eq!(json["schema"], 1);
            assert_eq!(json["root"]["type"], "document");
        }
        markstone_free(out, out_len);
    }
});
