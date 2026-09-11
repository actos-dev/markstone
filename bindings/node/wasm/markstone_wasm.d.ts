/* tslint:disable */
/* eslint-disable */

export function actosToAst(input: string): string;

export function actosToHtml(input: string): string;

export function actos_to_ast(input: string): string;

export function actos_to_html(input: string): string;

export function astSchemaVersion(): number;

export function ast_schema_version(): number;

export function toAst(input: string): string;

export function toHtml(input: string): string;

export function to_ast(input: string): string;

export function to_html(input: string): string;

export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly actosToAst: (a: number, b: number) => [number, number, number, number];
    readonly actosToHtml: (a: number, b: number) => [number, number, number, number];
    readonly actos_to_ast: (a: number, b: number) => [number, number, number, number];
    readonly actos_to_html: (a: number, b: number) => [number, number, number, number];
    readonly astSchemaVersion: () => number;
    readonly ast_schema_version: () => number;
    readonly toAst: (a: number, b: number) => [number, number, number, number];
    readonly toHtml: (a: number, b: number) => [number, number, number, number];
    readonly to_ast: (a: number, b: number) => [number, number, number, number];
    readonly to_html: (a: number, b: number) => [number, number, number, number];
    readonly version: () => [number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
