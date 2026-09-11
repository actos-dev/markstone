use markstone_actos::{AstDocument, Node, to_ast, to_ast_bytes, to_html, to_html_bytes};

#[test]
fn test_tag_valid_tags() {
    // Basic tag
    assert_eq!(
        to_html("Check #rust today!").unwrap(),
        "<p>Check <a href=\"/t/rust\" class=\"tag\">#rust</a> today!</p>\n"
    );

    // With hyphens
    assert_eq!(
        to_html("Check #c-sharp and #web-dev-123!").unwrap(),
        "<p>Check <a href=\"/t/c-sharp\" class=\"tag\">#c-sharp</a> and <a href=\"/t/web-dev-123\" class=\"tag\">#web-dev-123</a>!</p>\n"
    );

    // Single character (minimum length 1)
    assert_eq!(
        to_html("Check #a and #1 now!").unwrap(),
        "<p>Check <a href=\"/t/a\" class=\"tag\">#a</a> and <a href=\"/t/1\" class=\"tag\">#1</a> now!</p>\n"
    );

    // 32 characters (maximum length)
    let t32 = format!("a{}", "b".repeat(31));
    let input = format!("Tag #{t32}!");
    let html = to_html(&input).unwrap();
    assert_eq!(
        html,
        format!("<p>Tag <a href=\"/t/{t32}\" class=\"tag\">#{t32}</a>!</p>\n")
    );
}

