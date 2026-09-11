import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import fs from 'node:fs';

const require = createRequire(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

let nativeBinding = null;

function loadNativeBinding() {
  if (nativeBinding) return nativeBinding;

  const candidates = [
    path.join(__dirname, 'markstone.node'),
    path.join(__dirname, `markstone.${process.platform}-${process.arch}.node`),
    path.join(__dirname, `markstone.${process.platform}-${process.arch}-gnu.node`),
    path.join(__dirname, `markstone.${process.platform}-${process.arch}-musl.node`),
    path.join(__dirname, '../../target/release/libmarkstone_node.so'),
    path.join(__dirname, '../../target/release/markstone_node.dll'),
    path.join(__dirname, '../../target/release/libmarkstone_node.dylib'),
    path.join(__dirname, '../../target/debug/libmarkstone_node.so'),
    path.join(__dirname, '../../target/debug/markstone_node.dll'),
    path.join(__dirname, '../../target/debug/libmarkstone_node.dylib'),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      try {
        const addon = { exports: {} };
        process.dlopen(addon, candidate);
        nativeBinding = addon.exports;
        return nativeBinding;
      } catch (err) {
        // Try next candidate
      }
    }
  }

  throw new Error(
    `Failed to load native markstone addon for ${process.platform}-${process.arch}. ` +
    `Ensure prebuilt binaries are installed or build locally with cargo build.`
  );
}

const binding = loadNativeBinding();

function sanitizeInput(input) {
  if (typeof input === 'string') {
    return input;
  }
  if (Buffer.isBuffer(input) || input instanceof Uint8Array) {
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
  return binding.toHtml(sanitizeInput(input));
}

export function to_html(input) {
  return toHtml(input);
}

export function toAst(input) {
  return binding.toAst(sanitizeInput(input));
}

export function to_ast(input) {
  return toAst(input);
}

export function astSchemaVersion() {
  return binding.astSchemaVersion ? binding.astSchemaVersion() : binding.AST_SCHEMA_VERSION;
}

export function ast_schema_version() {
  return astSchemaVersion();
}

export const AST_SCHEMA_VERSION = binding.AST_SCHEMA_VERSION ?? 1;
export const version = typeof binding.version === 'function' ? binding.version() : (binding.version || '0.1.0');

export const actos = {
  toHtml(input) {
    return binding.actos.toHtml(sanitizeInput(input));
  },
  to_html(input) {
    return binding.actos.to_html(sanitizeInput(input));
  },
  toAst(input) {
    return binding.actos.toAst(sanitizeInput(input));
  },
  to_ast(input) {
    return binding.actos.to_ast(sanitizeInput(input));
  },
  astSchemaVersion,
  ast_schema_version,
  AST_SCHEMA_VERSION,
  version,
};

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
};
