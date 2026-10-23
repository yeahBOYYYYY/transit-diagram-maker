import { createEditor, provideEditor, type EditorStore } from '../../src/stores/editor'
import type { PointerInfo } from '../../src/tools'

export function freshStore(): EditorStore {
  const store = provideEditor(createEditor())
  store.refresh()
  store.setViewport(800, 600)
  return store
}

/** Pointer info for a position given in cell units (zoom 1, view origin 0,0). */
export function at(store: EditorStore, x: number, y: number, extra: Partial<PointerInfo> = {}): PointerInfo {
  const g = store.state.settings.grid_size
  const world = { x: x * g, y: y * g }
  return {
    screen: { x: world.x * store.state.view.zoom + store.state.view.x, y: world.y * store.state.view.zoom + store.state.view.y },
    world,
    cells: { x, y },
    cell: { x: Math.floor(x), y: Math.floor(y) },
    button: 0,
    buttons: 1,
    shift: false,
    alt: false,
    ctrl: false,
    hitId: null,
    handle: null,
    ...extra,
  }
}

export function key(k: string, extra: Partial<KeyboardEventInit> = {}): KeyboardEvent {
  const code = /^[a-z]$/i.test(k) ? `Key${k.toUpperCase()}` : k
  return new KeyboardEvent('keydown', { key: k, code, ...extra })
}
