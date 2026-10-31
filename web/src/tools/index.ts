// Pointer and keyboard behavior of each canvas tool.
import type { Cell, Id, Rect, Shape, ShapeKind, Vec2 } from '../core/types'
import { ancestorsOf, findItem, leafIds } from '../core/tree'
import { CELL_PX, brush, cellsBetween, rectFromPoints, rectsIntersect } from '../core/viewport'
import type { EditorStore, ToolId } from '../stores/editor'
import { prefs } from '../stores/prefs'
import { nudgeSelection } from '../stores/commands'
import { plainParagraphs } from '../core/richtext'

export interface PointerInfo {
  /** Position relative to the canvas element. */
  screen: Vec2
  /** World pixels. */
  world: Vec2
  /** World position in cell units. */
  cells: Vec2
  cell: Cell
  button: number
  buttons: number
  shift: boolean
  alt: boolean
  ctrl: boolean
  /** Topmost unlocked leaf layer under the pointer. */
  hitId: Id | null
  /** Overlay handle under the pointer ("rotate", "grow"), if any. */
  handle: string | null
  /** Cell a "grow" handle points into. */
  handleCell?: Cell | null
}

export interface Tool {
  cursor(p: PointerInfo | null): string
  down?(p: PointerInfo): void
  move?(p: PointerInfo, pressed: boolean): void
  up?(p: PointerInfo): void
  dblclick?(p: PointerInfo): void
  /** Return true when the key was consumed. */
  key?(e: KeyboardEvent): boolean
  /** Called when the tool is deactivated or the gesture is aborted. */
  reset?(): void
}

/** Rect of leaf layers in world px, measured from the DOM when possible. */
export type BoundsProvider = (ids: Id[]) => Map<Id, Rect>

function sameCell(a: Cell | undefined, b: Cell): boolean {
  return !!a && a.x === b.x && a.y === b.y
}

/** The pill station the selection consists of, if it is exactly one pill (or its station group). */
export function selectedPill(store: EditorStore): { id: Id; cells: Cell[]; offset: Vec2 } | null {
  const s = store.state
  if (s.selection.length !== 1) return null
  const stations = leafIds(s.tree, s.selection).filter((id) => findItem(s.tree, id)?.kind === 'station')
  if (stations.length !== 1) return null
  const n = store.node(stations[0])
  if (n?.type !== 'layer' || n.content.kind !== 'station' || n.content.shape !== 'pill') return null
  return { id: n.id, cells: n.content.cells, offset: n.content.offset ?? { x: 0, y: 0 } }
}

/** Empty cells around a pill (8 directions), each with the direction from the pill cell it extends. */
export function pillGrowTargets(cells: Cell[]): { cell: Cell; dx: number; dy: number }[] {
  const key = (c: Cell) => `${c.x},${c.y}`
  const taken = new Set(cells.map(key))
  const out = new Map<string, { cell: Cell; dx: number; dy: number }>()
  // Orthogonal neighbors first so a cell reachable both ways points straight.
  const dirs = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [1, -1], [-1, 1], [-1, -1]]
  for (const [dx, dy] of dirs) {
    for (const c of cells) {
      const n = { x: c.x + dx, y: c.y + dy }
      if (!taken.has(key(n)) && !out.has(key(n))) out.set(key(n), { cell: n, dx, dy })
    }
  }
  return [...out.values()]
}

function clearOverlay(store: EditorStore): void {
  store.state.overlay = { preview: '', marquee: null, cells: [], cellsColor: prefs.selectionColor }
}

// ----------------------------------------------------------------- hand

function handTool(store: EditorStore): Tool {
  let last: Vec2 | null = null
  return {
    cursor: () => (last ? 'grabbing' : 'grab'),
    down(p) {
      last = p.screen
    },
    move(p, pressed) {
      if (!pressed || !last) return
      store.panBy(p.screen.x - last.x, p.screen.y - last.y)
      last = p.screen
    },
    up() {
      last = null
    },
    reset() {
      last = null
    },
  }
}

// --------------------------------------------------------------- select

