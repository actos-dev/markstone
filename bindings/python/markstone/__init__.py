"""markstone - Fast, safe Markdown-to-HTML and AST engine with Actos extensions."""

import sys
from ._markstone import (
    to_html,
    to_ast,
    AST_SCHEMA_VERSION,
    __version__,
    MarkstoneError,
    InputTooLargeError,
    DepthExceededError,
    InvalidUtf8Error,
    actos,
)

# Register markstone.actos in sys.modules so `import markstone.actos` works
sys.modules[f"{__name__}.actos"] = actos

__all__ = [
    "to_html",
    "to_ast",
    "AST_SCHEMA_VERSION",
    "__version__",
    "MarkstoneError",
    "InputTooLargeError",
    "DepthExceededError",
    "InvalidUtf8Error",
    "actos",
]
