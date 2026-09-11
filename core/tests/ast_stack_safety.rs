use markstone_core::{MarkstoneError, to_ast};
use std::thread;

#[test]
fn test_stack_safety_deeply_nested_blocks_ast() {
    // Run in a thread with a small stack (128 KiB) to guarantee no stack overflow occurs
    let builder = thread::Builder::new()
        .name("deep_blocks".into())
        .stack_size(128 * 1024);

    let handler = builder
        .spawn(|| {
            // 1,000 nested blockquotes: must return DepthExceeded without blowing the stack
            let input = "> ".repeat(1000) + "deep";
            let result = to_ast(&input);
            assert_eq!(result, Err(MarkstoneError::DepthExceeded));

            // 5,000 nested blockquotes
            let input = "> ".repeat(5000) + "deep";
            let result = to_ast(&input);
            assert_eq!(result, Err(MarkstoneError::DepthExceeded));
        })
        .unwrap();

    handler.join().expect("thread failed or stack overflowed");
}

#[test]
fn test_stack_safety_deeply_nested_inlines_ast() {
    // Run in a thread with a small stack (128 KiB) to verify non-recursive conversion,
    // serialization, and iterative Drop all operate safely.
    let builder = thread::Builder::new()
        .name("deep_inlines".into())
        .stack_size(128 * 1024);

    let handler = builder
        .spawn(|| {
            // 2,000 nested emphasis delimiters
            let count = 2000;
            let input = "*".repeat(count) + "deep inlines" + &"*".repeat(count);
            let result = to_ast(&input);
            assert!(result.is_ok(), "to_ast failed on deep inlines");

            let json = result.unwrap();
            assert!(json.starts_with(r#"{"schema":1,"root":{"type":"document""#));
            assert!(json.contains("deep inlines"));

            // Verify that constructing the AST struct and dropping a 2,000-deep tree
            // in a 128 KiB stack thread does not overflow the stack (iterative Drop implementation)
            let doc = markstone_core::parse_to_ast_document(&input).unwrap();
            assert_eq!(doc.schema, 1);
            drop(doc);
        })
        .unwrap();

    handler.join().expect("thread failed or stack overflowed");
}

#[test]
fn test_stack_safety_multiple_nested_lists() {
    let builder = thread::Builder::new()
        .name("nested_lists".into())
        .stack_size(128 * 1024);

    let handler = builder
        .spawn(|| {
            // 20 nested lists (each adds List + Item = ~40 block depth <= 64 limit)
            let mut input = String::new();
            for i in 0..20 {
                input.push_str(&"  ".repeat(i));
                input.push_str("* item\n");
            }
            let result = to_ast(&input);
            assert!(result.is_ok());
            let json = result.unwrap();
            assert!(json.contains(r#""type":"list""#));

            // 40 nested lists = ~80 block depth > 64 limit: returns DepthExceeded
            let mut deep_input = String::new();
            for i in 0..40 {
                deep_input.push_str(&"  ".repeat(i));
                deep_input.push_str("* item\n");
            }
            let deep_result = to_ast(&deep_input);
            assert_eq!(deep_result, Err(MarkstoneError::DepthExceeded));
        })
        .unwrap();

    handler.join().expect("thread failed or stack overflowed");
}