function selectTool(store: EditorStore, boundsOf: BoundsProvider): Tool {
  type Mode = 'idle' | 'pending' | 'moving' | 'marquee' | 'rotating'
  let mode: Mode = 'idle'
  let start: PointerInfo | null = null
  let wasSelected = false
  let pivot: Vec2 = { x: 0, y: 0 }
  let startAngle = 0
  const s = store.state

  const selectionCenter = (): Vec2 | null => {
    const rects = [...boundsOf(leafIds(s.tree, s.selection)).values()]
    if (!rects.length) return null
    const x0 = Math.min(...rects.map((r) => r.x))
    const y0 = Math.min(...rects.map((r) => r.y))
    const x1 = Math.max(...rects.map((r) => r.x + r.w))
    const y1 = Math.max(...rects.map((r) => r.y + r.h))
    return { x: (x0 + x1) / 2, y: (y0 + y1) / 2 }
  }

  const angleAt = (p: PointerInfo) => (Math.atan2(p.world.y - pivot.y, p.world.x - pivot.x) * 180) / Math.PI

  return {
    cursor(p) {
      if (mode === 'moving') return 'move'
      if (mode === 'rotating' || p?.handle === 'rotate') return 'alias'
      return 'default'
    },
    down(p) {
      if (p.button !== 0) return
      // Pill arrows act on double click only.
      if (p.handle === 'grow') return
      start = p
      if (p.handle === 'rotate' && s.selection.length) {
        const c = selectionCenter()
        if (!c) return
        pivot = c
        startAngle = angleAt(p)
        store.engine.value.begin()
        mode = 'rotating'
        return
      }
      if (p.hitId !== null) {
        const chain = [p.hitId, ...ancestorsOf(s.tree, p.hitId)]
        const selected = chain.find((id) => s.selection.includes(id))
        wasSelected = selected !== undefined
        if (p.shift) {
          store.select([selected ?? store.pickTarget(p.hitId)], 'toggle')
          wasSelected = false
        } else if (!wasSelected) {
          store.select([store.pickTarget(p.hitId)])
          s.transformMode = 'move'
        }
        mode = 'pending'
        return
      }
      mode = 'marquee'
    },
    move(p, pressed) {
      if (!pressed || !start) return
      const g = CELL_PX
      if (mode === 'pending') {
        const d = Math.hypot(p.screen.x - start.screen.x, p.screen.y - start.screen.y)
        if (d < prefs.dragThreshold || s.selection.length === 0) return
        store.engine.value.begin()
        mode = 'moving'
      }
      if (mode === 'moving') {
        // The engine snaps the move to the cells (Alt: free, in tenths of a cell).
        let dx = (p.world.x - start.world.x) / g
        let dy = (p.world.y - start.world.y) / g
        if (p.shift) {
          if (Math.abs(dx) > Math.abs(dy)) dy = 0
          else dx = 0
        }
        store.run(
          (e) => {
            e.rebase()
            e.translate(s.selection, dx, dy, p.alt)
          },
          { quiet: true },
        )
      } else if (mode === 'marquee') {
        s.overlay = { ...s.overlay, marquee: rectFromPoints(start.world, p.world) }
      } else if (mode === 'rotating') {
        // The engine snaps rotations to 45° steps; with Alt, text and
        // pictograms turn freely (lines and stations stay on the grid).
        const deg = angleAt(p) - startAngle
        const pv = { x: pivot.x / g, y: pivot.y / g }
        const applied = store.run(
          (e) => {
            e.rebase()
            return e.rotate(s.selection, deg, pv, p.alt)
          },
          { quiet: true },
        )
        s.rotation = applied ?? null
      }
    },
    up(p) {
      const e = store.engine.value
      if (mode === 'pending') {
        if (wasSelected && !p.shift) s.transformMode = s.transformMode === 'move' ? 'rotate' : 'move'
      } else if (mode === 'moving' || mode === 'rotating') {
        e.commit()
        store.refresh()
        s.rotation = null
      } else if (mode === 'marquee' && start) {
        const rect = s.overlay.marquee
        if (rect && (rect.w > 2 || rect.h > 2)) {
          const candidates = s.fragments.filter((f) => !f.locked).map((f) => f.id)
          const boxes = boundsOf(candidates)
          const hits = candidates.filter((id) => {
            const b = boxes.get(id)
            return b && rectsIntersect(b, rect)
          })
          store.select([...new Set(hits.map((id) => store.pickTarget(id)))], p.shift ? 'add' : 'replace')
        } else if (!p.shift) {
          store.clearSelection()
        }
        s.overlay = { ...s.overlay, marquee: null }
      }
      mode = 'idle'
      start = null
    },
    dblclick(p) {
      const pill = selectedPill(store)
      if (pill && p.handle === 'grow' && p.handleCell) {
        const cell = p.handleCell
        if (store.run((e) => e.setPillCells(pill.id, [...pill.cells, cell])) !== undefined) store.notify('Pill extended')
        return
      }
      if (p.hitId === null) return
      if (pill && p.hitId === pill.id && pill.cells.length > 1) {
        // Double clicking one of the pill's own cells removes it.
        const at = { x: Math.floor(p.cells.x - pill.offset.x), y: Math.floor(p.cells.y - pill.offset.y) }
        const rest = pill.cells.filter((c) => c.x !== at.x || c.y !== at.y)
        if (rest.length < pill.cells.length) store.run((e) => e.setPillCells(pill.id, rest))
        return
      }
      const n = findItem(s.tree, p.hitId)
      if (n?.kind === 'text') store.startTextEdit(p.hitId)
    },
    key(ev) {
      const step = ev.shiftKey ? prefs.bigNudge : prefs.nudge
      const dirs: Record<string, [number, number]> = {
        ArrowLeft: [-step, 0],
        ArrowRight: [step, 0],
        ArrowUp: [0, -step],
        ArrowDown: [0, step],
      }
      const d = dirs[ev.key]
      if (d && !ev.ctrlKey && !ev.metaKey && s.selection.length) {
        nudgeSelection(store, d[0], d[1])
        return true
      }
      if (ev.key === 'Escape') {
        if (mode === 'moving' || mode === 'rotating') {
          store.engine.value.cancel()
          store.refresh()
          mode = 'idle'
          s.rotation = null
        } else store.clearSelection()
        return true
      }
      return false
    },
    reset() {
      if (mode === 'moving' || mode === 'rotating') {
        store.engine.value.commit()
        store.refresh()
      }
      mode = 'idle'
      s.rotation = null
    },
  }
}

