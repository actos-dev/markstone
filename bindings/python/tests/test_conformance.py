"""Pytest conformance and behavioral test suite for markstone Python bindings."""

import os
from pathlib import Path
import pytest
import markstone
import markstone.actos
from markstone import (
    to_html,
    to_ast,
    AST_SCHEMA_VERSION,
    __version__,
    MarkstoneError,
    InputTooLargeError,
    DepthExceededError,
    InvalidUtf8Error,
)
from markstone.actos import (
    to_html as actos_to_html,
    to_ast as actos_to_ast,
)


def find_cases_dir() -> Path:
    current = Path(__file__).resolve().parent
    while current.parent != current:
        cases_dir = current / "conformance" / "cases"
        if cases_dir.is_dir():
            return cases_dir
        current = current.parent
    raise FileNotFoundError("Could not find conformance/cases directory")


CASES_DIR = find_cases_dir()
CASE_DIRS = sorted([p for p in CASES_DIR.iterdir() if p.is_dir() and (p / "input.md").is_file()])


def test_conformance_cases_count():
    assert len(CASE_DIRS) == 77, f"Expected 77 cases, found {len(CASE_DIRS)}"


@pytest.mark.parametrize("case_dir", CASE_DIRS, ids=lambda p: p.name)
def test_case_conformance(case_dir: Path):
    input_text = (case_dir / "input.md").read_text(encoding="utf-8")

    # 1. generic-html
    expected_generic_html = (case_dir / "generic.html").read_bytes()
    actual_generic_html = to_html(input_text).encode("utf-8")
    assert actual_generic_html == expected_generic_html, (
        f"Mismatch in {case_dir.name} generic-html"
    )

    # 2. generic-ast
    expected_generic_ast = (case_dir / "generic.ast.json").read_bytes()
    actual_generic_ast = to_ast(input_text).encode("utf-8")
    assert actual_generic_ast == expected_generic_ast, (
        f"Mismatch in {case_dir.name} generic-ast"
    )

    # 3. actos-html
    expected_actos_html = (case_dir / "actos.html").read_bytes()
    actual_actos_html = actos_to_html(input_text).encode("utf-8")
    assert actual_actos_html == expected_actos_html, (
        f"Mismatch in {case_dir.name} actos-html"
    )

    # 4. actos-ast
    expected_actos_ast = (case_dir / "actos.ast.json").read_bytes()
    actual_actos_ast = actos_to_ast(input_text).encode("utf-8")
    assert actual_actos_ast == expected_actos_ast, (
        f"Mismatch in {case_dir.name} actos-ast"
    )


def test_constants_and_metadata():
    assert __version__ == "0.1.0"
    assert AST_SCHEMA_VERSION == 1
    assert markstone.actos.__version__ == "0.1.0"
    assert markstone.actos.AST_SCHEMA_VERSION == 1


def test_exception_hierarchy():
    assert issubclass(InputTooLargeError, MarkstoneError)
    assert issubclass(DepthExceededError, MarkstoneError)
    assert issubclass(InvalidUtf8Error, MarkstoneError)
    assert issubclass(MarkstoneError, Exception)


def test_input_too_large_error():
    # 4 MiB limit: 4 * 1024 * 1024 + 1
    too_large = "a" * (4 * 1024 * 1024 + 1)
    
    with pytest.raises(InputTooLargeError):
        to_html(too_large)

    with pytest.raises(InputTooLargeError):
        to_ast(too_large)

    with pytest.raises(InputTooLargeError):
        actos_to_html(too_large)

    with pytest.raises(InputTooLargeError):
        actos_to_ast(too_large)


def test_depth_exceeded_error():
    # 64 block nesting depth limit: 65 blockquotes
    too_deep = "> " * 65 + "deep text\n"

    with pytest.raises(DepthExceededError):
        to_html(too_deep)

    with pytest.raises(DepthExceededError):
        to_ast(too_deep)

    with pytest.raises(DepthExceededError):
        actos_to_html(too_deep)

    with pytest.raises(DepthExceededError):
        actos_to_ast(too_deep)


def test_invalid_utf8_error():
    invalid_bytes = b"\xff\xfe\xfd"

    with pytest.raises(InvalidUtf8Error):
        to_html(invalid_bytes)

    with pytest.raises(InvalidUtf8Error):
        to_ast(invalid_bytes)

    with pytest.raises(InvalidUtf8Error):
        actos_to_html(invalid_bytes)

    with pytest.raises(InvalidUtf8Error):
        actos_to_ast(invalid_bytes)


def test_bytes_input_support():
    valid_bytes = b"# Hello from bytes\n\nContent here."
    expected_html = "<h1>Hello from bytes</h1>\n<p>Content here.</p>\n"
    assert to_html(valid_bytes) == expected_html
    assert actos_to_html(valid_bytes) == expected_html
    assert '"type":"heading"' in to_ast(valid_bytes)
    assert '"type":"heading"' in actos_to_ast(valid_bytes)


def test_type_error_on_invalid_argument_type():
    with pytest.raises(TypeError):
        to_html(12345)  # type: ignore

    with pytest.raises(TypeError):
        actos_to_html(None)  # type: ignore
