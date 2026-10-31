// Tool behavior driven with synthetic pointer events against the real engine.
import { describe, expect, it } from 'vitest'
import { at, freshStore, key } from './helpers'
import { plainParagraphs, plainText } from '../../src/core/richtext'
import { createTools, pillGrowTargets, placeIconAt, placeIcon, resizeShape, selectedPill, shapeFromDrag, shapeHandles } from '../../src/tools'
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

describe('drawing into an existing line', () => {
  it('picking the pen with a line selected adds a disconnected segment to it', () => {
    const { store, tools } = setup()
    const a = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], { color: '#ff0000' }))!
    store.select([a.group_id])
    store.setTool('pen')
    expect(store.state.penTarget).toBe(a.line_id)
    click(store, tools.pen, 0.5, 5.5)
    click(store, tools.pen, 4.5, 5.5)
    tools.pen.key?.(key('Enter'))
    expect(store.state.tree).toHaveLength(1)
    const segs = store.state.tree[0].children.map((c) => store.node(c.id))
    expect(segs).toHaveLength(2)
    expect(segs.every((n) => n?.type === 'layer' && n.content.kind === 'line' && n.content.line_id === a.line_id)).toBe(true)
    expect(segs.every((n) => n?.type === 'layer' && n.content.kind === 'line' && n.content.style.color === '#ff0000')).toBe(true)
  })

  it('starts a new line when nothing is selected and forgets deleted targets', () => {
    const { store } = setup()
    const a = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))!
    store.clearSelection()
    store.setTool('pen')
    expect(store.state.penTarget).toBeNull()
    store.state.penTarget = a.line_id
    expect(store.lines()).toEqual([{ lineId: a.line_id, name: 'Line 1' }])
    store.run((e) => e.delete([a.group_id]))
    expect(store.state.penTarget).toBeNull()
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

  it('places a tick in the cell corner nearest to the pointer and previews it', () => {
    const { store, tools } = setup()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 10, y: 0 }], {}))
    prefs.station.kind = 'tick'
    tools.station.move?.(at(store, 4.8, 0.5), false)
    expect(store.state.overlay.preview).toContain('<path')
    click(store, tools.station, 4.8, 0.5)
    const st = store.state.fragments.find((f) => f.kind === 'station')!
    const n = store.node(st.id)
    expect(n?.type === 'layer' && n.content.kind === 'station' && n.content.shape === 'tick' && n.content.along).toBe(1)
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

