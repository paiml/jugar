/* tslint:disable */
/* eslint-disable */

/**
 * Demo: Create a simple Rube Goldberg machine
 */
export function create_demo_contraption(): string;

/**
 * Demo: Fork a contraption (Kaizen cycle)
 */
export function demo_fork_workflow(): string;

/**
 * Demo: Poka-Yoke material safety
 */
export function demo_poka_yoke(): string;

/**
 * Demo: Serialization round-trip
 */
export function demo_serialization(): string;

/**
 * Demo: Storage and search
 */
export function demo_storage(): string;

/**
 * Demo: Complexity Thermometer (Mieruka)
 */
export function demo_thermometer(physics_ms: number, render_ms: number, ui_ms: number): string;

/**
 * Get engine version
 */
export function get_engine_version(): string;

/**
 * Get max objects limit
 */
export function get_max_objects(): number;

export function main(): void;

/**
 * Run all demos and return summary
 */
export function run_all_demos(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly create_demo_contraption: () => [number, number];
  readonly demo_fork_workflow: () => [number, number];
  readonly demo_poka_yoke: () => [number, number];
  readonly demo_serialization: () => [number, number];
  readonly demo_storage: () => [number, number];
  readonly demo_thermometer: (a: number, b: number, c: number) => [number, number];
  readonly get_engine_version: () => [number, number];
  readonly get_max_objects: () => number;
  readonly main: () => void;
  readonly run_all_demos: () => [number, number];
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
