# Markstone Spec Conformance

This directory documents markstone's conformance to **CommonMark 0.31** and **GitHub Flavored Markdown (GFM)** specifications.

## Baseline Specifications

- **CommonMark**: Version 0.31 (https://spec.commonmark.org/0.31/)
- **GitHub Flavored Markdown**: GFM Extension Specification (https://github.github.com/gfm/)
  - Tables
  - Task List Items
  - Strikethrough
  - Autolinks
  - Footnotes

## Markstone Security Policy & Intentional Spec Deviations

Markstone is engineered from the ground up for untrusted input in high-security, multi-user platforms (specifically Actos). Consequently, markstone departs from standard CommonMark / GFM in a small number of strictly documented ways:

| Spec Feature | CommonMark / GFM Default | Markstone Behavior | Rationale |
|---|---|---|---|
| **Raw HTML Blocks** | Passed through unescaped | Completely dropped from AST and HTML output | Eliminates XSS vectors (`<script>`, `<iframe>`, `<object>`, `<embed>`, inline event handlers). |
| **Inline Raw HTML** | Passed through unescaped | Completely dropped from AST and HTML output | Eliminates tag-based injection inside paragraphs, headings, etc. |
| **Dangerous URL Schemes** | Allowed (`javascript:`, `data:`, `vbscript:`) | Permitted schemes restricted to `http`, `https`, `mailto`. Disallowed links degrade to text; disallowed images are dropped. | Prevents script execution via link clicks and SVG/HTML payloads. |
| **External Link Attributes** | No `rel` attributes added | External links automatically receive `rel="nofollow noopener noreferrer"`. Relative links do not. | Protects against tabnabbing, referrer leakage, and search indexing abuse. |
| **Trojan Source Characters** | Preserved | Stripped before parsing: zero-width (`U+200B`..`U+200D`, `U+FEFF`, `U+2060`, `U+00AD`) and bidi control characters (`U+202A`..`U+202E`, `U+2066`..`U+2069`). | Prevents visual spoofing, homoglyph attacks, and bidi direction reversal exploits. |
| **Heading IDs / Anchors** | Many implementations inject auto-generated `id` attributes | Heading IDs are **not** generated. | Eliminates DOM clobbering, CSS injection, and accidental anchor collision. |
| **Code Block Language Tag** | Preserved verbatim | Sanitized to allow only `[A-Za-z0-9_+-]`. Any other character stripped. Emitted as `<code class="language-...">`. | Prevents attribute injection via fenced code info strings. |
| **Block Nesting Depth** | Unbounded | Maximum block nesting depth is 64. Exceeding returns `MarkstoneError::DepthExceeded`. | Protects AST consumers from stack exhaustion across all language bindings. |
| **Input Size** | Unbounded | Maximum input size is 4 MiB (4,194,304 bytes). Exceeding returns `MarkstoneError::InputTooLarge`. | Prevents memory exhaustion attacks. |

## Test Suites in This Directory

- `commonmark_0.31.json`: Representative CommonMark 0.31 example test suite covering all CommonMark syntax constructs.
- `gfm.json`: GitHub Flavored Markdown extension suite covering tables, task lists, strikethrough, autolinks, and footnotes.
