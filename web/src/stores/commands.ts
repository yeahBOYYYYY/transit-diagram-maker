// Implementation of every catalogued action.
import type { ActionId } from '../core/actions'
import type { DocSettings } from '../core/types'
import { matches } from '../core/shortcuts'
import { findItem, isEffectivelyLocked, leafIds } from '../core/tree'
import type { Engine } from '../core/engine'
import { canSaveInPlace, downloadText, pickSaveHandle, pickTransitFile, writeHandle } from '../core/files'
import type { EditorStore, ToolId } from './editor'
import { prefs } from './prefs'
import { CELL_PX } from '../core/viewport'

export interface CommandContext {
  store: EditorStore
}

function rotationPivot(store: EditorStore): { x: number; y: number } | null {
  const r = store.contentBounds(store.state.selection)
  if (!r) return null
  const g = CELL_PX
  return { x: (r.x + r.w / 2) / g, y: (r.y + r.h / 2) / g }
}

export function rotateSelection(store: EditorStore, deg: number): void {
  const pivot = rotationPivot(store)
  if (!pivot || store.state.selection.length === 0) return
  store.run((e) => e.rotate(store.state.selection, deg, pivot))
}

/** Whether a selection moves in whole cells (it contains lines). */
export function selectionOnGrid(store: EditorStore): boolean {
  return leafIds(store.state.tree, store.state.selection).some((id) => findItem(store.state.tree, id)?.kind === 'line')
}

/**
 * Move the selection so its position (see `Engine.position`) gets `value` on
 * one axis. Lines move in whole cells; stations moved by a fraction of a cell
 * leave their line, like an Alt-drag.
 */
export function moveSelectionTo(e: Engine, store: EditorStore, axis: 'x' | 'y', value: number): void {
  const sel = store.state.selection
  const pos = e.position(sel)
  if (!pos) return
  const onGrid = selectionOnGrid(store)
  const round = (v: number) => (onGrid ? Math.round(v) : Math.round(v * 10) / 10)
  const dx = axis === 'x' ? round(value - pos.x) : 0
  const dy = axis === 'y' ? round(value - pos.y) : 0
  if (dx === 0 && dy === 0) return
  e.translate(sel, dx, dy, !Number.isInteger(dx) || !Number.isInteger(dy))
}

export function nudgeSelection(store: EditorStore, dx: number, dy: number): void {
  if (store.state.selection.length === 0) return
  store.run((e) => e.translate(store.state.selection, dx, dy))
}

function toggleOnSelection(store: EditorStore, key: 'locked' | 'visible'): void {
  const sel = store.state.selection
  if (sel.length === 0) return
  const items = sel.map((id) => findItem(store.state.tree, id)).filter((x) => x !== null)
  const target = !items.every((i) => i[key])
  store.run((e) => {
    for (const it of items) {
      if (key === 'locked') e.setLocked(it.id, target)
      else e.setVisible(it.id, target)
    }
  })
}

function restackSelection(store: EditorStore, delta: number): void {
  if (store.state.selection.length) store.run((e) => e.restack(store.state.selection, delta))
}

export function hasSelection(store: EditorStore): boolean {
  return store.state.selection.length > 0
}

export function canGroup(store: EditorStore): boolean {
  return store.state.selection.length > 0
}

export function canUngroup(store: EditorStore): boolean {
  return store.state.selection.some((id) => findItem(store.state.tree, id)?.kind === 'group')
}

export function selectionDeletable(store: EditorStore): boolean {
  return store.state.selection.some((id) => !isEffectivelyLocked(store.state.tree, id))
}

const TOOL_ACTIONS: Partial<Record<ActionId, ToolId>> = {
  'tool.select': 'select',
  'tool.hand': 'hand',
  'tool.pen': 'pen',
  'tool.eraser': 'eraser',
  'tool.station': 'station',
  'tool.text': 'text',
  'tool.icon': 'icon',
}

/**
 * Save the document. Where the browser supports it, the first save asks for a
 * location and later saves write back to the same file (`saveAs` always asks);
 * otherwise the file is downloaded.
 */
export async function saveTransit(store: EditorStore, saveAs = false): Promise<void> {
  const s = store.state
  const text = store.engine.value.saveTransit()
  if (!canSaveInPlace()) {
    downloadText(`${s.docName}.transit`, text, 'application/json')
    s.dirty = false
    store.notify(`Saved ${s.docName}.transit`)
    return
  }
  try {
    const handle = (!saveAs && store.getFileHandle()) || (await pickSaveHandle(`${s.docName}.transit`))
    if (!handle) return
    await writeHandle(handle, text)
    store.setFileHandle(handle)
    s.docName = handle.name.replace(/\.transit$/i, '')
    s.dirty = false
    store.notify(`Saved ${handle.name}`)
  } catch (err) {
    store.notify(`Could not save: ${err instanceof Error ? err.message : String(err)}`, 'error')
  }
}

