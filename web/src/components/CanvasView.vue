<script setup lang="ts">
// The infinite canvas: grid, document fragments, tool overlays and the inline
// text editor. Owns pointer/keyboard dispatch to the active tool.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'
import type { Id, Rect, TextBox } from '../core/types'
import { LINE_SPACING, editorSelection, insertText, readEditor, renderEditor, setEditorSelection } from '../core/richtext'
import { leafIds } from '../core/tree'
import { CELL_PX, cellAt, screenToWorld, unionRects, worldToScreen } from '../core/viewport'
import { useEditor } from '../stores/editor'
import { prefs } from '../stores/prefs'
import { actionForEvent, runAction } from '../stores/commands'
import { createTools, iconPreview, pillGrowTargets, placeIconAt, placeIcon, selectedPill, type PointerInfo, type Tool } from '../tools'

const store = useEditor()
const s = store.state

const host = ref<HTMLDivElement | null>(null)
const docLayer = ref<SVGGElement | null>(null)
const textEditor = ref<HTMLDivElement | null>(null)
const lastPointer = shallowRef<PointerInfo | null>(null)
const pressed = ref(false)
const panning = ref<{ x: number; y: number } | null>(null)
const hoverId = ref<Id | null>(null)

// ------------------------------------------------------------- bounds

function boundsOf(ids: Id[]): Map<Id, Rect> {
  const out = new Map<Id, Rect>()
  const frags = new Map(s.fragments.map((f) => [f.id, f]))
  for (const id of ids) {
    // Lines and stations are stroked paths, which getBBox measures without
    // the stroke, and getBBox measures text by its font's glyphs; the engine's
    // bounds are exact for them (a text box is its measured width by its
    // lines' height, whatever the font).
    const f = frags.get(id)
    if (f?.kind !== 'icon' && f?.bounds) {
      out.set(id, f.bounds)
      continue
    }
    const el = docLayer.value?.querySelector<SVGGElement>(`g[data-id="${id}"]`)
    let r: Rect | null = null
    if (el && typeof el.getBBox === 'function') {
      try {
        const b = el.getBBox()
        if (b.width > 0 || b.height > 0) r = { x: b.x, y: b.y, w: b.width, h: b.height }
      } catch {
        r = null
      }
    }
    r ??= frags.get(id)?.bounds ?? null
    if (r) out.set(id, r)
  }
  return out
}

const tools = createTools(store, boundsOf)
const tool = computed<Tool>(() => (s.spaceHeld || panning.value ? tools.hand : tools[s.tool]))

watch(
  () => s.tool,
  (_n, old) => {
    tools[old]?.reset?.()
  },
)

// ------------------------------------------------------ selection boxes

const selectionRects = ref<{ id: Id; rect: Rect }[]>([])
const hoverRect = ref<Rect | null>(null)

function measureSelection(): void {
  selectionRects.value = s.selection
    .map((id) => {
      const leaves = leafIds(s.tree, [id])
      return { id, rect: unionRects([...boundsOf(leaves).values()]) }
    })
    .filter((x): x is { id: Id; rect: Rect } => x.rect !== null)
  hoverRect.value =
    hoverId.value !== null && !s.selection.includes(hoverId.value) ? (boundsOf([hoverId.value]).get(hoverId.value) ?? null) : null
}

watch(() => [s.revision, s.selection, hoverId.value], measureSelection, { flush: 'post', deep: true })

const toScreen = (r: Rect): Rect => {
  const a = worldToScreen(s.view, r.x, r.y)
  return { x: a.x, y: a.y, w: r.w * s.view.zoom, h: r.h * s.view.zoom }
}

const selectionBox = computed(() => {
  const r = unionRects(selectionRects.value.map((x) => x.rect))
  return r ? toScreen(r) : null
})

const handles = computed(() => {
  const b = selectionBox.value
  if (!b) return []
  const pad = 6
  return [
    { x: b.x - pad, y: b.y - pad },
    { x: b.x + b.w + pad, y: b.y - pad },
    { x: b.x + b.w + pad, y: b.y + b.h + pad },
    { x: b.x - pad, y: b.y + b.h + pad },
  ]
})

