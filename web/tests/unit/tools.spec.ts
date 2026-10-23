// Tool behavior driven with synthetic pointer events against the real engine.
import { describe, expect, it } from 'vitest'
import { at, freshStore, key } from './helpers'
import { createTools } from '../../src/tools'
import type { EditorStore } from '../../src/stores/editor'
import type { Id, Rect } from '../../src/core/types'
import { prefs } from '../../src/stores/prefs'

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

function lineCount(store: EditorStore) {
  return store.state.fragments.filter((f) => f.kind === 'line').length
}

function click(store: EditorStore, tool: ReturnType<typeof setup>['tools'][keyof ReturnType<typeof setup>['tools']], x: number, y: number, extra = {}) {
  const p = at(store, x, y, extra)
  tool.down?.(p)
  tool.up?.(p)
}

describe('line pen', () => {
  it('collects points, previews and finishes on Enter', () => {
    const { store, tools } = setup()
    const pen = tools.pen
    click(store, pen, 0.5, 0.5)
    pen.move?.(at(store, 5.5, 2.5), false)
    expect(store.state.overlay.preview).toContain('<path')
    click(store, pen, 5.5, 2.5)
    expect(pen.key?.(key('Enter'))).toBe(true)
    expect(lineCount(store)).toBe(1)
    expect(store.state.selection).toEqual([store.state.tree[0].id])
    expect(store.state.overlay.preview).toBe('')
  })

  it('finishes when clicking the last point again and supports backspace', () => {
    const { store, tools } = setup()
    const pen = tools.pen
    click(store, pen, 0.5, 0.5)
    click(store, pen, 3.5, 0.5)
    click(store, pen, 3.5, 4.5)
    expect(pen.key?.(key('Backspace'))).toBe(true)
    click(store, pen, 3.5, 0.5) // same as last remaining point -> finish
    expect(lineCount(store)).toBe(1)
    const n = store.node(store.state.fragments[0].id)
    expect(n?.type === 'layer' && n.content.kind === 'line' && n.content.points).toEqual([{ x: 0, y: 0 }, { x: 3, y: 0 }])
  })

  it('uses the pen options as style and ignores single points', () => {
    const { store, tools } = setup()
    prefs.line.color = '#00ff00'
    prefs.line.kind = 'hollow'
    click(store, tools.pen, 0.5, 0.5)
    tools.pen.key?.(key('Escape'))
    expect(lineCount(store)).toBe(0)
    click(store, tools.pen, 0.5, 0.5)
    click(store, tools.pen, 2.5, 0.5)
    tools.pen.dblclick?.(at(store, 2.5, 0.5))
    expect(store.state.fragments[0].svg).toContain('#00ff00')
    expect(store.state.fragments[0].svg).toContain('#ffffff')
  })
})

describe('eraser', () => {
  it('splits a line with one undo step per stroke', () => {
    const { store, tools } = setup()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 10, y: 0 }], {}))
    const er = tools.eraser
    er.down?.(at(store, 3.5, 0.5))
    er.move?.(at(store, 6.5, 0.5), true)
    er.up?.(at(store, 6.5, 0.5))
    expect(lineCount(store)).toBe(2)
    store.undo()
    expect(lineCount(store)).toBe(1)
  })

  it('shows a brush overlay sized by preference', () => {
    const { store, tools } = setup()
    prefs.eraserSize = 3
    tools.eraser.move?.(at(store, 1.5, 1.5), false)
    expect(store.state.overlay.cells).toHaveLength(9)
  })
})

