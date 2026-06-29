/* tslint:disable */
/* eslint-disable */

/**
 * Chroma subsampling format
 */
export enum ChromaSampling {
    /**
     * Both vertically and horizontally subsampled.
     */
    Cs420 = 0,
    /**
     * Horizontally subsampled.
     */
    Cs422 = 1,
    /**
     * Not subsampled.
     */
    Cs444 = 2,
    /**
     * Monochrome.
     */
    Cs400 = 3,
}

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly start: () => void;
    readonly wasm_bindgen__convert__closures_____invoke__ha9b7351d3aefd9c8: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__hec1fc1c1eeb60dcc: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h1ee7cfd2b6c1baf6: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h73cd4b605d5572fc: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h7fc1a3d56008c12a: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h7855e6978bb86342: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h7705b2eb39f955a8: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h2f83ce9f36d98df0: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h68d9852572363fdd: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hcf0b8d9dfec2675c: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h20431b34bdd31283: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hff3bcc39047f2784: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hf268c3f76239e8ce: (a: number, b: number) => number;
    readonly wasm_bindgen__convert__closures_____invoke__h94e29ec35d83312e: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
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
