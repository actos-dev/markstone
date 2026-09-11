//! Byte-for-byte difference detection and formatting for conformance validation.

/// Details about a mismatch between expected and actual byte sequences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteDiff {
    pub expected_len: usize,
    pub actual_len: usize,
    pub mismatch_offset: usize,
    pub expected_byte: Option<u8>,
    pub actual_byte: Option<u8>,
    pub line: usize,
    pub col: usize,
    pub context_expected: String,
    pub context_actual: String,
}

impl ByteDiff {
    /// Formats a comprehensive visual report detailing the byte diff.
    #[must_use]
    pub fn format_report(&self, case_name: &str, mode_name: &str) -> String {
        let mut s = String::new();
        s.push_str(
            "================================================================================\n",
        );
        s.push_str(&format!(
            "CONFORMANCE MISMATCH: case '{case_name}' [{mode_name}]\n"
        ));
        s.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        s.push_str(&format!(
            "Byte lengths: expected {} bytes, got {} bytes\n",
            self.expected_len, self.actual_len
        ));
        s.push_str(&format!(
            "First mismatch at byte offset {} (line {}, column {}):\n",
            self.mismatch_offset, self.line, self.col
        ));

        let exp_repr = match self.expected_byte {
            Some(b) => format!("{b:#04x} ({})", escape_byte(b)),
            None => "<EOF>".to_string(),
        };
        let act_repr = match self.actual_byte {
            Some(b) => format!("{b:#04x} ({})", escape_byte(b)),
            None => "<EOF>".to_string(),
        };

        s.push_str(&format!("  Expected byte: {exp_repr}\n"));
        s.push_str(&format!("  Actual byte:   {act_repr}\n\n"));

        s.push_str("Context (around mismatch offset):\n");
        s.push_str(&format!("  Expected: {:?}\n", self.context_expected));
        s.push_str(&format!("  Actual:   {:?}\n", self.context_actual));
        s.push_str(
            "================================================================================\n",
        );
        s
    }
}

fn escape_byte(b: u8) -> String {
    match b {
        b'\n' => "\\n".to_string(),
        b'\r' => "\\r".to_string(),
        b'\t' => "\\t".to_string(),
        b'\\' => "\\\\".to_string(),
        0x20..=0x7E => format!("'{}'", b as char),
        _ => format!("\\x{b:02x}"),
    }
}

/// Compares two byte slices byte-for-byte.
///
/// Returns `Ok(())` if identical, or `Err(ByteDiff)` with details about the first divergence.
pub fn compare_bytes(expected: &[u8], actual: &[u8]) -> Result<(), ByteDiff> {
    if expected == actual {
        return Ok(());
    }

    let min_len = expected.len().min(actual.len());
    let mut mismatch_offset = min_len;

    for i in 0..min_len {
        if expected[i] != actual[i] {
            mismatch_offset = i;
            break;
        }
    }

    let expected_byte = expected.get(mismatch_offset).copied();
    let actual_byte = actual.get(mismatch_offset).copied();

    // Compute line and column at mismatch_offset in expected (or actual if expected is shorter)
    let source_for_pos = if mismatch_offset < expected.len() {
        expected
    } else {
        actual
    };

    let mut line = 1;
    let mut col = 1;
    for &b in &source_for_pos[..mismatch_offset] {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    // Extract context windows around mismatch_offset
    let start_exp = mismatch_offset.saturating_sub(25);
    let end_exp = (mismatch_offset + 25).min(expected.len());
    let context_expected = String::from_utf8_lossy(&expected[start_exp..end_exp]).into_owned();

    let start_act = mismatch_offset.saturating_sub(25);
    let end_act = (mismatch_offset + 25).min(actual.len());
    let context_actual = String::from_utf8_lossy(&actual[start_act..end_act]).into_owned();

    Err(ByteDiff {
        expected_len: expected.len(),
        actual_len: actual.len(),
        mismatch_offset,
        expected_byte,
        actual_byte,
        line,
        col,
        context_expected,
        context_actual,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_bytes() {
        let a = b"<h1>Hello</h1>\n";
        let b = b"<h1>Hello</h1>\n";
        assert!(compare_bytes(a, b).is_ok());
    }

    #[test]
    fn test_mismatch_in_middle() {
        let a = b"<h1>Hello</h1>\n";
        let b = b"<h1>Hella</h1>\n";
        let err = compare_bytes(a, b).unwrap_err();
        assert_eq!(err.mismatch_offset, 8);
        assert_eq!(err.expected_byte, Some(b'o'));
        assert_eq!(err.actual_byte, Some(b'a'));
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 9);
    }

    #[test]
    fn test_length_truncation() {
        let a = b"hello world";
        let b = b"hello";
        let err = compare_bytes(a, b).unwrap_err();
        assert_eq!(err.mismatch_offset, 5);
        assert_eq!(err.expected_byte, Some(b' '));
        assert_eq!(err.actual_byte, None);
    }
}
