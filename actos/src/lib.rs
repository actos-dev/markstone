#![forbid(unsafe_code)]

//! `markstone-actos` provides Actos extensions (@mentions and #tags)
//! on top of the markstone generic pipeline.

pub mod ast_pass;
pub mod render;

pub use ast_pass::{is_valid_tag, is_valid_username, TAG_PATTERN, USERNAME_PATTERN};
pub use markstone_core::ast::{AstDocument, Node};
pub use markstone_core::error::MarkstoneError;
pub use markstone_core::render::MAX_INPUT_SIZE;
pub use markstone_core::depth::MAX_BLOCK_DEPTH;
pub use markstone_core::AST_SCHEMA_VERSION;
pub use markstone_core as core;

/// Markstone Actos version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Converts a Markdown string into safe HTML with Actos extensions (@mentions, #tags).
///
/// Runs the generic pipeline first, then executes an AST pass for mentions and tags.
/// All tree walks and rendering are implemented strictly without recursion.
///
/// # Errors
/// Returns [`MarkstoneError::InputTooLarge`] if input exceeds 4 MiB, or
/// [`MarkstoneError::DepthExceeded`] if block nesting depth exceeds 64.
pub fn to_html(input: &str) -> Result<String, MarkstoneError> {
    let mut doc = markstone_core::parse_to_ast_document(input)?;
    ast_pass::transform_ast(&mut doc);
    Ok(render::render_html(&doc))
}

/// Convenience helper to convert UTF-8 byte slices to safe HTML with Actos extensions.
///
/// Returns [`MarkstoneError::InvalidUtf8`] if the slice is not valid UTF-8.
pub fn to_html_bytes(input: &[u8]) -> Result<String, MarkstoneError> {
    let s = std::str::from_utf8(input).map_err(|_| MarkstoneError::InvalidUtf8)?;
    to_html(s)
}

/// Converts a Markdown string into an AST JSON string with Actos extensions (@mentions, #tags).
///
/// Runs the generic pipeline first, then executes an AST pass for mentions and tags.
/// All tree walks and serialization are implemented strictly without recursion.
///
/// # Errors
/// Returns [`MarkstoneError::InputTooLarge`] if input exceeds 4 MiB, or
/// [`MarkstoneError::DepthExceeded`] if block nesting depth exceeds 64.
pub fn to_ast(input: &str) -> Result<String, MarkstoneError> {
    let mut doc = markstone_core::parse_to_ast_document(input)?;
    ast_pass::transform_ast(&mut doc);
    Ok(doc.to_json())
}

/// Convenience helper to convert UTF-8 byte slices to an AST JSON string with Actos extensions.
///
/// Returns [`MarkstoneError::InvalidUtf8`] if the slice is not valid UTF-8.
pub fn to_ast_bytes(input: &[u8]) -> Result<String, MarkstoneError> {
    let s = std::str::from_utf8(input).map_err(|_| MarkstoneError::InvalidUtf8)?;
    to_ast(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actos_version() {
        assert_eq!(VERSION, "0.1.0");
        assert_eq!(core::VERSION, "0.1.0");
        assert_eq!(AST_SCHEMA_VERSION, 1);
    }

    #[test]
    fn test_patterns() {
        assert_eq!(USERNAME_PATTERN, r"^[a-z0-9_]{3,32}$");
        assert_eq!(TAG_PATTERN, r"^[a-z0-9][a-z0-9-]{0,31}$");
    }

    #[test]
    fn test_basic_mention_and_tag() {
        let input = "Hello @alice and #rust!";
        let html = to_html(input).unwrap();
        assert_eq!(
            html,
            "<p>Hello <a href=\"/u/alice\" class=\"mention\">@alice</a> and <a href=\"/t/rust\" class=\"tag\">#rust</a>!</p>\n"
        );

        let ast = to_ast(input).unwrap();
        assert!(ast.contains(r#""type":"mention""#));
        assert!(ast.contains(r#""username":"alice""#));
        assert!(ast.contains(r#""text":"@alice""#));
        assert!(ast.contains(r#""type":"tag""#));
        assert!(ast.contains(r#""name":"rust""#));
        assert!(ast.contains(r##""text":"#rust""##));
    }
}