#[test]
fn test_tag_starting_with_hyphen_rejection() {
    // Cannot start with a hyphen
    assert_eq!(
        to_html("Check #-tag today").unwrap(),
        "<p>Check #-tag today</p>\n"
    );
    assert_eq!(
        to_html("Check #--tag today").unwrap(),
        "<p>Check #--tag today</p>\n"
    );

    let ast_json = to_ast("#-tag").unwrap();
    assert!(!ast_json.contains(r#""type":"tag""#));
}

#[test]
fn test_tag_uppercase_rejection() {
    assert_eq!(
        to_html("Check #Rust today").unwrap(),
        "<p>Check #Rust today</p>\n"
    );
    assert_eq!(
        to_html("Check #C-Sharp today").unwrap(),
        "<p>Check #C-Sharp today</p>\n"
    );
    assert_eq!(
        to_html("Check #TAG today").unwrap(),
        "<p>Check #TAG today</p>\n"
    );

    let ast_json = to_ast("#Rust").unwrap();
    assert!(!ast_json.contains(r#""type":"tag""#));
}

#[test]
fn test_tag_underscore_rejection() {
    // Tags cannot contain underscores
    assert_eq!(
        to_html("Check #tag_name today").unwrap(),
        "<p>Check #tag_name today</p>\n"
    );
    assert_eq!(
        to_html("Check #_tag today").unwrap(),
        "<p>Check #_tag today</p>\n"
    );

    let ast_json = to_ast("#tag_name").unwrap();
    assert!(!ast_json.contains(r#""type":"tag""#));
}

#[test]
fn test_tag_too_long_rejection() {
    // 33 characters: too long
    let t33 = "a".repeat(33);
    let input = format!("Check #{t33} today");
    assert_eq!(
        to_html(&input).unwrap(),
        format!("<p>Check #{t33} today</p>\n")
    );

    let ast_json = to_ast(&input).unwrap();
    assert!(!ast_json.contains(r#""type":"tag""#));
}

#[test]
fn test_tag_preceding_char_rejection() {
    // Letter preceding
    assert_eq!(to_html("foo#bar").unwrap(), "<p>foo#bar</p>\n");
    assert_eq!(to_html("FOO#bar").unwrap(), "<p>FOO#bar</p>\n");

    // Digit preceding
    assert_eq!(to_html("123#tag").unwrap(), "<p>123#tag</p>\n");

    // Underscore preceding
    assert_eq!(to_html("foo_#bar").unwrap(), "<p>foo_#bar</p>\n");
}

#[test]
fn test_tag_valid_preceding_chars() {
    // Start of line
    assert_eq!(
        to_html("#rust").unwrap(),
        // Note: # followed immediately by non-space in ATX heading is NOT a heading unless space follows in CommonMark!
        // In CommonMark: "#rust" is a paragraph! "# rust" is a heading!
        "<p><a href=\"/t/rust\" class=\"tag\">#rust</a></p>\n"
    );

    // Whitespace
    assert_eq!(
        to_html("Learn #rust").unwrap(),
        "<p>Learn <a href=\"/t/rust\" class=\"tag\">#rust</a></p>\n"
    );

    // Parentheses
    assert_eq!(
        to_html("(#rust)").unwrap(),
        "<p>(<a href=\"/t/rust\" class=\"tag\">#rust</a>)</p>\n"
    );

    // Quotes
    assert_eq!(
        to_html("\"#rust\"").unwrap(),
        "<p>&quot;<a href=\"/t/rust\" class=\"tag\">#rust</a>&quot;</p>\n"
    );
}

#[test]
fn test_tag_trailing_punctuation_preservation() {
    let input = "Topics: #rust. Also #c-sharp! And #web-dev? Plus #coding, #ai: #ml;";
    let expected = "<p>Topics: <a href=\"/t/rust\" class=\"tag\">#rust</a>. Also <a href=\"/t/c-sharp\" class=\"tag\">#c-sharp</a>! And <a href=\"/t/web-dev\" class=\"tag\">#web-dev</a>? Plus <a href=\"/t/coding\" class=\"tag\">#coding</a>, <a href=\"/t/ai\" class=\"tag\">#ai</a>: <a href=\"/t/ml\" class=\"tag\">#ml</a>;</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_tag_multiple_in_one_text() {
    let input = "#rust and #c-sharp and #web-dev are popular.";
    let expected = "<p><a href=\"/t/rust\" class=\"tag\">#rust</a> and <a href=\"/t/c-sharp\" class=\"tag\">#c-sharp</a> and <a href=\"/t/web-dev\" class=\"tag\">#web-dev</a> are popular.</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);

    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let p = &doc.root.children().unwrap()[0];
    let inlines = p.children().unwrap();

    assert_eq!(inlines[0].node_type(), "tag");
    assert_eq!(inlines[2].node_type(), "tag");
    assert_eq!(inlines[4].node_type(), "tag");
}

#[test]
fn test_tag_ast_node_structure_and_pos() {
    let input = "Hello #rust!";
    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();

    let p = &doc.root.children().unwrap()[0];
    let inlines = p.children().unwrap();
    assert_eq!(inlines.len(), 3);

    // Text: "Hello "
    assert_eq!(inlines[0].node_type(), "text");
    match &inlines[0] {
        Node::Text { value, pos } => {
            assert_eq!(value, "Hello ");
            assert_eq!(pos, &[1, 1, 1, 6]);
        }
        _ => panic!("Expected text"),
    }

    // Tag: #rust
    assert_eq!(inlines[1].node_type(), "tag");
    match &inlines[1] {
        Node::Tag { name, text, pos } => {
            assert_eq!(name, "rust");
            assert_eq!(text, "#rust");
            assert_eq!(pos, &[1, 7, 1, 11]);
        }
        _ => panic!("Expected tag"),
    }

    // Text: "!"
    assert_eq!(inlines[2].node_type(), "text");
    match &inlines[2] {
        Node::Text { value, pos } => {
            assert_eq!(value, "!");
            assert_eq!(pos, &[1, 12, 1, 12]);
        }
        _ => panic!("Expected text"),
    }
}

#[test]
fn test_tag_bytes_helpers() {
    let input = b"Check #rust!";
    let html = to_html_bytes(input).unwrap();
    assert_eq!(
        html,
        "<p>Check <a href=\"/t/rust\" class=\"tag\">#rust</a>!</p>\n"
    );

    let ast = to_ast_bytes(input).unwrap();
    assert!(ast.contains(r#""name":"rust""#));

    // Invalid UTF-8
    assert_eq!(
        to_html_bytes(b"\xff\xfe"),
        Err(markstone_actos::MarkstoneError::InvalidUtf8)
    );
    assert_eq!(
        to_ast_bytes(b"\xff\xfe"),
        Err(markstone_actos::MarkstoneError::InvalidUtf8)
    );
}
