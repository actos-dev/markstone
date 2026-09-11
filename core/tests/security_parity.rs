use markstone_core::{
    AstDocument, MAX_INPUT_SIZE, MarkstoneError, Node, is_invisible_or_bidi, to_ast, to_html,
};

#[test]
fn test_parity_script_tags_dropped() {
    let input = "<script>alert('xss')</script>\n\n<script src=\"evil.js\"></script>\n";

    let html = to_html(input).unwrap();
    assert_eq!(html, "");

    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    assert_eq!(doc.root.children().unwrap().len(), 0);
    assert!(!ast_json.contains("script"));
    assert!(!ast_json.contains("alert"));
}

#[test]
fn test_parity_iframe_object_embed_tags_dropped() {
    let input = "<iframe src=\"evil.html\"></iframe>\n<object data=\"evil.swf\"></object>\n<embed src=\"evil.swf\">\n";

    let html = to_html(input).unwrap();
    assert_eq!(html, "");

    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    assert_eq!(doc.root.children().unwrap().len(), 0);
    assert!(!ast_json.contains("iframe"));
    assert!(!ast_json.contains("object"));
    assert!(!ast_json.contains("embed"));
}

#[test]
fn test_parity_inline_html_dropped() {
    let input = "Hello <span style=\"color:red\">world</span> and <div onclick=\"alert(1)\">click me</div>.\n";

    let html = to_html(input).unwrap();
    assert_eq!(html, "<p>Hello world and click me.</p>\n");

    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();

    let para = &doc.root.children().unwrap()[0];
    let inlines = para.children().unwrap();
    // In both paths: raw tags and handlers are dropped; inner text survives
    for node in inlines {
        assert_eq!(node.node_type(), "text");
    }
    assert!(!ast_json.contains("span"));
    assert!(!ast_json.contains("div"));
    assert!(!ast_json.contains("onclick"));
    assert!(!ast_json.contains("alert"));
}

#[test]
fn test_parity_img_onerror_dropped() {
    let input = "<img src=\"x\" onerror=\"alert('xss')\">\n";

    let html = to_html(input).unwrap();
    assert_eq!(html, "");

    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    assert_eq!(doc.root.children().unwrap().len(), 0);
    assert!(!ast_json.contains("onerror"));
    assert!(!ast_json.contains("alert"));
}

#[test]
fn test_parity_javascript_links_degraded() {
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

        let ast_json = to_ast(input).unwrap();
        assert!(!ast_json.contains(r#""type":"link""#));
        assert!(!ast_json.contains("javascript"));

        let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
        let para = &doc.root.children().unwrap()[0];
        let inlines = para.children().unwrap();
        assert_eq!(inlines.len(), 1);
        assert_eq!(inlines[0].node_type(), "text");
        match &inlines[0] {
            Node::Text { value, .. } => assert_eq!(value, "Click here"),
            _ => panic!("Expected text"),
        }
    }
}

#[test]
fn test_parity_data_urls_degraded() {
    let input = "[Payload](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)\n";

    let html = to_html(input).unwrap();
    assert!(!html.contains("<a"));
    assert_eq!(html, "<p>Payload</p>\n");

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"link""#));
    assert!(!ast_json.contains("data:"));

    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let para = &doc.root.children().unwrap()[0];
    let inlines = para.children().unwrap();
    assert_eq!(inlines.len(), 1);
    assert_eq!(inlines[0].node_type(), "text");
}

#[test]
fn test_parity_vbscript_and_file_urls_degraded() {
    let input = "[VBS](vbscript:msgbox(1))\n\n[File](file:///etc/passwd)\n\n[About](about:blank)\n";

    let html = to_html(input).unwrap();
    assert!(!html.contains("<a"));
    assert_eq!(html, "<p>VBS</p>\n<p>File</p>\n<p>About</p>\n");

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"link""#));
    assert!(!ast_json.contains("vbscript"));
    assert!(!ast_json.contains("file:"));

    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let paras = doc.root.children().unwrap();
    assert_eq!(paras.len(), 3);
    for p in paras {
        assert_eq!(p.children().unwrap()[0].node_type(), "text");
    }
}

#[test]
fn test_parity_degraded_link_preserves_formatting() {
    let input = "[**Bold warning** and *italic*](javascript:alert(1))\n";

    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<p><strong>Bold warning</strong> and <em>italic</em></p>\n"
    );
    assert!(!html.contains("<a"));

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"link""#));

    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let para = &doc.root.children().unwrap()[0];
    let inlines = para.children().unwrap();
    assert_eq!(inlines[0].node_type(), "strong");
    assert_eq!(inlines[1].node_type(), "text");
    assert_eq!(inlines[2].node_type(), "emphasis");
}

