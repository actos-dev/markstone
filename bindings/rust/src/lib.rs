#![forbid(unsafe_code)]

//! # markstone
//!
//! Fast, safe CommonMark + GFM markdown engine with Actos extensions.
//!
//! This crate is the primary Rust interface to markstone. It is a thin, idiomatic
//! wrapper directly over `markstone-core` and `markstone-actos` (no FFI overhead).
//!
//! ## Example
//!
//! ```rust
//! use markstone::{to_html, to_ast};
//!
//! let html = to_html("# Hello world").unwrap();
//! assert_eq!(html, "<h1>Hello world</h1>\n");
//!
//! let ast = to_ast("# Hello world").unwrap();
//! assert!(ast.contains(r#""type":"heading""#));
//!
//! // Actos extensions (@mentions, #tags):
//! let actos_html = markstone::actos::to_html("Hello @alice").unwrap();
//! assert!(actos_html.contains(r#"<a href="/u/alice" class="mention">@alice</a>"#));
//! ```

pub use markstone_core::ast::{AST_SCHEMA_VERSION, to_ast, to_ast_bytes};
pub use markstone_core::error::MarkstoneError;
pub use markstone_core::render::{to_html, to_html_bytes};

/// Actos-specific extensions (@mentions and #tags).
pub mod actos {
    pub use markstone_actos::{to_ast, to_ast_bytes, to_html, to_html_bytes};
}

/// markstone package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
