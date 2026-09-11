# markstone

Unified Node.js and browser package for markstone: fast, safe Markdown-to-HTML and AST engine with Actos extensions.

Uses native napi-rs addon on Node.js and WebAssembly (`wasm-bindgen`) in browsers via conditional `exports`. Both paths produce byte-for-byte identical output.

## Installation

```bash
npm install markstone
```

## Usage

### Node.js (Native Addon)

```javascript
import markstone, { toHtml, toAst, actos } from 'markstone';

// Generic CommonMark + GFM
const html = toHtml('# Hello world');
const ast = toAst('# Hello world');

// Actos extensions (@mentions and #tags)
const actosHtml = actos.toHtml('Hello @alice and #rust');
const actosAst = actos.toAst('Hello @alice and #rust');
```

### Browser (WebAssembly)

```javascript
import markstone, { init, toHtml, toAst, actos } from 'markstone';

// Initialize WASM binary once in browser
await init();

// Render
const html = toHtml('# Hello from the browser');
const actosHtml = actos.toHtml('Hello @alice and #rust');
```

## Security & Conformance

- No raw HTML or dangerous scripts pass through the AST sanitizer.
- Depth limit: 64 block levels (`DEPTH_EXCEEDED`).
- Input limit: 4 MiB (`INPUT_TOO_LARGE`).
- 100% byte-for-byte conformance parity between Node native and browser WASM.
