// Load the real WebAssembly engine synchronously for unit tests.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeEach } from 'vitest'
import { initSync } from '../../src/wasm/pkg/transit_wasm.js'
import { resetPrefs } from '../../src/stores/prefs'

initSync({ module: readFileSync(resolve(__dirname, '../../src/wasm/pkg/transit_wasm_bg.wasm')) })

beforeEach(() => {
  resetPrefs()
  localStorage.clear()
})
