<script setup lang="ts">
// The infinite canvas: grid, document fragments, tool overlays and the inline
// text editor. Owns pointer/keyboard dispatch to the active tool.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'
import type { Id, Rect, TextBox } from '../core/types'
import { leafIds } from '../core/tree'
import { CELL_PX, cellAt, screenToWorld, unionRects, worldToScreen } from '../core/viewport'
import { useEditor } from '../stores/editor'
import { prefs } from '../stores/prefs'
import { actionForEvent, runAction } from '../stores/commands'
import { createTools, type PointerInfo, type Tool } from '../tools'

const store = useEditor()
const s = store.state

const host = ref<HTMLDivElement | null>(null)
const docLayer = ref<SVGGElement | null>(null)
const textArea = ref<HTMLTextAreaElement | null>(null)
const lastPointer = shallowRef<PointerInfo | null>(null)
const pressed = ref(false)
const panning = ref<{ x: number; y: number } | null>(null)
const hoverId = ref<Id | null>(null)

// ------------------------------------------------------------- bounds

function boundsOf(ids: Id[]): Map<Id, Rect> {
  const out = new Map<Id, Rect>()
  const frags = new Map(s.fragments.map((f) => [f.id, f]))
  for (const id of ids) {
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
  }
}

function onPointerDown(e: PointerEvent): void {
  if (s.textEdit && (e.target as Element).closest('textarea')) return
  if (s.textEdit) store.finishTextEdit(true)
  host.value?.focus({ preventScroll: true })
  const p = info(e)
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
  tool.value.dblclick?.(info(e))
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

const editorStyle = computed(() => {
  const t = editing.value
  if (!t) return {}
  const g = CELL_PX
  const fs = t.size * g * s.view.zoom
  const p = worldToScreen(s.view, t.pos.x * g, t.pos.y * g)
  const lines = (s.textEdit?.value ?? '').split('\n')
  const cols = Math.max(4, ...lines.map((l) => l.length))
  const shift = t.align === 'middle' ? '-50%' : t.align === 'end' ? '-100%' : '0'
  return {
    left: `${p.x}px`,
    top: `${p.y}px`,
    fontSize: `${fs}px`,
    lineHeight: '1.2',
    fontFamily: t.font_family,
    fontWeight: t.bold ? 'bold' : 'normal',
    fontStyle: t.italic ? 'italic' : 'normal',
    color: t.color,
    textAlign: (t.align === 'middle' ? 'center' : t.align === 'end' ? 'right' : 'left') as 'center' | 'right' | 'left',
    width: `${cols * 0.62 + 1}em`,
    height: `${lines.length * 1.2 + 0.4}em`,
    transform: `translate(${shift}, -50%) rotate(${t.rotation}deg)`,
    transformOrigin: t.align === 'middle' ? '50% 50%' : t.align === 'end' ? '100% 50%' : '0 50%',
  }
})

watch(
  () => s.textEdit?.id,
  async (id) => {
    if (id === undefined) return
    await nextTick()
    textArea.value?.focus()
    textArea.value?.select()
  },
)

function onTextKey(e: KeyboardEvent): void {
  e.stopPropagation()
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    store.finishTextEdit(true)
  } else if (e.key === 'Escape') {
    e.preventDefault()
    store.finishTextEdit(false)
  }
}

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
})

onBeforeUnmount(() => {
  resize?.disconnect()
  host.value?.removeEventListener('wheel', onWheel)
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('keyup', onKeyUp)
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
      <rect width="100%" height="100%" :fill="prefs.canvasBackground" />
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
          <g class="preview" opacity="0.75" v-html="s.overlay.preview" />
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

    <textarea
      v-if="s.textEdit && editing"
      ref="textArea"
      v-model="s.textEdit.value"
      class="text-editor"
      data-testid="text-editor"
      spellcheck="false"
      :style="editorStyle"
      @keydown="onTextKey"
      @blur="store.finishTextEdit(true)"
      @pointerdown.stop
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
.text-editor {
  position: absolute;
  margin: 0;
  padding: 0 0.1em;
  border: 1px dashed var(--c-accent);
  border-radius: 0;
  background: rgba(255, 255, 255, 0.85);
  resize: none;
  overflow: hidden;
  white-space: pre;
}
</style>
