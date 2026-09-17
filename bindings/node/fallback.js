// Entry point for Node and other non-browser runtimes: the native addon when
// this platform has one, the WebAssembly build otherwise.
let binding;

try {
  binding = await import('./index.js');
} catch (_) {
  // No addon for this platform, or it failed to load.
}

if (!binding) {
  binding = await import('./browser.js');
  if (typeof process !== 'undefined' && process.versions?.node) {
    const { readFileSync } = await import('node:fs');
    const wasm = readFileSync(new URL('./wasm/markstone_wasm_bg.wasm', import.meta.url));
    binding.initSync({ module: wasm });
  }
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