// Arrows around a selected pill station: double click one to grow into that cell.
const pillArrows = computed(() => {
  void s.revision
  if (s.tool !== 'select') return []
  const pill = selectedPill(store)
  if (!pill) return []
  return pillGrowTargets(pill.cells).map(({ cell, dx, dy }) => {
    const c = worldToScreen(s.view, (cell.x + 0.5 + pill.offset.x) * CELL_PX, (cell.y + 0.5 + pill.offset.y) * CELL_PX)
    return { key: `${cell.x},${cell.y}`, x: c.x, y: c.y, angle: (Math.atan2(dy, dx) * 180) / Math.PI }
  })
})

// ---------------------------------------------------------------- grid

const grid = computed(() => {
  const g = CELL_PX * s.view.zoom
  const major = g * Math.max(1, prefs.gridMajorEvery)
  return {
    minor: g,
    major,
    showMinor: prefs.showGrid && g >= prefs.minGridPixels,
    showMajor: prefs.showGrid && major >= prefs.minGridPixels && prefs.gridMajorEvery > 1,
    transform: `translate(${s.view.x} ${s.view.y})`,
  }
})

const worldTransform = computed(() => `translate(${s.view.x} ${s.view.y}) scale(${s.view.zoom})`)

const overlayCells = computed(() => {
  const g = CELL_PX
  return s.overlay.cells.map((c) => ({ x: c.x * g, y: c.y * g, w: g, h: g }))
})

const marquee = computed(() => (s.overlay.marquee ? toScreen(s.overlay.marquee) : null))

// ------------------------------------------------------------- pointer

function hitTest(clientX: number, clientY: number, world: { x: number; y: number }): Id | null {
  const locked = new Set(s.fragments.filter((f) => f.locked).map((f) => f.id))
  const els = typeof document.elementsFromPoint === 'function' ? document.elementsFromPoint(clientX, clientY) : []
  for (const el of els) {
    const g = el.closest?.('g[data-id]')
    if (g && docLayer.value?.contains(g)) {
      const id = Number(g.getAttribute('data-id'))
      if (!locked.has(id)) return id
    }
  }
  // Text glyphs are thin; fall back to their boxes.
  for (const f of [...s.fragments].reverse()) {
    if (f.locked || (f.kind !== 'text' && f.kind !== 'icon')) continue
    const b = boundsOf([f.id]).get(f.id)
    if (b && world.x >= b.x && world.x <= b.x + b.w && world.y >= b.y && world.y <= b.y + b.h) return f.id
  }
  return null
}

function parseCell(v: string | null | undefined): { x: number; y: number } | null {
  const m = v?.match(/^(-?\d+),(-?\d+)$/)
  return m ? { x: Number(m[1]), y: Number(m[2]) } : null
}

function info(e: PointerEvent | MouseEvent): PointerInfo {
  const rect = host.value!.getBoundingClientRect()
  const screen = { x: e.clientX - rect.left, y: e.clientY - rect.top }
  const world = screenToWorld(s.view, screen.x, screen.y)
  const g = CELL_PX
  const handleEl = (e.target as Element | null)?.closest?.('[data-handle]')
  return {
    screen,
    world,
    cells: { x: world.x / g, y: world.y / g },
    cell: cellAt(world, g),
    button: e.button,
    buttons: e.buttons,
    shift: e.shiftKey,
    alt: e.altKey,
    ctrl: e.ctrlKey || e.metaKey,
    hitId: hitTest(e.clientX, e.clientY, world),
    handle: handleEl?.getAttribute('data-handle') ?? null,
    handleCell: parseCell(handleEl?.getAttribute('data-cell')),
  }
}

// The canvas captures the pointer on press, so a following dblclick is
// retargeted to the canvas itself; remember which handle was pressed.
let lastDown: PointerInfo | null = null

function onPointerDown(e: PointerEvent): void {
  if (s.textEdit && (e.target as Element).closest('[data-testid=text-editor]')) return
  if (s.textEdit) store.finishTextEdit(true)
  host.value?.focus({ preventScroll: true })
  const p = info(e)
  lastDown = p
  if (e.button === 1 || (e.button === 0 && s.spaceHeld)) {
    panning.value = p.screen
    tools.hand.down?.(p)
  } else if (e.button === 0) {
    pressed.value = true
    tool.value.down?.(p)
  }
  host.value?.setPointerCapture(e.pointerId)
  lastPointer.value = p
  e.preventDefault()
}

function onPointerMove(e: PointerEvent): void {
  const p = info(e)
  lastPointer.value = p
  s.cursor = p.cells
  if (panning.value) {
    tools.hand.move?.(p, true)
    return
  }
  tool.value.move?.(p, pressed.value)
  hoverId.value = s.tool === 'select' && !pressed.value && !s.spaceHeld ? p.hitId : null
}

