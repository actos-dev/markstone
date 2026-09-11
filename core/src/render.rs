use std::fmt::Write as _;
use comrak::html::{format_document_with_formatter, format_node_default, ChildRendering, Context};
use comrak::nodes::{AstNode, NodeValue};
use comrak::options::Plugins;
use comrak::{parse_document, Arena, Options};

use crate::depth::check_block_depth;
use crate::error::MarkstoneError;
use crate::sanitize::{sanitize_code_block_lang, strip_invisible_and_bidi, validate_url, UrlKind};

/// Maximum permitted input size: 4 MiB (4 * 1024 * 1024 bytes).
pub const MAX_INPUT_SIZE: usize = 4 * 1024 * 1024;

/// Constructs comrak options for CommonMark + GFM extensions.
#[must_use]
pub fn comrak_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.table = true;
    opts.extension.strikethrough = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts.extension.header_id_prefix = None;
    opts
}

/// Renders a single AST node during the iterative HTML rendering walk.
fn render_node<'a>(
    context: &mut Context<()>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> Result<ChildRendering, std::fmt::Error> {
    match node.data().value {
        // Raw HTML blocks and inline raw HTML are completely dropped from output
        NodeValue::HtmlBlock(_) | NodeValue::HtmlInline(_) => Ok(ChildRendering::Skip),

        // Code blocks: sanitize language tag to [A-Za-z0-9_+-] and render <pre><code class="language-...">
        NodeValue::CodeBlock(ref ncb) => {
            if entering {
                context.cr()?;
                let lang = sanitize_code_block_lang(&ncb.info);
                if let Some(lang) = lang {
                    write!(context, "<pre><code class=\"language-{lang}\">")?;
                } else {
                    context.write_str("<pre><code>")?;
                }
                context.escape(&ncb.literal)?;
                context.write_str("</code></pre>")?;
                context.lf()?;
            }
            Ok(ChildRendering::Skip)
        }

        // Links: validate destination URL schemes.
        // If rejected, degrade to plain text (do not write <a> / </a>; let children render).
        // External links receive rel="nofollow noopener noreferrer".
        NodeValue::Link(ref nl) => {
            match validate_url(&nl.url) {
                Some(url_kind) => {
                    if entering {
                        context.write_str("<a href=\"")?;
                        context.escape_href(&nl.url)?;
                        context.write_str("\"")?;
                        if url_kind == UrlKind::External {
                            context.write_str(" rel=\"nofollow noopener noreferrer\"")?;
                        }
                        if !nl.title.is_empty() {
                            context.write_str(" title=\"")?;
                            context.escape(&nl.title)?;
                            context.write_str("\"")?;
                        }
                        context.write_str(">")?;
                    } else {
                        context.write_str("</a>")?;
                    }
                    Ok(ChildRendering::HTML)
                }
                None => {
                    // Degrade link to plain text: omit <a> and </a>, render child text/inlines
                    Ok(ChildRendering::HTML)
                }
            }
        }

        // Images: validate source URL schemes.
        // If rejected, drop the entire image node.
        NodeValue::Image(ref nl) => {
            if validate_url(&nl.url).is_none() {
                // Drop image node on enter and exit
                Ok(ChildRendering::Skip)
            } else {
                format_node_default(context, node, entering)
            }
        }

        // All other nodes (headings, paragraphs, lists, tables, task items, footnotes, etc.)
        // are formatted by comrak's standard formatter.
        _ => format_node_default(context, node, entering),
    }
}

/// Converts a Markdown string into safe, sanitized HTML.
///
/// # Security Guarantees:
/// - Input size limited to 4 MiB ([`MarkstoneError::InputTooLarge`]).
/// - Block nesting depth limited to 64 ([`MarkstoneError::DepthExceeded`]).
/// - Invisible and bidi control characters stripped before parsing.
/// - Tree walk and HTML rendering implemented iteratively without recursion.
/// - Raw HTML blocks and inline raw HTML completely dropped from output.
/// - Text nodes safely HTML-escaped.
/// - Links and images validated: only `http`, `https`, `mailto` and relative URLs allowed.
///   Disallowed links degrade to plain text; disallowed images are dropped.
/// - External links get `rel="nofollow noopener noreferrer"`.
/// - Code block language tags sanitized to `[A-Za-z0-9_+-]`.
/// - No heading IDs generated.
///
/// # Errors
/// Returns [`MarkstoneError::InputTooLarge`] if input exceeds 4 MiB, or
/// [`MarkstoneError::DepthExceeded`] if block nesting depth exceeds 64.
pub fn to_html(input: &str) -> Result<String, MarkstoneError> {
    if input.len() > MAX_INPUT_SIZE {
        return Err(MarkstoneError::InputTooLarge);
    }

    let sanitized = strip_invisible_and_bidi(input);
    let options = comrak_options();
    let arena = Arena::new();
    let root = parse_document(&arena, &sanitized, &options);

    check_block_depth(root)?;

    let mut output = String::new();
    let plugins = Plugins::default();
    format_document_with_formatter(root, &options, &mut output, &plugins, render_node, ())
        .map_err(|e| MarkstoneError::Internal(e.to_string()))?;

    Ok(output)
}

/// Convenience helper to convert UTF-8 byte slices to safe HTML.
///
/// Returns [`MarkstoneError::InvalidUtf8`] if the slice is not valid UTF-8.
pub fn to_html_bytes(input: &[u8]) -> Result<String, MarkstoneError> {
    let s = std::str::from_utf8(input).map_err(|_| MarkstoneError::InvalidUtf8)?;
    to_html(s)
}
