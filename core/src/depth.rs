use comrak::nodes::AstNode;
use crate::error::MarkstoneError;

/// Maximum permitted block nesting depth.
pub const MAX_BLOCK_DEPTH: usize = 64;

/// Iteratively checks that the block nesting depth in the AST does not exceed
/// [`MAX_BLOCK_DEPTH`].
///
/// Implemented strictly without recursion using an explicit heap-allocated worklist.
/// Even inputs with hundreds of levels of nesting will not blow the call stack.
pub fn check_block_depth<'a>(root: &'a AstNode<'a>) -> Result<(), MarkstoneError> {
    let mut stack = Vec::new();

    // Document is the root container (depth 0).
    for child in root.children() {
        let depth = if child.data().value.block() { 1 } else { 0 };
        if depth > MAX_BLOCK_DEPTH {
            return Err(MarkstoneError::DepthExceeded);
        }
        stack.push((child, depth));
    }

    while let Some((node, current_depth)) = stack.pop() {
        for child in node.children() {
            let next_depth = if child.data().value.block() {
                current_depth + 1
            } else {
                current_depth
            };
            if next_depth > MAX_BLOCK_DEPTH {
                return Err(MarkstoneError::DepthExceeded);
            }
            stack.push((child, next_depth));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{parse_document, Arena, Options};

    #[test]
    fn test_depth_within_limit() {
        let arena = Arena::new();
        let opts = Options::default();
        // A single paragraph has block depth 1
        let doc = parse_document(&arena, "Hello, world!", &opts);
        assert!(check_block_depth(doc).is_ok());

        // 10 nested blockquotes + 1 paragraph = depth 11 <= 64
        let input = "> ".repeat(10) + "nested";
        let doc = parse_document(&arena, &input, &opts);
        assert!(check_block_depth(doc).is_ok());

        // 63 nested blockquotes + 1 paragraph = depth 64 <= 64
        let input = "> ".repeat(63) + "nested";
        let doc = parse_document(&arena, &input, &opts);
        assert!(check_block_depth(doc).is_ok());
    }

    #[test]
    fn test_depth_exceeded() {
        let arena = Arena::new();
        let opts = Options::default();

        // 64 nested blockquotes + 1 paragraph = depth 65 > 64
        let input = "> ".repeat(64) + "nested";
        let doc = parse_document(&arena, &input, &opts);
        assert_eq!(check_block_depth(doc), Err(MarkstoneError::DepthExceeded));

        // 200 nested blockquotes - verified without stack overflow
        let input = "> ".repeat(200) + "deep";
        let doc = parse_document(&arena, &input, &opts);
        assert_eq!(check_block_depth(doc), Err(MarkstoneError::DepthExceeded));
    }
}
