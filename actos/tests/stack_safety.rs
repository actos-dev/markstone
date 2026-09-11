use markstone_actos::{to_ast, to_html, MarkstoneError};
use std::thread;

#[test]
fn test_stack_safety_deeply_nested_blocks_actos() {
    // Run in a thread with a 128 KiB stack to verify no stack overflow
    let builder = thread::Builder::new()
        .name("deep_blocks_actos".into())
        .stack_size(128 * 1024);

    let handler = builder
        .spawn(|| {
            // 1,000 nested blockquotes
            let input = "> ".repeat(1000) + "@alice #rust";
            assert_eq!(to_ast(&input), Err(MarkstoneError::DepthExceeded));
            assert_eq!(to_html(&input), Err(MarkstoneError::DepthExceeded));

            // 5,000 nested blockquotes
            let input5000 = "> ".repeat(5000) + "@alice";
            assert_eq!(to_ast(&input5000), Err(MarkstoneError::DepthExceeded));
            assert_eq!(to_html(&input5000), Err(MarkstoneError::DepthExceeded));
        })
        .unwrap();

    handler.join().expect("thread failed or stack overflowed");
}

#[test]
fn test_stack_safety_deeply_nested_inlines_actos() {
    let builder = thread::Builder::new()
        .name("deep_inlines_actos".into())
        .stack_size(128 * 1024);

    let handler = builder
        .spawn(|| {
            // 1,000 nested emphasis delimiters containing @mention and #tag
            let count = 1000;
            let input = "*".repeat(count) + "@alice and #rust" + &"*".repeat(count);

            // AST path
            let ast_result = to_ast(&input);
            assert!(ast_result.is_ok(), "to_ast failed on deep inlines with mention/tag");
            let json = ast_result.unwrap();
            assert!(json.contains(r#""username":"alice""#));
            assert!(json.contains(r#""name":"rust""#));

            // HTML path
            let html_result = to_html(&input);
            assert!(html_result.is_ok(), "to_html failed on deep inlines with mention/tag");
            let html = html_result.unwrap();
            assert!(html.contains("<a href=\"/u/alice\" class=\"mention\">@alice</a>"));
            assert!(html.contains("<a href=\"/t/rust\" class=\"tag\">#rust</a>"));
        })
        .unwrap();

    handler.join().expect("thread failed or stack overflowed");
}

#[test]
fn test_stack_safety_exact_limits_actos() {
    let builder = thread::Builder::new()
        .name("exact_limits_actos".into())
        .stack_size(128 * 1024);

    let handler = builder
        .spawn(|| {
            // 63 nested blockquotes + 1 paragraph = depth 64 <= 64: must succeed
            let input64 = "> ".repeat(63) + "Hello @alice and #rust!";
            let html = to_html(&input64).unwrap();
            assert!(html.contains("<a href=\"/u/alice\" class=\"mention\">@alice</a>"));
            assert!(html.contains("<a href=\"/t/rust\" class=\"tag\">#rust</a>"));

            let ast = to_ast(&input64).unwrap();
            assert!(ast.contains(r#""username":"alice""#));

            // 64 nested blockquotes + 1 paragraph = depth 65 > 64: must return DepthExceeded
            let input65 = "> ".repeat(64) + "Hello @alice and #rust!";
            assert_eq!(to_html(&input65), Err(MarkstoneError::DepthExceeded));
            assert_eq!(to_ast(&input65), Err(MarkstoneError::DepthExceeded));
        })
        .unwrap();

    handler.join().expect("thread failed or stack overflowed");
}
