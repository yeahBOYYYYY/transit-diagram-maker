import { describe, expect, it } from 'vitest'
import {
  brush,
  cellAt,
  cellsBetween,
  fitRect,
  rectFromPoints,
  rectsIntersect,
  screenToWorld,
  snap,
  unionRects,
  worldToScreen,
  zoomAt,
} from '../../src/core/viewport'

describe('viewport math', () => {
  const view = { x: 100, y: -50, zoom: 2 }
  it('round-trips screen and world coordinates', () => {
    const w = screenToWorld(view, 300, 150)
    expect(w).toEqual({ x: 100, y: 100 })
    expect(worldToScreen(view, w.x, w.y)).toEqual({ x: 300, y: 150 })
  })
  it('zooms around a fixed screen point and clamps', () => {
    const z = zoomAt(view, 300, 150, 2)
    expect(z.zoom).toBe(4)
    expect(screenToWorld(z, 300, 150)).toEqual(screenToWorld(view, 300, 150))
    expect(zoomAt(view, 0, 0, 1000, 0.1, 8).zoom).toBe(8)
    expect(zoomAt(view, 0, 0, 0.0001, 0.1, 8).zoom).toBe(0.1)
  })
  it('fits a rectangle into the viewport', () => {
    const v = fitRect({ x: 0, y: 0, w: 400, h: 200 }, 1000, 600, 100)
    expect(v.zoom).toBe(2)
    expect(worldToScreen(v, 200, 100)).toEqual({ x: 500, y: 300 })
  })
  it('finds the cell under a point including negatives', () => {
    expect(cellAt({ x: 25, y: -1 }, 24)).toEqual({ x: 1, y: -1 })
  })
})

describe('rects and cells', () => {
  it('builds and intersects rectangles', () => {
    const r = rectFromPoints({ x: 10, y: 10 }, { x: 0, y: 4 })
    expect(r).toEqual({ x: 0, y: 4, w: 10, h: 6 })
    expect(rectsIntersect(r, { x: 9, y: 9, w: 5, h: 5 })).toBe(true)
    expect(rectsIntersect(r, { x: 11, y: 0, w: 1, h: 1 })).toBe(false)
  })
  it('unions rectangles ignoring nulls', () => {
    expect(unionRects([null, { x: 0, y: 0, w: 1, h: 1 }, undefined, { x: 2, y: -1, w: 1, h: 1 }])).toEqual({ x: 0, y: -1, w: 3, h: 2 })
    expect(unionRects([])).toBeNull()
  })
  it('interpolates cells without gaps', () => {
    const run = cellsBetween({ x: 0, y: 0 }, { x: 4, y: -2 })
    expect(run[0]).toEqual({ x: 0, y: 0 })
    expect(run[run.length - 1]).toEqual({ x: 4, y: -2 })
    for (let i = 1; i < run.length; i++) {
      expect(Math.abs(run[i].x - run[i - 1].x)).toBeLessThanOrEqual(1)
      expect(Math.abs(run[i].y - run[i - 1].y)).toBeLessThanOrEqual(1)
    }
    expect(cellsBetween({ x: 3, y: 3 }, { x: 3, y: 3 })).toEqual([{ x: 3, y: 3 }])
  })
  it('makes square brushes', () => {
    expect(brush({ x: 5, y: 5 }, 1)).toEqual([{ x: 5, y: 5 }])
    const b3 = brush({ x: 0, y: 0 }, 3)
    expect(b3).toHaveLength(9)
    expect(b3).toContainEqual({ x: -1, y: -1 })
    expect(b3).toContainEqual({ x: 1, y: 1 })
  })
  it('snaps values', () => {
    expect(snap(1.13, 0.25)).toBe(1.25)
    expect(snap(1.13, 0)).toBe(1.13)
  })
})
