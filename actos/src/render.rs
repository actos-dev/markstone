use markstone_core::ast::{AstDocument, Node};
use markstone_core::sanitize::{UrlKind, validate_url};

/// Escapes HTML special characters: `&`, `<`, `>`, `"`.
pub fn escape_html(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

/// Escapes href attributes: `&`, `"`, `<`, `>`.
pub fn escape_href(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

/// Extracts plain text from inline nodes non-recursively (used for image `alt` text).
pub fn plain_text_non_recursive(nodes: &[Node], out: &mut String) {
    let mut stack: Vec<(&[Node], usize)> = vec![(nodes, 0)];
    while let Some((slice, idx)) = stack.last_mut() {
        if *idx < slice.len() {
            let node = &slice[*idx];
            *idx += 1;
            match node {
                Node::Text { value, .. } | Node::Code { value, .. } => {
                    out.push_str(value);
                }
                Node::Mention { text, .. } | Node::Tag { text, .. } => {
                    out.push_str(text);
                }
                Node::SoftBreak { .. } | Node::LineBreak { .. } => {
                    out.push(' ');
                }
                _ => {
                    if let Some(children) = node.children() {
                        stack.push((children, 0));
                    }
                }
            }
        } else {
            stack.pop();
        }
    }
}

struct RenderFrame<'a> {
    node: &'a Node,
    next_child_idx: usize,
    tight_list: bool,
    table_alignments: &'a [String],
    table_col_idx: usize,
    is_table_header: bool,
    footnote_backref: Option<(&'a str, usize)>,
    last_paragraph_ptr: Option<*const Node>,
}

/// Renders a slice of nodes non-recursively into `out`.
fn render_nodes_non_recursive<'a>(
    root_node: &'a Node,
    initial_tight_list: bool,
    footnote_backref: Option<(&'a str, usize)>,
    last_paragraph_ptr: Option<*const Node>,
    out: &mut String,
    collected_footnotes: &mut Vec<&'a Node>,
) {
    let mut stack: Vec<RenderFrame<'a>> = Vec::new();

    // Entering root
    render_node_enter(root_node, initial_tight_list, false, &[], 0, out);

    if root_node.children().is_some() {
        stack.push(RenderFrame {
            node: root_node,
            next_child_idx: 0,
            tight_list: initial_tight_list,
            table_alignments: match root_node {
                Node::Table { alignments, .. } => alignments.as_slice(),
                _ => &[],
            },
            table_col_idx: 0,
            is_table_header: false,
            footnote_backref,
            last_paragraph_ptr,
        });
    } else {
        render_node_exit(root_node, initial_tight_list, false, None, out);
        return;
    }

    while let Some(frame) = stack.last_mut() {
        let node = frame.node;
        let children = node.children().unwrap();
        let child_idx = frame.next_child_idx;

        if child_idx < children.len() {
            let child = &children[child_idx];
            frame.next_child_idx += 1;

            // Handle footnotes collection at document root level
            if matches!(node, Node::Document { .. })
                && matches!(child, Node::FootnoteDefinition { .. })
            {
                collected_footnotes.push(child);
                continue;
            }

            // Handle table section wrappers (thead and tbody)
            if matches!(node, Node::Table { .. }) {
                if child_idx == 0 {
                    out.push_str("<thead>\n");
                } else if child_idx == 1 {
                    out.push_str("<tbody>\n");
                }
            }

            let child_is_header = match child {
                Node::TableRow { header, .. } => *header,
                _ => frame.is_table_header,
            };

            let child_tight_list = match child {
                Node::List { tight, .. } => *tight,
                _ => frame.tight_list,
            };

            let table_alignments = match child {
                Node::Table { alignments, .. } => alignments.as_slice(),
                _ => frame.table_alignments,
            };

            let col_idx = if matches!(child, Node::TableCell { .. }) {
                let idx = frame.table_col_idx;
                frame.table_col_idx += 1;
                idx
            } else {
                0
            };

            let parent_tight = frame.tight_list;
            let frame_backref = frame.footnote_backref;
            let frame_para_ptr = frame.last_paragraph_ptr;

            if matches!(child, Node::Image { .. }) {
                render_node_enter(
                    child,
                    parent_tight,
                    child_is_header,
                    table_alignments,
                    col_idx,
                    out,
                );
            } else if child.children().is_some() {
                render_node_enter(
                    child,
                    parent_tight,
                    child_is_header,
                    table_alignments,
                    col_idx,
                    out,
                );

                stack.push(RenderFrame {
                    node: child,
                    next_child_idx: 0,
                    tight_list: child_tight_list,
                    table_alignments,
                    table_col_idx: 0,
                    is_table_header: child_is_header,
                    footnote_backref: frame_backref,
                    last_paragraph_ptr: frame_para_ptr,
                });
            } else {
                // Leaf node: render full HTML directly
                render_leaf_node(
                    child,
                    parent_tight,
                    child_is_header,
                    table_alignments,
                    col_idx,
                    out,
                );
            }
        } else {
            let finished = stack.pop().unwrap();

            // Table closing wrappers
            if matches!(finished.node, Node::Table { .. }) {
                let num_children = finished.node.children().unwrap().len();
                if num_children > 1 {
                    out.push_str("</tbody>\n");
                }
            }

            let parent_tight = stack.last().is_some_and(|f| f.tight_list);

            let backref = if let Some(target_ptr) = finished.last_paragraph_ptr {
                if std::ptr::eq(finished.node, target_ptr) {
                    finished.footnote_backref
                } else {
                    None
                }
            } else {
                None
            };

            render_node_exit(
                finished.node,
                parent_tight,
                finished.is_table_header,
                backref,
                out,
            );

            // Close thead for first table row
            if let Some(parent) = stack.last() {
                if matches!(parent.node, Node::Table { .. }) && parent.next_child_idx == 1 {
                    out.push_str("</thead>\n");
                }
            }
        }
    }
}

fn render_node_enter(
    node: &Node,
    parent_tight_list: bool,
    is_header_cell: bool,
    table_alignments: &[String],
    table_col_idx: usize,
    out: &mut String,
) {
    match node {
        Node::Document { .. } => {}
        Node::Paragraph { .. } => {
            if !parent_tight_list {
                out.push_str("<p>");
            }
        }
        Node::Heading { level, .. } => {
            out.push_str("<h");
            out.push_str(&level.to_string());
            out.push('>');
        }
        Node::BlockQuote { .. } => {
            out.push_str("<blockquote>\n");
        }
        Node::List { ordered, start, .. } => {
            if *ordered {
                if let Some(start_val) = start {
                    if *start_val != 1 {
                        out.push_str("<ol start=\"");
                        out.push_str(&start_val.to_string());
                        out.push_str("\">\n");
                        return;
                    }
                }
                out.push_str("<ol>\n");
            } else {
                out.push_str("<ul>\n");
            }
        }
        Node::ListItem { .. } => {
            if parent_tight_list {
                out.push_str("<li>");
            } else {
                out.push_str("<li>\n");
            }
        }
        Node::TaskItem { checked, .. } => {
            out.push_str("<li><input type=\"checkbox\"");
            if *checked {
                out.push_str(" checked=\"\"");
            }
            out.push_str(" disabled=\"\" /> ");
        }
        Node::Table { .. } => {
            out.push_str("<table>\n");
        }
        Node::TableRow { .. } => {
            out.push_str("<tr>\n");
        }
        Node::TableCell { .. } => {
            let tag = if is_header_cell { "th" } else { "td" };
            out.push('<');
            out.push_str(tag);
            if let Some(align) = table_alignments.get(table_col_idx) {
                if align != "none" {
                    out.push_str(" align=\"");
                    out.push_str(align);
                    out.push('"');
                }
            }
            out.push('>');
        }
        Node::Emphasis { .. } => {
            out.push_str("<em>");
        }
        Node::Strong { .. } => {
            out.push_str("<strong>");
        }
        Node::Strikethrough { .. } => {
            out.push_str("<del>");
        }
        Node::Link { url, title, .. } => {
            out.push_str("<a href=\"");
            escape_href(url, out);
            out.push('"');
            if validate_url(url) == Some(UrlKind::External) {
                out.push_str(" rel=\"nofollow noopener noreferrer\"");
            }
            if !title.is_empty() {
                out.push_str(" title=\"");
                escape_html(title, out);
                out.push('"');
            }
            out.push('>');
        }
        Node::Image {
            url,
            title,
            children,
            ..
        } => {
            out.push_str("<img src=\"");
            escape_href(url, out);
            out.push_str("\" alt=\"");
            let mut alt = String::new();
            plain_text_non_recursive(children, &mut alt);
            escape_html(&alt, out);
            out.push('"');
            if !title.is_empty() {
                out.push_str(" title=\"");
                escape_html(title, out);
                out.push('"');
            }
            out.push_str(" />");
        }
        Node::FootnoteDefinition { .. } => {}
        _ => {}
    }
}

fn render_node_exit(
    node: &Node,
    parent_tight_list: bool,
    is_header_cell: bool,
    footnote_backref: Option<(&str, usize)>,
    out: &mut String,
) {
    match node {
        Node::Document { .. } => {}
        Node::Paragraph { .. } => {
            if let Some((name, idx)) = footnote_backref {
                out.push_str(" <a href=\"#fnref-");
                escape_html(name, out);
                out.push_str("\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"");
                out.push_str(&idx.to_string());
                out.push_str("\" aria-label=\"Back to reference ");
                out.push_str(&idx.to_string());
                out.push_str("\">↩</a>");
            }
            if !parent_tight_list {
                out.push_str("</p>\n");
            }
        }
        Node::Heading { level, .. } => {
            out.push_str("</h");
            out.push_str(&level.to_string());
            out.push_str(">\n");
        }
        Node::BlockQuote { .. } => {
            out.push_str("</blockquote>\n");
        }
        Node::List { ordered, .. } => {
            if *ordered {
                out.push_str("</ol>\n");
            } else {
                out.push_str("</ul>\n");
            }
        }
        Node::ListItem { .. } | Node::TaskItem { .. } => {
            out.push_str("</li>\n");
        }
        Node::Table { .. } => {
            out.push_str("</table>\n");
        }
        Node::TableRow { .. } => {
            out.push_str("</tr>\n");
        }
        Node::TableCell { .. } => {
            let tag = if is_header_cell { "th" } else { "td" };
            out.push_str("</");
            out.push_str(tag);
            out.push_str(">\n");
        }
        Node::Emphasis { .. } => {
            out.push_str("</em>");
        }
        Node::Strong { .. } => {
            out.push_str("</strong>");
        }
        Node::Strikethrough { .. } => {
            out.push_str("</del>");
        }
        Node::Link { .. } => {
            out.push_str("</a>");
        }
        Node::Image { .. } => {}
        Node::FootnoteDefinition { .. } => {}
        _ => {}
    }
}

fn render_leaf_node(
    node: &Node,
    _parent_tight_list: bool,
    _is_header_cell: bool,
    _table_alignments: &[String],
    _table_col_idx: usize,
    out: &mut String,
) {
    match node {
        Node::Text { value, .. } => {
            escape_html(value, out);
        }
        Node::Code { value, .. } => {
            out.push_str("<code>");
            escape_html(value, out);
            out.push_str("</code>");
        }
        Node::CodeBlock {
            language, value, ..
        } => {
            if language.is_empty() {
                out.push_str("<pre><code>");
            } else {
                out.push_str("<pre><code class=\"language-");
                escape_html(language, out);
                out.push_str("\">");
            }
            escape_html(value, out);
            out.push_str("</code></pre>\n");
        }
        Node::ThematicBreak { .. } => {
            out.push_str("<hr />\n");
        }
        Node::SoftBreak { .. } => {
            out.push('\n');
        }
        Node::LineBreak { .. } => {
            out.push_str("<br />\n");
        }
        Node::FootnoteReference { name, .. } => {
            out.push_str("<sup class=\"footnote-ref\"><a href=\"#fn-");
            escape_html(name, out);
            out.push_str("\" id=\"fnref-");
            escape_html(name, out);
            out.push_str("\" data-footnote-ref>");
            escape_html(name, out);
            out.push_str("</a></sup>");
        }
        Node::Mention { username, .. } => {
            out.push_str("<a href=\"/u/");
            escape_href(username, out);
            out.push_str("\" class=\"mention\">@");
            escape_html(username, out);
            out.push_str("</a>");
        }
        Node::Tag { name, .. } => {
            out.push_str("<a href=\"/t/");
            escape_href(name, out);
            out.push_str("\" class=\"tag\">#");
            escape_html(name, out);
            out.push_str("</a>");
        }
        _ => {}
    }
}

/// Converts an [`AstDocument`] containing Actos extensions to safe HTML.
///
/// Implemented strictly without recursion.
#[must_use]
pub fn render_html(doc: &AstDocument) -> String {
    let mut out = String::with_capacity(1024);
    let mut collected_footnotes: Vec<&Node> = Vec::new();

    render_nodes_non_recursive(
        &doc.root,
        false,
        None,
        None,
        &mut out,
        &mut collected_footnotes,
    );

    // Render footnotes section if any footnote definitions were collected
    if !collected_footnotes.is_empty() {
        out.push_str("<section class=\"footnotes\" data-footnotes>\n<ol>\n");
        for (idx, fndef) in collected_footnotes.iter().enumerate() {
            let fn_idx = idx + 1;
            if let Node::FootnoteDefinition { name, children, .. } = fndef {
                out.push_str("<li id=\"fn-");
                escape_html(name, &mut out);
                out.push_str("\">\n");

                // Find the last Paragraph node in children to attach backref
                let last_para_ptr = children.iter().rev().find_map(|c| match c {
                    Node::Paragraph { .. } => Some(c as *const Node),
                    _ => None,
                });

                for child in children {
                    render_nodes_non_recursive(
                        child,
                        false,
                        Some((name, fn_idx)),
                        last_para_ptr,
                        &mut out,
                        &mut Vec::new(),
                    );
                }

                if last_para_ptr.is_none() {
                    out.push_str("<p><a href=\"#fnref-");
                    escape_html(name, &mut out);
                    out.push_str("\" class=\"footnote-backref\" data-footnote-backref data-footnote-backref-idx=\"");
                    out.push_str(&fn_idx.to_string());
                    out.push_str("\" aria-label=\"Back to reference ");
                    out.push_str(&fn_idx.to_string());
                    out.push_str("\">↩</a></p>\n");
                }

                out.push_str("</li>\n");
            }
        }
        out.push_str("</ol>\n</section>\n");
    }

    out
}
