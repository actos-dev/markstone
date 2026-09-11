use markstone_core::{
    AST_SCHEMA_VERSION, AstDocument, Node, parse_to_ast_document, to_ast, to_ast_bytes,
};

#[test]
fn test_schema_version_and_document_structure() {
    assert_eq!(AST_SCHEMA_VERSION, 1);

    let json = to_ast("Hello").unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.schema, 1);
    assert_eq!(doc.root.node_type(), "document");
    assert_eq!(doc.root.pos(), [1, 1, 1, 5]);
    assert_eq!(doc.root.children().unwrap().len(), 1);

    // Byte helper
    let json_bytes = to_ast_bytes(b"Hello").unwrap();
    assert_eq!(json, json_bytes);

    // Invalid UTF-8 bytes error
    let invalid = b"\xff\xfe\xfd";
    assert_eq!(
        to_ast_bytes(invalid),
        Err(markstone_core::MarkstoneError::InvalidUtf8)
    );
}

#[test]
fn test_empty_document() {
    let json = to_ast("").unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.schema, 1);
    assert!(matches!(
        doc.root,
        Node::Document { pos, ref children } if pos == [1, 1, 0, 0] && children.is_empty()
    ));
}

#[test]
fn test_heading_all_levels() {
    for level in 1..=6 {
        let input = format!("{} Heading {}", "#".repeat(level), level);
        let json = to_ast(&input).unwrap();
        let doc: AstDocument = serde_json::from_str(&json).unwrap();

        let heading = &doc.root.children().unwrap()[0];
        assert_eq!(heading.node_type(), "heading");
        match heading {
            Node::Heading {
                level: lvl,
                pos,
                children,
            } => {
                assert_eq!(*lvl, level as u8);
                assert_eq!(pos[0], 1);
                assert_eq!(children.len(), 1);
                assert_eq!(children[0].node_type(), "text");
            }
            _ => panic!("Expected Heading node"),
        }
    }
}

#[test]
fn test_paragraph_and_text_unescaped() {
    // Crucial requirement: text values are UNESCAPED raw text
    let input = "Cats & Dogs (x < 5 && y > 10) \"Fish\" 'Hamsters'";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let para = &doc.root.children().unwrap()[0];
    assert_eq!(para.node_type(), "paragraph");

    let text_node = &para.children().unwrap()[0];
    assert_eq!(text_node.node_type(), "text");
    match text_node {
        Node::Text { value, pos } => {
            assert_eq!(value, "Cats & Dogs (x < 5 && y > 10) \"Fish\" 'Hamsters'");
            assert_eq!(*pos, [1, 1, 1, 47]);
        }
        _ => panic!("Expected Text node"),
    }

    // Verify JSON string contains unescaped text (not &amp;, &lt;, etc.)
    assert!(!json.contains("&amp;"));
    assert!(!json.contains("&lt;"));
    assert!(!json.contains("&gt;"));
    assert!(json.contains("Cats & Dogs (x < 5 && y > 10)"));
}

#[test]
fn test_raw_html_dropped_from_ast() {
    // HtmlBlock dropped
    let block_html = "<script>alert('xss')</script>\n\n<iframe src=\"evil.html\"></iframe>\n";
    let block_json = to_ast(block_html).unwrap();
    let block_doc: AstDocument = serde_json::from_str(&block_json).unwrap();
    assert_eq!(block_doc.root.children().unwrap().len(), 0);
    assert!(!block_json.contains("script"));
    assert!(!block_json.contains("alert"));
    assert!(!block_json.contains("iframe"));

    // HtmlInline dropped
    let inline_html = "Hello <span style=\"color:red\">world</span> and <div onclick=\"alert(1)\">click me</div>.\n";
    let inline_json = to_ast(inline_html).unwrap();
    let inline_doc: AstDocument = serde_json::from_str(&inline_json).unwrap();

    let para = &inline_doc.root.children().unwrap()[0];
    let inlines = para.children().unwrap();
    // All inlines are Text nodes; tags are dropped
    for node in inlines {
        assert_eq!(node.node_type(), "text");
    }
    assert!(!inline_json.contains("span"));
    assert!(!inline_json.contains("div"));
    assert!(!inline_json.contains("onclick"));
}