#[test]
fn test_parity_dangerous_images_dropped() {
    let input = "![XSS](javascript:alert(1))\n\n![SVG Payload](data:image/svg+xml;base64,PHN2ZyBvbmxvYWQ9YWxlcnQoMSk+)\n\n![Passwd](file:///etc/passwd)\n";

    let html = to_html(input).unwrap();
    assert_eq!(html, "<p></p>\n<p></p>\n<p></p>\n");
    assert!(!html.contains("<img"));

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"image""#));
    assert!(!ast_json.contains("javascript"));
    assert!(!ast_json.contains("data:"));
    assert!(!ast_json.contains("file:"));

    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let paras = doc.root.children().unwrap();
    assert_eq!(paras.len(), 3);
    for p in paras {
        assert_eq!(p.children().unwrap().len(), 0);
    }
}

#[test]
fn test_parity_code_block_language_sanitization() {
    let input = "```<script>alert(1)</script>\ncode\n```\n";

    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<pre><code class=\"language-scriptalert1script\">code\n</code></pre>\n"
    );

    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let cb = &doc.root.children().unwrap()[0];
    match cb {
        Node::CodeBlock {
            language, value, ..
        } => {
            assert_eq!(language, "scriptalert1script");
            assert_eq!(value, "code\n");
        }
        _ => panic!("Expected CodeBlock"),
    }
}

#[test]
fn test_parity_bidi_and_invisible_stripped() {
    let input = "Hello\u{200B}\u{200C}\u{200D} \u{FEFF}\u{2060}\u{00AD}world!\n\n\u{202A}Trojan\u{202E} \u{2066}Source\u{2069}\n";

    let html = to_html(input).unwrap();
    assert_eq!(html, "<p>Hello world!</p>\n<p>Trojan Source</p>\n");

    let ast_json = to_ast(input).unwrap();
    for c in ast_json.chars() {
        assert!(!is_invisible_or_bidi(c));
    }

    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let paras = doc.root.children().unwrap();
    match &paras[0].children().unwrap()[0] {
        Node::Text { value, .. } => assert_eq!(value, "Hello world!"),
        _ => panic!("Expected text"),
    }
    match &paras[1].children().unwrap()[0] {
        Node::Text { value, .. } => assert_eq!(value, "Trojan Source"),
        _ => panic!("Expected text"),
    }
}

#[test]
fn test_parity_input_size_limit() {
    let exact = "a".repeat(MAX_INPUT_SIZE);
    assert!(to_html(&exact).is_ok());
    assert!(to_ast(&exact).is_ok());

    let exceeded = "a".repeat(MAX_INPUT_SIZE + 1);
    assert_eq!(to_html(&exceeded), Err(MarkstoneError::InputTooLarge));
    assert_eq!(to_ast(&exceeded), Err(MarkstoneError::InputTooLarge));
}

#[test]
fn test_parity_depth_limit() {
    // Exact 64 depth: 63 nested blockquotes + 1 paragraph = 64
    let exact = "> ".repeat(63) + "within limit";
    assert!(to_html(&exact).is_ok());
    assert!(to_ast(&exact).is_ok());

    // Exceeded 65 depth: 64 nested blockquotes + 1 paragraph = 65
    let exceeded = "> ".repeat(64) + "exceeded limit";
    assert_eq!(to_html(&exceeded), Err(MarkstoneError::DepthExceeded));
    assert_eq!(to_ast(&exceeded), Err(MarkstoneError::DepthExceeded));

    // Highly nested (1000 blockquotes)
    let deep = "> ".repeat(1000) + "deep";
    assert_eq!(to_html(&deep), Err(MarkstoneError::DepthExceeded));
    assert_eq!(to_ast(&deep), Err(MarkstoneError::DepthExceeded));
}
