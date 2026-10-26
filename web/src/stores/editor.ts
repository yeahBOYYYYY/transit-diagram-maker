// Central application state. The Rust engine owns the document; this store
// mirrors what the UI needs (render fragments, layer tree, settings) and keeps
// purely visual state (selection, view, active tool, overlays).
import { reactive, shallowRef } from 'vue'
import { Engine } from '../core/engine'
import type { AssetList, Cell, DocNode, DocSettings, Fragment, Id, Paragraph, Rect, RunStyle, TextAlign, TextBox, TextDirection, TreeItem } from '../core/types'
import {
  type RunKey,
  type TextRange,
  RUN_KEYS,
  applyStyle,
  clearRunProperty,
  cloneParagraphs,
  coversAll,
  isCollapsed,
  plainText,
  resolvedStyleAt,
  setAlign,
  wholeRange,
} from '../core/richtext'
import { registerFonts } from '../core/fonts'
import { leafIds, findItem, topLevelOf } from '../core/tree'
import { fitRect, unionRects, zoomAt, type View } from '../core/viewport'
import { prefs } from './prefs'
import { docNameOf, type FileHandle } from '../core/files'

export type ToolId = 'select' | 'hand' | 'pen' | 'eraser' | 'station' | 'text'

export interface Overlay {
  /** Markup drawn in world space above the document (pen / pill previews). */
  preview: string
  /** Marquee in world px. */
  marquee: Rect | null
  /** Cells highlighted in world space (eraser brush, hover target). */
  cells: Cell[]
  cellsColor: string
}

/**
 * An inline text edit. It runs as one engine transaction (opened before a new
 * text is created): every change writes the whole draft, so committing
 * records one undo step and cancelling restores the text (or removes a new one).
 */
export interface TextEditState {
  id: Id
  /** Created by this edit session (cancel removes it). */
  isNew: boolean
  paragraphs: Paragraph[]
  /** Box properties changed during the edit. */
  box: Partial<TextBox>
  /** Last selection inside the editor (kept while a panel has the focus). */
  selection: TextRange | null
  /** Bumped when the draft changes from outside the editor, so it redraws. */
  version: number
}