function onPointerUp(e: PointerEvent): void {
  const p = info(e)
  if (panning.value) {
    tools.hand.up?.(p)
    panning.value = null
  } else if (pressed.value) {
    pressed.value = false
    tool.value.up?.(p)
  }
  host.value?.releasePointerCapture?.(e.pointerId)
}

function onDblClick(e: MouseEvent): void {
  let p = info(e)
  const d = lastDown
  if (!p.handle && d?.handle && Math.hypot(d.screen.x - p.screen.x, d.screen.y - p.screen.y) < 4) {
    p = { ...p, handle: d.handle, handleCell: d.handleCell }
  }
  tool.value.dblclick?.(p)
}

// Pictograms dragged from the Icons panel.
const ICON_MIME = 'application/x-transit-icon'
function onDragOver(e: DragEvent): void {
  if (!e.dataTransfer?.types.includes(ICON_MIME)) return
  e.preventDefault()
  e.dataTransfer.dropEffect = 'copy'
  const p = info(e)
  s.cursor = p.cells
  s.overlay = { ...s.overlay, preview: iconPreview(store, placeIconAt(p)), cells: [] }
}
function onDrop(e: DragEvent): void {
  const name = e.dataTransfer?.getData(ICON_MIME)
  if (!name) return
  e.preventDefault()
  clearDropPreview()
  if (s.textEdit) store.finishTextEdit(true)
  placeIcon(store, name, placeIconAt(info(e)))
  host.value?.focus({ preventScroll: true })
}
function clearDropPreview(): void {
  s.overlay = { ...s.overlay, preview: '' }
}

function onLeave(): void {
  s.cursor = null
  hoverId.value = null
  if (s.tool !== 'pen' && s.tool !== 'select') s.overlay = { ...s.overlay, cells: [], preview: s.tool === 'station' ? '' : s.overlay.preview }
}

function onWheel(e: WheelEvent): void {
  e.preventDefault()
  const rect = host.value!.getBoundingClientRect()
  if (e.ctrlKey || e.metaKey) {
    const factor = Math.pow(prefs.zoomStep, -e.deltaY / 100)
    store.zoomBy(factor, e.clientX - rect.left, e.clientY - rect.top)
  } else if (e.shiftKey) {
    store.panBy(-(e.deltaY || e.deltaX), 0)
  } else {
    store.panBy(-e.deltaX, -e.deltaY)
  }
}

// ------------------------------------------------------------ keyboard

function isTyping(t: EventTarget | null): boolean {
  const el = t as HTMLElement | null
  return !!el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT' || el.isContentEditable)
}

function onKeyDown(e: KeyboardEvent): void {
  if (isTyping(e.target) || s.dialog) return
  if (e.code === 'Space') {
    if (!s.spaceHeld) s.spaceHeld = true
    e.preventDefault()
    return
  }
  if (tool.value.key?.(e)) {
    e.preventDefault()
    return
  }
  const action = actionForEvent(e)
  if (action) {
    e.preventDefault()
    runAction(store, action)
  }
}

function onKeyUp(e: KeyboardEvent): void {
  if (e.code === 'Space') s.spaceHeld = false
}

// --------------------------------------------------------- text editor

const editing = computed(() => {
  const ed = s.textEdit
  if (!ed) return null
  // Depend on revision so changes to the node are picked up.
  void s.revision
  const n = store.node(ed.id)
  if (!n || n.type !== 'layer' || n.content.kind !== 'text') return null
  return n.content as TextBox & { kind: 'text' }
})

// The editor sits exactly over the text box (top-left corner, same rotation);
// run sizes are relative to its font size, so zooming needs no redraw.
const editorStyle = computed(() => {
  const t = editing.value
  if (!t) return {}
  const g = CELL_PX
  const p = worldToScreen(s.view, t.pos.x * g, t.pos.y * g)
  return {
    left: `${p.x}px`,
    top: `${p.y}px`,
    fontSize: `${t.size * g * s.view.zoom}px`,
    lineHeight: String(LINE_SPACING),
    fontFamily: t.font_family,
    fontWeight: t.bold ? 'bold' : 'normal',
    fontStyle: t.italic ? 'italic' : 'normal',
    color: t.color,
    direction: t.direction,
    transform: `rotate(${t.rotation}deg)`,
  }
})

