// Whole workflows through several layers at once: tools and commands drive the
// store, the store drives the real wasm engine, and files go through the same
// save / open / export paths the menus use.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { at, freshStore, key } from '../unit/helpers'
import { createTools } from '../../src/tools'
import { actionForEvent, exportSvg, openTransit, runAction, saveTransit } from '../../src/stores/commands'
import { prefs } from '../../src/stores/prefs'
import type { EditorStore } from '../../src/stores/editor'
import type { Id, Rect } from '../../src/core/types'

type PickerWindow = { showSaveFilePicker?: unknown; showOpenFilePicker?: unknown }
const win = window as unknown as PickerWindow

afterEach(() => {
  delete win.showSaveFilePicker
  delete win.showOpenFilePicker
  vi.restoreAllMocks()
})

function setup() {
  const store = freshStore()
  store.state.view = { x: 0, y: 0, zoom: 1 }
  const bounds = (ids: Id[]) => {
    const m = new Map<Id, Rect>()
    for (const f of store.state.fragments) if (ids.includes(f.id) && f.bounds) m.set(f.id, f.bounds)
    return m
  }
  return { store, tools: createTools(store, bounds) }
}

type Tools = ReturnType<typeof setup>['tools']

function click(store: EditorStore, tool: Tools[keyof Tools], x: number, y: number, extra = {}) {
  tool.down?.(at(store, x, y, extra))
  tool.up?.(at(store, x, y, extra))
}

/** Press a key the way the canvas does: through the bindings, then the action. */
function press(store: EditorStore, k: string, mods: KeyboardEventInit = {}) {
  const id = actionForEvent(key(k, mods))
  expect(id, `${k} is bound`).not.toBeNull()
  runAction(store, id!)
}

/** A map drawn with the tools: two lines, three stations and a title. */
function drawMap(store: EditorStore, tools: Tools) {
  prefs.line.color = '#e32017'
  click(store, tools.pen, 0.5, 0.5)
  click(store, tools.pen, 8.5, 0.5)
  tools.pen.key?.(key('Enter'))
  prefs.line.color = '#0098d4'
  click(store, tools.pen, 4.5, -3.5)
  click(store, tools.pen, 4.5, 5.5)
  tools.pen.key?.(key('Enter'))
  click(store, tools.station, 2.5, 0.5)
  click(store, tools.station, 4.5, 3.5)
  prefs.station.kind = 'tick'
  click(store, tools.station, 6.5, 0.5)
  store.run((e) => e.addText({ text: 'Metro', pos: { x: 0, y: -3 } }))
}

const kinds = (store: EditorStore) => store.state.fragments.map((f) => f.kind).sort()

function fakeSaveHandle(name: string) {
  const writes: string[] = []
  const handle = {
    name,
    getFile: async () => new File([writes.at(-1) ?? ''], name),
    createWritable: async () => ({ write: async (d: string) => void writes.push(d), close: async () => {} }),
  }
  return { handle, writes }
}

describe('drawing, saving and opening a map', () => {
  it('reopens exactly what was drawn, as a clean document', async () => {
    const { store, tools } = setup()
    drawMap(store, tools)
    // Every station gets a label, plus the title.
    expect(kinds(store)).toEqual(['line', 'line', 'station', 'station', 'station', 'text', 'text', 'text', 'text'])
    const lineColors = store.state.fragments.filter((f) => f.kind === 'line').map((f) => f.svg.match(/stroke="(#[0-9a-f]+)"/)![1])
    expect(lineColors).toEqual(['#e32017', '#0098d4'])

    const file = fakeSaveHandle('Metro.transit.svg')
    win.showSaveFilePicker = vi.fn().mockResolvedValue(file.handle)
    await saveTransit(store)
    expect(store.state.dirty).toBe(false)
    expect(store.state.docName).toBe('Metro')

    const other = setup().store
    win.showOpenFilePicker = vi.fn().mockResolvedValue([file.handle])
    await openTransit(other)
    expect(other.state.docName).toBe('Metro')
    expect(other.state.dirty).toBe(false)
    expect(other.state.canUndo).toBe(false)
    expect(other.state.fragments).toEqual(store.state.fragments)
    expect(other.state.tree).toEqual(store.state.tree)
    // Saving the reopened document writes back the very same file.
    expect(other.engine.value.saveTransit()).toBe(file.writes[0])
  })

  it('reports a file that is not a transit diagram and keeps the current document', async () => {
    const { store, tools } = setup()
    drawMap(store, tools)
    const before = store.state.fragments
    const bad = new File(['<svg xmlns="http://www.w3.org/2000/svg"/>'], 'other.svg')
    win.showOpenFilePicker = vi.fn().mockResolvedValue([{ name: bad.name, getFile: async () => bad }])
    await openTransit(store)
    expect(store.state.messageKind).toBe('error')
    expect(store.state.fragments).toEqual(before)
  })

  it('exports the same shapes the canvas shows', () => {
    const { store, tools } = setup()
    drawMap(store, tools)
    let blob: Blob | null = null
    vi.spyOn(URL, 'createObjectURL').mockImplementation((b) => ((blob = b as Blob), 'blob:x'))
    vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {})
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {})
    exportSvg(store, { cell_px: 24, margin: 1, background: '' })
    return blob!.text().then((svg) => {
      expect(svg).not.toContain('transit:')
      for (const f of store.state.fragments) expect(svg, `${f.kind} ${f.id}`).toContain(f.svg)
    })
  })
})