export function createEditor(engine = new Engine()) {
  const state = reactive({
    revision: 0,
    fragments: [] as Fragment[],
    tree: [] as TreeItem[],
    settings: engine.settings() as DocSettings,
    /** Fonts and icons uploaded into the document. */
    assets: engine.assetList() as AssetList,
    selection: [] as Id[],
    tool: 'select' as ToolId,
    /** Line the pen adds segments to (its line id), or null to start a new line. */
    penTarget: null as Id | null,
    transformMode: 'move' as 'move' | 'rotate',
    view: { x: 0, y: 0, zoom: 1 } as View,
    viewport: { width: 800, height: 600 },
    canUndo: false,
    canRedo: false,
    docName: 'Untitled-1',
    dirty: false,
    overlay: { preview: '', marquee: null, cells: [], cellsColor: '#31a8ff' } as Overlay,
    cursor: null as { x: number; y: number } | null,
    message: '',
    messageKind: 'info' as 'info' | 'error',
    textEdit: null as TextEditState | null,
    /** Angle shown while rotating. */
    rotation: null as number | null,
    spaceHeld: false,
    /** Tab shown in the upper dock panel. */
    dockTab: 'Properties' as 'Properties' | 'Icons',
    dialog: null as null | 'document' | 'assets' | 'export' | 'preferences' | 'shortcuts' | 'about' | 'reference',
    /** Open the Preferences dialog asking to restore every default. */
    restorePrompt: false,
  })

  const engineRef = shallowRef(engine)
  /** Where the document was last saved or opened from, so Save can write back to it. */
  let fileHandle: FileHandle | null = null
  let messageTimer: ReturnType<typeof setTimeout> | null = null

  function refresh(): void {
    state.fragments = engine.fragments()
    state.tree = engine.tree()
    state.settings = engine.settings()
    state.assets = engine.assetList()
    registerFonts(engine, state.assets.fonts)
    state.canUndo = engine.canUndo()
    state.canRedo = engine.canRedo()
    state.selection = state.selection.filter((id) => findItem(state.tree, id) !== null)
    if (state.penTarget !== null && !lines().some((l) => l.lineId === state.penTarget)) state.penTarget = null
    state.revision++
  }

  function notify(message: string, kind: 'info' | 'error' = 'info'): void {
    state.message = message
    state.messageKind = kind
    if (messageTimer) clearTimeout(messageTimer)
    messageTimer = setTimeout(() => (state.message = ''), kind === 'error' ? prefs.errorDuration : prefs.messageDuration)
  }

  /** Run an engine mutation, refresh the mirror and report failures. */
  function run<T>(fn: (e: Engine) => T, opts: { quiet?: boolean } = {}): T | undefined {
    try {
      const out = fn(engine)
      state.dirty = true
      refresh()
      return out
    } catch (err) {
      if (!opts.quiet) notify(errorMessage(err), 'error')
      refresh()
      return undefined
    }
  }

  // ------------------------------------------------------------ selection

  function select(ids: Id[], mode: 'replace' | 'add' | 'toggle' = 'replace'): void {
    if (mode === 'replace') state.selection = [...new Set(ids)]
    else if (mode === 'add') state.selection = [...new Set([...state.selection, ...ids])]
    else {
      const set = new Set(state.selection)
      for (const id of ids) {
        if (set.has(id)) set.delete(id)
        else set.add(id)
      }
      state.selection = [...set]
    }
    if (state.selection.length === 0) state.transformMode = 'move'
  }

  function clearSelection(): void {
    select([])
  }

  function selectAll(): void {
    select(state.tree.filter((t) => !t.locked).map((t) => t.id))
  }

  /** Which node a click on leaf `id` should select, honoring the auto-select mode. */
  function pickTarget(id: Id): Id {
    return prefs.autoSelect === 'group' ? topLevelOf(state.tree, id) : id
  }

  function selectedLeaves(): Id[] {
    return leafIds(state.tree, state.selection)
  }

  function node(id: Id): DocNode | null {
    return engine.node(id)
  }

  // ----------------------------------------------------------------- view

  function setViewport(width: number, height: number): void {
    state.viewport = { width, height }
  }

  function zoomBy(factor: number, sx = state.viewport.width / 2, sy = state.viewport.height / 2): void {
    state.view = zoomAt(state.view, sx, sy, factor, prefs.minZoom, prefs.maxZoom)
  }

  function setZoom(zoom: number): void {
    zoomBy(zoom / state.view.zoom)
  }

  function panBy(dx: number, dy: number): void {
    state.view = { ...state.view, x: state.view.x + dx, y: state.view.y + dy }
  }

  function contentBounds(ids?: Id[]): Rect | null {
    const want = ids ? new Set(leafIds(state.tree, ids)) : null
    return unionRects(state.fragments.filter((f) => !want || want.has(f.id)).map((f) => f.bounds))
  }

  function fitToScreen(): void {
    const r = contentBounds()
    if (!r) {
      state.view = { x: state.viewport.width / 2, y: state.viewport.height / 2, zoom: 1 }
      return
    }
    state.view = fitRect(r, state.viewport.width, state.viewport.height, prefs.fitMargin, prefs.minZoom, Math.min(prefs.maxZoom, prefs.fitMaxZoom))
  }

  function actualSize(): void {
    const r = contentBounds()
    const cx = r ? r.x + r.w / 2 : 0
    const cy = r ? r.y + r.h / 2 : 0
    state.view = { zoom: 1, x: state.viewport.width / 2 - cx, y: state.viewport.height / 2 - cy }
  }

  // ---------------------------------------------------------------- tools

  /** Line id of a line group or segment, if `id` is one. */
  function lineIdOf(id: Id): Id | null {
    const n = engine.node(id)
    if (n?.type === 'group' && n.kind.role === 'line') return n.kind.line_id
    if (n?.type === 'layer' && n.content.kind === 'line') return n.content.line_id
    return null
  }

  /** Every line in the document, top of the stack first. */
  function lines(): { lineId: Id; name: string }[] {
    const out: { lineId: Id; name: string }[] = []
    const walk = (items: TreeItem[]) =>
      items.forEach((it) => {
        if (it.kind === 'group' && it.role === 'line') {
          const lineId = lineIdOf(it.id)
          if (lineId !== null) out.push({ lineId, name: it.name })
        }
        walk(it.children)
      })
    walk(state.tree)
    return out
  }

  function setTool(tool: ToolId): void {
    if (state.textEdit) finishTextEdit(true)
    // Picking the pen with a line selected continues that line.
    if (tool === 'pen' && state.tool !== 'pen') {
      state.penTarget = state.selection.length === 1 ? lineIdOf(state.selection[0]) : null
    }
    state.tool = tool
    state.overlay = { preview: '', marquee: null, cells: [], cellsColor: prefs.selectionColor }
    state.transformMode = 'move'
  }

  // -------------------------------------------------------------- history

  function undo(): void {
    if (state.textEdit) finishTextEdit(false)
    if (engine.undo()) {
      state.dirty = true
      refresh()
    }
  }

  function redo(): void {
    if (engine.redo()) {
      state.dirty = true
      refresh()
    }
  }

  // ---------------------------------------------------------- text editing

  /** Edit a text box in place. A new text's transaction was already opened by the text tool. */
  function startTextEdit(id: Id, isNew = false): void {
    const n = engine.node(id)
    if (!n || n.type !== 'layer' || n.content.kind !== 'text') return
    if (!isNew) engine.begin()
    const paragraphs = cloneParagraphs(n.content.paragraphs)
    state.textEdit = { id, isNew, paragraphs, box: {}, selection: wholeRange(paragraphs), version: 0 }
    select([id])
  }

  function editedText(): TextBox | null {
    const ed = state.textEdit
    const n = ed ? engine.node(ed.id) : null
    return n?.type === 'layer' && n.content.kind === 'text' ? n.content : null
  }

  /** Show the draft on the canvas. Its values are absolute, so writing it again replaces the last one. */
  function pushTextDraft(): void {
    const ed = state.textEdit
    if (!ed) return
    run((e) => e.updateContent(ed.id, { ...ed.box, paragraphs: ed.paragraphs }), { quiet: true })
  }

  /** The editor's content changed (typing); `redraw` also redraws the editor. */
  function setTextDraft(paragraphs: Paragraph[], selection: TextRange | null, redraw = false): void {
    const ed = state.textEdit
    if (!ed) return
    ed.paragraphs = paragraphs
    if (selection) ed.selection = selection
    if (redraw) ed.version++
    pushTextDraft()
  }

  /**
   * Formatting of the text being edited, or of `id`: what the options bar and
   * the Properties panel show (the first selected character's while editing).
   */
  function textFormat(id?: Id): (Required<RunStyle> & { align: TextAlign; direction: TextDirection }) | null {
    const ed = state.textEdit
    const editing = ed && (id === undefined || id === ed.id)
    const n = engine.node(editing ? ed.id : (id ?? -1))
    if (n?.type !== 'layer' || n.content.kind !== 'text') return null
    const t = n.content
    if (editing) return { ...resolvedStyleAt(ed.paragraphs, ed.selection, t), direction: t.direction }
    return { font_family: t.font_family, size: t.size, color: t.color, bold: t.bold, italic: t.italic, align: t.align, direction: t.direction }
  }

  /**
   * Format text like a word processor. While editing, a selection gets the
   * formatting itself, and nothing selected (or everything) changes the box.
   * Outside the editor the whole box of `id` changes. `align` applies to the
   * paragraphs the selection touches.
   */
  function formatText(patch: Partial<Record<RunKey, unknown>> & { align?: TextAlign; direction?: TextDirection }, id?: Id): void {
    const ed = state.textEdit
    if (!ed || (id !== undefined && id !== ed.id)) {
      if (id !== undefined) run((e) => e.updateContent(id, patch))
      return
    }
    const sel = ed.selection ?? wholeRange(ed.paragraphs)
    const whole = isCollapsed(sel) || coversAll(ed.paragraphs, sel)
    const { align, direction, ...chars } = patch
    let ps = ed.paragraphs
    if (direction) ed.box = { ...ed.box, direction }
    if (Object.keys(chars).length) {
      if (whole) {
        for (const k of Object.keys(chars) as RunKey[]) if (RUN_KEYS.includes(k)) ps = clearRunProperty(ps, k)
        ed.box = { ...ed.box, ...(chars as Partial<TextBox>) }
      } else {
        ps = applyStyle(ps, sel, chars)
      }
    }
    if (align) {
      if (ps.length === 1 || coversAll(ps, sel)) {
        ps = clearRunProperty(ps, 'align')
        ed.box = { ...ed.box, align }
      } else {
        ps = setAlign(ps, sel, align)
      }
    }
    setTextDraft(ps, null, true)
  }

  /** Commit (or cancel) the inline text editor. New, cancelled or emptied texts are removed. */
  function finishTextEdit(commit: boolean): void {
    const edit = state.textEdit
    if (!edit) return
    state.textEdit = null
    if (!commit) {
      engine.cancel()
      refresh()
      return
    }
    run((e) => {
      if (plainText(edit.paragraphs).trim() === '') {
        e.rebase()
        if (!edit.isNew) e.delete([edit.id])
        else {
          e.cancel()
          return
        }
      }
      e.commit()
    })
  }

  // ---------------------------------------------------------------- files

  function newDocument(): void {
    engine.newDocument()
    state.selection = []
    state.dirty = false
    state.docName = 'Untitled-1'
    fileHandle = null
    refresh()
    state.view = { x: state.viewport.width / 2, y: state.viewport.height / 2, zoom: 1 }
  }

  function openTransit(text: string, name: string, handle: FileHandle | null = null): boolean {
    try {
      engine.loadTransit(text)
    } catch (err) {
      notify(errorMessage(err), 'error')
      return false
    }
    state.selection = []
    state.dirty = false
    state.docName = docNameOf(name)
    fileHandle = handle
    refresh()
    fitToScreen()
    notify(`Opened ${name}`)
    return true
  }

  return {
    state,
    engine: engineRef,
    refresh,
    notify,
    run,
    select,
    clearSelection,
    selectAll,
    pickTarget,
    selectedLeaves,
    node,
    setViewport,
    zoomBy,
    setZoom,
    panBy,
    contentBounds,
    fitToScreen,
    actualSize,
    setTool,
    lineIdOf,
    lines,
    undo,
    redo,
    startTextEdit,
    editedText,
    setTextDraft,
    textFormat,
    formatText,
    finishTextEdit,
    newDocument,
    openTransit,
    getFileHandle: () => fileHandle,
    setFileHandle: (h: FileHandle | null) => (fileHandle = h),
  }
}

export type EditorStore = ReturnType<typeof createEditor>

export function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message
  return String(err)
}

let instance: EditorStore | null = null

export function provideEditor(store: EditorStore): EditorStore {
  instance = store
  return store
}

export function useEditor(): EditorStore {
  if (!instance) instance = createEditor()
  return instance
}