describe('text tool and pictograms', () => {
  it('creates text on the nearest grid intersection and opens the inline editor', () => {
    const { store, tools } = setup()
    tools.text.down?.(at(store, 3.4, 2.6))
    expect(store.state.textEdit?.isNew).toBe(true)
    const id = store.state.textEdit!.id
    store.setTextDraft(plainParagraphs('Central'), null)
    store.finishTextEdit(true)
    const n = store.node(id)
    expect(n?.type === 'layer' && n.content.kind === 'text' && plainText(n.content.paragraphs)).toBe('Central')
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.pos, 'its top-left corner').toEqual({ x: 3, y: 3 })
    // a single undo removes the whole creation
    store.undo()
    expect(store.state.fragments).toHaveLength(0)
  })

  it('places text off the grid with Alt', () => {
    const { store, tools } = setup()
    tools.text.down?.(at(store, 3.1, 2.1, { alt: true }))
    const id = store.state.textEdit!.id
    store.finishTextEdit(true)
    const n = store.node(id)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.pos).toEqual({ x: 3.1, y: 2.1 })
  })

  it('drops pictograms centered in a cell at the station size', () => {
    const { store } = setup()
    placeIcon(store, 'ferry', placeIconAt(at(store, 2.2, 3.9)))
    const n = store.node(store.state.selection[0])
    expect(n?.type === 'layer' && n.content.kind === 'icon' && n.content).toMatchObject({
      icon: 'ferry',
      pos: { x: 2.5, y: 3.5 },
      size: store.state.settings.station_outer_radius * 2,
    })
    placeIcon(store, 'ferry', placeIconAt(at(store, 2.2, 3.9, { alt: true })))
    const m = store.node(store.state.selection[0])
    expect(m?.type === 'layer' && m.content.kind === 'icon' && m.content.pos).toEqual({ x: 2.2, y: 3.9 })
  })

  it('dragged pictograms and text move from cell to cell, or freely with Alt', () => {
    const { store, tools } = setup()
    const i = store.run((e) => e.addIcon({ pos: { x: 2.5, y: 2.5 } }))!
    // A whole number of cells wide and tall, so every corner of the text snaps alike.
    const t = store.run((e) => e.addText({ pos: { x: 6.2, y: 1.3 }, size: 2.5 }))!
    store.run((e) => e.fitTextWidths([[t, 2]]))
    const pos = (id: Id) => {
      const n = store.node(id)
      return n?.type === 'layer' && (n.content.kind === 'icon' || n.content.kind === 'text') ? n.content.pos : null
    }
    const drag = (id: Id, from: [number, number], to: [number, number], alt = false) => {
      tools.select.down?.(at(store, from[0], from[1], { hitId: id }))
      tools.select.move?.(at(store, to[0], to[1], { hitId: id, alt }), true)
      tools.select.up?.(at(store, to[0], to[1], { alt }))
    }
    // A small drag keeps the pictogram in its cell, a longer one moves whole cells.
    drag(i, [2.5, 2.5], [2.8, 2.6])
    expect(pos(i)).toEqual({ x: 2.5, y: 2.5 })
    drag(i, [2.5, 2.5], [4.3, 3.1])
    expect(pos(i)).toEqual({ x: 4.5, y: 3.5 })
    // Text off the grid puts a corner on the nearest grid intersection.
    drag(t, [6.2, 1.3], [7.2, 1.3])
    expect(pos(t)).toEqual({ x: 7, y: 1 })
    // Alt moves text in tenths of a cell too.
    drag(t, [7, 1], [7.33, 1], true)
    expect(pos(t)).toEqual({ x: 7.3, y: 1 })
    // Alt moves in tenths of a cell.
    drag(i, [4.5, 3.5], [4.83, 3.21], true)
    expect(pos(i)).toEqual({ x: 4.8, y: 3.2 })
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

  it('grows a selected pill by double clicking an arrow and shrinks it by double clicking a cell', () => {
    const { store, tools } = setup()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 8, y: 0 }], {}))
    store.run((e) => e.createLine([{ x: 0, y: 1 }, { x: 8, y: 1 }], {}))
    const st = store.run((e) => e.addPill([{ x: 3, y: 0 }], { label: false }))!
    store.select([st.station_id])
    const pill = selectedPill(store)!
    expect(pillGrowTargets(pill.cells)).toHaveLength(8)
    const below = pillGrowTargets(pill.cells).find((t) => t.cell.x === 3 && t.cell.y === 1)!
    expect([below.dx, below.dy]).toEqual([0, 1])
    // A single click on an arrow does nothing; a double click grows.
    tools.select.down?.(at(store, 3.5, 1.5, { handle: 'grow', handleCell: { x: 3, y: 1 } }))
    tools.select.up?.(at(store, 3.5, 1.5, { handle: 'grow', handleCell: { x: 3, y: 1 } }))
    expect(store.state.selection).toEqual([st.station_id])
    tools.select.dblclick?.(at(store, 3.5, 1.5, { handle: 'grow', handleCell: { x: 3, y: 1 } }))
    const cells = () => {
      const n = store.node(st.station_id)
      return n?.type === 'layer' && n.content.kind === 'station' && n.content.shape === 'pill' ? n.content.cells : []
    }
    expect(cells()).toEqual([{ x: 3, y: 0 }, { x: 3, y: 1 }])
    expect(pillGrowTargets(cells())).toHaveLength(10)
    tools.select.dblclick?.(at(store, 3.5, 0.5, { hitId: st.station_id }))
    expect(cells()).toEqual([{ x: 3, y: 1 }])
  })

  it('alt-dragging a station moves it off its line', () => {
    const { store, tools } = setup()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 8, y: 0 }], {}))
    const st = store.run((e) => e.addStation('dot', { x: 3, y: 0 }, { label: false }))!
    const sel = tools.select
    sel.down?.(at(store, 3.5, 0.5, { hitId: st.station_id }))
    sel.move?.(at(store, 4.2, 1.8, { hitId: st.station_id, alt: true }), true)
    sel.up?.(at(store, 4.2, 1.8, { alt: true }))
    const n = store.node(st.station_id)
    expect(n?.type === 'layer' && n.content.kind === 'station' && n.content.offset).toEqual({ x: 0.7, y: 1.3 })
    // Without Alt the same drag snaps back onto the line.
    sel.down?.(at(store, 4.2, 1.8, { hitId: st.station_id }))
    sel.move?.(at(store, 5.2, 1.8, { hitId: st.station_id }), true)
    sel.up?.(at(store, 5.2, 1.8))
    const m = store.node(st.station_id)
    expect(m?.type === 'layer' && m.content.kind === 'station' && m.content.offset).toEqual({ x: 0, y: 0 })
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
    // Nudged text keeps its corner on a grid intersection like any other move.
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.pos).toEqual({ x: 6, y: 1 })
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
    expect(store.state.textEdit).toMatchObject({ id: t, isNew: false })
    expect(plainText(store.state.textEdit!.paragraphs)).toBe('Edit me')
  })
})

