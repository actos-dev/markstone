use markstone_core::{to_html, MarkstoneError, MAX_INPUT_SIZE};

#[test]
fn test_input_size_limit_exact_4mib() {
    // 4 MiB of plain text is allowed
    let input = "a".repeat(MAX_INPUT_SIZE);
    let result = to_html(&input);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.starts_with("<p>"));
    assert!(html.ends_with("</p>\n"));
}

#[test]
fn test_input_size_limit_exceeded() {
    // 4 MiB + 1 byte must return InputTooLarge
    let input = "a".repeat(MAX_INPUT_SIZE + 1);
    let result = to_html(&input);
    assert_eq!(result, Err(MarkstoneError::InputTooLarge));

    // 10 MiB
    let input = "a".repeat(10 * 1024 * 1024);
    let result = to_html(&input);
    assert_eq!(result, Err(MarkstoneError::InputTooLarge));
}

#[test]
fn test_block_nesting_depth_exact_64() {
    // 63 nested blockquotes + 1 paragraph = block nesting depth 64 (within limit)
    let input = "> ".repeat(63) + "nested content";
    let result = to_html(&input);
    assert!(result.is_ok());
}

#[test]
fn test_block_nesting_depth_exceeded_65() {
    // 64 nested blockquotes + 1 paragraph = block nesting depth 65 (exceeds limit 64)
    let input = "> ".repeat(64) + "nested content";
    let result = to_html(&input);
    assert_eq!(result, Err(MarkstoneError::DepthExceeded));
}

#[test]
fn test_stack_safety_with_deeply_nested_blocks() {
    // 1,000 nested blockquotes: must not blow the stack!
    let input = "> ".repeat(1000) + "deep";
    let result = to_html(&input);
    assert_eq!(result, Err(MarkstoneError::DepthExceeded));

    // 5,000 nested blockquotes
    let input = "> ".repeat(5000) + "deep";
    let result = to_html(&input);
    assert_eq!(result, Err(MarkstoneError::DepthExceeded));
}

#[test]
fn test_stack_safety_with_deeply_nested_inlines() {
    // Highly nested emphasis delimiters
    let count = 2000;
    let input = "*".repeat(count) + "hello" + &"*".repeat(count);
    let result = to_html(&input);
    assert!(result.is_ok());
}
