use markstone_core::to_html;
use std::collections::{HashMap, HashSet};

/// Configures ammonia with the Actos platform allowlist.
///
/// This serves as the oracle: any valid, safe HTML produced by markstone
/// must pass through this sanitizer completely unchanged (byte-for-byte).
fn platform_sanitizer() -> ammonia::Builder<'static> {
    let allowed_tags = HashSet::from([
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "strong",
        "em",
        "b",
        "i",
        "ul",
        "ol",
        "li",
        "code",
        "pre",
        "blockquote",
        "hr",
        "br",
        "a",
        "img",
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
        "del",
        "s",
        "section",
        "sup",
    ]);

    let mut tag_attrs = HashMap::new();
    tag_attrs.insert(
        "a",
        HashSet::from([
            "href",
            "rel",
            "title",
            "id",
            "class",
            "data-footnote-ref",
            "data-footnote-backref",
            "data-footnote-backref-idx",
            "aria-label",
        ]),
    );
    tag_attrs.insert(
        "img",
        HashSet::from(["src", "alt", "title", "width", "height"]),
    );
    tag_attrs.insert("code", HashSet::from(["class"]));
    tag_attrs.insert("ol", HashSet::from(["start"]));
    tag_attrs.insert("li", HashSet::from(["id"]));
    tag_attrs.insert("th", HashSet::from(["align", "colspan", "rowspan"]));
    tag_attrs.insert("td", HashSet::from(["align", "colspan", "rowspan"]));
    tag_attrs.insert("section", HashSet::from(["class", "data-footnotes"]));
    tag_attrs.insert("sup", HashSet::from(["class"]));

    let allowed_schemes = HashSet::from(["http", "https", "mailto"]);

    let mut builder = ammonia::Builder::new();
    builder
        .tags(allowed_tags)
        .tag_attributes(tag_attrs)
        .generic_attributes(HashSet::new())
        .url_schemes(allowed_schemes)
        .link_rel(None);
    builder
}

fn assert_oracle_byte_for_byte(input: &str) {
    let rendered = to_html(input).unwrap();
    let oracle_output = platform_sanitizer().clean(&rendered).to_string();
    assert_eq!(
        rendered, oracle_output,
        "Ammonia oracle mismatch!\nOriginal markdown:\n{input}\nRendered by markstone:\n{rendered}\nAmmonia output:\n{oracle_output}"
    );
}

#[test]
fn test_oracle_security_corpus() {
    let security_inputs = [
        // Raw script and iframe injections
        "<script>alert(1)</script>",
        "<script src=\"evil.js\"></script>",
        "<iframe src=\"https://evil.com\"></iframe>",
        "<object data=\"evil.swf\"></object>",
        "<embed src=\"evil.swf\">",
        "<style>body { display: none; }</style>",
        // Event handlers in HTML
        "<img src=\"x\" onerror=\"alert(1)\">",
        "<div onclick=\"alert(1)\">click me</div>",
        "<svg onload=\"alert(1)\">",
        "<body onload=\"alert(1)\">",
        // Dangerous link schemes
        "[XSS](javascript:alert(1))",
        "[XSS](JAVASCRIPT:alert(1))",
        "[XSS](  javascript:alert(1))",
        "[XSS](java\0script:alert(1))",
        "[Data](data:text/html,<script>alert(1)</script>)",
        "[VBS](vbscript:msgbox(1))",
        "[File](file:///etc/passwd)",
        // Dangerous image schemes
        "![img](javascript:alert(1))",
        "![img](data:image/svg+xml,<svg onload=alert(1)>)",
        "![img](file:///etc/shadow)",
        "![img](vbscript:alert(1))",
        // Code block tag injections
        "```<script>alert(1)</script>\ncode\n```",
        "```\" onclick=\"alert(1)\"\ncode\n```",
        "```rust\" data-evil=\"true\ncode\n```",
        // Bidi Trojan Source characters
        "Hello \u{202A}Trojan\u{202E} \u{2066}Source\u{2069}!",
        "Zero \u{200B}\u{200C}\u{200D}width \u{FEFF}\u{2060}\u{00AD}chars",
    ];

    for input in security_inputs {
        assert_oracle_byte_for_byte(input);
    }
}

#[test]
fn test_oracle_golden_corpus() {
    let golden_inputs = [
        "# Heading 1\n## Heading 2\n### Heading 3\n",
        "A simple paragraph with **bold**, *italic*, and `code` inline.",
        "> A blockquote\n>\n> Second line of quote.",
        "> Level 1\n>> Level 2\n>>> Level 3",
        "1. First item\n2. Second item\n3. Third item",
        "- Item A\n- Item B\n- Item C",
        "5. Fifth\n6. Sixth",
        "```rust\nfn main() {\n    let x = 42;\n}\n```",
        "```python\ndef hello():\n    return 42\n```",
        "    indented code line 1\n    indented code line 2",
        "[External](https://example.com) and [External HTTP](http://example.com)",
        "[External with title](https://example.com \"Title text\")",
        "[Relative link](/about) and [Section link](#overview) and [Query link](?page=2)",
        "[Email link](mailto:support@actos.app)",
        "This is ~~strikethrough text~~.",
        "| Name | Age | Role |\n| :--- | :--: | ---: |\n| Alice | 30 | Admin |\n| Bob | 25 | User |",
        "Combining **bold** with [links](https://actos.app) and `code` in one paragraph.",
    ];

    for input in golden_inputs {
        assert_oracle_byte_for_byte(input);
    }
}