// ------------------------------------------------------------------ pen

function penTool(store: EditorStore): Tool {
  let points: Cell[] = []
  let hover: Cell | null = null
  const s = store.state

  /** Style of the line the pen adds to, or the pen defaults for a new line. */
  const style = () => {
    if (s.penTarget === null) return prefs.line
    for (const f of s.fragments) {
      const n = f.kind === 'line' ? store.node(f.id) : null
      if (n?.type === 'layer' && n.content.kind === 'line' && n.content.line_id === s.penTarget) return n.content.style
    }
    return prefs.line
  }

  const update = () => {
    if (points.length && hover) {
      s.overlay = {
        ...s.overlay,
        preview: store.engine.value.previewLine([...points, hover], style()),
        cells: points.slice(),
        cellsColor: prefs.selectionColor,
      }
    } else {
      s.overlay = { ...s.overlay, preview: '', cells: hover ? [hover] : [], cellsColor: prefs.selectionColor }
    }
  }

  const finish = () => {
    const pts = points
    points = []
    const distinct = pts.filter((c, i) => i === 0 || !sameCell(pts[i - 1], c))
    if (distinct.length >= 2) {
      const target = s.penTarget
      const created = store.run((e) => (target === null ? e.createLine(distinct, prefs.line) : e.addSegment(target, distinct)))
      if (created) {
        store.select([created.group_id])
        store.notify(target === null ? 'Line created' : 'Segment added to the line')
      }
    }
    update()
  }

  return {
    cursor: () => 'crosshair',
    down(p) {
      if (p.button !== 0) return
      if (points.length && sameCell(points[points.length - 1], p.cell)) {
        finish()
        return
      }
      points.push(p.cell)
      hover = p.cell
      update()
    },
    move(p) {
      if (!sameCell(hover ?? undefined, p.cell)) {
        hover = p.cell
        update()
      }
    },
    dblclick() {
      if (points.length) finish()
    },
    key(e) {
      if (!points.length) return false
      if (e.key === 'Enter' || e.key === 'Escape') {
        finish()
        return true
      }
      if (e.key === 'Backspace' || e.key === 'Delete') {
        points.pop()
        update()
        return true
      }
      return false
    },
    reset() {
      if (points.length >= 2) finish()
      points = []
      hover = null
      clearOverlay(store)
    },
  }
}

