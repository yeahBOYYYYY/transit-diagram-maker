import { describe, expect, it, vi } from 'vitest'
import { freshStore } from './helpers'
import { prefs } from '../../src/stores/prefs'
import { fontChoices } from '../../src/core/fonts'

describe('uploaded assets', () => {
  it('mirrors uploaded fonts and icons and offers them in the pickers', () => {
    const store = freshStore()
    const family = store.run((e) => e.addFont('City Sans', 'data:font/woff2;base64,d09GMgAB'))!
    expect(family).toBe('"City Sans", sans-serif')
    const name = store.run((e) => e.addIconAsset('Star', '<svg viewBox="0 0 10 10"><path d="M0 0H10V10Z"/></svg>'))!
    expect(store.state.assets.fonts).toEqual(['City Sans'])
    expect(store.state.assets.icons.map((i) => [i.name, i.label])).toEqual([[name, 'Star']])
    expect(fontChoices(store.state.assets.fonts)).toContain(family)
    // Invalid uploads are reported, not stored.
    store.run((e) => e.addIconAsset('bad', '<html/>'))
    expect(store.state.messageKind).toBe('error')
    expect(store.state.assets.icons).toHaveLength(1)
    // Undo removes the upload again.
    store.undo()
    expect(store.state.assets.icons).toHaveLength(0)
  })
})

describe('editor store', () => {
  it('mirrors engine state after mutations', () => {
    const store = freshStore()
    expect(store.state.fragments).toEqual([])
    const line = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))!
    expect(store.state.fragments).toHaveLength(1)
    expect(store.state.tree[0].id).toBe(line.group_id)
    expect(store.state.canUndo).toBe(true)
    expect(store.state.dirty).toBe(true)
  })

  it('reports errors instead of throwing', () => {
    const store = freshStore()
    const out = store.run((e) => e.addStation('dot', { x: 0, y: 0 }, {}))
    expect(out).toBeUndefined()
    expect(store.state.messageKind).toBe('error')
    expect(store.state.message).toMatch(/line/)
  })

  it('selects with replace / add / toggle and drops stale ids', () => {
    const store = freshStore()
    const a = store.run((e) => e.addText({ text: 'a' }))!
    const b = store.run((e) => e.addText({ text: 'b' }))!
    store.select([a])
    store.select([b], 'add')
    expect(store.state.selection).toEqual([a, b])
    store.select([a], 'toggle')
    expect(store.state.selection).toEqual([b])
    store.run((e) => e.delete([b]))
    expect(store.state.selection).toEqual([])
    store.state.transformMode = 'rotate'
    store.clearSelection()
    expect(store.state.transformMode).toBe('move')
  })

  it('select all skips locked top-level nodes', () => {
    const store = freshStore()
    const a = store.run((e) => e.addText({}))!
    const b = store.run((e) => e.addText({}))!
    store.run((e) => e.setLocked(a, true))
    store.selectAll()
    expect(store.state.selection).toEqual([b])
  })

  it('picks layers or top-level groups depending on auto-select', () => {
    const store = freshStore()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))!
    expect(store.pickTarget(l.layer_id)).toBe(l.layer_id)
    prefs.autoSelect = 'group'
    expect(store.pickTarget(l.layer_id)).toBe(l.group_id)
    store.select([l.group_id])
    expect(store.selectedLeaves()).toEqual([l.layer_id])
  })

  it('zooms, pans and fits the view', () => {
    const store = freshStore()
    store.state.view = { x: 0, y: 0, zoom: 1 }
    store.zoomBy(2, 100, 100)
    expect(store.state.view).toEqual({ x: -100, y: -100, zoom: 2 })
    store.panBy(10, -5)
    expect(store.state.view).toMatchObject({ x: -90, y: -105 })
    store.setZoom(1)
    expect(store.state.view.zoom).toBe(1)
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 10, y: 0 }], {}))
    store.fitToScreen()
    expect(store.state.view.zoom).toBeGreaterThan(1)
    store.actualSize()
    expect(store.state.view.zoom).toBe(1)
  })

  it('undo / redo through the store', () => {
    const store = freshStore()
    store.run((e) => e.addText({}))
    store.undo()
    expect(store.state.fragments).toHaveLength(0)
    expect(store.state.canRedo).toBe(true)
    store.redo()
    expect(store.state.fragments).toHaveLength(1)
  })

  it('cancelling a new text removes it without a history entry', () => {
    const store = freshStore()
    const e = store.engine.value
    e.begin()
    const id = store.run((en) => en.addText({ text: 'Text' }))!
    store.startTextEdit(id, true)
    expect(store.state.textEdit?.value).toBe('Text')
    store.finishTextEdit(false)
    expect(store.state.fragments).toHaveLength(0)
    expect(store.state.canUndo).toBe(false)
  })

  it('committing text edits updates the layer; empty text deletes it', () => {
    const store = freshStore()
    const id = store.run((e) => e.addText({ text: 'Old' }))!
    store.startTextEdit(id)
    store.state.textEdit!.value = 'New name'
    store.finishTextEdit(true)
    const n = store.node(id)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.text).toBe('New name')
    store.startTextEdit(id)
    store.state.textEdit!.value = '   '
    store.finishTextEdit(true)
    expect(store.node(id)).toBeNull()
  })

  it('opens documents and rejects invalid files', () => {
    const store = freshStore()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))
    const saved = store.engine.value.saveTransit()
    store.newDocument()
    expect(store.state.fragments).toHaveLength(0)
    expect(store.openTransit(saved, 'map.transit.svg')).toBe(true)
    expect(store.state.docName).toBe('map')
    expect(store.state.dirty).toBe(false)
    expect(store.state.fragments).toHaveLength(1)
    expect(store.openTransit('garbage', 'bad.transit.svg')).toBe(false)
    expect(store.openTransit(store.engine.value.exportSvg(), 'picture.svg')).toBe(false)
    expect(store.state.message).toContain('no Transit Diagram Maker data')
    expect(store.state.messageKind).toBe('error')
    expect(store.state.fragments).toHaveLength(1)
  })

  it('messages expire', () => {
    vi.useFakeTimers()
    const store = freshStore()
    store.notify('hello')
    expect(store.state.message).toBe('hello')
    vi.advanceTimersByTime(5000)
    expect(store.state.message).toBe('')
    vi.useRealTimers()
  })

  it('setTool resets overlays and tracks the pen slot', () => {
    const store = freshStore()
    store.state.overlay.preview = '<path/>'
    store.setTool('eraser')
    expect(store.state.penSlot).toBe('eraser')
    expect(store.state.overlay.preview).toBe('')
    store.setTool('select')
    expect(store.state.penSlot).toBe('eraser')
    store.setTool('pen')
    expect(store.state.penSlot).toBe('pen')
  })
})
