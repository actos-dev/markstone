use std::borrow::Cow;
use pyo3::exceptions::{PyException, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

pyo3::create_exception!(markstone, MarkstoneError, PyException);
pyo3::create_exception!(markstone, InputTooLargeError, MarkstoneError);
pyo3::create_exception!(markstone, DepthExceededError, MarkstoneError);
pyo3::create_exception!(markstone, InvalidUtf8Error, MarkstoneError);

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AST_SCHEMA_VERSION: u32 = markstone_core::AST_SCHEMA_VERSION;

fn map_markstone_error(err: markstone_core::MarkstoneError) -> PyErr {
    match err {
        markstone_core::MarkstoneError::InputTooLarge => {
            InputTooLargeError::new_err("input exceeds maximum size of 4 MiB")
        }
        markstone_core::MarkstoneError::DepthExceeded => {
            DepthExceededError::new_err("block nesting depth exceeds maximum of 64")
        }
        markstone_core::MarkstoneError::InvalidUtf8 => {
            InvalidUtf8Error::new_err("invalid UTF-8 input")
        }
        markstone_core::MarkstoneError::Internal(msg) => {
            MarkstoneError::new_err(format!("internal error: {msg}"))
        }
    }
}

fn extract_input<'a>(input: &'a Bound<'_, PyAny>) -> PyResult<Cow<'a, str>> {
    if let Ok(cow) = input.extract::<Cow<'a, str>>() {
        Ok(cow)
    } else if let Ok(py_bytes) = input.downcast::<PyBytes>() {
        let s = std::str::from_utf8(py_bytes.as_bytes()).map_err(|_| {
            InvalidUtf8Error::new_err("invalid UTF-8 byte sequence")
        })?;
        Ok(Cow::Borrowed(s))
    } else {
        Err(PyTypeError::new_err("input must be a string or bytes"))
    }
}

#[pyfunction]
#[pyo3(name = "to_html")]
fn py_to_html(input: &Bound<'_, PyAny>) -> PyResult<String> {
    let s = extract_input(input)?;
    markstone_core::to_html(&s).map_err(map_markstone_error)
}

#[pyfunction]
#[pyo3(name = "to_ast")]
fn py_to_ast(input: &Bound<'_, PyAny>) -> PyResult<String> {
    let s = extract_input(input)?;
    markstone_core::to_ast(&s).map_err(map_markstone_error)
}

#[pyfunction]
#[pyo3(name = "to_html")]
fn py_actos_to_html(input: &Bound<'_, PyAny>) -> PyResult<String> {
    let s = extract_input(input)?;
    markstone_actos::to_html(&s).map_err(map_markstone_error)
}

#[pyfunction]
#[pyo3(name = "to_ast")]
fn py_actos_to_ast(input: &Bound<'_, PyAny>) -> PyResult<String> {
    let s = extract_input(input)?;
    markstone_actos::to_ast(&s).map_err(map_markstone_error)
}

#[pymodule]
fn _markstone(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", VERSION)?;
    m.add("AST_SCHEMA_VERSION", AST_SCHEMA_VERSION)?;

    m.add("MarkstoneError", m.py().get_type::<MarkstoneError>())?;
    m.add("InputTooLargeError", m.py().get_type::<InputTooLargeError>())?;
    m.add("DepthExceededError", m.py().get_type::<DepthExceededError>())?;
    m.add("InvalidUtf8Error", m.py().get_type::<InvalidUtf8Error>())?;

    m.add_function(wrap_pyfunction!(py_to_html, m)?)?;
    m.add_function(wrap_pyfunction!(py_to_ast, m)?)?;

    // Submodule `actos`
    let actos_mod = PyModule::new(m.py(), "actos")?;
    actos_mod.add_function(wrap_pyfunction!(py_actos_to_html, &actos_mod)?)?;
    actos_mod.add_function(wrap_pyfunction!(py_actos_to_ast, &actos_mod)?)?;
    actos_mod.add("__version__", VERSION)?;
    actos_mod.add("AST_SCHEMA_VERSION", AST_SCHEMA_VERSION)?;
    m.add_submodule(&actos_mod)?;

    Ok(())
}
