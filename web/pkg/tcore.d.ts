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
    readonly wasm_bindgen__convert__closures_____invoke__h0f29f1253e1d51bd: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h7593f7d68f881ffe: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h90897e904d33052b: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h29b57ae09aa741ad: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hbd70e34ec71e5c4a: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h80d06db7173953eb: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h7c7d7d4887e16f10: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hcabe3a3b226e479c: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h3f4dac78ba48c6cd: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h30736dd0420e24ed: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hd01a34e2c91d5e07: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hde0f2cad6603a448: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h5922f524a0fa6374: (a: number, b: number) => number;
    readonly wasm_bindgen__convert__closures_____invoke__hf30697e446667207: (a: number, b: number) => void;
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