describe('station tool', () => {
  it('places dot stations with labels on lines only', () => {
    const { store, tools } = setup()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 10, y: 0 }], {}))
    click(store, tools.station, 4.5, 0.5)
    expect(store.state.fragments.map((f) => f.kind)).toEqual(['line', 'station', 'text'])
    click(store, tools.station, 4.5, 8.5)
    expect(store.state.messageKind).toBe('error')
    expect(store.state.fragments).toHaveLength(3)
  })

  it('previews the snapped station when hovering a line', () => {
    const { store, tools } = setup()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 10, y: 0 }], {}))
    tools.station.move?.(at(store, 4.5, 1.2), false)
    expect(store.state.overlay.cells).toEqual([{ x: 4, y: 0 }])
    expect(store.state.overlay.preview).toContain('<path')
    tools.station.move?.(at(store, 4.5, 9), false)
    expect(store.state.overlay.cellsColor).toBe('#ff4d4f')
  })

  it('paints pill stations by dragging', () => {
    const { store, tools } = setup()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 10, y: 0 }], {}))
    store.run((e) => e.createLine([{ x: 0, y: 1 }, { x: 10, y: 1 }], {}))
    prefs.station.kind = 'pill'
    tools.station.down?.(at(store, 5.5, 0.5))
    tools.station.move?.(at(store, 5.5, 1.5), true)
    tools.station.up?.(at(store, 5.5, 1.5))
    const st = store.state.fragments.find((f) => f.kind === 'station')!
    const n = store.node(st.id)
    expect(n?.type === 'layer' && n.content.kind === 'station' && n.content.line_ids).toHaveLength(2)
  })
})

describe('text and icon tools', () => {
  it('creates text and opens the inline editor', () => {
    const { store, tools } = setup()
    tools.text.down?.(at(store, 3.1, 2.1))
    expect(store.state.textEdit?.isNew).toBe(true)
    const id = store.state.textEdit!.id
    store.state.textEdit!.value = 'Central'
    store.finishTextEdit(true)
    const n = store.node(id)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.text).toBe('Central')
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.pos).toEqual({ x: 3, y: 2 })
    // a single undo removes the whole creation
    store.undo()
    expect(store.state.fragments).toHaveLength(0)
  })

  it('drops icons centered in a cell at the station size', () => {
    const { store, tools } = setup()
    prefs.icon.icon = 'ferry'
    tools.icon.down?.(at(store, 2.2, 3.9))
    const n = store.node(store.state.selection[0])
    expect(n?.type === 'layer' && n.content.kind === 'icon' && n.content).toMatchObject({
      icon: 'ferry',
      pos: { x: 2.5, y: 3.5 },
      size: store.state.settings.station_outer_radius * 2,
    })
    tools.icon.down?.(at(store, 2.2, 3.9, { alt: true }))
    const m = store.node(store.state.selection[0])
    expect(m?.type === 'layer' && m.content.kind === 'icon' && m.content.pos).toEqual({ x: 2.2, y: 3.9 })
  })
})

