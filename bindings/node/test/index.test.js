import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import markstoneNode, {
  toHtml as nodeToHtml,
  to_html as nodeToHtmlSnake,
  toAst as nodeToAst,
  to_ast as nodeToAstSnake,
  actos as nodeActos,
  AST_SCHEMA_VERSION as nodeSchemaVersion,
  version as nodeVersion,
} from '../index.js';

import markstoneWasm, {
  initSync as wasmInitSync,
  toHtml as wasmToHtml,
  to_html as wasmToHtmlSnake,
  toAst as wasmToAst,
  to_ast as wasmToAstSnake,
  actos as wasmActos,
  AST_SCHEMA_VERSION as wasmSchemaVersion,
  version as wasmVersion,
} from '../browser.js';

import fallbackMarkstone from '../fallback.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const wasmPath = path.join(__dirname, '../wasm/markstone_wasm_bg.wasm');
wasmInitSync({ module: fs.readFileSync(wasmPath) });

test('Node native bindings basic rendering', () => {
  const html = nodeToHtml('# Hello world');
  assert.equal(html, '<h1>Hello world</h1>\n');

  const htmlSnake = nodeToHtmlSnake('# Hello world');
  assert.equal(htmlSnake, '<h1>Hello world</h1>\n');

  const astJson = nodeToAst('# Hello world');
  const ast = JSON.parse(astJson);
  assert.equal(ast.schema, 1);
  assert.equal(ast.root.type, 'document');
  assert.equal(ast.root.children[0].type, 'heading');
  assert.equal(ast.root.children[0].level, 1);

  const astSnakeJson = nodeToAstSnake('# Hello world');
  assert.equal(astSnakeJson, astJson);
});