describe('shape tool', () => {
  function drag(store: EditorStore, tool: ReturnType<typeof setup>['tools']['shape'], from: [number, number], to: [number, number], extra = {}) {
    tool.down?.(at(store, from[0], from[1], extra))
    tool.move?.(at(store, to[0], to[1], extra), true)
    const preview = store.state.overlay.preview
    tool.up?.(at(store, to[0], to[1], extra))
    return preview
  }
  function shapeOf(store: EditorStore) {
    const n = store.node(store.state.selection[0])
    if (n?.type !== 'layer' || n.content.kind !== 'shape') throw new Error('no shape selected')
    return n.content
  }

  it('draws a rectangle over every cell it was dragged across, as one undo step', () => {
    const { store, tools } = setup()
    prefs.shape.kind = 'rect'
    const depth = store.engine.value.canUndo()
    expect(depth).toBe(false)
    expect(drag(store, tools.shape, [4.2, 3.7], [1.6, 1.2])).toContain('<rect')
    expect(shapeOf(store)).toMatchObject({ shape: 'rect', pos: { x: 1, y: 1 }, width: 4, height: 3, rotation: 0, color: prefs.shape.rect.color })
    expect(store.state.tree[0].name).toBe('Rectangle 1')
    store.undo()
    expect(store.state.tree).toHaveLength(0)
    // A click is one cell; Shift makes a square; Alt follows the pointer in tenths.
    click(store, tools.shape, 2.5, 2.5)
    expect(shapeOf(store)).toMatchObject({ pos: { x: 2, y: 2 }, width: 1, height: 1 })
    drag(store, tools.shape, [0.5, 0.5], [2.5, 5.5], { shift: true })
    expect(shapeOf(store)).toMatchObject({ width: 6, height: 6 })
    drag(store, tools.shape, [0.33, 0.5], [2.04, 1.26], { alt: true })
    expect(shapeOf(store)).toMatchObject({ pos: { x: 0.3, y: 0.5 }, width: 1.7, height: 0.8 })
  })

  it('draws lines between grid intersections in 8 directions, or anywhere with Alt', () => {
    const { store, tools } = setup()
    prefs.shape.kind = 'line'
    expect(drag(store, tools.shape, [0.8, 1.1], [5.2, 1.9])).toContain('<line')
    expect(shapeOf(store)).toMatchObject({ shape: 'line', from: { x: 1, y: 1 }, to: { x: 5, y: 1 }, thickness: 0.1 })
    drag(store, tools.shape, [0, 0], [3.2, 2.6])
    expect(shapeOf(store)).toMatchObject({ from: { x: 0, y: 0 }, to: { x: 3, y: 3 } })
    drag(store, tools.shape, [0, 0], [3.2, 1.06], { alt: true })
    expect(shapeOf(store)).toMatchObject({ from: { x: 0, y: 0 }, to: { x: 3.2, y: 1.1 } })
    // A click draws nothing.
    const count = store.state.tree.length
    click(store, tools.shape, 4, 4)
    expect(store.state.tree).toHaveLength(count)
  })

  it('draws circles centered on a cell with a radius in tenths of a cell; Escape cancels', () => {
    const { store, tools } = setup()
    prefs.shape.kind = 'circle'
    expect(drag(store, tools.shape, [2.3, 2.8], [4.1, 2.6])).toContain('<circle')
    expect(shapeOf(store)).toMatchObject({ shape: 'circle', center: { x: 2.5, y: 2.5 }, radius: 1.6 })
    tools.shape.down?.(at(store, 8, 8))
    tools.shape.move?.(at(store, 10, 8), true)
    expect(tools.shape.key?.(key('Escape'))).toBe(true)
    expect(store.state.overlay.preview).toBe('')
    tools.shape.up?.(at(store, 10, 8))
    expect(store.state.tree).toHaveLength(1)
  })
})

