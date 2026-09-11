#![forbid(unsafe_code)]

//! `markstone-core` provides pure CommonMark + GFM parsing, AST manipulation,
//! sanitization, and non-recursive safe HTML rendering.

pub mod ast;
pub mod depth;
pub mod error;
pub mod render;
pub mod sanitize;

pub use ast::{
    parse_to_ast_document, to_ast, to_ast_bytes, AST_SCHEMA_VERSION, AstDocument, Node,
};
pub use depth::MAX_BLOCK_DEPTH;
pub use error::MarkstoneError;
pub use render::{comrak_options, to_html, to_html_bytes, MAX_INPUT_SIZE};
pub use sanitize::{is_invisible_or_bidi, strip_invisible_and_bidi, validate_url, UrlKind};

/// Markstone core version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_basic_to_html() {
        let html = to_html("# Hello\n\nThis is a test.").unwrap();
        assert_eq!(html, "<h1>Hello</h1>\n<p>This is a test.</p>\n");
    }

    #[test]
    fn test_basic_to_ast() {
        let ast_json = to_ast("# Hello\n\nThis is a test.").unwrap();
        assert!(ast_json.contains(r#""schema":1"#));
        assert!(ast_json.contains(r#""type":"heading""#));
        assert!(ast_json.contains(r#""level":1"#));
        assert!(ast_json.contains(r#""type":"text""#));
        assert!(ast_json.contains(r#""value":"Hello""#));
        let doc: AstDocument = serde_json::from_str(&ast_json).unwrap();
        assert_eq!(doc.schema, 1);
        assert_eq!(doc.root.node_type(), "document");
    }
}
