//! Built-in conformance test cases and case discovery utilities.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::mode::Mode;

/// Built-in case definition for generating golden files.
#[derive(Debug, Clone, Copy)]
pub struct CaseDefinition {
    pub name: &'static str,
    pub input: &'static str,
}

/// A loaded test case with its input and expected golden outputs.
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub dir: PathBuf,
    pub input: String,
    pub expected: HashMap<Mode, Vec<u8>>,
}

/// Catalog of comprehensive built-in test cases covering all CommonMark, GFM,
/// Actos extensions, security rules, limits, and edge cases.
pub const BUILTIN_CASES: &[CaseDefinition] = &[
    // --- Headings ---
    CaseDefinition {
        name: "heading_atx_levels",
        input: "# Level 1\n## Level 2\n### Level 3\n#### Level 4\n##### Level 5\n###### Level 6\n",
    },
    CaseDefinition {
        name: "heading_setext",
        input: "Setext Level 1\n==============\n\nSetext Level 2\n--------------\n",
    },
    CaseDefinition {
        name: "heading_special_chars",
        input: "# Heading & Symbols: `code`, 100% safe <script>alert(1)</script>!\n",
    },
    // --- Paragraphs & Line Breaks ---
    CaseDefinition {
        name: "paragraph_basic",
        input: "First paragraph content here.\n\nSecond paragraph content with multiple sentences.\n",
    },
    CaseDefinition {
        name: "paragraph_soft_break",
        input: "First line of a paragraph.\nSecond line of the same paragraph.\nThird line.\n",
    },
    CaseDefinition {
        name: "paragraph_hard_break",
        input: "Line with two spaces at end  \nLine with backslash at end\\\nFinal line.\n",
    },
    // --- Blockquotes ---
    CaseDefinition {
        name: "blockquote_simple",
        input: "> Single blockquote paragraph.\n> Continuing the blockquote.\n",
    },
    CaseDefinition {
        name: "blockquote_nested",
        input: "> Level 1 quote\n>> Level 2 nested\n>>> Level 3 deeply nested\n",
    },
    CaseDefinition {
        name: "blockquote_multiparagraph",
        input: "> First paragraph in quote.\n>\n> Second paragraph in quote.\n",
    },
    // --- Lists ---
    CaseDefinition {
        name: "list_unordered_tight",
        input: "- Alpha\n- Beta\n- Gamma\n",
    },
    CaseDefinition {
        name: "list_unordered_loose",
        input: "- Item one\n\n- Item two\n\n- Item three\n",
    },
    CaseDefinition {
        name: "list_ordered_simple",
        input: "1. First step\n2. Second step\n3. Third step\n",
    },
    CaseDefinition {
        name: "list_ordered_custom_start",
        input: "42. Answer\n43. Next\n44. Final\n",
    },
    CaseDefinition {
        name: "list_nested",
        input: "- Fruits\n  - Apple\n  - Orange\n- Numbers\n  1. One\n  2. Two\n",
    },
    CaseDefinition {
        name: "list_with_paragraphs",
        input: "1. Item one header.\n\n   Item one additional description paragraph.\n\n2. Item two header.\n",
    },
    // --- GFM Task Lists ---
    CaseDefinition {
        name: "task_list_checked_unchecked",
        input: "- [ ] Incomplete task\n- [x] Completed task\n- [X] Also completed with capital X\n",
    },
    CaseDefinition {
        name: "task_list_nested",
        input: "- [ ] Root goal\n  - [x] Subtask A\n  - [ ] Subtask B\n",
    },
    // --- Inlines (Emphasis, Strong, Strikethrough) ---
    CaseDefinition {
        name: "inline_emphasis",
        input: "*asterisk emphasis* and _underscore emphasis_.\n",
    },
    CaseDefinition {
        name: "inline_strong",
        input: "**asterisk bold** and __underscore bold__.\n",
    },
    CaseDefinition {
        name: "inline_strikethrough",
        input: "This is ~~strikethrough text~~ and ~~deleted~~.\n",
    },
    CaseDefinition {
        name: "inline_nested",
        input: "***bold italic*** and **bold with *italic* inside** and ~~**strikethrough bold**~~.\n",
    },
    // --- Code ---
    CaseDefinition {
        name: "code_span_basic",
        input: "Inline `code span` and ``code with `backtick` inside``.\n",
    },
    CaseDefinition {
        name: "code_span_special_chars",
        input: "`<div> & \"quotes\" & <script>alert(1)</script>`\n",
    },
    CaseDefinition {
        name: "code_block_fenced",
        input: "```rust\nfn main() {\n    println!(\"Hello, Actos!\");\n}\n```\n",
    },
    CaseDefinition {
        name: "code_block_indented",
        input: "    let a = 1;\n    let b = 2;\n    a + b\n",
    },
    CaseDefinition {
        name: "code_block_sanitized_lang",
        input: "```rust\" onclick=\"alert(1)\"\nlet x = 42;\n```\n",
    },
    // --- Thematic Break ---
    CaseDefinition {
        name: "thematic_break",
        input: "Top\n\n***\n\nMiddle\n\n---\n\nBottom\n\n___\n",
    },
    // --- Links ---
    CaseDefinition {
        name: "link_https",
        input: "Check out [Actos](https://actos.app) now.\n",
    },
    CaseDefinition {
        name: "link_http",
        input: "Insecure [Example](http://example.com) link.\n",
    },
    CaseDefinition {
        name: "link_mailto",
        input: "Send feedback to [Support](mailto:support@actos.app).\n",
    },
    CaseDefinition {
        name: "link_relative",
        input: "Read [Docs](/docs/intro) and jump to [Section](#top) or [Query](?page=2).\n",
    },
    CaseDefinition {
        name: "link_with_title",
        input: "Visit [Actos Home](https://actos.app \"Official Actos Website\").\n",
    },
    CaseDefinition {
        name: "link_autolink",
        input: "Direct URL: <https://actos.app> and direct email: <info@example.com>.\n",
    },
    CaseDefinition {
        name: "link_disallowed_javascript",
        input: "Click [Attack](javascript:alert(1)) here.\n",
    },
    CaseDefinition {
        name: "link_disallowed_data",
        input: "Open [Payload](data:text/html,<script>alert(1)</script>) now.\n",
    },
    CaseDefinition {
        name: "link_disallowed_vbscript",
        input: "Run [VBS](vbscript:msgbox(\"xss\")) here.\n",
    },
    CaseDefinition {
        name: "link_disallowed_file",
        input: "Read [Passwd](file:///etc/passwd) secret.\n",
    },
    // --- Images ---
    CaseDefinition {
        name: "image_https",
        input: "![Actos Logo](https://actos.app/logo.png \"Actos Logo\")\n",
    },
    CaseDefinition {
        name: "image_http",
        input: "![Remote Graphic](http://example.com/banner.jpg)\n",
    },
    CaseDefinition {
        name: "image_relative",
        input: "![Local Icon](/assets/icon.svg \"App Icon\")\n",
    },
    CaseDefinition {
        name: "image_disallowed_javascript",
        input: "![XSS](javascript:alert(1))\n",
    },
    CaseDefinition {
        name: "image_disallowed_data",
        input: "![Data SVG](data:image/svg+xml,<svg onload=alert(1)>)\n",
    },
    CaseDefinition {
        name: "image_disallowed_vbscript",
        input: "![VBS](vbscript:alert(1))\n",
    },
    CaseDefinition {
        name: "image_disallowed_file",
        input: "![Shadow](file:///etc/shadow)\n",
    },
    // --- GFM Tables ---
    CaseDefinition {
        name: "table_simple",
        input: "| Column A | Column B |\n| --- | --- |\n| Cell 1 | Cell 2 |\n",
    },
    CaseDefinition {
        name: "table_alignments",
        input: "| Left | Center | Right | Default |\n| :--- | :---: | ---: | --- |\n| L1 | C1 | R1 | D1 |\n| L2 | C2 | R2 | D2 |\n",
    },
    CaseDefinition {
        name: "table_with_inlines",
        input: "| Style | Output |\n| --- | --- |\n| Bold | **text** |\n| Code | `val` |\n| Link | [Actos](https://actos.app) |\n",
    },
    // --- GFM Footnotes ---
    CaseDefinition {
        name: "footnotes_simple",
        input: "Statement with footnote[^1].\n\n[^1]: Footnote body explanation.\n",
    },
    CaseDefinition {
        name: "footnotes_multiple",
        input: "See beta[^beta] and alpha[^alpha].\n\n[^alpha]: Alpha note.\n[^beta]: Beta note.\n",
    },
    // --- Raw HTML Stripping ---
    CaseDefinition {
        name: "raw_html_block_script",
        input: "<script>\nalert('xss');\n</script>\n\nSafe text follows.\n",
    },
    CaseDefinition {
        name: "raw_html_block_tags",
        input: "<div class=\"malicious\">\n  <iframe src=\"https://evil.com\"></iframe>\n  <object data=\"bad.swf\"></object>\n  <embed src=\"bad.swf\">\n</div>\n",
    },
    CaseDefinition {
        name: "raw_html_inline",
        input: "Text with <span onclick=\"evil()\">inline html</span> and <img src=\"x\" onerror=\"alert(1)\"/> stripped.\n",
    },
    // --- Invisible & Bidi Control Characters ---
    CaseDefinition {
        name: "invisible_bidi_characters",
        input: "Clean \u{200B}zero\u{200C}width\u{200D} and \u{FEFF}bom\u{2060} plus \u{202A}bidi\u{202E} override.\n",
    },
    // --- Actos Mentions ---
    CaseDefinition {
        name: "mention_valid",
        input: "Greetings @alice, @bob_123, and @charlie!\n",
    },
    CaseDefinition {
        name: "mention_length_bounds",
        input: "Min length: @abc\nMax length: @a1234567890123456789012345678901\n",
    },
    CaseDefinition {
        name: "mention_too_short",
        input: "Too short @ab is not a mention.\n",
    },
    CaseDefinition {
        name: "mention_too_long",
        input: "Over 32 chars: @a12345678901234567890123456789012 should not match fully.\n",
    },
    CaseDefinition {
        name: "mention_uppercase",
        input: "Uppercase @Alice and @BOB are ignored in Actos.\n",
    },
    CaseDefinition {
        name: "mention_preceding_char",
        input: "Email test@example.com is untouched, but (@alice) and [@bob] are valid mentions.\n",
    },
    CaseDefinition {
        name: "mention_trailing_punctuation",
        input: "Punctuation: @alice. Next: @bob? Finished: @charlie!\n",
    },
    CaseDefinition {
        name: "mention_inside_code_span",
        input: "Code `@alice` and `call(@bob)` are not parsed as mentions.\n",
    },
    CaseDefinition {
        name: "mention_inside_code_block",
        input: "```\n@alice in code block\n```\n",
    },
    CaseDefinition {
        name: "mention_inside_link_text",
        input: "[@alice profile](https://actos.app/u/alice)\n",
    },
    CaseDefinition {
        name: "mention_in_table",
        input: "| User | Status |\n| --- | --- |\n| @alice | Active |\n| @bob | Idle |\n",
    },
    // --- Actos Tags ---
    CaseDefinition {
        name: "tag_valid",
        input: "Posts tagged with #rust, #web-dev, and #v1-0.\n",
    },
    CaseDefinition {
        name: "tag_length_bounds",
        input: "Min tag: #a\nMax tag: #a1234567890123456789012345678901\n",
    },
    CaseDefinition {
        name: "tag_hyphens",
        input: "Valid #rust-lang and #a-b-c, but invalid #--bad starting with hyphen.\n",
    },
    CaseDefinition {
        name: "tag_underscore_rejected",
        input: "Tag with underscore #rust_lang is not permitted.\n",
    },
    CaseDefinition {
        name: "tag_uppercase_rejected",
        input: "Uppercase #Rust is ignored in Actos.\n",
    },
    CaseDefinition {
        name: "tag_preceding_char",
        input: "Anchor foo#bar is untouched, but (#rust) is valid.\n",
    },
    CaseDefinition {
        name: "tag_trailing_punctuation",
        input: "Discuss #rust. Tagged #web! Also #ai,\n",
    },
    CaseDefinition {
        name: "tag_inside_code",
        input: "Code `#rust` and indented:\n\n    #rust\n",
    },
    CaseDefinition {
        name: "tag_inside_link_text",
        input: "[Explore #rust](https://actos.app/t/rust)\n",
    },
    // --- Edge Cases & Unicode ---
    CaseDefinition {
        name: "empty_input",
        input: "",
    },
    CaseDefinition {
        name: "whitespace_only",
        input: "   \n\t\n  \n",
    },
    CaseDefinition {
        name: "unicode_multilingual",
        input: "Turkish: Çiçekler, ılık yağmur, şölen günleri ve Ağrı Dağı.\nGerman: Schöne Grüße aus Köln, Übermut und Spaß.\nCJK: こんにちは世界、你好世界、안녕하세요 세계.\nEmoji: 🚀 Actos 🦀 Rust 🎉\n",
    },
    CaseDefinition {
        name: "complex_mixed_document",
        input: "# Actos Release v1.0\n\nWelcome to **Actos**! Mentioning @alice and @bob regarding #release-notes.\n\n> Note: Follow our [guide](https://actos.app/guide) or email [team](mailto:team@actos.app).\n\n## Features\n\n- [x] Markdown parsing\n- [x] AST output\n- [ ] Live preview\n\n| Component | Status | Assignee |\n| :--- | :---: | ---: |\n| Core | Done | @alice |\n| Actos Extensions | Done | @bob |\n\n```rust\nfn main() {\n    println!(\"Hello #rust\");\n}\n```\n\nFootnote reference[^note].\n\n[^note]: Additional details for @charlie.\n",
    },
];