// --------------------------------------------------------------- eraser

function eraserTool(store: EditorStore): Tool {
  let last: Cell | null = null
  const s = store.state
  const show = (c: Cell) => {
    s.overlay = { ...s.overlay, preview: '', cells: brush(c, prefs.eraserSize), cellsColor: prefs.warningColor }
  }
  const eraseRun = (from: Cell, to: Cell) => {
    const cells = cellsBetween(from, to).flatMap((c) => brush(c, prefs.eraserSize))
    store.run((e) => e.erase(cells), { quiet: true })
  }
  return {
    cursor: () => 'cell',
    down(p) {
      if (p.button !== 0) return
      store.engine.value.begin()
      last = p.cell
      eraseRun(p.cell, p.cell)
      show(p.cell)
    },
    move(p, pressed) {
      show(p.cell)
      if (pressed && last && !sameCell(last, p.cell)) {
        eraseRun(last, p.cell)
        last = p.cell
      }
    },
    up() {
      if (last) {
        store.engine.value.commit()
        store.refresh()
      }
      last = null
    },
    reset() {
      if (last) {
        store.engine.value.commit()
        store.refresh()
      }
      last = null
      clearOverlay(store)
    },
  }
}

// -------------------------------------------------------------- station

function stationTool(store: EditorStore): Tool {
  let pill: Cell[] | null = null
  let last: Cell | null = null
  const s = store.state
  const opts = () => {
    const { kind: _kind, ...rest } = prefs.station
    return rest
  }
  const hover = (p: PointerInfo) => {
    const e = store.engine.value
    if (pill) {
      s.overlay = { ...s.overlay, preview: e.previewPill(pill, prefs.station.outer_color, prefs.station.inner_color), cells: [] }
      return
    }
    const hit = e.nearestLineCell(p.cells.x, p.cells.y, prefs.station.snap_distance)
    if (!hit) {
      s.overlay = { ...s.overlay, preview: '', cells: [p.cell], cellsColor: prefs.warningColor }
      return
    }
    const preview = e.previewStation(prefs.station.kind, p.cell, p.cells, opts())
    s.overlay = { ...s.overlay, preview, cells: [hit.cell], cellsColor: prefs.selectionColor }
  }
  return {
    cursor: () => 'crosshair',
    down(p) {
      if (p.button !== 0) return
      if (prefs.station.kind === 'pill') {
        pill = [p.cell]
        last = p.cell
        hover(p)
        return
      }
      const created = store.run((e) => e.addStation(prefs.station.kind, p.cell, opts(), p.cells))
      if (created) store.select([created.station_id])
      hover(p)
    },
    move(p, pressed) {
      if (pill && pressed && last && !sameCell(last, p.cell)) {
        for (const c of cellsBetween(last, p.cell)) if (!pill.some((q) => sameCell(q, c))) pill.push(c)
        last = p.cell
      }
      hover(p)
    },
    up(p) {
      if (!pill) return
      const cells = pill
      pill = null
      last = null
      const created = store.run((e) => e.addPill(cells, opts()))
      if (created) store.select([created.station_id])
      hover(p)
    },
    reset() {
      pill = null
      last = null
      clearOverlay(store)
    },
  }
}

// ----------------------------------------------------------------- text

