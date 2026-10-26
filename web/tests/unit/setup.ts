// Shared setup for every vitest test: load the real WebAssembly engine synchronously, reset the preferences.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, beforeEach } from 'vitest'
import { cleanup } from '@testing-library/vue'
import '@testing-library/jest-dom/vitest'
import { initSync } from '../../src/wasm/pkg/transit_wasm.js'
import { resetPrefs } from '../../src/stores/prefs'

initSync({ module: readFileSync(resolve(__dirname, '../../src/wasm/pkg/transit_wasm_bg.wasm')) })

beforeEach(() => {
  resetPrefs()
  localStorage.clear()
})

// Testing Library only cleans up by itself when test globals are enabled.
afterEach(() => cleanup())
