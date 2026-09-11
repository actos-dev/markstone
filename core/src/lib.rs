#![forbid(unsafe_code)]

//! `markstone-core` provides pure CommonMark + GFM parsing, AST manipulation,
//! sanitization, and rendering.

/// Markstone core version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}