describe('selection tool', () => {
  it('selects on click, moves on drag as one undo step', () => {
    const { store, tools } = setup()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))!
    const sel = tools.select
    sel.down?.(at(store, 1.5, 0.5, { hitId: l.layer_id }))
    sel.move?.(at(store, 1.5, 3.5, { hitId: l.layer_id }), true)
    sel.up?.(at(store, 1.5, 3.5))
    expect(store.state.selection).toEqual([l.layer_id])
    const n = store.node(l.layer_id)
    expect(n?.type === 'layer' && n.content.kind === 'line' && n.content.points[0]).toEqual({ x: 0, y: 3 })
    store.undo()
    const m = store.node(l.layer_id)
    expect(m?.type === 'layer' && m.content.kind === 'line' && m.content.points[0]).toEqual({ x: 0, y: 0 })
  })

  it('clicking a selected object toggles rotate mode', () => {
    const { store, tools } = setup()
    const t = store.run((e) => e.addText({ pos: { x: 1, y: 1 } }))!
    click(store, tools.select, 1, 1, { hitId: t })
    expect(store.state.transformMode).toBe('move')
    click(store, tools.select, 1, 1, { hitId: t })
    expect(store.state.transformMode).toBe('rotate')
    click(store, tools.select, 1, 1, { hitId: t })
    expect(store.state.transformMode).toBe('move')
  })

  it('rotates the selection with a handle drag', () => {
    const { store, tools } = setup()
    const t = store.run((e) => e.addText({ pos: { x: 2, y: 2 }, text: 'Hi' }))!
    store.select([t])
    const b = store.state.fragments[0].bounds!
    const cx = (b.x + b.w / 2) / 24
    const cy = (b.y + b.h / 2) / 24
    tools.select.down?.(at(store, cx + 2, cy, { handle: 'rotate' }))
    tools.select.move?.(at(store, cx, cy + 2), true)
    expect(store.state.rotation).toBeCloseTo(90)
    tools.select.up?.(at(store, cx, cy + 2))
    const n = store.node(t)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.rotation).toBeCloseTo(90)
    expect(store.state.rotation).toBeNull()
  })

  it('snaps text rotation to 45° steps while dragging', () => {
    const { store, tools } = setup()
    const t = store.run((e) => e.addText({ pos: { x: 2, y: 2 }, text: 'Hi' }))!
    store.select([t])
    const b = store.state.fragments[0].bounds!
    const cx = (b.x + b.w / 2) / 24
    const cy = (b.y + b.h / 2) / 24
    tools.select.down?.(at(store, cx + 2, cy, { handle: 'rotate' }))
    // ~30° swing lands on 45°
    tools.select.move?.(at(store, cx + 2 * Math.cos(Math.PI / 6), cy + 2 * Math.sin(Math.PI / 6)), true)
    expect(store.state.rotation).toBe(45)
    tools.select.up?.(at(store, cx, cy))
    const n = store.node(t)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.rotation).toBe(45)
  })

  it('marquee selects intersecting unlocked layers', () => {
    const { store, tools } = setup()
    const a = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))!
    const b = store.run((e) => e.createLine([{ x: 0, y: 5 }, { x: 4, y: 5 }], {}))!
    const c = store.run((e) => e.createLine([{ x: 0, y: 2 }, { x: 4, y: 2 }], {}))!
    store.run((e) => e.setLocked(c.layer_id, true))
    tools.select.down?.(at(store, -1, -1))
    tools.select.move?.(at(store, 2, 6), true)
    expect(store.state.overlay.marquee).not.toBeNull()
    tools.select.up?.(at(store, 2, 6))
    expect(store.state.selection.sort()).toEqual([a.layer_id, b.layer_id].sort())
    expect(store.state.overlay.marquee).toBeNull()
  })

  it('clicking empty canvas deselects; arrows nudge; Escape cancels', () => {
    const { store, tools } = setup()
    const t = store.run((e) => e.addText({ pos: { x: 1, y: 1 } }))!
    store.select([t])
    expect(tools.select.key?.(key('ArrowRight', { shiftKey: true }))).toBe(true)
    const n = store.node(t)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.pos.x).toBe(6)
    click(store, tools.select, 20, 20)
    expect(store.state.selection).toEqual([])
    store.select([t])
    tools.select.down?.(at(store, 6, 1, { hitId: t }))
    tools.select.move?.(at(store, 9, 1, { hitId: t }), true)
    tools.select.key?.(key('Escape'))
    const m = store.node(t)
    expect(m?.type === 'layer' && m.content.kind === 'text' && m.content.pos.x).toBe(6)
  })

  it('double clicking text starts editing', () => {
    const { store, tools } = setup()
    const t = store.run((e) => e.addText({ text: 'Edit me' }))!
    tools.select.dblclick?.(at(store, 0, 0, { hitId: t }))
    expect(store.state.textEdit).toMatchObject({ id: t, isNew: false, value: 'Edit me' })
  })
})

describe('hand tool', () => {
  it('pans the view while dragging', () => {
    const { store, tools } = setup()
    tools.hand.down?.(at(store, 0, 0))
    tools.hand.move?.(at(store, 2, 1), true)
    tools.hand.up?.(at(store, 2, 1))
    expect(store.state.view).toMatchObject({ x: 48, y: 24 })
    expect(tools.hand.cursor(null)).toBe('grab')
  })
})
