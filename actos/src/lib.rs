#![forbid(unsafe_code)]

//! `markstone-actos` provides Actos extensions (@mentions and #tags)
//! on top of the markstone generic pipeline.

pub use markstone_core as core;

/// Markstone Actos version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actos_version() {
        assert_eq!(VERSION, "0.1.0");
        assert_eq!(core::VERSION, "0.1.0");
    }
}
