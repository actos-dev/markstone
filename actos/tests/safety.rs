use markstone_actos::{MAX_INPUT_SIZE, MarkstoneError, to_ast, to_html};

#[test]
fn test_mention_attribute_injection_impossible() {
    // Attempting attribute injection via username
    let inputs = [
        "@user\" onclick=\"alert(1)\"",
        "@user' onclick='alert(1)'",
        "@user<script>alert(1)</script>",
        "@user>click me</a>",
        "@user/profile",
        "@user?param=1",
        "@user&param=1",
        "@user\0null",
    ];

    for input in inputs {
        let html = to_html(input).unwrap();
        // The mention anchor tag <a> must NEVER have injected attributes or script tags
        assert!(!html.contains("<a href=\"/u/user\" onclick="));
        assert!(!html.contains("<a href=\"/u/user\" onfocus="));
        assert!(!html.contains("<script>"));
        if html.contains("class=\"mention\"") {
            assert!(html.contains("<a href=\"/u/user\" class=\"mention\">@user</a>"));
        }
    }
}

#[test]
fn test_tag_attribute_injection_impossible() {
    let inputs = [
        "#tag\" onclick=\"alert(1)\"",
        "#tag' onfocus='alert(1)'",
        "#tag<script>",
        "#tag>inject</a>",
        "#tag/sub",
        "#tag?foo=bar",
        "#tag&amp;",
    ];

    for input in inputs {
        let html = to_html(input).unwrap();
        assert!(!html.contains("<a href=\"/t/tag\" onclick="));
        assert!(!html.contains("<a href=\"/t/tag\" onfocus="));
        assert!(!html.contains("<script>"));
        if html.contains("class=\"tag\"") {
            assert!(html.contains("<a href=\"/t/tag\" class=\"tag\">#tag</a>"));
        }
    }
}

#[test]
fn test_security_parity_with_core() {
    // Dangerous schemes still degraded to plain text
    let js_link = "[Click](javascript:alert(1))";
    assert_eq!(to_html(js_link).unwrap(), "<p>Click</p>\n");

    let data_link = "[Data](data:text/html,<script>alert(1)</script>)";
    assert_eq!(to_html(data_link).unwrap(), "<p>Data</p>\n");

    let vbs_link = "[VBS](vbscript:msgbox(1))";
    assert_eq!(to_html(vbs_link).unwrap(), "<p>VBS</p>\n");

    // Raw HTML dropped
    let raw = "<script>alert('xss')</script>\n\nHello @alice";
    assert_eq!(
        to_html(raw).unwrap(),
        "<p>Hello <a href=\"/u/alice\" class=\"mention\">@alice</a></p>\n"
    );

    // Bidi characters stripped
    let bidi = "Hello \u{202A}@alice\u{202E}!";
    assert_eq!(
        to_html(bidi).unwrap(),
        "<p>Hello <a href=\"/u/alice\" class=\"mention\">@alice</a>!</p>\n"
    );

    // External link rel
    let ext = "[Actos](https://actos.app)";
    assert_eq!(
        to_html(ext).unwrap(),
        "<p><a href=\"https://actos.app\" rel=\"nofollow noopener noreferrer\">Actos</a></p>\n"
    );
}

#[test]
fn test_input_size_limit_actos() {
    // 4 MiB exact is accepted
    let exact_input = "a".repeat(MAX_INPUT_SIZE);
    assert!(to_html(&exact_input).is_ok());
    assert!(to_ast(&exact_input).is_ok());

    // 4 MiB + 1 byte is rejected
    let too_large = "a".repeat(MAX_INPUT_SIZE + 1);
    assert_eq!(to_html(&too_large), Err(MarkstoneError::InputTooLarge));
    assert_eq!(to_ast(&too_large), Err(MarkstoneError::InputTooLarge));
}
