use markstone::*;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Arc;
use std::thread;

#[test]
fn test_thread_concurrency() {
    const NUM_THREADS: usize = 20;
    const ITERATIONS_PER_THREAD: usize = 100;

    let test_inputs: Arc<Vec<&'static str>> = Arc::new(vec![
        "# Hello from thread\n\nParagraph with **bold** and *italic* text.",
        "## Actos surface\n\nContact @admin or @support regarding #release-1.",
        "| Name | Score |\n|------|-------|\n| Alice | 100 |\n| Bob | 95 |",
        "```rust\nfn main() {\n    println!(\"Hello!\");\n}\n```",
        "- [x] Completed task\n- [ ] Incomplete task\n",
        "",
        "   \n\t\n   ",
        "Embedded \0 NUL \0 characters \0 in text",
        "> Quote level 1\n>> Quote level 2\n>>> Quote level 3\n",
    ]);

    let mut handles = Vec::with_capacity(NUM_THREADS);

    for thread_idx in 0..NUM_THREADS {
        let inputs = Arc::clone(&test_inputs);
        handles.push(thread::spawn(move || {
            for iter in 0..ITERATIONS_PER_THREAD {
                let input = inputs[(thread_idx + iter) % inputs.len()];
                let bytes = input.as_bytes();

                // Test version & schema getters
                let ver_ptr = markstone_version();
                assert!(!ver_ptr.is_null());
                // SAFETY: static C string
                let ver_str = unsafe { CStr::from_ptr(ver_ptr) };
                assert_eq!(ver_str.to_str().unwrap(), "0.1.0");
                assert_eq!(markstone_ast_schema_version(), 1);

                // 1. markstone_to_html
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
                // SAFETY: buffer contains out_len bytes + NUL
                unsafe {
                    assert_eq!(*out.add(out_len), 0);
                }
                markstone_free(out, out_len);

                // 2. markstone_to_ast
                let mut out: *mut c_char = std::ptr::null_mut();
                let mut out_len: usize = 0;
                let status = markstone_to_ast(
                    bytes.as_ptr() as *const c_char,
                    bytes.len(),
                    &mut out,
                    &mut out_len,
                );
                assert_eq!(status, MarkstoneStatus::Ok);
                assert!(!out.is_null());
                unsafe {
                    assert_eq!(*out.add(out_len), 0);
                }
                markstone_free(out, out_len);

                // 3. markstone_actos_to_html
                let mut out: *mut c_char = std::ptr::null_mut();
                let mut out_len: usize = 0;
                let status = markstone_actos_to_html(
                    bytes.as_ptr() as *const c_char,
                    bytes.len(),
                    &mut out,
                    &mut out_len,
                );
                assert_eq!(status, MarkstoneStatus::Ok);
                assert!(!out.is_null());
                unsafe {
                    assert_eq!(*out.add(out_len), 0);
                }
                markstone_free(out, out_len);

                // 4. markstone_actos_to_ast
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
                unsafe {
                    assert_eq!(*out.add(out_len), 0);
                }
                markstone_free(out, out_len);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread panicked during concurrency test");
    }
}
