use napi::Error;
use napi_derive::napi;

fn map_error(err: markstone_core::MarkstoneError) -> Error<&'static str> {
    let code = match &err {
        markstone_core::MarkstoneError::InputTooLarge => "INPUT_TOO_LARGE",
        markstone_core::MarkstoneError::DepthExceeded => "DEPTH_EXCEEDED",
        markstone_core::MarkstoneError::InvalidUtf8 => "INVALID_UTF8",
        markstone_core::MarkstoneError::Internal(_) => "INTERNAL",
    };
    Error::new(code, err.to_string())
}

#[napi(js_name = "toHtml")]
pub fn to_html(input: String) -> Result<String, Error<&'static str>> {
    markstone_core::to_html(&input).map_err(map_error)
}

#[napi(js_name = "to_html")]
pub fn to_html_snake(input: String) -> Result<String, Error<&'static str>> {
    to_html(input)
}

#[napi(js_name = "toAst")]
pub fn to_ast(input: String) -> Result<String, Error<&'static str>> {
    markstone_core::to_ast(&input).map_err(map_error)
}

#[napi(js_name = "to_ast")]
pub fn to_ast_snake(input: String) -> Result<String, Error<&'static str>> {
    to_ast(input)
}

#[napi(namespace = "actos", js_name = "toHtml")]
pub fn actos_to_html(input: String) -> Result<String, Error<&'static str>> {
    markstone_actos::to_html(&input).map_err(map_error)
}

#[napi(namespace = "actos", js_name = "to_html")]
pub fn actos_to_html_snake(input: String) -> Result<String, Error<&'static str>> {
    actos_to_html(input)
}

#[napi(namespace = "actos", js_name = "toAst")]
pub fn actos_to_ast(input: String) -> Result<String, Error<&'static str>> {
    markstone_actos::to_ast(&input).map_err(map_error)
}

#[napi(namespace = "actos", js_name = "to_ast")]
pub fn actos_to_ast_snake(input: String) -> Result<String, Error<&'static str>> {
    actos_to_ast(input)
}

#[napi(js_name = "astSchemaVersion")]
pub fn ast_schema_version() -> u32 {
    markstone_core::AST_SCHEMA_VERSION
}

#[napi(js_name = "ast_schema_version")]
pub fn ast_schema_version_snake() -> u32 {
    markstone_core::AST_SCHEMA_VERSION
}

#[napi(js_name = "AST_SCHEMA_VERSION")]
pub const AST_SCHEMA_VERSION: u32 = markstone_core::AST_SCHEMA_VERSION;

#[napi(js_name = "version")]
pub fn version() -> String {
    markstone_core::VERSION.to_string()
}
