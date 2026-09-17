import { fileURLToPath } from 'node:url';
import path from 'node:path';
import fs from 'node:fs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Published packages carry one prebuilt addon per platform, named after the
// platform tag below. `markstone.node` is a local build (`npm run
// build:native`) and wins when present.
function isMusl() {
  if (process.platform !== 'linux') return false;
  const report = process.report?.getReport?.();
  const header = typeof report === 'string' ? JSON.parse(report).header : report?.header;
  return !header?.glibcVersionRuntime;
}

function platformTag() {
  const base = `${process.platform}-${process.arch}`;
  if (process.platform !== 'linux') return base;
  return `${base}-${isMusl() ? 'musl' : 'gnu'}`;
}

function loadNativeBinding() {
  const tag = platformTag();
  const candidates = [
    path.join(__dirname, 'markstone.node'),
    path.join(__dirname, `markstone.${tag}.node`),
  ];

  const failures = [];
  for (const candidate of candidates) {
    if (!fs.existsSync(candidate)) continue;
    try {
      const addon = { exports: {} };
      process.dlopen(addon, candidate);
      return addon.exports;
    } catch (err) {
      failures.push(`${path.basename(candidate)}: ${err.message}`);
    }
  }

  const detail = failures.length > 0 ? ` (${failures.join('; ')})` : '';
  throw new Error(`markstone has no native addon for ${tag}${detail}`);
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
