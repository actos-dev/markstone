use markstone_core::to_html;

#[test]
fn test_script_tags_completely_dropped() {
    let input = "<script>alert('xss')</script>\n\n<script src=\"evil.js\"></script>\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "");
    assert!(!html.contains("script"));
    assert!(!html.contains("alert"));
}

#[test]
fn test_iframe_object_embed_tags_dropped() {
    let input = "<iframe src=\"evil.html\"></iframe>\n<object data=\"evil.swf\"></object>\n<embed src=\"evil.swf\">\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "");
    assert!(!html.contains("iframe"));
    assert!(!html.contains("object"));
    assert!(!html.contains("embed"));
}

#[test]
fn test_inline_html_and_event_handlers_dropped() {
    let input = "Hello <span style=\"color:red\">world</span> and <div onclick=\"alert(1)\">click me</div>.\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "<p>Hello world and click me.</p>\n");
    assert!(!html.contains("<span"));
    assert!(!html.contains("<div"));
    assert!(!html.contains("onclick"));
    assert!(!html.contains("alert"));
}

#[test]
fn test_img_onerror_in_raw_html_dropped() {
    let input = "<img src=\"x\" onerror=\"alert('xss')\">\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "");
    assert!(!html.contains("onerror"));
    assert!(!html.contains("alert"));
}

#[test]
fn test_javascript_links_degraded_to_plain_text() {
    let cases = [
        "[Click here](javascript:alert(1))",
        "[Click here](javaSCRIPT:alert(1))",
        "[Click here](  javascript:alert(1))",
        "[Click here](<javascript :alert(1)>)",
        "[Click here](java\0script:alert(1))",
        "[Click here](<java\tscript:alert(1)>)",
        "[Click here](javascript&#x3a;alert(1))",
    ];

    for input in cases {
        let html = to_html(input).unwrap();
        assert!(!html.contains("<a"));
        assert!(!html.contains("javascript"));
        assert_eq!(html, "<p>Click here</p>\n");
    }
}

#[test]
fn test_data_urls_degraded_to_plain_text() {
    let input = "[Payload](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)\n";
    let html = to_html(input).unwrap();
    assert!(!html.contains("<a"));
    assert!(!html.contains("data:"));
    assert_eq!(html, "<p>Payload</p>\n");
}

#[test]
fn test_vbscript_and_file_urls_degraded() {
    let input = "[VBS](vbscript:msgbox(1))\n\n[File](file:///etc/passwd)\n\n[About](about:blank)\n";
    let html = to_html(input).unwrap();
    assert!(!html.contains("<a"));
    assert_eq!(html, "<p>VBS</p>\n<p>File</p>\n<p>About</p>\n");
}

#[test]
fn test_degraded_link_preserves_formatting() {
    let input = "[**Bold warning** and *italic*](javascript:alert(1))\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "<p><strong>Bold warning</strong> and <em>italic</em></p>\n");
    assert!(!html.contains("<a"));
}

#[test]
fn test_dangerous_images_dropped_completely() {
    let input = "![XSS](javascript:alert(1))\n\n![SVG Payload](data:image/svg+xml;base64,PHN2ZyBvbmxvYWQ9YWxlcnQoMSk+)\n\n![Passwd](file:///etc/passwd)\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "<p></p>\n<p></p>\n<p></p>\n");
    assert!(!html.contains("<img"));
    assert!(!html.contains("javascript"));
    assert!(!html.contains("data:"));
    assert!(!html.contains("file:"));
}

#[test]
fn test_code_block_language_tag_injection() {
    // Attempting to break out of class="language-..."
    let input = "```<script>alert(1)</script>\ncode\n```\n";
    let html = to_html(input).unwrap();
    assert!(!html.contains("<script"));
    assert!(!html.contains("alert(1)"));
    assert_eq!(html, "<pre><code class=\"language-scriptalert1script\">code\n</code></pre>\n");

    let input_attr = "```\" onclick=\"alert(1)\"\ncode\n```\n";
    let html = to_html(input_attr).unwrap();
    assert!(!html.contains("onclick"));
    assert!(!html.contains("alert"));
    // Since all characters outside [A-Za-z0-9_+-] are stripped, language tag is empty -> <pre><code>
    assert_eq!(html, "<pre><code>code\n</code></pre>\n");
}

#[test]
fn test_bidi_and_invisible_control_characters_stripped() {
    // Trojan Source and invisible characters:
    // U+200B..U+200D, U+FEFF, U+2060, U+00AD, U+202A..U+202E, U+2066..U+2069
    let input = "Hello\u{200B}\u{200C}\u{200D} \u{FEFF}\u{2060}\u{00AD}world!\n\n\u{202A}Trojan\u{202E} \u{2066}Source\u{2069}\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "<p>Hello world!</p>\n<p>Trojan Source</p>\n");

    // Ensure none of the stripped codepoints remain in the output
    for c in html.chars() {
        assert!(!markstone_core::is_invisible_or_bidi(c));
    }
}

#[test]
fn test_external_links_have_rel_and_relative_do_not() {
    let input = "[Ext](https://example.com) [Rel](/page)\n";
    let html = to_html(input).unwrap();
    assert!(html.contains("<a href=\"https://example.com\" rel=\"nofollow noopener noreferrer\">Ext</a>"));
    assert!(html.contains("<a href=\"/page\">Rel</a>"));
    assert!(!html.contains("<a href=\"/page\" rel"));
}