#[test]
fn test_block_quote() {
    let input = "> Line 1\n> Line 2\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let bq = &doc.root.children().unwrap()[0];
    assert_eq!(bq.node_type(), "block_quote");
    assert_eq!(bq.pos(), [1, 1, 2, 8]);
    assert_eq!(bq.children().unwrap().len(), 1);
    assert_eq!(bq.children().unwrap()[0].node_type(), "paragraph");
}

#[test]
fn test_lists_ordered_and_unordered() {
    // Unordered tight list: `start` must be omitted
    let unordered = "- item 1\n- item 2\n";
    let json = to_ast(unordered).unwrap();
    assert!(!json.contains(r#""start""#));
    assert!(json.contains(r#""ordered":false"#));
    assert!(json.contains(r#""tight":true"#));

    let doc: AstDocument = serde_json::from_str(&json).unwrap();
    let list = &doc.root.children().unwrap()[0];
    match list {
        Node::List {
            ordered,
            start,
            tight,
            children,
            ..
        } => {
            assert!(!*ordered);
            assert_eq!(*start, None);
            assert!(*tight);
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].node_type(), "list_item");
            assert_eq!(children[1].node_type(), "list_item");
        }
        _ => panic!("Expected List node"),
    }

    // Ordered list with custom start
    let ordered = "3. Third\n4. Fourth\n";
    let json = to_ast(ordered).unwrap();
    assert!(json.contains(r#""ordered":true"#));
    assert!(json.contains(r#""start":3"#));

    let doc: AstDocument = serde_json::from_str(&json).unwrap();
    let list = &doc.root.children().unwrap()[0];
    match list {
        Node::List {
            ordered,
            start,
            children,
            ..
        } => {
            assert!(*ordered);
            assert_eq!(*start, Some(3));
            assert_eq!(children.len(), 2);
        }
        _ => panic!("Expected List node"),
    }
}

#[test]
fn test_task_list_items() {
    let input = "- [ ] Pending task\n- [x] Done task\n- [X] Also done\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let list = &doc.root.children().unwrap()[0];
    let items = list.children().unwrap();
    assert_eq!(items.len(), 3);

    match &items[0] {
        Node::TaskItem {
            checked,
            pos,
            children,
        } => {
            assert!(!*checked);
            assert_eq!(*pos, [1, 1, 1, 18]);
            assert_eq!(children[0].node_type(), "paragraph");
        }
        _ => panic!("Expected TaskItem node"),
    }

    match &items[1] {
        Node::TaskItem { checked, .. } => {
            assert!(*checked);
        }
        _ => panic!("Expected TaskItem node"),
    }

    match &items[2] {
        Node::TaskItem { checked, .. } => {
            assert!(*checked);
        }
        _ => panic!("Expected TaskItem node"),
    }
}

#[test]
fn test_code_blocks_and_code_spans() {
    let input = "```rust\nlet x = 1 < 2 && 3 > 0;\n```\n\nInline `x & y`\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let children = doc.root.children().unwrap();
    // Block code
    match &children[0] {
        Node::CodeBlock {
            language,
            value,
            pos,
        } => {
            assert_eq!(language, "rust");
            assert_eq!(value, "let x = 1 < 2 && 3 > 0;\n");
            assert_eq!(*pos, [1, 1, 3, 3]);
        }
        _ => panic!("Expected CodeBlock node"),
    }

    // Inline code
    let para = &children[1];
    let para_children = para.children().unwrap();
    match &para_children[1] {
        Node::Code { value, pos } => {
            assert_eq!(value, "x & y");
            assert_eq!(*pos, [5, 8, 5, 14]);
        }
        _ => panic!("Expected Code node"),
    }
}

#[test]
fn test_thematic_break() {
    let input = "---\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let tb = &doc.root.children().unwrap()[0];
    assert_eq!(tb.node_type(), "thematic_break");
    assert_eq!(tb.pos(), [1, 1, 1, 3]);
}

#[test]
fn test_tables() {
    let input =
        "| Left | Center | Right | None |\n| :--- | :----: | ----: | ---- |\n| 1 | 2 | 3 | 4 |\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let table = &doc.root.children().unwrap()[0];
    assert_eq!(table.node_type(), "table");
    match table {
        Node::Table {
            alignments,
            pos,
            children,
        } => {
            assert_eq!(alignments, &["left", "center", "right", "none"]);
            assert_eq!(*pos, [1, 1, 3, 17]);
            assert_eq!(children.len(), 2); // Header row and Data row

            // Header row
            match &children[0] {
                Node::TableRow {
                    header,
                    children: cells,
                    ..
                } => {
                    assert!(*header);
                    assert_eq!(cells.len(), 4);
                    assert_eq!(cells[0].node_type(), "table_cell");
                }
                _ => panic!("Expected TableRow header"),
            }

            // Data row
            match &children[1] {
                Node::TableRow { header, .. } => {
                    assert!(!*header);
                }
                _ => panic!("Expected TableRow data"),
            }
        }
        _ => panic!("Expected Table node"),
    }
}

#[test]
fn test_inlines_emphasis_strong_strikethrough() {
    let input = "*em* **strong** ~~strike~~\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let para = &doc.root.children().unwrap()[0];
    let inlines = para.children().unwrap();

    assert_eq!(inlines[0].node_type(), "emphasis");
    assert_eq!(inlines[0].pos(), [1, 1, 1, 4]);

    assert_eq!(inlines[2].node_type(), "strong");
    assert_eq!(inlines[2].pos(), [1, 6, 1, 15]);

    assert_eq!(inlines[4].node_type(), "strikethrough");
    assert_eq!(inlines[4].pos(), [1, 17, 1, 26]);
}

#[test]
fn test_links_and_images() {
    let input =
        "[Actos](https://actos.org \"Actos Platform\")\n\n![Logo](/logo.png \"Site Logo\")\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let children = doc.root.children().unwrap();

    // Link
    let p1 = &children[0];
    let link = &p1.children().unwrap()[0];
    match link {
        Node::Link {
            url,
            title,
            pos,
            children,
        } => {
            assert_eq!(url, "https://actos.org");
            assert_eq!(title, "Actos Platform");
            assert_eq!(*pos, [1, 1, 1, 43]);
            assert_eq!(children[0].node_type(), "text");
        }
        _ => panic!("Expected Link node"),
    }

    // Image
    let p2 = &children[1];
    let img = &p2.children().unwrap()[0];
    match img {
        Node::Image {
            url,
            title,
            pos,
            children,
        } => {
            assert_eq!(url, "/logo.png");
            assert_eq!(title, "Site Logo");
            assert_eq!(*pos, [3, 1, 3, 30]);
            assert_eq!(children[0].node_type(), "text");
        }
        _ => panic!("Expected Image node"),
    }
}

#[test]
fn test_softbreak_and_linebreak() {
    let input = "Line one\nLine two  \nLine three\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let para = &doc.root.children().unwrap()[0];
    let inlines = para.children().unwrap();

    // Softbreak between Line one and Line two
    assert_eq!(inlines[1].node_type(), "soft_break");
    assert_eq!(inlines[1].pos(), [1, 9, 1, 9]);

    // Hard line break between Line two and Line three
    assert_eq!(inlines[3].node_type(), "line_break");
    assert_eq!(inlines[3].pos(), [2, 9, 2, 11]);
}

#[test]
fn test_footnotes() {
    let input = "Reference[^note]\n\n[^note]: Footnote text\n";
    let json = to_ast(input).unwrap();
    let doc: AstDocument = serde_json::from_str(&json).unwrap();

    let children = doc.root.children().unwrap();

    // Paragraph with FootnoteReference
    let p = &children[0];
    let inlines = p.children().unwrap();
    match &inlines[1] {
        Node::FootnoteReference { name, pos } => {
            assert_eq!(name, "note");
            assert_eq!(*pos, [1, 10, 1, 16]);
        }
        _ => panic!("Expected FootnoteReference"),
    }

    // FootnoteDefinition
    match &children[1] {
        Node::FootnoteDefinition {
            name,
            pos,
            children,
        } => {
            assert_eq!(name, "note");
            assert_eq!(*pos, [3, 1, 3, 22]);
            assert_eq!(children[0].node_type(), "paragraph");
        }
        _ => panic!("Expected FootnoteDefinition"),
    }
}

#[test]
fn test_parse_to_ast_document_direct() {
    let doc = parse_to_ast_document("# Direct AST").unwrap();
    assert_eq!(doc.schema, AST_SCHEMA_VERSION);
    assert_eq!(doc.root.node_type(), "document");
}