describe('editing with the keyboard', () => {
  it('select all, duplicate, rotate, delete, then undo and redo every step', () => {
    const { store, tools } = setup()
    drawMap(store, tools)
    const drawn = store.state.fragments
    const history: string[] = [JSON.stringify(drawn)]
    const step = (k: string, mods: KeyboardEventInit = {}) => {
      press(store, k, mods)
      history.push(JSON.stringify(store.state.fragments))
    }
    press(store, 'a', { ctrlKey: true })
    expect(store.state.selection.length).toBeGreaterThan(0)
    step('j', { ctrlKey: true })
    expect(store.state.fragments).toHaveLength(drawn.length * 2)
    step('r')
    step('Delete')
    expect(store.state.fragments).toHaveLength(drawn.length)
    for (let i = history.length - 2; i >= 0; i--) {
      press(store, 'z', { ctrlKey: true })
      expect(JSON.stringify(store.state.fragments), `undo to step ${i}`).toBe(history[i])
    }
    for (let i = 1; i < history.length; i++) {
      press(store, 'y', { ctrlKey: true })
      expect(JSON.stringify(store.state.fragments), `redo to step ${i}`).toBe(history[i])
    }
  })

  it('switches tools with their letters and keeps tool options across switches', () => {
    const { store } = setup()
    press(store, 'p')
    expect(store.state.tool).toBe('pen')
    prefs.line.color = '#123456'
    press(store, 'e')
    expect(store.state.tool).toBe('eraser')
    press(store, 'p')
    expect(prefs.line.color).toBe('#123456')
  })
})

describe('moving a line with its stations', () => {
  it('drags a whole line group as one undo step, and the stations stay on it', () => {
    const { store, tools } = setup()
    drawMap(store, tools)
    const line = store.state.tree.find((t) => t.kind === 'group' && t.role === 'line' && t.name === 'Line 1')!
    const stationsBefore = store.state.fragments.filter((f) => f.kind === 'station').map((f) => f.svg)
    const before = store.state.fragments
    const sel = tools.select
    sel.down?.(at(store, 1.5, 0.5, { hitId: line.id }))
    sel.move?.(at(store, 1.5, 2.5, { hitId: line.id }), true)
    sel.move?.(at(store, 1.5, 4.5, { hitId: line.id }), true)
    sel.up?.(at(store, 1.5, 4.5))
    const stationsAfter = store.state.fragments.filter((f) => f.kind === 'station').map((f) => f.svg)
    expect(stationsAfter).not.toEqual(stationsBefore)
    // Every station is still drawn on one of the lines: none was dropped.
    expect(stationsAfter).toHaveLength(stationsBefore.length)
    // One undo takes back the whole drag.
    store.undo()
    expect(store.state.fragments).toEqual(before)
  })
})

describe('erasing through a station', () => {
  it('splits the line and keeps the stations that are not erased', () => {
    const { store, tools } = setup()
    drawMap(store, tools)
    const stations = store.state.fragments.filter((f) => f.kind === 'station').length
    tools.eraser.down?.(at(store, 4.5, 2.5))
    tools.eraser.up?.(at(store, 4.5, 2.5))
    expect(store.state.fragments.filter((f) => f.kind === 'line')).toHaveLength(3)
    expect(store.state.fragments.filter((f) => f.kind === 'station')).toHaveLength(stations)
    const blue = store.state.tree.find((t) => t.kind === 'group' && t.role === 'line' && t.name === 'Line 2')!
    expect(blue.kind === 'group' && blue.children.filter((c) => c.kind === 'line')).toHaveLength(2)
  })
})
