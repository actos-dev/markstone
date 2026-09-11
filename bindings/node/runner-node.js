#!/usr/bin/env node
import fs from 'node:fs';
import markstone, { toHtml, toAst, actos } from './index.js';

let mode = null;
let inputPath = null;

for (let i = 2; i < process.argv.length; i++) {
  if (process.argv[i] === '--mode') {
    mode = process.argv[++i];
  } else if (!process.argv[i].startsWith('-')) {
    inputPath = process.argv[i];
  }
}

if (!mode) {
  console.error('Error: --mode is required');
  process.exit(1);
}

let content;
if (inputPath && inputPath !== '-') {
  content = fs.readFileSync(inputPath, 'utf8');
} else {
  content = fs.readFileSync(0, 'utf8');
}

let output;
if (mode === 'generic-html') {
  output = toHtml(content);
} else if (mode === 'generic-ast') {
  output = toAst(content);
} else if (mode === 'actos-html') {
  output = actos.toHtml(content);
} else if (mode === 'actos-ast') {
  output = actos.toAst(content);
} else {
  console.error(`Unknown mode: ${mode}`);
  process.exit(1);
}

process.stdout.write(output);
