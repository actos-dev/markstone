import initWasm, {
  initSync as wasmInitSync,
  toHtml as wasmToHtml,
  to_html as wasm_to_html,
  toAst as wasmToAst,
  to_ast as wasm_to_ast,
  actosToHtml as wasmActosToHtml,
  actos_to_html as wasm_actos_to_html,
  actosToAst as wasmActosToAst,
  actos_to_ast as wasm_actos_to_ast,
  astSchemaVersion as wasmAstSchemaVersion,
  ast_schema_version as wasm_ast_schema_version,
  version as wasmVersion,
} from './wasm/markstone_wasm.js';

let isInitialized = false;

function ensureInitialized() {
  if (isInitialized) return;
  // If in Node.js, attempt synchronous auto-initialization from local wasm file
  if (typeof process !== 'undefined' && process.versions && process.versions.node) {
    try {
      // In CommonJS or Node environments with require
      const fs = typeof require !== 'undefined' ? require('node:fs') : null;
      const path = typeof require !== 'undefined' ? require('node:path') : null;
      if (fs && path) {
        const wasmPath = path.join(__dirname, 'wasm', 'markstone_wasm_bg.wasm');
        if (fs.existsSync(wasmPath)) {
          wasmInitSync({ module: fs.readFileSync(wasmPath) });
          isInitialized = true;
          return;
        }
      }
    } catch (_) {
      // Fall through
    }
  }

  // Check if wasm-bindgen already initialized wasm
  try {
    wasmVersion();
    isInitialized = true;
    return;
  } catch (_) {
    // Not yet initialized
  }

  throw new Error(
    'markstone WASM module is not initialized. Please call "await init()" before using markstone in the browser.'
  );
}

function sanitizeInput(input) {
  if (typeof input === 'string') {
    return input;
  }
  if (
    (typeof Buffer !== 'undefined' && Buffer.isBuffer(input)) ||
    input instanceof Uint8Array
  ) {
    try {
      const decoder = new TextDecoder('utf-8', { fatal: true });
      return decoder.decode(input);
    } catch (_) {
      const err = new Error('invalid utf-8 byte sequence');
      err.code = 'INVALID_UTF8';
      throw err;
    }
  }
  throw new TypeError('input must be a string or Buffer');
}

export function toHtml(input) {
  ensureInitialized();
  return wasmToHtml(sanitizeInput(input));
}

export function to_html(input) {
  return toHtml(input);
}

export function toAst(input) {
  ensureInitialized();
  return wasmToAst(sanitizeInput(input));
}

export function to_ast(input) {
  return toAst(input);
}

export function astSchemaVersion() {
  ensureInitialized();
  return wasmAstSchemaVersion();
}

export function ast_schema_version() {
  return astSchemaVersion();
}

export const AST_SCHEMA_VERSION = 1;
export const version = '0.1.0';

export const actos = {
  toHtml(input) {
    ensureInitialized();
    return wasmActosToHtml(sanitizeInput(input));
  },
  to_html(input) {
    return actos.toHtml(input);
  },
  toAst(input) {
    ensureInitialized();
    return wasmActosToAst(sanitizeInput(input));
  },
  to_ast(input) {
    return actos.toAst(input);
  },
  astSchemaVersion,
  ast_schema_version,
  AST_SCHEMA_VERSION,
  version,
};

export async function init(moduleOrPath) {
  const result = await initWasm(moduleOrPath);
  isInitialized = true;
  return result;
}

export function initSync(module) {
  const result = wasmInitSync(module);
  isInitialized = true;
  return result;
}

export default {
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
};
