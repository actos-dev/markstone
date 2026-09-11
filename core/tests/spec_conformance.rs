use markstone_core::to_html;

#[test]
fn test_headings_atx_and_setext() {
    let input = "# Heading 1\n## Heading 2\n### Heading 3\n#### Heading 4\n##### Heading 5\n###### Heading 6\n\nSetext 1\n========\n\nSetext 2\n--------\n";
    let expected = "<h1>Heading 1</h1>\n<h2>Heading 2</h2>\n<h3>Heading 3</h3>\n<h4>Heading 4</h4>\n<h5>Heading 5</h5>\n<h6>Heading 6</h6>\n<h1>Setext 1</h1>\n<h2>Setext 2</h2>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_no_heading_id_generation() {
    let input = "# Some Important Heading with punctuation & symbols!\n";
    let html = to_html(input).unwrap();
    assert_eq!(html, "<h1>Some Important Heading with punctuation &amp; symbols!</h1>\n");
    assert!(!html.contains("id="));
    assert!(!html.contains("class=\"anchor\""));
}

#[test]
fn test_paragraphs_and_softbreaks() {
    let input = "First line\nsecond line.\n\nNew paragraph.\n";
    let expected = "<p>First line\nsecond line.</p>\n<p>New paragraph.</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_hardbreaks() {
    let input = "Line one  \nLine two\\\nLine three.\n";
    let expected = "<p>Line one<br />\nLine two<br />\nLine three.</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_blockquotes() {
    let input = "> Single blockquote\n>\n> Second line\n";
    let expected = "<blockquote>\n<p>Single blockquote</p>\n<p>Second line</p>\n</blockquote>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_nested_blockquotes() {
    let input = "> Level 1\n>> Level 2\n>>> Level 3\n";
    let expected = "<blockquote>\n<p>Level 1</p>\n<blockquote>\n<p>Level 2</p>\n<blockquote>\n<p>Level 3</p>\n</blockquote>\n</blockquote>\n</blockquote>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_lists_unordered_and_ordered() {
    let input = "- Item A\n- Item B\n- Item C\n\n1. First\n2. Second\n3. Third\n";
    let expected = "<ul>\n<li>Item A</li>\n<li>Item B</li>\n<li>Item C</li>\n</ul>\n<ol>\n<li>First</li>\n<li>Second</li>\n<li>Third</li>\n</ol>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_ordered_list_custom_start() {
    let input = "5. Fifth\n6. Sixth\n";
    let expected = "<ol start=\"5\">\n<li>Fifth</li>\n<li>Sixth</li>\n</ol>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_emphasis_and_strong() {
    let input = "*italic* _italic_ **bold** __bold__ ***bold italic*** `code span`\n";
    let expected = "<p><em>italic</em> <em>italic</em> <strong>bold</strong> <strong>bold</strong> <em><strong>bold italic</strong></em> <code>code span</code></p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_code_blocks_fenced_and_indented() {
    let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n    indented code block\n";
    let expected = "<pre><code class=\"language-rust\">fn main() {\n    println!(&quot;Hello&quot;);\n}\n</code></pre>\n<pre><code>indented code block\n</code></pre>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_thematic_break() {
    let input = "***\n\n---\n\n___\n";
    let expected = "<hr />\n<hr />\n<hr />\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_links_external_and_relative() {
    let input = "[External](https://example.com) [HTTP](http://example.com) [Relative](/docs/api) [Mail](mailto:info@example.com)\n";
    let expected = "<p><a href=\"https://example.com\" rel=\"nofollow noopener noreferrer\">External</a> <a href=\"http://example.com\" rel=\"nofollow noopener noreferrer\">HTTP</a> <a href=\"/docs/api\">Relative</a> <a href=\"mailto:info@example.com\">Mail</a></p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_link_with_title() {
    let input = "[Actos](https://actos.app \"Actos Platform\")\n";
    let expected = "<p><a href=\"https://actos.app\" rel=\"nofollow noopener noreferrer\" title=\"Actos Platform\">Actos</a></p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_valid_image() {
    let input = "![Logo](https://example.com/logo.png \"Logo\")\n\n![Relative](/assets/img.svg)\n";
    let expected = "<p><img src=\"https://example.com/logo.png\" alt=\"Logo\" title=\"Logo\" /></p>\n<p><img src=\"/assets/img.svg\" alt=\"Relative\" /></p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_gfm_tables() {
    let input = "| Item | Quantity | Price |\n| :--- | :------: | ----: |\n| Apple | 10 | $1.50 |\n| Banana | 20 | $0.75 |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th align=\"left\">Item</th>\n<th align=\"center\">Quantity</th>\n<th align=\"right\">Price</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td align=\"left\">Apple</td>\n<td align=\"center\">10</td>\n<td align=\"right\">$1.50</td>\n</tr>\n<tr>\n<td align=\"left\">Banana</td>\n<td align=\"center\">20</td>\n<td align=\"right\">$0.75</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_gfm_strikethrough() {
    let input = "This is ~~deleted text~~.\n";
    let expected = "<p>This is <del>deleted text</del>.</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_gfm_task_lists() {
    let input = "- [ ] Unchecked task\n- [x] Checked task\n- [X] Also checked\n";
    let expected = "<ul>\n<li><input type=\"checkbox\" disabled=\"\" /> Unchecked task</li>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> Checked task</li>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> Also checked</li>\n</ul>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_gfm_autolinks() {
    let input = "Visit https://example.org or email mailto:user@test.org directly.\n";
    let expected = "<p>Visit <a href=\"https://example.org\" rel=\"nofollow noopener noreferrer\">https://example.org</a> or email <a href=\"mailto:user@test.org\">mailto:user@test.org</a> directly.</p>\n";
    assert_eq!(to_html(input).unwrap(), expected);
}

#[test]
fn test_gfm_footnotes() {
    let input = "Here is a note[^1].\n\n[^1]: Note content.\n";
    let html = to_html(input).unwrap();
    assert!(html.contains("<sup class=\"footnote-ref\"><a href=\"#fn-1\" id=\"fnref-1\" data-footnote-ref>1</a></sup>"));
    assert!(html.contains("<section class=\"footnotes\" data-footnotes>"));
    assert!(html.contains("<li id=\"fn-1\">"));
    assert!(html.contains("class=\"footnote-backref\""));
}
