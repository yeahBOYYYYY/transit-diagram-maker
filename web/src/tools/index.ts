// Pointer and keyboard behavior of each canvas tool.
import type { Cell, Id, Rect, Vec2 } from '../core/types'
import { ancestorsOf, findItem, leafIds } from '../core/tree'
import { brush, cellsBetween, rectFromPoints, rectsIntersect, snap } from '../core/viewport'
import type { EditorStore, ToolId } from '../stores/editor'
import { prefs } from '../stores/prefs'
import { nudgeSelection } from '../stores/commands'

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
  /** Overlay handle under the pointer ("rotate"), if any. */
  handle: string | null
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
      const g = s.settings.grid_size
      if (mode === 'pending') {
        const d = Math.hypot(p.screen.x - start.screen.x, p.screen.y - start.screen.y)
        if (d < prefs.dragThreshold || s.selection.length === 0) return
        store.engine.value.begin()
        mode = 'moving'
      }
      if (mode === 'moving') {
        let dx = (p.world.x - start.world.x) / g
        let dy = (p.world.y - start.world.y) / g
        if (!p.alt) {
          dx = snap(dx, prefs.textSnap)
          dy = snap(dy, prefs.textSnap)
        }
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
        // The engine snaps every rotation to 45° steps.
        const deg = angleAt(p) - startAngle
        const pv = { x: pivot.x / g, y: pivot.y / g }
        const applied = store.run(
          (e) => {
            e.rebase()
            return e.rotate(s.selection, deg, pv)
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
      if (p.hitId === null) return
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

  const update = () => {
    if (points.length && hover) {
      s.overlay = {
        ...s.overlay,
        preview: store.engine.value.previewLine([...points, hover], prefs.line),
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
      const created = store.run((e) => e.createLine(distinct, prefs.line))
      if (created) {
        store.select([created.group_id])
        store.notify('Line created')
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
    s.overlay = { ...s.overlay, preview: '', cells: brush(c, prefs.eraserSize), cellsColor: '#ff4d4f' }
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
    const hit = e.nearestLineCell(p.cells.x, p.cells.y, 1.5)
    if (!hit) {
      s.overlay = { ...s.overlay, preview: '', cells: [p.cell], cellsColor: '#ff4d4f' }
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
      const pos = p.alt
        ? p.cells
        : { x: snap(p.cells.x, prefs.textSnap), y: snap(p.cells.y, prefs.textSnap) }
      const e = store.engine.value
      e.begin()
      const id = store.run((en) => en.addText({ ...prefs.text, pos, text: 'Text' }))
      if (id === undefined) {
        e.cancel()
        return
      }
      store.startTextEdit(id, true)
    },
  }
}

// ----------------------------------------------------------------- icon

function iconTool(store: EditorStore): Tool {
  const s = store.state
  const place = (p: PointerInfo): Vec2 => (p.alt ? p.cells : { x: p.cell.x + 0.5, y: p.cell.y + 0.5 })
  return {
    cursor: () => 'copy',
    move(p) {
      const g = s.settings.grid_size
      const size = (prefs.icon.size ?? s.settings.icon_size ?? s.settings.station_outer_radius * 2) * g
      const c = place(p)
      const x = c.x * g - size / 2
      const y = c.y * g - size / 2
      s.overlay = {
        ...s.overlay,
        preview: `<rect x="${x}" y="${y}" width="${size}" height="${size}" rx="${size * 0.2}" fill="${prefs.icon.background}" opacity="0.45"/>`,
        cells: [],
      }
    },
    down(p) {
      if (p.button !== 0) return
      const size = prefs.icon.size ?? s.settings.icon_size ?? s.settings.station_outer_radius * 2
      const id = store.run((e) =>
        e.addIcon({
          icon: prefs.icon.icon,
          color: prefs.icon.color,
          background: prefs.icon.background,
          size,
          pos: place(p),
        }),
      )
      if (id !== undefined) store.select([id])
    },
    reset() {
      clearOverlay(store)
    },
  }
}

export function createTools(store: EditorStore, boundsOf: BoundsProvider): Record<ToolId, Tool> {
  return {
    select: selectTool(store, boundsOf),
    hand: handTool(store),
    pen: penTool(store),
    eraser: eraserTool(store),
    station: stationTool(store),
    text: textTool(store),
    icon: iconTool(store),
  }
}
