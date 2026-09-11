use wasm_bindgen::prelude::*;

fn map_markstone_error(err: markstone_core::MarkstoneError) -> JsValue {
    let code = match &err {
        markstone_core::MarkstoneError::InputTooLarge => "INPUT_TOO_LARGE",
        markstone_core::MarkstoneError::DepthExceeded => "DEPTH_EXCEEDED",
        markstone_core::MarkstoneError::InvalidUtf8 => "INVALID_UTF8",
        markstone_core::MarkstoneError::Internal(_) => "INTERNAL",
    };
    let js_err = js_sys::Error::new(&err.to_string());
    let _ = js_sys::Reflect::set(
        &js_err,
        &JsValue::from_str("code"),
        &JsValue::from_str(code),
    );
    JsValue::from(js_err)
}

#[wasm_bindgen]
pub fn to_html(input: &str) -> Result<String, JsValue> {
    markstone_core::to_html(input).map_err(map_markstone_error)
}

#[wasm_bindgen(js_name = toHtml)]
pub fn to_html_camel(input: &str) -> Result<String, JsValue> {
    to_html(input)
}

#[wasm_bindgen]
pub fn to_ast(input: &str) -> Result<String, JsValue> {
    markstone_core::to_ast(input).map_err(map_markstone_error)
}

#[wasm_bindgen(js_name = toAst)]
pub fn to_ast_camel(input: &str) -> Result<String, JsValue> {
    to_ast(input)
}

#[wasm_bindgen]
pub fn actos_to_html(input: &str) -> Result<String, JsValue> {
    markstone_actos::to_html(input).map_err(map_markstone_error)
}

#[wasm_bindgen(js_name = actosToHtml)]
pub fn actos_to_html_camel(input: &str) -> Result<String, JsValue> {
    actos_to_html(input)
}

#[wasm_bindgen]
pub fn actos_to_ast(input: &str) -> Result<String, JsValue> {
    markstone_actos::to_ast(input).map_err(map_markstone_error)
}

#[wasm_bindgen(js_name = actosToAst)]
pub fn actos_to_ast_camel(input: &str) -> Result<String, JsValue> {
    actos_to_ast(input)
}

#[wasm_bindgen]
pub fn ast_schema_version() -> u32 {
    markstone_core::AST_SCHEMA_VERSION
}

#[wasm_bindgen(js_name = astSchemaVersion)]
pub fn ast_schema_version_camel() -> u32 {
    ast_schema_version()
}

#[wasm_bindgen]
pub fn version() -> String {
    markstone_core::VERSION.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_version() {
        assert_eq!(version(), "0.1.0");
        assert_eq!(ast_schema_version(), 1);
    }

    #[test]
    fn test_wasm_to_html() {
        let html = to_html("# Hello").unwrap();
        assert_eq!(html, "<h1>Hello</h1>\n");
    }

    #[test]
    fn test_wasm_actos_to_html() {
        let html = actos_to_html("Hello @alice").unwrap();
        assert!(html.contains(r#"<a href="/u/alice" class="mention">@alice</a>"#));
    }
}