/// Locate the conformance cases directory by inspecting common locations.
#[must_use]
pub fn find_cases_dir() -> Option<PathBuf> {
    if let Ok(path_str) = std::env::var("MARKSTONE_CASES_DIR") {
        let p = PathBuf::from(path_str);
        if p.is_dir() {
            return Some(p);
        }
    }

    let candidates = [
        "conformance/cases",
        "../conformance/cases",
        "../../conformance/cases",
        "../../../conformance/cases",
    ];

    for candidate in &candidates {
        let p = PathBuf::from(candidate);
        if p.is_dir() {
            if let Ok(canon) = p.canonicalize() {
                return Some(canon);
            }
            return Some(p);
        }
    }

    // Try walking up from current_dir
    if let Ok(mut cur) = std::env::current_dir() {
        for _ in 0..6 {
            let candidate = cur.join("conformance").join("cases");
            if candidate.is_dir() {
                return Some(candidate);
            }
            if !cur.pop() {
                break;
            }
        }
    }

    None
}

/// Loads all test cases from the specified directory.
///
/// If `filter` is provided, only cases whose directory name contains the filter substring are returned.
///
/// # Errors
/// Returns an `io::Error` if reading directories fails.
pub fn load_cases(cases_dir: &Path, filter: Option<&str>) -> io::Result<Vec<TestCase>> {
    let mut cases = Vec::new();

    if !cases_dir.is_dir() {
        return Ok(cases);
    }

    let mut entries: Vec<_> = fs::read_dir(cases_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let dir_path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if let Some(f) = filter {
            if !name.contains(f) {
                continue;
            }
        }

        let input_path = dir_path.join("input.md");
        if !input_path.is_file() {
            continue;
        }

        let input = fs::read_to_string(&input_path)?;

        let mut expected = HashMap::new();
        for mode in Mode::ALL {
            let file_path = dir_path.join(mode.filename());
            if file_path.is_file() {
                let bytes = fs::read(&file_path)?;
                expected.insert(mode, bytes);
            }
        }

        cases.push(TestCase {
            name,
            dir: dir_path,
            input,
            expected,
        });
    }

    Ok(cases)
}