describe('resizing shapes', () => {
  const pt = (x: number, y: number) => ({ cells: { x, y }, cell: { x: Math.floor(x), y: Math.floor(y) } })

  it('drags a rectangle corner onto the grid, keeping the opposite corner', () => {
    const rect = shapeFromDrag('rect', pt(1, 1), pt(3, 2), false, false)!
    expect(shapeHandles(rect)).toEqual([{ x: 1, y: 1 }, { x: 4, y: 1 }, { x: 4, y: 3 }, { x: 1, y: 3 }])
    expect(resizeShape(rect, 2, { x: 6.3, y: 5.8 }, false, false)).toEqual({ pos: { x: 1, y: 1 }, width: 5, height: 5 })
    expect(resizeShape(rect, 0, { x: -0.4, y: 2.2 }, false, false)).toEqual({ pos: { x: 0, y: 2 }, width: 4, height: 1 })
    expect(resizeShape(rect, 2, { x: 6.33, y: 5.81 }, true, false)).toEqual({ pos: { x: 1, y: 1 }, width: 5.3, height: 4.8 })
    expect(resizeShape(rect, 2, { x: 9, y: 2 }, false, true)).toMatchObject({ width: 8, height: 8 })
    // Never smaller than a cell (a tenth with Alt).
    expect(resizeShape(rect, 2, { x: 0, y: 0 }, false, false)).toMatchObject({ width: 1, height: 1 })
  })

  it('moves one end of a line and changes a circle radius in tenths', () => {
    const line = shapeFromDrag('line', pt(0, 0), pt(4, 0), false, false)!
    expect(resizeShape(line, 1, { x: 3.2, y: 2.9 }, false, false)).toEqual({ to: { x: 3, y: 3 } })
    expect(resizeShape(line, 0, { x: 1.04, y: 1.96 }, true, false)).toEqual({ from: { x: 1, y: 2 } })
    const circle = shapeFromDrag('circle', pt(2.5, 2.5), pt(3.5, 2.5), false, false)!
    expect(circle).toMatchObject({ radius: 1 })
    expect(shapeHandles(circle)[2]).toEqual({ x: 3.5, y: 3.5 })
    expect(resizeShape(circle, 2, { x: 4.03, y: 3.1 }, false, false)).toEqual({ radius: 1.5 })
  })

  it('the selection tool resizes with a handle as one undo step', () => {
    const { store, tools } = setup()
    const id = store.run((e) => e.addShape(shapeFromDrag('rect', pt(1, 1), pt(2, 2), false, false)!))!
    store.select([id])
    tools.select.down?.(at(store, 3, 3, { handle: 'resize', handleIndex: 2 }))
    tools.select.move?.(at(store, 5.2, 4.1), true)
    tools.select.up?.(at(store, 5.2, 4.1))
    const n = store.node(id)
    expect(n?.type === 'layer' && n.content).toMatchObject({ pos: { x: 1, y: 1 }, width: 4, height: 3 })
    store.undo()
    const back = store.node(id)
    expect(back?.type === 'layer' && back.content).toMatchObject({ width: 2, height: 2 })
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
