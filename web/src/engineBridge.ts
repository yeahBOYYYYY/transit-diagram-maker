import { EditorAction, EditorStateResponse } from './types';

interface WasmExports {
  memory: WebAssembly.Memory;
  engine_init: (width: number, height: number) => void;
  engine_alloc: (len: number) => number;
  engine_dealloc: (ptr: number, len: number) => void;
  engine_dispatch: (ptr: number, len: number) => number;
  engine_response_len: () => number;
}

class TransitEngineBridge {
  private exports: WasmExports | null = null;
  private isInitialized = false;
  private textEncoder = new TextEncoder();
  private textDecoder = new TextDecoder('utf-8');
  private stateListeners: Array<(state: EditorStateResponse) => void> = [];
  private currentState: EditorStateResponse | null = null;

  async init(width: number, height: number): Promise<EditorStateResponse> {
    try {
      const wasmUrl = new URL('/transit_diagram_maker.wasm', window.location.origin).href;
      const response = await fetch(wasmUrl);
      const wasmBytes = await response.arrayBuffer();

      const wasmModule = await WebAssembly.instantiate(wasmBytes, {});

      this.exports = wasmModule.instance.exports as unknown as WasmExports;
      this.exports.engine_init(width, height);
      this.isInitialized = true;

      // Initial dispatch to fetch starting document state
      return this.dispatch({ type: 'ResizeViewport', payload: { width, height } });
    } catch (err) {
      console.error('Failed to load Transit Diagram WASM module:', err);
      throw err;
    }
  }

  dispatch(action: EditorAction): EditorStateResponse {
    if (!this.exports || !this.isInitialized) {
      throw new Error('TransitEngineBridge is not initialized');
    }

    const actionJson = JSON.stringify(action);
    const actionBytes = this.textEncoder.encode(actionJson);

    // Allocate memory in WASM space
    const inputPtr = this.exports.engine_alloc(actionBytes.length);
    const memoryBuffer = new Uint8Array(this.exports.memory.buffer);
    memoryBuffer.set(actionBytes, inputPtr);

    // Execute engine action
    const responsePtr = this.exports.engine_dispatch(inputPtr, actionBytes.length);
    const responseLen = this.exports.engine_response_len();

    // Read result
    const responseSlice = new Uint8Array(this.exports.memory.buffer, responsePtr, responseLen);
    const responseJson = this.textDecoder.decode(responseSlice);

    // Deallocate input buffer
    this.exports.engine_dealloc(inputPtr, actionBytes.length);

    const state: EditorStateResponse = JSON.parse(responseJson);
    this.currentState = state;

    // Notify listeners
    for (const listener of this.stateListeners) {
      listener(state);
    }

    return state;
  }

  getState(): EditorStateResponse | null {
    return this.currentState;
  }

  subscribe(listener: (state: EditorStateResponse) => void): () => void {
    this.stateListeners.push(listener);
    if (this.currentState) {
      listener(this.currentState);
    }
    return () => {
      const idx = this.stateListeners.indexOf(listener);
      if (idx !== -1) {
        this.stateListeners.splice(idx, 1);
      }
    };
  }
}

export const engineBridge = new TransitEngineBridge();
