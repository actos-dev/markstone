export * from './index.js';

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;
export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Initializes the WebAssembly module asynchronously.
 * In a browser, passing no arguments fetches the wasm binary relative to the module.
 */
export function init(moduleOrPath?: InitInput | Promise<InitInput>): Promise<unknown>;

/**
 * Synchronously initializes the WebAssembly module from bytes or compiled module.
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): unknown;

import markstone from './index.js';

export interface MarkstoneBrowserBinding extends typeof markstone {
  init: typeof init;
  initSync: typeof initSync;
}

declare const markstoneBrowser: MarkstoneBrowserBinding;
export default markstoneBrowser;
