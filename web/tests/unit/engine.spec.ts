// Integration of the TypeScript bridge with the real WebAssembly engine.
import { describe, expect, it } from 'vitest'
import { Engine } from '../../src/core/engine'

describe('Engine bridge', () => {
  it('creates lines and exposes fragments, tree and bounds', () => {
    const e = new Engine()
    const line = e.createLine([{ x: 0, y: 0 }, { x: 4, y: 2 }], { color: '#ff0000' })
    expect(line.group_id).toBeTypeOf('number')
    const frags = e.fragments()
    expect(frags).toHaveLength(1)
    expect(frags[0]).toMatchObject({ id: line.layer_id, kind: 'line', locked: false })
    expect(frags[0].svg).toContain('#ff0000')
    expect(e.tree()[0]).toMatchObject({ id: line.group_id, kind: 'group', role: 'line' })
    expect(e.bounds([line.group_id])).toEqual({ x: 0, y: 0, w: 120, h: 72 })
    const node = e.node(line.layer_id)
    expect(node?.type).toBe('layer')
    if (node?.type === 'layer' && node.content.kind === 'line') {
      expect(node.content.points).toEqual([{ x: 0, y: 0 }, { x: 2, y: 0 }, { x: 4, y: 2 }])
    }
  })

  it('throws readable errors', () => {
    const e = new Engine()
    expect(() => e.addStation('dot', { x: 0, y: 0 }, {})).toThrow(/line/)
    expect(() => e.loadTransit('{}')).toThrow()
    expect(() => e.createLine([{ x: 1, y: 1 }], {})).toThrow(/two distinct cells/)
  })

  it('supports undo / redo and transactions', () => {
    const e = new Engine()
    expect(e.canUndo()).toBe(false)
    const l = e.createLine([{ x: 0, y: 0 }, { x: 5, y: 0 }], {})
    e.begin()
    for (let i = 1; i <= 3; i++) {
      e.rebase()
      e.translate([l.group_id], i, 0)
    }
    e.commit()
    e.undo()
    const n = e.node(l.layer_id)
    expect(n?.type === 'layer' && n.content.kind === 'line' && n.content.points[0]).toEqual({ x: 0, y: 0 })
    expect(e.canRedo()).toBe(true)
    e.redo()
    const m = e.node(l.layer_id)
    expect(m?.type === 'layer' && m.content.kind === 'line' && m.content.points[0]).toEqual({ x: 3, y: 0 })
  })

  it('round-trips .transit files and exports SVG', () => {
    const e = new Engine()
    e.createLine([{ x: 0, y: 0 }, { x: 5, y: 0 }], { kind: 'dashed' })
    e.addStation('tick', { x: 2, y: 0 }, { tick_side: -1 })
    e.addText({ text: 'Hello', pos: { x: 1, y: 3 } })
    const saved = e.saveTransit()
    const other = new Engine()
    other.loadTransit(saved)
    expect(other.saveTransit()).toBe(saved)
    expect(other.fragments().map((f) => f.kind)).toEqual(['line', 'station', 'text', 'text'])
    const svg = other.exportSvg()
    expect(svg).toMatch(/^<\?xml/)
    expect(svg).toContain('stroke-dasharray')
    expect(svg).toContain('<svg xmlns="http://www.w3.org/2000/svg"')
    expect(svg.trim().endsWith('</svg>')).toBe(true)
  })

  it('erases cells into separate segments', () => {
    const e = new Engine()
    e.createLine([{ x: 0, y: 0 }, { x: 8, y: 0 }], {})
    expect(e.erase([{ x: 4, y: 0 }])).toBe(1)
    expect(e.fragments().filter((f) => f.kind === 'line')).toHaveLength(2)
  })

  it('provides previews, routing and hits', () => {
    const e = new Engine()
    expect(e.route({ x: 0, y: 0 }, { x: 3, y: 1 }, 'diagonal_first')).toEqual([{ x: 0, y: 0 }, { x: 1, y: 1 }, { x: 3, y: 1 }])
    expect(e.previewLine([{ x: 0, y: 0 }, { x: 3, y: 0 }], {})).toContain('<path')
    expect(e.previewPill([{ x: 0, y: 0 }], '#000', '#fff')).toContain('stroke-linecap="round"')
    const l = e.createLine([{ x: 0, y: 0 }, { x: 3, y: 0 }], {})
    expect(e.lineHitAt({ x: 2, y: 0 })?.layer_id).toBe(l.layer_id)
    expect(e.nearestLineCell(2.5, 1.9, 1.5)?.cell).toEqual({ x: 2, y: 0 })
    expect(Engine.iconLibrary().map((i) => i.name)).toEqual(expect.arrayContaining(['wheelchair', 'airport', 'ferry', 'rail', 'parking']))
  })
})