/** Redraw the editor from the draft; `focus` also focuses it and restores the selection. */
async function drawEditor(focus: boolean): Promise<void> {
  await nextTick()
  const el = textEditor.value
  const ed = s.textEdit
  const t = editing.value
  if (!el || !ed || !t) return
  renderEditor(el, ed.paragraphs, t)
  const active = document.activeElement
  if (focus || active === el || active === document.body || active === null) {
    el.focus({ preventScroll: true })
    if (ed.selection) setEditorSelection(el, ed.selection)
  }
}

watch(
  () => s.textEdit?.id,
  (id) => {
    if (id !== undefined) void drawEditor(true)
  },
)
watch(
  () => s.textEdit?.version,
  (v, old) => {
    if (v !== undefined && old !== undefined) void drawEditor(false)
  },
)

function onEditorInput(): void {
  const el = textEditor.value
  if (!el) return
  const sel = editorSelection(el)
  store.setTextDraft(readEditor(el).paragraphs, sel)
}

/** Replace the selection with `text` through the model (pasting, new paragraphs). */
function insertIntoEditor(text: string): void {
  const el = textEditor.value
  const ed = s.textEdit
  if (!el || !ed) return
  const sel = editorSelection(el) ?? ed.selection
  if (!sel) return
  const { paragraphs, caret } = insertText(readEditor(el).paragraphs, sel, text)
  store.setTextDraft(paragraphs, { start: caret, end: caret }, true)
}

function onTextKey(e: KeyboardEvent): void {
  e.stopPropagation()
  const mod = e.ctrlKey || e.metaKey
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    store.finishTextEdit(true)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    insertIntoEditor('\n')
  } else if (e.key === 'Escape') {
    e.preventDefault()
    store.finishTextEdit(false)
  } else if (mod && (e.key === 'b' || e.key === 'B' || e.key === 'i' || e.key === 'I')) {
    e.preventDefault()
    const f = store.textFormat()
    if (!f) return
    if (e.key.toLowerCase() === 'b') store.formatText({ bold: !f.bold })
    else store.formatText({ italic: !f.italic })
  } else if (mod && ['z', 'y', 'u'].includes(e.key.toLowerCase())) {
    // The browser's own undo and underline would bypass the model.
    e.preventDefault()
  }
}

function onBeforeInput(e: InputEvent): void {
  if (e.inputType === 'insertParagraph' || e.inputType === 'insertLineBreak') {
    e.preventDefault()
    insertIntoEditor('\n')
  } else if (e.inputType.startsWith('format') || e.inputType.startsWith('history') || e.inputType === 'insertFromDrop') {
    e.preventDefault()
  }
}

function onEditorPaste(e: ClipboardEvent): void {
  e.preventDefault()
  insertIntoEditor(e.clipboardData?.getData('text/plain') ?? '')
}

/** Leaving the editor commits it, except for the panels that format its text. */
function onEditorFocusOut(e: FocusEvent): void {
  const to = e.relatedTarget as Element | null
  if (to && !textEditor.value?.contains(to) && !to.closest('[data-keeps-text-edit]')) store.finishTextEdit(true)
}

function onSelectionChange(): void {
  const el = textEditor.value
  const ed = s.textEdit
  if (!el || !ed) return
  const sel = editorSelection(el)
  if (sel) ed.selection = sel
}

// ------------------------------------------------------- text widths

/**
 * Text boxes are as wide as their widest line. The engine can only estimate
 * that, so after each render the lines are measured here and the widths are
 * handed back (not an undo step: it describes the text already there).
 */
function fitTexts(): void {
  const layer = docLayer.value
  if (!layer) return
  const widths: [Id, number][] = []
  for (const f of s.fragments) {
    if (f.kind !== 'text') continue
    const lines = layer.querySelectorAll<SVGTextContentElement>(`g[data-id="${f.id}"] tspan[data-line]`)
    let w = 0
    for (const l of lines) {
      if (typeof l.getComputedTextLength === 'function') w = Math.max(w, l.getComputedTextLength())
    }
    if (!(w > 0)) continue
    const n = store.node(f.id)
    if (n?.type !== 'layer' || n.content.kind !== 'text') continue
    if (Math.abs(w / CELL_PX - n.content.width) > 0.002) widths.push([f.id, w / CELL_PX])
  }
  if (widths.length && store.engine.value.fitTextWidths(widths)) store.refresh()
}

watch(() => s.revision, fitTexts, { flush: 'post' })

// ------------------------------------------------------------ lifecycle

