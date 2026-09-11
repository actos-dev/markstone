#![forbid(unsafe_code)]

//! `markstone-core` provides pure CommonMark + GFM parsing, AST manipulation,
//! sanitization, and non-recursive safe HTML rendering.

pub mod depth;
pub mod error;
pub mod render;
pub mod sanitize;

pub use depth::MAX_BLOCK_DEPTH;
pub use error::MarkstoneError;
pub use render::{to_html, to_html_bytes, MAX_INPUT_SIZE};
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
}
