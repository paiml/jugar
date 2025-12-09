/* tslint:disable */
/* eslint-disable */

export class WebPlatform {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns the current configuration as JSON.
   */
  getConfig(): string;
  /**
   * Returns AI model metadata and current state as JSON.
   */
  getAiInfo(): string;
  /**
   * Creates a new `WebPlatform` with default configuration.
   */
  static newDefault(): WebPlatform;
  /**
   * Resets the timer (useful when tab becomes visible again).
   */
  resetTimer(): void;
  /**
   * Returns the AI model as JSON string for download.
   */
  getAiModel(): string;
  /**
   * Gets the current game mode as string.
   */
  getGameMode(): string;
  /**
   * Sets the game mode ("demo", "1p", "2p").
   */
  setGameMode(mode: string): void;
  /**
   * Gets the current AI difficulty level.
   */
  getAiDifficulty(): number;
  /**
   * Sets the AI difficulty level (0-9).
   */
  setAiDifficulty(level: number): void;
  /**
   * Sets the canvas offset from viewport origin.
   */
  setCanvasOffset(x: number, y: number): void;
  /**
   * Creates a new `WebPlatform` with configuration from JSON.
   *
   * # Arguments
   *
   * * `config_json` - JSON string with configuration
   *
   * # Errors
   *
   * Returns a JavaScript error if the configuration is invalid.
   */
  constructor(config_json: string);
  /**
   * Processes a single frame.
   *
   * This is called from `requestAnimationFrame`. All game logic runs here.
   *
   * # Arguments
   *
   * * `timestamp` - `DOMHighResTimeStamp` from `requestAnimationFrame`
   * * `input_events_json` - JSON array of input events since last frame
   *
   * # Returns
   *
   * JSON string with render commands for Canvas2D execution.
   */
  frame(timestamp: number, input_events_json: string): string;
  /**
   * Handles canvas resize.
   *
   * # Arguments
   *
   * * `width` - New canvas width in pixels
   * * `height` - New canvas height in pixels
   */
  resize(width: number, height: number): void;
  /**
   * Gets the current speed multiplier value.
   */
  getSpeed(): number;
  /**
   * Returns current debug statistics as JSON.
   */
  getStats(): string;
  /**
   * Sets the speed multiplier (1, 5, 10, 50, 100, 1000).
   */
  setSpeed(speed: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_webplatform_free: (a: number, b: number) => void;
  readonly webplatform_frame: (a: number, b: number, c: number, d: number) => [number, number];
  readonly webplatform_getAiDifficulty: (a: number) => number;
  readonly webplatform_getAiInfo: (a: number) => [number, number];
  readonly webplatform_getAiModel: (a: number) => [number, number];
  readonly webplatform_getConfig: (a: number) => [number, number];
  readonly webplatform_getGameMode: (a: number) => [number, number];
  readonly webplatform_getSpeed: (a: number) => number;
  readonly webplatform_getStats: (a: number) => [number, number];
  readonly webplatform_new: (a: number, b: number) => [number, number, number];
  readonly webplatform_newDefault: () => number;
  readonly webplatform_resetTimer: (a: number) => void;
  readonly webplatform_resize: (a: number, b: number, c: number) => void;
  readonly webplatform_setAiDifficulty: (a: number, b: number) => void;
  readonly webplatform_setCanvasOffset: (a: number, b: number, c: number) => void;
  readonly webplatform_setGameMode: (a: number, b: number, c: number) => void;
  readonly webplatform_setSpeed: (a: number, b: number) => void;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
