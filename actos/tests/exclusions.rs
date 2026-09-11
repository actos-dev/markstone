use markstone_actos::{to_ast, to_html, AstDocument, Node};

#[test]
fn test_link_text_exclusion() {
    // [@alice](https://example.com) must NOT produce a nested link in HTML
    let input = "[@alice](https://example.com)";
    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<p><a href=\"https://example.com\" rel=\"nofollow noopener noreferrer\">@alice</a></p>\n"
    );
    // Crucial: no class="mention" or nested /u/ link
    assert!(!html.contains("class=\"mention\""));
    assert!(!html.contains("/u/alice"));

    // AST verification: Node::Link whose child is Node::Text, NOT Node::Mention
    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let p = &doc.root.children().unwrap()[0];
    let link = &p.children().unwrap()[0];
    assert_eq!(link.node_type(), "link");
    let link_children = link.children().unwrap();
    assert_eq!(link_children.len(), 1);
    assert_eq!(link_children[0].node_type(), "text");
    match &link_children[0] {
        Node::Text { value, .. } => assert_eq!(value, "@alice"),
        _ => panic!("Expected text inside link"),
    }
}

#[test]
fn test_link_text_tag_exclusion() {
    // [#rust](https://example.com) must NOT produce tag link in HTML or AST
    let input = "[#rust](https://example.com)";
    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<p><a href=\"https://example.com\" rel=\"nofollow noopener noreferrer\">#rust</a></p>\n"
    );
    assert!(!html.contains("class=\"tag\""));
    assert!(!html.contains("/t/rust"));

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"tag""#));
}

#[test]
fn test_link_text_with_formatting_exclusion() {
    // Formatting inside link: [**@alice** and *#rust*](https://example.com)
    let input = "[**@alice** and *#rust*](https://example.com)";
    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<p><a href=\"https://example.com\" rel=\"nofollow noopener noreferrer\"><strong>@alice</strong> and <em>#rust</em></a></p>\n"
    );
    assert!(!html.contains("class=\"mention\""));
    assert!(!html.contains("class=\"tag\""));

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
    assert!(!ast_json.contains(r#""type":"tag""#));
}

#[test]
fn test_code_span_exclusion() {
    // Inline code spans must remain untouched
    let input = "foo `@bar` baz and `#notatag` here";
    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<p>foo <code>@bar</code> baz and <code>#notatag</code> here</p>\n"
    );
    assert!(!html.contains("class=\"mention\""));
    assert!(!html.contains("class=\"tag\""));

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
    assert!(!ast_json.contains(r#""type":"tag""#));
}

#[test]
fn test_code_block_exclusion() {
    // Fenced code block
    let input = "```rust\n// @alice\nfn main() {\n    let s = \"#notatag\";\n}\n```\n";
    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<pre><code class=\"language-rust\">// @alice\nfn main() {\n    let s = &quot;#notatag&quot;;\n}\n</code></pre>\n"
    );
    assert!(!html.contains("class=\"mention\""));
    assert!(!html.contains("class=\"tag\""));

    let ast_json = to_ast(input).unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
    assert!(!ast_json.contains(r#""type":"tag""#));

    // Indented code block
    let input2 = "    @alice\n    #notatag\n";
    let html2 = to_html(input2).unwrap();
    assert_eq!(
        html2,
        "<pre><code>@alice\n#notatag\n</code></pre>\n"
    );
    assert!(!html2.contains("class=\"mention\""));
    assert!(!html2.contains("class=\"tag\""));
}

#[test]
fn test_autolink_exclusion() {
    // Bracketed autolink
    let input = "Check <https://example.com/@alice> now.";
    let html = to_html(input).unwrap();
    assert!(html.contains("<a href=\"https://example.com/@alice\""));
    assert!(!html.contains("class=\"mention\""));
    assert!(!html.contains("<a href=\"/u/alice\""));

    // GFM autolink
    let input2 = "Visit https://example.com/#rust or https://example.com/@alice directly.";
    let html2 = to_html(input2).unwrap();
    assert!(!html2.contains("class=\"mention\""));
    assert!(!html2.contains("class=\"tag\""));
    assert!(!html2.contains("<a href=\"/u/"));
    assert!(!html2.contains("<a href=\"/t/"));
}

#[test]
fn test_image_alt_text_exclusion() {
    let input = "![@alice profile](https://example.com/pic.png \"#rust logo\")";
    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<p><img src=\"https://example.com/pic.png\" alt=\"@alice profile\" title=\"#rust logo\" /></p>\n"
    );
    assert!(!html.contains("class=\"mention\""));
    assert!(!html.contains("class=\"tag\""));
}
