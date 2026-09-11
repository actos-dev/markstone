//! Conformance test modes corresponding to the four markstone output functions.

use std::fmt;

/// The evaluation mode for a test case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Pure CommonMark + GFM HTML output (`markstone_to_html`).
    GenericHtml,
    /// Pure CommonMark + GFM AST JSON output (`markstone_to_ast`).
    GenericAst,
    /// Actos CommonMark + GFM + Mentions/Tags HTML output (`markstone_actos_to_html`).
    ActosHtml,
    /// Actos CommonMark + GFM + Mentions/Tags AST JSON output (`markstone_actos_to_ast`).
    ActosAst,
}

impl Mode {
    /// Array of all available modes.
    pub const ALL: [Mode; 4] = [
        Mode::GenericHtml,
        Mode::GenericAst,
        Mode::ActosHtml,
        Mode::ActosAst,
    ];

    /// CLI name for the mode.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Mode::GenericHtml => "generic-html",
            Mode::GenericAst => "generic-ast",
            Mode::ActosHtml => "actos-html",
            Mode::ActosAst => "actos-ast",
        }
    }

    /// Associated golden filename inside each test case directory.
    #[must_use]
    pub const fn filename(&self) -> &'static str {
        match self {
            Mode::GenericHtml => "generic.html",
            Mode::GenericAst => "generic.ast.json",
            Mode::ActosHtml => "actos.html",
            Mode::ActosAst => "actos.ast.json",
        }
    }

    /// Parse mode from string.
    #[must_use]
    pub fn parse_mode(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

/// Error returned when parsing an invalid mode name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseModeError;

impl fmt::Display for ParseModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid markstone conformance mode")
    }
}

impl std::error::Error for ParseModeError {}

impl std::str::FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "generic-html" | "generic_html" => Ok(Mode::GenericHtml),
            "generic-ast" | "generic_ast" => Ok(Mode::GenericAst),
            "actos-html" | "actos_html" => Ok(Mode::ActosHtml),
            "actos-ast" | "actos_ast" => Ok(Mode::ActosAst),
            _ => Err(ParseModeError),
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
