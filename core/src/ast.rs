use comrak::nodes::{AstNode, ListType, NodeValue, TableAlignment};
use comrak::{Arena, parse_document};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::depth::check_block_depth;
use crate::error::MarkstoneError;
use crate::render::{MAX_INPUT_SIZE, comrak_options};
use crate::sanitize::{sanitize_code_block_lang, strip_invisible_and_bidi, validate_url};

/// Current AST JSON schema version. Increments when the schema breaks.
pub const AST_SCHEMA_VERSION: u32 = 1;

/// Top-level AST document structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstDocument {
    /// Schema version number.
    pub schema: u32,
    /// Root node of the document (always a `document` node).
    pub root: Node,
}

impl AstDocument {
    /// Serializes the AST document into a JSON string using an explicit heap stack (strictly non-recursive).
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str(r#"{"schema":"#);
        out.push_str(&self.schema.to_string());
        out.push_str(r#","root":"#);
        serialize_node_non_recursive(&self.root, &mut out);
        out.push('}');
        out
    }
}

impl fmt::Display for AstDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}

/// AST node enum representing all supported markdown constructs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
    Document {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    Paragraph {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    Heading {
        level: u8,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    BlockQuote {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    List {
        ordered: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        start: Option<u32>,
        tight: bool,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    ListItem {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    TaskItem {
        checked: bool,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    CodeBlock {
        language: String,
        value: String,
        pos: [usize; 4],
    },
    ThematicBreak {
        pos: [usize; 4],
    },
    Table {
        alignments: Vec<String>,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    TableRow {
        header: bool,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    TableCell {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    Text {
        value: String,
        pos: [usize; 4],
    },
    Emphasis {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    Strong {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    Strikethrough {
        pos: [usize; 4],
        children: Vec<Node>,
    },
    Code {
        value: String,
        pos: [usize; 4],
    },
    Link {
        url: String,
        title: String,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    Image {
        url: String,
        title: String,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    SoftBreak {
        pos: [usize; 4],
    },
    LineBreak {
        pos: [usize; 4],
    },
    FootnoteDefinition {
        name: String,
        pos: [usize; 4],
        children: Vec<Node>,
    },
    FootnoteReference {
        name: String,
        pos: [usize; 4],
    },
    Mention {
        username: String,
        text: String,
        pos: [usize; 4],
    },
    Tag {
        name: String,
        text: String,
        pos: [usize; 4],
    },
}

impl Node {
    /// Returns the 1-based source position `[start_line, start_col, end_line, end_col]`.
    #[must_use]
    pub const fn pos(&self) -> [usize; 4] {
        match self {
            Self::Document { pos, .. }
            | Self::Paragraph { pos, .. }
            | Self::Heading { pos, .. }
            | Self::BlockQuote { pos, .. }
            | Self::List { pos, .. }
            | Self::ListItem { pos, .. }
            | Self::TaskItem { pos, .. }
            | Self::CodeBlock { pos, .. }
            | Self::ThematicBreak { pos }
            | Self::Table { pos, .. }
            | Self::TableRow { pos, .. }
            | Self::TableCell { pos, .. }
            | Self::Text { pos, .. }
            | Self::Emphasis { pos, .. }
            | Self::Strong { pos, .. }
            | Self::Strikethrough { pos, .. }
            | Self::Code { pos, .. }
            | Self::Link { pos, .. }
            | Self::Image { pos, .. }
            | Self::SoftBreak { pos }
            | Self::LineBreak { pos }
            | Self::FootnoteDefinition { pos, .. }
            | Self::FootnoteReference { pos, .. }
            | Self::Mention { pos, .. }
            | Self::Tag { pos, .. } => *pos,
        }
    }

    /// Returns the string representation of this node's type.
    #[must_use]
    pub const fn node_type(&self) -> &'static str {
        match self {
            Self::Document { .. } => "document",
            Self::Paragraph { .. } => "paragraph",
            Self::Heading { .. } => "heading",
            Self::BlockQuote { .. } => "block_quote",
            Self::List { .. } => "list",
            Self::ListItem { .. } => "list_item",
            Self::TaskItem { .. } => "task_item",
            Self::CodeBlock { .. } => "code_block",
            Self::ThematicBreak { .. } => "thematic_break",
            Self::Table { .. } => "table",
            Self::TableRow { .. } => "table_row",
            Self::TableCell { .. } => "table_cell",
            Self::Text { .. } => "text",
            Self::Emphasis { .. } => "emphasis",
            Self::Strong { .. } => "strong",
            Self::Strikethrough { .. } => "strikethrough",
            Self::Code { .. } => "code",
            Self::Link { .. } => "link",
            Self::Image { .. } => "image",
            Self::SoftBreak { .. } => "soft_break",
            Self::LineBreak { .. } => "line_break",
            Self::FootnoteDefinition { .. } => "footnote_definition",
            Self::FootnoteReference { .. } => "footnote_reference",
            Self::Mention { .. } => "mention",
            Self::Tag { .. } => "tag",
        }
    }

    /// Returns a slice of children, if this node takes children.
    #[must_use]
    pub fn children(&self) -> Option<&[Self]> {
        match self {
            Self::Document { children, .. }
            | Self::Paragraph { children, .. }
            | Self::Heading { children, .. }
            | Self::BlockQuote { children, .. }
            | Self::List { children, .. }
            | Self::ListItem { children, .. }
            | Self::TaskItem { children, .. }
            | Self::Table { children, .. }
            | Self::TableRow { children, .. }
            | Self::TableCell { children, .. }
            | Self::Emphasis { children, .. }
            | Self::Strong { children, .. }
            | Self::Strikethrough { children, .. }
            | Self::Link { children, .. }
            | Self::Image { children, .. }
            | Self::FootnoteDefinition { children, .. } => Some(children),
            Self::CodeBlock { .. }
            | Self::ThematicBreak { .. }
            | Self::Text { .. }
            | Self::Code { .. }
            | Self::SoftBreak { .. }
            | Self::LineBreak { .. }
            | Self::FootnoteReference { .. }
            | Self::Mention { .. }
            | Self::Tag { .. } => None,
        }
    }

    /// Returns a mutable reference to children, if this node takes children.
    #[must_use]
    pub fn children_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Document { children, .. }
            | Self::Paragraph { children, .. }
            | Self::Heading { children, .. }
            | Self::BlockQuote { children, .. }
            | Self::List { children, .. }
            | Self::ListItem { children, .. }
            | Self::TaskItem { children, .. }
            | Self::Table { children, .. }
            | Self::TableRow { children, .. }
            | Self::TableCell { children, .. }
            | Self::Emphasis { children, .. }
            | Self::Strong { children, .. }
            | Self::Strikethrough { children, .. }
            | Self::Link { children, .. }
            | Self::Image { children, .. }
            | Self::FootnoteDefinition { children, .. } => Some(children),
            Self::CodeBlock { .. }
            | Self::ThematicBreak { .. }
            | Self::Text { .. }
            | Self::Code { .. }
            | Self::SoftBreak { .. }
            | Self::LineBreak { .. }
            | Self::FootnoteReference { .. }
            | Self::Mention { .. }
            | Self::Tag { .. } => None,
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        let mut stack = Vec::new();
        if let Some(children) = self.children_mut() {
            stack.append(children);
        }
        while let Some(mut node) = stack.pop() {
            if let Some(children) = node.children_mut() {
                stack.append(children);
            }
        }
    }
}

/// Helper to convert comrak sourcepos to 1-based `[start_line, start_col, end_line, end_col]`.
#[inline]
fn sourcepos_to_pos(sp: comrak::nodes::Sourcepos) -> [usize; 4] {
    [sp.start.line, sp.start.column, sp.end.line, sp.end.column]
}

/// Escapes a string as a valid JSON string literal.
fn write_json_str(out: &mut String, s: &str) {
    if let Ok(escaped) = serde_json::to_string(s) {
        out.push_str(&escaped);
    } else {
        out.push_str("\"\"");
    }
}

/// Appends the `,pos:[...]` field to `out`.
fn write_pos_field(out: &mut String, pos: &[usize; 4]) {
    out.push_str(r#","pos":["#);
    out.push_str(&pos[0].to_string());
    out.push(',');
    out.push_str(&pos[1].to_string());
    out.push(',');
    out.push_str(&pos[2].to_string());
    out.push(',');
    out.push_str(&pos[3].to_string());
    out.push(']');
}

/// Serializes the node tag and its non-children attributes into `out`.
fn write_node_header(node: &Node, out: &mut String) {
    out.push('{');
    out.push_str(r#""type":""#);
    out.push_str(node.node_type());
    out.push('"');

    match node {
        Node::Document { pos, .. }
        | Node::Paragraph { pos, .. }
        | Node::BlockQuote { pos, .. }
        | Node::ListItem { pos, .. }
        | Node::TableCell { pos, .. }
        | Node::Emphasis { pos, .. }
        | Node::Strong { pos, .. }
        | Node::Strikethrough { pos, .. } => {
            write_pos_field(out, pos);
        }
        Node::Heading { level, pos, .. } => {
            out.push_str(r#","level":"#);
            out.push_str(&level.to_string());
            write_pos_field(out, pos);
        }
        Node::List {
            ordered,
            start,
            tight,
            pos,
            ..
        } => {
            out.push_str(r#","ordered":"#);
            out.push_str(if *ordered { "true" } else { "false" });
            if let Some(start_val) = start {
                out.push_str(r#","start":"#);
                out.push_str(&start_val.to_string());
            }
            out.push_str(r#","tight":"#);
            out.push_str(if *tight { "true" } else { "false" });
            write_pos_field(out, pos);
        }
        Node::TaskItem { checked, pos, .. } => {
            out.push_str(r#","checked":"#);
            out.push_str(if *checked { "true" } else { "false" });
            write_pos_field(out, pos);
        }
        Node::CodeBlock {
            language,
            value,
            pos,
        } => {
            out.push_str(r#","language":"#);
            write_json_str(out, language);
            out.push_str(r#","value":"#);
            write_json_str(out, value);
            write_pos_field(out, pos);
        }
        Node::ThematicBreak { pos } | Node::SoftBreak { pos } | Node::LineBreak { pos } => {
            write_pos_field(out, pos);
        }
        Node::Table {
            alignments, pos, ..
        } => {
            out.push_str(r#","alignments":["#);
            for (i, align) in alignments.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json_str(out, align);
            }
            out.push(']');
            write_pos_field(out, pos);
        }
        Node::TableRow { header, pos, .. } => {
            out.push_str(r#","header":"#);
            out.push_str(if *header { "true" } else { "false" });
            write_pos_field(out, pos);
        }
        Node::Text { value, pos } | Node::Code { value, pos } => {
            out.push_str(r#","value":"#);
            write_json_str(out, value);
            write_pos_field(out, pos);
        }
        Node::Link {
            url, title, pos, ..
        }
        | Node::Image {
            url, title, pos, ..
        } => {
            out.push_str(r#","url":"#);
            write_json_str(out, url);
            out.push_str(r#","title":"#);
            write_json_str(out, title);
            write_pos_field(out, pos);
        }
        Node::FootnoteDefinition { name, pos, .. } => {
            out.push_str(r#","name":"#);
            write_json_str(out, name);
            write_pos_field(out, pos);
        }
        Node::FootnoteReference { name, pos } => {
            out.push_str(r#","name":"#);
            write_json_str(out, name);
            write_pos_field(out, pos);
        }
        Node::Mention {
            username,
            text,
            pos,
        } => {
            out.push_str(r#","username":"#);
            write_json_str(out, username);
            out.push_str(r#","text":"#);
            write_json_str(out, text);
            write_pos_field(out, pos);
        }
        Node::Tag { name, text, pos } => {
            out.push_str(r#","name":"#);
            write_json_str(out, name);
            out.push_str(r#","text":"#);
            write_json_str(out, text);
            write_pos_field(out, pos);
        }
    }
}

struct SerializerFrame<'a> {
    node: &'a Node,
    next_child_idx: usize,
}

/// Serializes a node and its descendants to JSON strictly without recursion.
fn serialize_node_non_recursive(root: &Node, out: &mut String) {
    let mut stack: Vec<SerializerFrame> = Vec::new();
    write_node_header(root, out);

    if root.children().is_some() {
        out.push_str(r#","children":["#);
        stack.push(SerializerFrame {
            node: root,
            next_child_idx: 0,
        });
    } else {
        out.push('}');
        return;
    }

    while let Some(frame) = stack.last_mut() {
        let children = frame.node.children().unwrap();
        if frame.next_child_idx < children.len() {
            let child = &children[frame.next_child_idx];
            if frame.next_child_idx > 0 {
                out.push(',');
            }
            frame.next_child_idx += 1;
            write_node_header(child, out);
            if child.children().is_some() {
                out.push_str(r#","children":["#);
                stack.push(SerializerFrame {
                    node: child,
                    next_child_idx: 0,
                });
            } else {
                out.push('}');
            }
        } else {
            out.push(']');
            out.push('}');
            stack.pop();
        }
    }
}

enum ConvertResult {
    Single(Node),
    Spliced(Vec<Node>),
    Dropped,
}

struct ConvertFrame<'a> {
    comrak_node: &'a AstNode<'a>,
    children: Vec<Node>,
    next_child: Option<&'a AstNode<'a>>,
}

/// Converts a comrak AST tree to a [`Node`] strictly without recursion using a heap worklist.
fn convert_comrak_to_node<'a>(root: &'a AstNode<'a>) -> Result<Node, MarkstoneError> {
    let mut stack: Vec<ConvertFrame<'a>> = Vec::new();
    stack.push(ConvertFrame {
        comrak_node: root,
        children: Vec::new(),
        next_child: root.first_child(),
    });

    let mut last_result: Option<ConvertResult> = None;

    while let Some(frame) = stack.last_mut() {
        if let Some(child) = frame.next_child {
            frame.next_child = child.next_sibling();

            // Drop raw HTML and images with disallowed URLs upfront
            match child.data().value {
                NodeValue::HtmlBlock(_) | NodeValue::HtmlInline(_) => {
                    continue;
                }
                NodeValue::Image(ref nl) if validate_url(&nl.url).is_none() => {
                    continue;
                }
                _ => {}
            }

            stack.push(ConvertFrame {
                comrak_node: child,
                children: Vec::new(),
                next_child: child.first_child(),
            });
        } else {
            let finished_frame = stack.pop().unwrap();
            let result = convert_single_node(finished_frame.comrak_node, finished_frame.children)?;

            if let Some(parent_frame) = stack.last_mut() {
                match result {
                    ConvertResult::Single(node) => parent_frame.children.push(node),
                    ConvertResult::Spliced(nodes) => parent_frame.children.extend(nodes),
                    ConvertResult::Dropped => {}
                }
            } else {
                last_result = Some(result);
            }
        }
    }

    match last_result {
        Some(ConvertResult::Single(node)) => Ok(node),
        Some(ConvertResult::Spliced(mut nodes)) => {
            if nodes.len() == 1 {
                Ok(nodes.pop().unwrap())
            } else {
                let sp = sourcepos_to_pos(root.data().sourcepos);
                Ok(Node::Document {
                    pos: sp,
                    children: nodes,
                })
            }
        }
        _ => {
            let sp = sourcepos_to_pos(root.data().sourcepos);
            Ok(Node::Document {
                pos: sp,
                children: Vec::new(),
            })
        }
    }
}

/// Converts a single finished comrak node and its collected child nodes into a [`ConvertResult`].
fn convert_single_node<'a>(
    cnode: &'a AstNode<'a>,
    children: Vec<Node>,
) -> Result<ConvertResult, MarkstoneError> {
    let pos = sourcepos_to_pos(cnode.data().sourcepos);
    match cnode.data().value {
        NodeValue::Document => Ok(ConvertResult::Single(Node::Document { pos, children })),
        NodeValue::Paragraph => Ok(ConvertResult::Single(Node::Paragraph { pos, children })),
        NodeValue::Heading(ref nh) => Ok(ConvertResult::Single(Node::Heading {
            level: nh.level,
            pos,
            children,
        })),
        NodeValue::BlockQuote => Ok(ConvertResult::Single(Node::BlockQuote { pos, children })),
        NodeValue::List(ref nl) => {
            let ordered = nl.list_type == ListType::Ordered;
            let start = if ordered { Some(nl.start as u32) } else { None };
            Ok(ConvertResult::Single(Node::List {
                ordered,
                start,
                tight: nl.tight,
                pos,
                children,
            }))
        }
        NodeValue::Item(_) => Ok(ConvertResult::Single(Node::ListItem { pos, children })),
        NodeValue::TaskItem(ref nti) => Ok(ConvertResult::Single(Node::TaskItem {
            checked: nti.symbol.is_some(),
            pos,
            children,
        })),
        NodeValue::CodeBlock(ref ncb) => {
            let language = sanitize_code_block_lang(&ncb.info).unwrap_or_default();
            let value = ncb.literal.clone();
            Ok(ConvertResult::Single(Node::CodeBlock {
                language,
                value,
                pos,
            }))
        }
        NodeValue::ThematicBreak => Ok(ConvertResult::Single(Node::ThematicBreak { pos })),
        NodeValue::Table(ref nt) => {
            let alignments = nt
                .alignments
                .iter()
                .map(|a| {
                    match a {
                        TableAlignment::None => "none",
                        TableAlignment::Left => "left",
                        TableAlignment::Center => "center",
                        TableAlignment::Right => "right",
                    }
                    .to_string()
                })
                .collect();
            Ok(ConvertResult::Single(Node::Table {
                alignments,
                pos,
                children,
            }))
        }
        NodeValue::TableRow(header) => Ok(ConvertResult::Single(Node::TableRow {
            header,
            pos,
            children,
        })),
        NodeValue::TableCell => Ok(ConvertResult::Single(Node::TableCell { pos, children })),
        NodeValue::Text(ref text) => Ok(ConvertResult::Single(Node::Text {
            value: text.to_string(),
            pos,
        })),
        NodeValue::Emph => Ok(ConvertResult::Single(Node::Emphasis { pos, children })),
        NodeValue::Strong => Ok(ConvertResult::Single(Node::Strong { pos, children })),
        NodeValue::Strikethrough => {
            Ok(ConvertResult::Single(Node::Strikethrough { pos, children }))
        }
        NodeValue::Code(ref nc) => Ok(ConvertResult::Single(Node::Code {
            value: nc.literal.clone(),
            pos,
        })),
        NodeValue::Link(ref nl) => {
            match validate_url(&nl.url) {
                Some(_) => Ok(ConvertResult::Single(Node::Link {
                    url: nl.url.clone(),
                    title: nl.title.clone(),
                    pos,
                    children,
                })),
                None => {
                    // Disallowed link: degrade to plain text (omit link node, splice children)
                    Ok(ConvertResult::Spliced(children))
                }
            }
        }
        NodeValue::Image(ref nl) => {
            match validate_url(&nl.url) {
                Some(_) => Ok(ConvertResult::Single(Node::Image {
                    url: nl.url.clone(),
                    title: nl.title.clone(),
                    pos,
                    children,
                })),
                None => {
                    // Disallowed image: drop completely
                    Ok(ConvertResult::Dropped)
                }
            }
        }
        NodeValue::SoftBreak => Ok(ConvertResult::Single(Node::SoftBreak { pos })),
        NodeValue::LineBreak => Ok(ConvertResult::Single(Node::LineBreak { pos })),
        NodeValue::FootnoteDefinition(ref nfd) => {
            Ok(ConvertResult::Single(Node::FootnoteDefinition {
                name: nfd.name.clone(),
                pos,
                children,
            }))
        }
        NodeValue::FootnoteReference(ref nfr) => {
            Ok(ConvertResult::Single(Node::FootnoteReference {
                name: nfr.name.clone(),
                pos,
            }))
        }
        NodeValue::HtmlBlock(_) | NodeValue::HtmlInline(_) => Ok(ConvertResult::Dropped),
        _ => {
            if children.is_empty() {
                Ok(ConvertResult::Dropped)
            } else {
                Ok(ConvertResult::Spliced(children))
            }
        }
    }
}

/// Parses Markdown input and converts it into an [`AstDocument`].
///
/// # Security & Limits
/// - Input size limited to 4 MiB ([`MarkstoneError::InputTooLarge`]).
/// - Block nesting depth limited to 64 ([`MarkstoneError::DepthExceeded`]).
/// - Invisible and bidi control characters stripped before parsing.
/// - Tree walk and AST conversion implemented iteratively without recursion.
/// - Raw HTML blocks and inline raw HTML completely dropped.
/// - Links and images validated against safe URL schemes. Disallowed links degrade to
///   plain text; disallowed images are dropped.
/// - Code block languages sanitized to `[A-Za-z0-9_+-]`.
/// - Text nodes carry unescaped raw text.
///
/// # Errors
/// Returns [`MarkstoneError::InputTooLarge`] if input exceeds 4 MiB, or
/// [`MarkstoneError::DepthExceeded`] if block nesting depth exceeds 64.
pub fn parse_to_ast_document(input: &str) -> Result<AstDocument, MarkstoneError> {
    if input.len() > MAX_INPUT_SIZE {
        return Err(MarkstoneError::InputTooLarge);
    }

    let sanitized = strip_invisible_and_bidi(input);
    let options = comrak_options();
    let arena = Arena::new();
    let root = parse_document(&arena, &sanitized, &options);

    check_block_depth(root)?;

    let ast_root = convert_comrak_to_node(root)?;

    Ok(AstDocument {
        schema: AST_SCHEMA_VERSION,
        root: ast_root,
    })
}

/// Converts a Markdown string into an AST JSON string.
///
/// # Security Guarantees
/// - Input size limited to 4 MiB ([`MarkstoneError::InputTooLarge`]).
/// - Block nesting depth limited to 64 ([`MarkstoneError::DepthExceeded`]).
/// - Invisible and bidi control characters stripped before parsing.
/// - Tree conversion and JSON serialization implemented iteratively without recursion.
/// - Disallowed links degrade to plain text; disallowed images and raw HTML are dropped.
/// - Text nodes contain unescaped raw text.
///
/// # Errors
/// Returns [`MarkstoneError::InputTooLarge`] if input exceeds 4 MiB, or
/// [`MarkstoneError::DepthExceeded`] if block nesting depth exceeds 64.
pub fn to_ast(input: &str) -> Result<String, MarkstoneError> {
    let doc = parse_to_ast_document(input)?;
    Ok(doc.to_json())
}

/// Convenience helper to convert UTF-8 byte slices to an AST JSON string.
///
/// Returns [`MarkstoneError::InvalidUtf8`] if the slice is not valid UTF-8.
pub fn to_ast_bytes(input: &[u8]) -> Result<String, MarkstoneError> {
    let s = std::str::from_utf8(input).map_err(|_| MarkstoneError::InvalidUtf8)?;
    to_ast(s)
}