function textTool(store: EditorStore): Tool {
  return {
    cursor: () => 'text',
    down(p) {
      if (p.button !== 0) return
      if (store.state.textEdit) {
        store.finishTextEdit(true)
        return
      }
      if (p.hitId !== null && findItem(store.state.tree, p.hitId)?.kind === 'text') {
        store.startTextEdit(p.hitId)
        return
      }
      const pos = placeTextAt(p)
      const e = store.engine.value
      e.begin()
      const id = store.run((en) => en.addText({ ...prefs.text, pos, paragraphs: plainParagraphs(prefs.newText || 'Text') }))
      if (id === undefined) {
        e.cancel()
        return
      }
      store.startTextEdit(id, true)
    },
  }
}

// ---------------------------------------------------------------- shape

type DragPoint = Pick<PointerInfo, 'cells' | 'cell'>

/**
 * Turn `d` into the nearest of the 8 directions, keeping it on grid
 * intersections: along an axis it keeps its length on that axis, on a
 * diagonal both sides get the average of their lengths.
 */
function eightWay(d: Vec2): Vec2 {
  const ax = Math.abs(d.x)
  const ay = Math.abs(d.y)
  if (ax > 2 * ay) return { x: d.x, y: 0 }
  if (ay > 2 * ax) return { x: 0, y: d.y }
  const n = Math.round((ax + ay) / 2)
  return { x: Math.sign(d.x) * n, y: Math.sign(d.y) * n }
}

const tenth = (v: number) => Math.round(v * 10) / 10

/**
 * The shape a drag from `a` to `b` draws, or null when it is too small. A
 * line goes between grid intersections in one of the 8 directions; a
 * rectangle covers every cell from `a` to `b`; a circle is centered on the
 * cell of `a` with a radius in half cells. With `free` (Alt) points and sizes
 * follow the pointer in tenths of a cell; `square` (Shift) makes rectangles
 * square and keeps free lines in the 8 directions.
 */
export function shapeFromDrag(kind: ShapeKind, a: DragPoint, b: DragPoint, free: boolean, square: boolean): Shape | null {
  const st = prefs.shape
  if (kind === 'line') {
    const snap = (p: Vec2) => (free ? { x: tenth(p.x), y: tenth(p.y) } : { x: Math.round(p.x), y: Math.round(p.y) })
    const from = snap(a.cells)
    let d = { x: snap(b.cells).x - from.x, y: snap(b.cells).y - from.y }
    if (!free) d = eightWay(d)
    else if (square) {
      const e = eightWay({ x: Math.round(d.x * 10), y: Math.round(d.y * 10) })
      d = { x: e.x / 10, y: e.y / 10 }
    }
    if (d.x === 0 && d.y === 0) return null
    const { color, filled, border_color, border_width, thickness, cap, dashed, dash_length, dash_gap } = st.line
    const to = { x: tenth(from.x + d.x), y: tenth(from.y + d.y) }
    return { shape: 'line', from, to, thickness, cap, dashed, dash_length, dash_gap, color, filled, border_color, border_width }
  }
  if (kind === 'rect') {
    let x0: number, y0: number, x1: number, y1: number
    if (free) {
      [x0, y0, x1, y1] = [a.cells.x, a.cells.y, b.cells.x, b.cells.y].map(tenth)
    } else {
      // Every cell from the pressed one to the one under the pointer.
      x0 = a.cell.x + (b.cell.x < a.cell.x ? 1 : 0)
      y0 = a.cell.y + (b.cell.y < a.cell.y ? 1 : 0)
      x1 = b.cell.x + (b.cell.x < a.cell.x ? 0 : 1)
      y1 = b.cell.y + (b.cell.y < a.cell.y ? 0 : 1)
    }
    if (square) {
      const side = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0))
      x1 = x0 + (x1 < x0 ? -side : side)
      y1 = y0 + (y1 < y0 ? -side : side)
    }
    const width = tenth(Math.abs(x1 - x0))
    const height = tenth(Math.abs(y1 - y0))
    if (width < 0.1 || height < 0.1) return null
    return { shape: 'rect', pos: { x: Math.min(x0, x1), y: Math.min(y0, y1) }, width, height, rotation: 0, ...st.rect }
  }
  const center = free ? { x: tenth(a.cells.x), y: tenth(a.cells.y) } : { x: a.cell.x + 0.5, y: a.cell.y + 0.5 }
  const dist = Math.hypot(b.cells.x - center.x, b.cells.y - center.y)
  const radius = free ? tenth(dist) : Math.max(0.5, Math.round(dist * 2) / 2)
  if (radius < 0.1) return null
  return { shape: 'circle', center, radius, ...st.circle }
}

