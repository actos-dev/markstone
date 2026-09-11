let binding;

try {
  binding = await import('./index.js');
} catch (_) {
  // Native addon failed to load or unsupported platform; fallback to wasm
}

if (!binding) {
  binding = await import('./browser.js');
}

export const {
  toHtml,
  to_html,
  toAst,
  to_ast,
  astSchemaVersion,
  ast_schema_version,
  AST_SCHEMA_VERSION,
  version,
  actos,
  init,
  initSync,
} = binding;

export default binding.default || binding;