let resize: ResizeObserver | null = null
onMounted(() => {
  const el = host.value!
  const r = el.getBoundingClientRect()
  store.setViewport(r.width, r.height)
  s.view = { x: Math.round(r.width / 2), y: Math.round(r.height / 2), zoom: 1 }
  if (typeof ResizeObserver !== 'undefined') {
    resize = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect
      store.setViewport(width, height)
    })
    resize.observe(el)
  }
  el.addEventListener('wheel', onWheel, { passive: false })
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('keyup', onKeyUp)
  window.addEventListener('blur', () => (s.spaceHeld = false))
  document.addEventListener('selectionchange', onSelectionChange)
  // Web fonts change the text widths once they have loaded.
  document.fonts?.addEventListener?.('loadingdone', fitTexts)
  fitTexts()
})

onBeforeUnmount(() => {
  resize?.disconnect()
  host.value?.removeEventListener('wheel', onWheel)
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('keyup', onKeyUp)
  document.removeEventListener('selectionchange', onSelectionChange)
  document.fonts?.removeEventListener?.('loadingdone', fitTexts)
})

const cursor = computed(() => tool.value.cursor(lastPointer.value))

defineExpose({ boundsOf })
</script>

<template>
  <div
    ref="host"
    class="canvas-host"
    tabindex="0"
    data-testid="canvas"
    :style="{ cursor, background: prefs.pasteboardColor }"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
    @pointerleave="onLeave"
    @dblclick="onDblClick"
    @dragover="onDragOver"
    @dragleave="clearDropPreview"
    @drop="onDrop"
    @contextmenu.prevent
  >
    <svg class="canvas" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <pattern id="grid-minor" :width="grid.minor" :height="grid.minor" patternUnits="userSpaceOnUse" :patternTransform="grid.transform">
          <path :d="`M${grid.minor} 0H0V${grid.minor}`" fill="none" :stroke="prefs.gridColor" stroke-width="1" shape-rendering="crispEdges" />
        </pattern>
        <pattern id="grid-major" :width="grid.major" :height="grid.major" patternUnits="userSpaceOnUse" :patternTransform="grid.transform">
          <path :d="`M${grid.major} 0H0V${grid.major}`" fill="none" :stroke="prefs.gridMajorColor" stroke-width="1" shape-rendering="crispEdges" />
        </pattern>
      </defs>
      <rect width="100%" height="100%" :fill="prefs.canvasBackground" data-paper />
      <rect v-if="grid.showMinor" width="100%" height="100%" fill="url(#grid-minor)" />
      <rect v-if="grid.showMajor" width="100%" height="100%" fill="url(#grid-major)" />

      <g :transform="worldTransform">
        <g ref="docLayer" class="doc" data-testid="doc-layer">
          <g
            v-for="f in s.fragments"
            :key="f.id"
            :data-id="f.id"
            :data-kind="f.kind"
            :class="{ locked: f.locked, editing: s.textEdit?.id === f.id }"
            v-html="f.svg"
          />
        </g>
        <g class="overlay-world" pointer-events="none">
          <rect
            v-for="(c, i) in overlayCells"
            :key="i"
            :x="c.x"
            :y="c.y"
            :width="c.w"
            :height="c.h"
            :fill="s.overlay.cellsColor"
            fill-opacity="0.16"
            :stroke="s.overlay.cellsColor"
            stroke-width="1"
            vector-effect="non-scaling-stroke"
          />
          <g class="preview" :opacity="prefs.previewOpacity" v-html="s.overlay.preview" />
        </g>
      </g>

      <!-- screen space overlays -->
      <g class="overlay-screen">
        <rect
          v-if="hoverRect"
          v-bind="{ x: toScreen(hoverRect).x, y: toScreen(hoverRect).y, width: toScreen(hoverRect).w, height: toScreen(hoverRect).h }"
          fill="none"
          :stroke="prefs.selectionColor"
          stroke-opacity="0.6"
          stroke-width="1"
          pointer-events="none"
        />
        <rect
          v-for="r in selectionRects"
          :key="r.id"
          :x="toScreen(r.rect).x - 2"
          :y="toScreen(r.rect).y - 2"
          :width="toScreen(r.rect).w + 4"
          :height="toScreen(r.rect).h + 4"
          fill="none"
          :stroke="prefs.selectionColor"
          stroke-width="1"
          pointer-events="none"
          data-testid="selection-rect"
        />
        <template v-if="selectionBox && s.tool === 'select'">
          <rect
            v-if="selectionRects.length > 1 || s.transformMode === 'rotate'"
            :x="selectionBox.x - 6"
            :y="selectionBox.y - 6"
            :width="selectionBox.w + 12"
            :height="selectionBox.h + 12"
            fill="none"
            :stroke="prefs.selectionColor"
            stroke-dasharray="4 3"
            pointer-events="none"
          />
          <template v-if="s.transformMode === 'rotate'">
            <g v-for="(h, i) in handles" :key="i" class="rotate-handle" data-handle="rotate" data-testid="rotate-handle">
              <circle :cx="h.x" :cy="h.y" r="9" fill="transparent" />
              <circle :cx="h.x" :cy="h.y" r="4.5" fill="#fff" :stroke="prefs.selectionColor" stroke-width="1.5" />
            </g>
            <g pointer-events="none" :stroke="prefs.selectionColor" stroke-width="1.2">
              <circle :cx="selectionBox.x + selectionBox.w / 2" :cy="selectionBox.y + selectionBox.h / 2" r="4" fill="none" />
              <path
                :d="`M${selectionBox.x + selectionBox.w / 2 - 8} ${selectionBox.y + selectionBox.h / 2}h16M${selectionBox.x + selectionBox.w / 2} ${selectionBox.y + selectionBox.h / 2 - 8}v16`"
              />
            </g>
          </template>
          <template v-else-if="pillArrows.length">
            <g
              v-for="a in pillArrows"
              :key="a.key"
              class="grow-handle"
              data-handle="grow"
              :data-cell="a.key"
              data-testid="pill-grow"
              :transform="`translate(${a.x} ${a.y}) rotate(${a.angle})`"
            >
              <title>Double-click to extend the pill into this cell</title>
              <circle r="11" fill="transparent" />
              <path d="M-4.5 -6L5.5 0L-4.5 6Z" :fill="prefs.selectionColor" stroke="#fff" stroke-width="1.5" stroke-linejoin="round" />
            </g>
          </template>
          <template v-else>
            <rect
              v-for="(h, i) in handles"
              :key="i"
              :x="h.x - 3"
              :y="h.y - 3"
              width="6"
              height="6"
              fill="#fff"
              :stroke="prefs.selectionColor"
              pointer-events="none"
            />
          </template>
        </template>
        <rect
          v-if="marquee"
          :x="marquee.x"
          :y="marquee.y"
          :width="marquee.w"
          :height="marquee.h"
          :fill="prefs.selectionColor"
          fill-opacity="0.08"
          :stroke="prefs.selectionColor"
          stroke-dasharray="3 3"
          pointer-events="none"
          data-testid="marquee"
        />
        <g v-if="s.rotation !== null && lastPointer" pointer-events="none">
          <rect :x="lastPointer.screen.x + 14" :y="lastPointer.screen.y + 10" width="52" height="20" rx="3" fill="#202020" opacity="0.9" />
          <text :x="lastPointer.screen.x + 40" :y="lastPointer.screen.y + 24" text-anchor="middle" fill="#fff" font-size="11">{{ Math.round(s.rotation) }}°</text>
        </g>
      </g>
    </svg>

    <div
      v-if="s.textEdit && editing"
      ref="textEditor"
      class="text-editor"
      data-testid="text-editor"
      contenteditable="true"
      spellcheck="false"
      :style="editorStyle"
      @input="onEditorInput"
      @beforeinput="onBeforeInput"
      @paste="onEditorPaste"
      @keydown="onTextKey"
      @focusout="onEditorFocusOut"
      @pointerdown.stop
      @dblclick.stop
    />
  </div>
</template>

<style scoped>
.canvas-host {
  position: relative;
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  outline: none;
  touch-action: none;
}
.canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  display: block;
}
.doc :deep(g.locked) {
  pointer-events: none;
}
.doc :deep(g.editing) {
  opacity: 0;
}
.rotate-handle {
  cursor: alias;
}
.grow-handle {
  cursor: copy;
}
.grow-handle:hover path {
  transform: scale(1.3);
}
/* Laid out like the text box: no padding, as wide as its widest line. */
.text-editor {
  position: absolute;
  display: inline-block;
  min-width: 0.5em;
  margin: 0;
  padding: 0;
  outline: 1px dashed var(--c-accent);
  outline-offset: 2px;
  background: rgba(255, 255, 255, 0.85);
  transform-origin: 0 0;
  white-space: pre;
  cursor: text;
}
</style>
