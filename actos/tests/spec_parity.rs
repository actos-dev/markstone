use markstone_actos::to_html as actos_to_html;
use markstone_core::to_html as core_to_html;

#[test]
fn test_generic_markdown_parity() {
    let test_cases = [
        "# Heading 1\n## Heading 2\n### Heading 3\n#### Heading 4\n##### Heading 5\n###### Heading 6\n\nSetext 1\n========\n\nSetext 2\n--------\n",
        "# Some Important Heading with punctuation & symbols!\n",
        "First line\nsecond line.\n\nNew paragraph.\n",
        "Line one  \nLine two\\\nLine three.\n",
        "> Single blockquote\n>\n> Second line\n",
        "> Level 1\n>> Level 2\n>>> Level 3\n",
        "- Item A\n- Item B\n- Item C\n\n1. First\n2. Second\n3. Third\n",
        "5. Fifth\n6. Sixth\n",
        "*italic* _italic_ **bold** __bold__ ***bold italic*** `code span`\n",
        "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n    indented code block\n",
        "***\n\n---\n\n___\n",
        "[External](https://example.com) [HTTP](http://example.com) [Relative](/docs/api) [Mail](mailto:info@example.com)\n",
        "[Actos](https://actos.app \"Actos Platform\")\n",
        "![Logo](https://example.com/logo.png \"Logo\")\n\n![Relative](/assets/img.svg)\n",
        "| Item | Quantity | Price |\n| :--- | :------: | ----: |\n| Apple | 10 | $1.50 |\n| Banana | 20 | $0.75 |\n",
        "This is ~~deleted text~~.\n",
        "- [ ] Unchecked task\n- [x] Checked task\n- [X] Also checked\n",
        "Visit https://example.org or email mailto:user@test.org directly.\n",
        "Here is a note[^1].\n\n[^1]: Note content.\n",
    ];

    for input in test_cases {
        let core_out = core_to_html(input).unwrap();
        let actos_out = actos_to_html(input).unwrap();
        assert_eq!(
            actos_out, core_out,
            "Parity mismatch on input:\n{input}\nCore:\n{core_out}\nActos:\n{actos_out}"
        );
    }
}
