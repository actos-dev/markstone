use markstone::*;
use std::os::raw::c_char;

#[test]
fn test_memory_stability_10000_conversions() {
    const TOTAL_ITERATIONS: usize = 12_000;

    let inputs = [
        "Simple paragraph with text.",
        "# Heading 1\n\nSome paragraph with **bold** and *italic* and `code`.",
        "Check out @alice and #rust-lang in the feed.",
        "| Col1 | Col2 |\n|------|------|\n| Val1 | Val2 |\n| Val3 | Val4 |\n",
        "```json\n{\"test\": true}\n```\n",
        "",
    ];

    for i in 0..TOTAL_ITERATIONS {
        let input = inputs[i % inputs.len()];
        let bytes = input.as_bytes();

        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;

        let status = match i % 4 {
            0 => markstone_to_html(
                bytes.as_ptr() as *const c_char,
                bytes.len(),
                &mut out,
                &mut out_len,
            ),
            1 => markstone_to_ast(
                bytes.as_ptr() as *const c_char,
                bytes.len(),
                &mut out,
                &mut out_len,
            ),
            2 => markstone_actos_to_html(
                bytes.as_ptr() as *const c_char,
                bytes.len(),
                &mut out,
                &mut out_len,
            ),
            _ => markstone_actos_to_ast(
                bytes.as_ptr() as *const c_char,
                bytes.len(),
                &mut out,
                &mut out_len,
            ),
        };

        assert_eq!(status, MarkstoneStatus::Ok);
        assert!(!out.is_null());
        // SAFETY: verify terminating NUL
        unsafe {
            assert_eq!(*out.add(out_len), 0);
        }

        markstone_free(out, out_len);
    }
}
