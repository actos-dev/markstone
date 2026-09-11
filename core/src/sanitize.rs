use std::borrow::Cow;

/// Checks if a character is an invisible or bidirectional control character
/// that must be stripped from the input before parsing.
///
/// Stripped ranges/characters:
/// - `U+200B`..=`U+200D` (Zero-width space, Zero-width non-joiner, Zero-width joiner)
/// - `U+FEFF` (Zero-width no-break space / Byte order mark)
/// - `U+2060` (Word joiner)
/// - `U+00AD` (Soft hyphen)
/// - `U+202A`..=`U+202E` (Left-to-Right Embedding, Right-to-Left Embedding, Pop Directional Formatting, Left-to-Right Override, Right-to-Left Override)
/// - `U+2066`..=`U+2069` (Left-to-Right Isolate, Right-to-Left Isolate, First Strong Isolate, Pop Directional Isolate)
#[inline]
pub const fn is_invisible_or_bidi(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'..='\u{200D}'
            | '\u{FEFF}'
            | '\u{2060}'
            | '\u{00AD}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
    )
}

/// Strips invisible and bidi control characters from the input text.
///
/// If no such characters are present, returns a borrowed reference to avoid allocations.
#[must_use]
pub fn strip_invisible_and_bidi(input: &str) -> Cow<'_, str> {
    if input.chars().any(is_invisible_or_bidi) {
        Cow::Owned(input.chars().filter(|c| !is_invisible_or_bidi(*c)).collect())
    } else {
        Cow::Borrowed(input)
    }
}

/// Kind of validated URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlKind {
    /// External web link (http, https, or protocol-relative).
    External,
    /// Mailto link.
    Mailto,
    /// Relative URL (path-relative, document-relative, query, or fragment).
    Relative,
}

/// Validates a URL destination.
///
/// Only `http`, `https`, `mailto` schemes and valid relative URLs are allowed.
/// Dangerous schemes like `javascript:`, `data:`, `vbscript:`, `file:`, etc.
/// return `None`.
#[must_use]
pub fn validate_url(raw_url: &str) -> Option<UrlKind> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Some(UrlKind::Relative);
    }

    // Filter out ASCII whitespace and control characters to inspect effective scheme
    let clean: String = trimmed
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && !c.is_ascii_control())
        .collect();

    if clean.is_empty() {
        return Some(UrlKind::Relative);
    }

    // Check for scheme: in URI syntax (RFC 3986), if ':' appears before any '/', '?', '#',
    // the preceding text is the scheme candidate.
    let colon_pos = clean.find(':');
    let slash_pos = clean.find('/');
    let question_pos = clean.find('?');
    let hash_pos = clean.find('#');

    let first_sep = [slash_pos, question_pos, hash_pos]
        .into_iter()
        .flatten()
        .min();

    if let Some(cp) = colon_pos {
        if first_sep.is_none() || cp < first_sep.unwrap() {
            let scheme = clean[..cp].to_ascii_lowercase();
            let mut chars = scheme.chars();
            let first_ok = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
            let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.');

            if first_ok && rest_ok {
                return match scheme.as_str() {
                    "http" | "https" => Some(UrlKind::External),
                    "mailto" => Some(UrlKind::Mailto),
                    _ => None, // Disallowed scheme
                };
            }
            // Invalid scheme syntax before path separator: reject as unsafe
            return None;
        }
    }

    // Protocol-relative URL (e.g. "//example.com/foo")
    if clean.starts_with("//") {
        return Some(UrlKind::External);
    }

    // Valid relative URL
    Some(UrlKind::Relative)
}

/// Sanitizes code block language tag.
///
/// Keeps only characters in `[A-Za-z0-9_+-]`.
/// Returns `None` if the sanitized language tag is empty.
#[must_use]
pub fn sanitize_code_block_lang(info: &str) -> Option<String> {
    let token = info.split_whitespace().next()?;
    let sanitized: String = token
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '+' || *c == '-')
        .collect();

    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_invisible_and_bidi() {
        let input = "a\u{200B}b\u{FEFF}c\u{2060}d\u{00AD}e\u{202A}f\u{202E}g\u{2066}h\u{2069}i";
        assert_eq!(strip_invisible_and_bidi(input), "abcdefghi");

        let clean = "Hello, world!";
        assert!(matches!(strip_invisible_and_bidi(clean), Cow::Borrowed(_)));
    }

    #[test]
    fn test_validate_url() {
        assert_eq!(validate_url("https://example.com"), Some(UrlKind::External));
        assert_eq!(validate_url("http://example.com/page"), Some(UrlKind::External));
        assert_eq!(validate_url("HTTPS://EXAMPLE.COM"), Some(UrlKind::External));
        assert_eq!(validate_url("mailto:alice@example.com"), Some(UrlKind::Mailto));
        assert_eq!(validate_url("//cdn.example.com/lib.js"), Some(UrlKind::External));
        assert_eq!(validate_url("/path/to/page"), Some(UrlKind::Relative));
        assert_eq!(validate_url("./path/to/page"), Some(UrlKind::Relative));
        assert_eq!(validate_url("../path/to/page"), Some(UrlKind::Relative));
        assert_eq!(validate_url("#heading"), Some(UrlKind::Relative));
        assert_eq!(validate_url("?query=1"), Some(UrlKind::Relative));
        assert_eq!(validate_url("path/to:file"), Some(UrlKind::Relative));

        // Rejected dangerous schemes
        assert_eq!(validate_url("javascript:alert(1)"), None);
        assert_eq!(validate_url("JAVASCRIPT:alert(1)"), None);
        assert_eq!(validate_url("  javascript:alert(1)"), None);
        assert_eq!(validate_url("java\0script:alert(1)"), None);
        assert_eq!(validate_url("data:text/html,<script>"), None);
        assert_eq!(validate_url("vbscript:msgbox(1)"), None);
        assert_eq!(validate_url("file:///etc/passwd"), None);
        assert_eq!(validate_url("blob:https://example.com"), None);
        assert_eq!(validate_url("about:blank"), None);
        assert_eq!(validate_url("ftp://example.com"), None);
    }

    #[test]
    fn test_sanitize_code_block_lang() {
        assert_eq!(sanitize_code_block_lang("rust"), Some("rust".to_string()));
        assert_eq!(sanitize_code_block_lang("c++"), Some("c++".to_string()));
        assert_eq!(sanitize_code_block_lang("c#"), Some("c".to_string()));
        assert_eq!(sanitize_code_block_lang("python extra metadata"), Some("python".to_string()));
        assert_eq!(sanitize_code_block_lang("<script>"), Some("script".to_string()));
        assert_eq!(sanitize_code_block_lang("\" onclick=\"alert(1)\""), None);
        assert_eq!(sanitize_code_block_lang(""), None);
    }
}
