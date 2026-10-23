// Screen <-> world conversions. World units are pixels at zoom 1 (cells ×
// CELL_PX); screen = world × zoom + offset.
import type { Cell, Rect, Vec2 } from './types'

/** Pixels per cell on the canvas at 100% zoom (the engine's `CELL_PX`). */
export const CELL_PX = 24

export interface View {
  x: number
  y: number
  zoom: number
}

export function screenToWorld(view: View, sx: number, sy: number): Vec2 {
  return { x: (sx - view.x) / view.zoom, y: (sy - view.y) / view.zoom }
}

export function worldToScreen(view: View, wx: number, wy: number): Vec2 {
  return { x: wx * view.zoom + view.x, y: wy * view.zoom + view.y }
}

/** Convert world pixels to cell units. */
export function toCells(p: Vec2, grid: number): Vec2 {
  return { x: p.x / grid, y: p.y / grid }
}

export function cellAt(p: Vec2, grid: number): Cell {
  return { x: Math.floor(p.x / grid), y: Math.floor(p.y / grid) }
}

export function clampZoom(z: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, z))
}

/** Zoom by `factor` keeping the screen point (sx, sy) fixed. */
export function zoomAt(view: View, sx: number, sy: number, factor: number, min = 0.05, max = 32): View {
  const zoom = clampZoom(view.zoom * factor, min, max)
  const k = zoom / view.zoom
  return { zoom, x: sx - (sx - view.x) * k, y: sy - (sy - view.y) * k }
}

/** View that fits `rect` (world px) into a viewport of the given size. */
export function fitRect(rect: Rect, width: number, height: number, margin = 40, min = 0.05, max = 8): View {
  const w = Math.max(rect.w, 1)
  const h = Math.max(rect.h, 1)
  const zoom = clampZoom(Math.min((width - margin * 2) / w, (height - margin * 2) / h), min, max)
  return {
    zoom,
    x: width / 2 - (rect.x + rect.w / 2) * zoom,
    y: height / 2 - (rect.y + rect.h / 2) * zoom,
  }
}

export function rectFromPoints(a: Vec2, b: Vec2): Rect {
  return { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(a.x - b.x), h: Math.abs(a.y - b.y) }
}

export function rectsIntersect(a: Rect, b: Rect): boolean {
  return a.x <= b.x + b.w && b.x <= a.x + a.w && a.y <= b.y + b.h && b.y <= a.y + a.h
}

export function unionRects(rects: (Rect | null | undefined)[]): Rect | null {
  let acc: Rect | null = null
  for (const r of rects) {
    if (!r) continue
    if (!acc) {
      acc = { ...r }
      continue
    }
    const x0 = Math.min(acc.x, r.x)
    const y0 = Math.min(acc.y, r.y)
    const x1 = Math.max(acc.x + acc.w, r.x + r.w)
    const y1 = Math.max(acc.y + acc.h, r.y + r.h)
    acc = { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
  }
  return acc
}

/** Every cell on the straight run from a to b (inclusive) so fast drags skip nothing. */
export function cellsBetween(a: Cell, b: Cell): Cell[] {
  const dx = b.x - a.x
  const dy = b.y - a.y
  const n = Math.max(Math.abs(dx), Math.abs(dy))
  if (n === 0) return [{ ...a }]
  const out: Cell[] = []
  for (let i = 0; i <= n; i++) {
    const t = i / n
    out.push({ x: Math.round(a.x + dx * t), y: Math.round(a.y + dy * t) })
  }
  return out
}

/** Square brush of `size` cells centered on `c`. */
export function brush(c: Cell, size: number): Cell[] {
  const s = Math.max(1, Math.floor(size))
  const r0 = -Math.floor((s - 1) / 2)
  const out: Cell[] = []
  for (let dy = 0; dy < s; dy++) for (let dx = 0; dx < s; dx++) out.push({ x: c.x + r0 + dx, y: c.y + r0 + dy })
  return out
}

export function snap(v: number, step: number): number {
  return step > 0 ? Math.round(v / step) * step : v
}