/** Download the drawing as SVG, optionally with one-off settings and scale (pixels per cell). */
export function exportSvg(store: EditorStore, settings?: DocSettings, cellPx = CELL_PX): void {
  const svg = settings ? store.engine.value.exportSvgWith(settings, cellPx) : store.engine.value.exportSvg()
  downloadText(`${store.state.docName}.svg`, svg, 'image/svg+xml')
  store.notify(`Exported ${store.state.docName}.svg`)
}

export async function openTransit(store: EditorStore): Promise<void> {
  try {
    const file = await pickTransitFile()
    if (file) store.openTransit(file.text, file.name, file.handle)
  } catch (err) {
    store.notify(`Could not open: ${err instanceof Error ? err.message : String(err)}`, 'error')
  }
}

export function runAction(store: EditorStore, id: ActionId): void {
  const s = store.state
  const tool = TOOL_ACTIONS[id]
  if (tool) {
    store.setTool(tool)
    return
  }
  switch (id) {
    case 'file.new':
      if (!s.dirty || confirm('Discard unsaved changes and start a new document?')) store.newDocument()
      return
    case 'file.open':
      void openTransit(store)
      return
    case 'file.save':
      void saveTransit(store)
      return
    case 'file.saveAs':
      void saveTransit(store, true)
      return
    case 'file.exportSvg':
      s.dialog = 'export'
      return
    case 'edit.undo':
      store.undo()
      return
    case 'edit.redo':
      store.redo()
      return
    case 'edit.delete':
      if (s.selection.length) store.run((e) => e.delete(s.selection))
      return
    case 'edit.duplicate':
      if (s.selection.length) {
        const ids = store.run((e) => e.duplicate(s.selection))
        if (ids) store.select(ids)
      }
      return
    case 'edit.selectAll':
      store.selectAll()
      return
    case 'edit.deselect':
      store.clearSelection()
      return
    case 'edit.rotateCw':
      rotateSelection(store, 45)
      return
    case 'edit.rotateCcw':
      rotateSelection(store, -45)
      return
    case 'edit.preferences':
      s.dialog = 'preferences'
      return
    case 'edit.shortcuts':
      s.dialog = 'shortcuts'
      return
    case 'document.settings':
      s.dialog = 'document'
      return
    case 'layer.newGroup': {
      const id = store.run((e) => e.newGroup())
      if (id !== undefined) store.select([id])
      return
    }
    case 'layer.group': {
      if (!s.selection.length) return
      const id = store.run((e) => e.group(s.selection))
      if (id !== undefined) store.select([id])
      return
    }
    case 'layer.ungroup': {
      const groups = s.selection.filter((g) => findItem(s.tree, g)?.kind === 'group')
      if (!groups.length) return
      const released: number[] = []
      store.run((e) => groups.forEach((g) => released.push(...e.ungroup(g))))
      store.select(released)
      return
    }
    case 'layer.toggleLock':
      toggleOnSelection(store, 'locked')
      return
    case 'layer.toggleVisible':
      toggleOnSelection(store, 'visible')
      return
    case 'layer.bringForward':
      restackSelection(store, 1)
      return
    case 'layer.sendBackward':
      restackSelection(store, -1)
      return
    case 'layer.bringToFront':
      restackSelection(store, 100000)
      return
    case 'layer.sendToBack':
      restackSelection(store, -100000)
      return
    case 'view.zoomIn':
      store.zoomBy(prefs.zoomStep * prefs.zoomStep)
      return
    case 'view.zoomOut':
      store.zoomBy(1 / (prefs.zoomStep * prefs.zoomStep))
      return
    case 'view.fit':
      store.fitToScreen()
      return
    case 'view.actual':
      store.actualSize()
      return
    case 'view.toggleGrid':
      prefs.showGrid = !prefs.showGrid
      return
    case 'help.shortcuts':
      s.dialog = 'reference'
      return
    case 'help.about':
      s.dialog = 'about'
      return
  }
}

/** Find the action bound to a keyboard event. */
export function actionForEvent(e: KeyboardEvent): ActionId | null {
  for (const [id, binding] of Object.entries(prefs.shortcuts) as [ActionId, string][]) {
    if (binding && matches(binding, e)) return id
  }
  return null
}

export function isEnabled(store: EditorStore, id: ActionId): boolean {
  switch (id) {
    case 'edit.undo':
      return store.state.canUndo
    case 'edit.redo':
      return store.state.canRedo
    case 'edit.delete':
      return selectionDeletable(store)
    case 'edit.duplicate':
    case 'edit.deselect':
    case 'edit.rotateCw':
    case 'edit.rotateCcw':
    case 'layer.toggleLock':
    case 'layer.toggleVisible':
    case 'layer.bringForward':
    case 'layer.sendBackward':
    case 'layer.bringToFront':
    case 'layer.sendToBack':
      return hasSelection(store)
    case 'layer.group':
      return canGroup(store)
    case 'layer.ungroup':
      return canUngroup(store)
    default:
      return true
  }
}