function shapeTool(store: EditorStore): Tool {
  let start: PointerInfo | null = null
  const s = store.state
  const hover = (p: PointerInfo) => {
    // Rectangles and circles start from the cell under the pointer.
    const cells = prefs.shape.kind === 'line' || p.alt ? [] : [p.cell]
    s.overlay = { ...s.overlay, preview: '', cells, cellsColor: prefs.selectionColor }
  }
  const draw = (p: PointerInfo) => {
    const shape = start && shapeFromDrag(prefs.shape.kind, start, p, p.alt, p.shift)
    s.overlay = { ...s.overlay, preview: shape ? store.engine.value.previewShape(shape) : '', cells: [] }
    return shape
  }
  return {
    cursor: () => 'crosshair',
    down(p) {
      if (p.button !== 0) return
      start = p
      draw(p)
    },
    move(p, pressed) {
      if (start && pressed) draw(p)
      else hover(p)
    },
    up(p) {
      if (!start) return
      const shape = draw(p)
      start = null
      hover(p)
      if (!shape) {
        if (prefs.shape.kind === 'line') store.notify('Drag to draw a straight line')
        return
      }
      const id = store.run((e) => e.addShape(shape))
      if (id !== undefined) {
        store.select([id])
        store.notify(`${findItem(s.tree, id)?.name ?? 'Shape'} drawn`)
      }
    },
    key(e) {
      if (e.key !== 'Escape' || !start) return false
      start = null
      clearOverlay(store)
      return true
    },
    reset() {
      start = null
      clearOverlay(store)
    },
  }
}

// ----------------------------------------------------------------- icon

/** Where a pictogram placed at `p` is centered: its cell center, or exactly under the pointer with Alt. */
export function placeIconAt(p: Pick<PointerInfo, 'alt' | 'cells' | 'cell'>): Vec2 {
  return p.alt ? p.cells : { x: p.cell.x + 0.5, y: p.cell.y + 0.5 }
}

/** Top-left corner of a text placed at `p`: the nearest grid intersection, or exactly under the pointer with Alt. */
export function placeTextAt(p: Pick<PointerInfo, 'alt' | 'cells'>): Vec2 {
  return p.alt ? p.cells : { x: Math.round(p.cells.x), y: Math.round(p.cells.y) }
}

/** Size of new pictograms, in cells. */
export function newIconSize(store: EditorStore): number {
  const s = store.state.settings
  return prefs.icon.size ?? s.icon_size ?? s.station_outer_radius * 2
}

/** Add a pictogram with the Icons panel's colors and size, and select it. */
export function placeIcon(store: EditorStore, icon: string, pos: Vec2): void {
  const id = store.run((e) => e.addIcon({ icon, color: prefs.icon.color, background: prefs.icon.background, size: newIconSize(store), pos }))
  if (id !== undefined) store.select([id])
}

/** Outline shown where a pictogram dragged from the Icons panel would land. */
export function iconPreview(store: EditorStore, pos: Vec2): string {
  const g = CELL_PX
  const size = newIconSize(store) * g
  const x = pos.x * g - size / 2
  const y = pos.y * g - size / 2
  return `<rect x="${x}" y="${y}" width="${size}" height="${size}" rx="${size * 0.2}" fill="${prefs.icon.background}" opacity="0.45"/>`
}

export function createTools(store: EditorStore, boundsOf: BoundsProvider): Record<ToolId, Tool> {
  return {
    select: selectTool(store, boundsOf),
    hand: handTool(store),
    pen: penTool(store),
    eraser: eraserTool(store),
    station: stationTool(store),
    text: textTool(store),
    shape: shapeTool(store),
  }
}