test('Node native bindings Actos extensions', () => {
  const html = nodeActos.toHtml('Hello @alice and #markstone');
  assert.match(html, /<a href="\/u\/alice" class="mention">@alice<\/a>/);
  assert.match(html, /<a href="\/t\/markstone" class="tag">#markstone<\/a>/);

  const htmlSnake = nodeActos.to_html('Hello @alice and #markstone');
  assert.equal(htmlSnake, html);

  const astJson = nodeActos.toAst('Hello @alice and #markstone');
  const ast = JSON.parse(astJson);
  const mentionNode = ast.root.children[0].children.find((c) => c.type === 'mention');
  assert.ok(mentionNode);
  assert.equal(mentionNode.username, 'alice');
  assert.equal(mentionNode.text, '@alice');

  const tagNode = ast.root.children[0].children.find((c) => c.type === 'tag');
  assert.ok(tagNode);
  assert.equal(tagNode.name, 'markstone');
  assert.equal(tagNode.text, '#markstone');
});

test('Node native error mapping and limits', () => {
  // Depth exceeded
  assert.throws(
    () => {
      nodeToHtml('> '.repeat(70) + 'deep');
    },
    (err) => {
      assert.equal(err.code, 'DEPTH_EXCEEDED');
      assert.match(err.message, /depth/i);
      return true;
    }
  );

  // Input too large
  assert.throws(
    () => {
      nodeToHtml('a'.repeat(4 * 1024 * 1024 + 1));
    },
    (err) => {
      assert.equal(err.code, 'INPUT_TOO_LARGE');
      assert.match(err.message, /4 MiB/i);
      return true;
    }
  );

  // Invalid UTF-8 in buffer
  assert.throws(
    () => {
      nodeToHtml(Buffer.from([0xff, 0xfe]));
    },
    (err) => {
      assert.equal(err.code, 'INVALID_UTF8');
      return true;
    }
  );

  // Type error
  assert.throws(
    () => {
      nodeToHtml(12345);
    },
    (err) => err instanceof TypeError
  );
});

test('Browser WASM bindings basic rendering', () => {
  const html = wasmToHtml('# Hello wasm');
  assert.equal(html, '<h1>Hello wasm</h1>\n');

  const htmlSnake = wasmToHtmlSnake('# Hello wasm');
  assert.equal(htmlSnake, '<h1>Hello wasm</h1>\n');

  const astJson = wasmToAst('# Hello wasm');
  const ast = JSON.parse(astJson);
  assert.equal(ast.schema, 1);
  assert.equal(ast.root.type, 'document');

  const astSnakeJson = wasmToAstSnake('# Hello wasm');
  assert.equal(astSnakeJson, astJson);
});

test('Browser WASM bindings Actos extensions', () => {
  const html = wasmActos.toHtml('Hello @bob and #rust');
  assert.match(html, /<a href="\/u\/bob" class="mention">@bob<\/a>/);
  assert.match(html, /<a href="\/t\/rust" class="tag">#rust<\/a>/);

  const astJson = wasmActos.toAst('Hello @bob and #rust');
  const ast = JSON.parse(astJson);
  assert.ok(ast.root.children[0].children.some((c) => c.type === 'mention'));
  assert.ok(ast.root.children[0].children.some((c) => c.type === 'tag'));
});

test('Browser WASM error mapping and limits', () => {
  assert.throws(
    () => {
      wasmToHtml('> '.repeat(70) + 'deep');
    },
    (err) => {
      assert.equal(err.code, 'DEPTH_EXCEEDED');
      return true;
    }
  );

  assert.throws(
    () => {
      wasmToHtml('a'.repeat(4 * 1024 * 1024 + 1));
    },
    (err) => {
      assert.equal(err.code, 'INPUT_TOO_LARGE');
      return true;
    }
  );

  assert.throws(
    () => {
      wasmToHtml(Buffer.from([0xff, 0xfe]));
    },
    (err) => {
      assert.equal(err.code, 'INVALID_UTF8');
      return true;
    }
  );
});

test('Byte-for-byte output parity between Native Node and Browser WASM', () => {
  const sampleInputs = [
    '# Simple Header\n\nParagraph text.',
    '* Item 1\n* Item 2\n  * Subitem',
    '| A | B |\n|---|---|\n| 1 | 2 |',
    '```python\nprint("code")\n```',
    'Hello @user and #tag in [a link](https://example.com)',
    '<script>alert("xss")</script>',
    '[evil](javascript:alert(1))',
  ];

  for (const input of sampleInputs) {
    const nodeHtml = nodeToHtml(input);
    const wasmHtml = wasmToHtml(input);
    assert.equal(nodeHtml, wasmHtml, `HTML mismatch for: ${input}`);

    const nodeAst = nodeToAst(input);
    const wasmAst = wasmToAst(input);
    assert.equal(nodeAst, wasmAst, `AST mismatch for: ${input}`);

    const nodeActosHtml = nodeActos.toHtml(input);
    const wasmActosHtml = wasmActos.toHtml(input);
    assert.equal(nodeActosHtml, wasmActosHtml, `Actos HTML mismatch for: ${input}`);

    const nodeActosAst = nodeActos.toAst(input);
    const wasmActosAst = wasmActos.toAst(input);
    assert.equal(nodeActosAst, wasmActosAst, `Actos AST mismatch for: ${input}`);
  }
});

test('Fallback module exports are functional', () => {
  assert.equal(fallbackMarkstone.toHtml('# Fallback'), '<h1>Fallback</h1>\n');
  assert.equal(fallbackMarkstone.AST_SCHEMA_VERSION, 1);
  assert.equal(fallbackMarkstone.version, '0.1.0');
});

test('Version constants match', () => {
  assert.equal(nodeVersion, '0.1.0');
  assert.equal(wasmVersion, '0.1.0');
  assert.equal(nodeSchemaVersion, 1);
  assert.equal(wasmSchemaVersion, 1);
  assert.equal(nodeActos.version, '0.1.0');
  assert.equal(wasmActos.version, '0.1.0');
  assert.equal(nodeActos.AST_SCHEMA_VERSION, 1);
  assert.equal(wasmActos.AST_SCHEMA_VERSION, 1);
});
