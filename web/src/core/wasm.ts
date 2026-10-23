import init from '../wasm/pkg/transit_wasm.js'

let ready: Promise<unknown> | null = null

/** Load the WebAssembly module once. */
export function loadWasm(): Promise<unknown> {
  ready ??= init()
  return ready
}
