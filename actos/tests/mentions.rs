use markstone_actos::{AstDocument, Node, to_ast, to_ast_bytes, to_html, to_html_bytes};

#[test]
fn test_mention_valid_usernames() {
    // 3 characters (minimum)
    let html = to_html("Hello @bob!").unwrap();
    assert_eq!(
        html,
        "<p>Hello <a href=\"/u/bob\" class=\"mention\">@bob</a>!</p>\n"
    );

    // 32 characters (maximum)
    let u32 = "a".repeat(32);
    let input = format!("Hello @{u32}!");
    let html = to_html(&input).unwrap();
    assert_eq!(
        html,
        format!("<p>Hello <a href=\"/u/{u32}\" class=\"mention\">@{u32}</a>!</p>\n")
    );

    // With numbers and underscores
    let input = "Talk to @alice_123 and @_bot and @user_name.";
    let html = to_html(input).unwrap();
    assert_eq!(
        html,
        "<p>Talk to <a href=\"/u/alice_123\" class=\"mention\">@alice_123</a> and <a href=\"/u/_bot\" class=\"mention\">@_bot</a> and <a href=\"/u/user_name\" class=\"mention\">@user_name</a>.</p>\n"
    );
}

#[test]
fn test_mention_too_short_rejection() {
    // Empty @
    assert_eq!(to_html("Hello @ world").unwrap(), "<p>Hello @ world</p>\n");

    // 1 char: @a
    assert_eq!(
        to_html("Hello @a world").unwrap(),
        "<p>Hello @a world</p>\n"
    );

    // 2 chars: @ab
    assert_eq!(
        to_html("Hello @ab world").unwrap(),
        "<p>Hello @ab world</p>\n"
    );

    // AST verification
    let ast_json = to_ast("Hello @ab world").unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let p = &doc.root.children().unwrap()[0];
    assert_eq!(p.children().unwrap().len(), 1);
    assert_eq!(p.children().unwrap()[0].node_type(), "text");
}

#[test]
fn test_mention_too_long_rejection() {
    // 33 chars: too long
    let u33 = "a".repeat(33);
    let input = format!("Hello @{u33} world");
    let html = to_html(&input).unwrap();
    assert_eq!(html, format!("<p>Hello @{u33} world</p>\n"));

    let ast_json = to_ast(&input).unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
}

#[test]
fn test_mention_uppercase_rejection() {
    assert_eq!(to_html("Hello @Alice").unwrap(), "<p>Hello @Alice</p>\n");
    assert_eq!(to_html("Hello @FOO").unwrap(), "<p>Hello @FOO</p>\n");
    assert_eq!(to_html("Hello @fooBar").unwrap(), "<p>Hello @fooBar</p>\n");
    assert_eq!(
        to_html("Hello @foo_Bar").unwrap(),
        "<p>Hello @foo_Bar</p>\n"
    );

    let ast_json = to_ast("@Foo").unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
}

#[test]
fn test_mention_hyphen_rejection() {
    // Hyphens are not permitted in usernames (only [a-z0-9_])
    assert_eq!(
        to_html("Hello @a-b world").unwrap(),
        "<p>Hello @a-b world</p>\n"
    );
    assert_eq!(
        to_html("Hello @alice-bob world").unwrap(),
        "<p>Hello @alice-bob world</p>\n"
    );

    let ast_json = to_ast("@alice-bob").unwrap();
    assert!(!ast_json.contains(r#""type":"mention""#));
}

#[test]
fn test_mention_preceding_char_rejection() {
    // mail@example.com is an email autolink in GFM; it must NOT contain a mention link
    let mail_html = to_html("mail@example.com").unwrap();
    assert!(!mail_html.contains("class=\"mention\""));
    assert!(!mail_html.contains("/u/"));

    // Letter preceding (plain text, non-autolink)
    assert_eq!(to_html("abc@def").unwrap(), "<p>abc@def</p>\n");
    assert_eq!(to_html("ABC@def").unwrap(), "<p>ABC@def</p>\n");
    assert_eq!(to_html("foo@bar").unwrap(), "<p>foo@bar</p>\n");

    // Digit preceding
    assert_eq!(to_html("user1@example").unwrap(), "<p>user1@example</p>\n");

    // Underscore preceding
    assert_eq!(to_html("user_@example").unwrap(), "<p>user_@example</p>\n");
}

#[test]
fn test_mention_valid_preceding_chars() {
    // Start of line
    assert_eq!(
        to_html("@alice").unwrap(),
        "<p><a href=\"/u/alice\" class=\"mention\">@alice</a></p>\n"
    );

    // Whitespace
    assert_eq!(
        to_html("Hello @alice").unwrap(),
        "<p>Hello <a href=\"/u/alice\" class=\"mention\">@alice</a></p>\n"
    );

    // Parentheses
    assert_eq!(
        to_html("(@alice)").unwrap(),
        "<p>(<a href=\"/u/alice\" class=\"mention\">@alice</a>)</p>\n"
    );

    // Quotes
    assert_eq!(
        to_html("\"@alice\"").unwrap(),
        "<p>&quot;<a href=\"/u/alice\" class=\"mention\">@alice</a>&quot;</p>\n"
    );
    assert_eq!(
        to_html("'@alice'").unwrap(),
        "<p>'<a href=\"/u/alice\" class=\"mention\">@alice</a>'</p>\n"
    );

    // Punctuation
    assert_eq!(
        to_html("Hello, @alice!").unwrap(),
        "<p>Hello, <a href=\"/u/alice\" class=\"mention\">@alice</a>!</p>\n"
    );
    assert_eq!(
        to_html("Hello:@alice").unwrap(),
        "<p>Hello:<a href=\"/u/alice\" class=\"mention\">@alice</a></p>\n"
    );
}

#[test]
fn test_mention_trailing_punctuation_preservation() {
    let input = "Check @alice. Then @bob! Also @carol? And @dave, @eve: @frank;";
    let expected = "<p>Check <a href=\"/u/alice\" class=\"mention\">@alice</a>. Then <a href=\"/u/bob\" class=\"mention\">@bob</a>! Also <a href=\"/u/carol\" class=\"mention\">@carol</a>? And <a href=\"/u/dave\" class=\"mention\">@dave</a>, <a href=\"/u/eve\" class=\"mention\">@eve</a>: <a href=\"/u/frank\" class=\"mention\">@frank</a>;</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_mention_multiple_in_one_text() {
    let input = "@alice and @bob and @carol are collaborating.";
    let expected = "<p><a href=\"/u/alice\" class=\"mention\">@alice</a> and <a href=\"/u/bob\" class=\"mention\">@bob</a> and <a href=\"/u/carol\" class=\"mention\">@carol</a> are collaborating.</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);

    let ast_json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
    let p = &doc.root.children().unwrap()[0];
    let inlines = p.children().unwrap();

    // 0: mention alice, 1: text " and ", 2: mention bob, 3: text " and ", 4: mention carol, 5: text " are collaborating."
    assert_eq!(inlines.len(), 6);
    assert_eq!(inlines[0].node_type(), "mention");
    assert_eq!(inlines[2].node_type(), "mention");
    assert_eq!(inlines[4].node_type(), "mention");
}

#[test]
fn test_mention_ast_node_structure_and_pos() {
    let input = "Hello @alice!";
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

    // Mention: @alice
    assert_eq!(inlines[1].node_type(), "mention");
    match &inlines[1] {
        Node::Mention {
            username,
            text,
            pos,
        } => {
            assert_eq!(username, "alice");
            assert_eq!(text, "@alice");
            assert_eq!(pos, &[1, 7, 1, 12]);
        }
        _ => panic!("Expected mention"),
    }

    // Text: "!"
    assert_eq!(inlines[2].node_type(), "text");
    match &inlines[2] {
        Node::Text { value, pos } => {
            assert_eq!(value, "!");
            assert_eq!(pos, &[1, 13, 1, 13]);
        }
        _ => panic!("Expected text"),
    }
}

#[test]
fn test_mention_bytes_helpers() {
    let input = b"Hello @alice!";
    let html = to_html_bytes(input).unwrap();
    assert_eq!(
        html,
        "<p>Hello <a href=\"/u/alice\" class=\"mention\">@alice</a>!</p>\n"
    );

    let ast = to_ast_bytes(input).unwrap();
    assert!(ast.contains(r#""username":"alice""#));

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
