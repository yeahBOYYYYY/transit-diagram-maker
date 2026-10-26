// Application preferences (not part of the document), persisted in localStorage.
import { reactive, watch } from 'vue'
import type { ExportOptions, Icon, LineStyle, StationKind, StationOptions, TextAlign, TextDirection } from '../core/types'
import { DEFAULT_SHORTCUTS, type ActionId } from '../core/actions'
import { FONT_FAMILIES } from '../core/fonts'
import { STROKE_KINDS } from '../core/lineStyles'

export interface Prefs {
  historyLimit: number
  /** How long status bar messages stay (milliseconds); errors stay `errorDuration`. */
  messageDuration: number
  errorDuration: number
  /** Ask before New discards unsaved changes. */
  confirmDiscard: boolean
  /** Text of a new text box. */
  newText: string
  /** Pause (milliseconds) that ends one undo step of continuous panel edits. */
  panelEditPause: number
  zoomStep: number
  minZoom: number
  maxZoom: number
  /** Space around the drawing when fitting it on screen (pixels). */
  fitMargin: number
  /** Largest zoom Fit on Screen uses. */
  fitMaxZoom: number
  nudge: number
  bigNudge: number
  dragThreshold: number
  /** Last used SVG export options. */
  export: ExportOptions
  canvasBackground: string
  gridColor: string
  gridMajorColor: string
  gridMajorEvery: number
  showGrid: boolean
  minGridPixels: number
  selectionColor: string
  /** Eraser brush and invalid station placement. */
  warningColor: string
  /** Opacity of pen, station and drop previews (0 to 1). */
  previewOpacity: number
  pasteboardColor: string
  autoSelect: 'layer' | 'group'
  eraserSize: number
  showThumbnails: boolean
  panels: { tools: boolean; options: boolean; properties: boolean; layers: boolean }
  /** Height of the Properties panel in the dock (px); the Layers panel takes the rest. */
  propertiesHeight: number
  shortcuts: Record<ActionId, string>
  line: LineStyle
  station: StationOptions & { kind: StationKind }
  text: { size: number; color: string; font_family: string; bold: boolean; italic: boolean; align: TextAlign; direction: TextDirection }
  icon: Pick<Icon, 'icon' | 'color' | 'background'> & { size: number | null }
}


export function defaultPrefs(): Prefs {
  return {
    historyLimit: 200,
    messageDuration: 3000,
    errorDuration: 6000,
    confirmDiscard: true,
    newText: 'Text',
    panelEditPause: 600,
    zoomStep: 1.15,
    minZoom: 0.05,
    maxZoom: 16,
    fitMargin: 60,
    fitMaxZoom: 4,
    nudge: 1,
    bigNudge: 5,
    dragThreshold: 3,
    export: { cell_px: 24, margin: 1, background: '#ffffff' },
    canvasBackground: '#ffffff',
    gridColor: '#ececec',
    gridMajorColor: '#d6d6d6',
    gridMajorEvery: 5,
    showGrid: true,
    minGridPixels: 5,
    selectionColor: '#31a8ff',
    warningColor: '#ff4d4f',
    previewOpacity: 0.75,
    pasteboardColor: '#1e1e1e',
    autoSelect: 'layer',
    eraserSize: 1,
    showThumbnails: true,
    panels: { tools: true, options: true, properties: true, layers: true },
    propertiesHeight: 330,
    shortcuts: { ...DEFAULT_SHORTCUTS },
    line: {
      color: '#e03c31',
      kind: 'plain',
      inner_color: '#ffffff',
      inner_width_pct: 33,
      dash_length: 1,
      dash_gap: 1,
      dash_width_pct: 30,
      dash_start_offset: 0,
      dash_end_offset: 0,
      start_cap: 'round',
      end_cap: 'round',
    },
    station: {
      kind: 'dot',
      outer_color: '#000000',
      inner_color: '#ffffff',
      tick_color: null,
      tick_side: 1,
      tick_width: null,
      tick_length: null,
      label: true,
      label_color: '#1a1a1a',
      snap_distance: 1.5,
    },
    text: { size: 0.75, color: '#1a1a1a', font_family: FONT_FAMILIES[0], bold: false, italic: false, align: 'middle', direction: 'ltr' },
    icon: { icon: 'wheelchair', color: '#ffffff', background: '#1d4f91', size: null },
  }
}

const KEY = 'transit-diagram-maker.prefs'

function deepMerge<T>(base: T, over: unknown): T {
  if (typeof base !== 'object' || base === null || typeof over !== 'object' || over === null) {
    return (over === undefined ? base : (over as T)) ?? base
  }
  const out: Record<string, unknown> = { ...(base as Record<string, unknown>) }
  for (const [k, v] of Object.entries(over as Record<string, unknown>)) {
    if (!(k in out)) continue // drop unknown keys from older versions
    const b = out[k]
    out[k] = typeof b === 'object' && b !== null && !Array.isArray(b) ? deepMerge(b, v) : typeof v === typeof b || b === null ? v : b
  }
  return out as T
}

export function loadPrefs(storage: Pick<Storage, 'getItem'> | null = safeStorage()): Prefs {
  try {
    const raw = storage?.getItem(KEY)
    if (!raw) return defaultPrefs()
    const p = deepMerge(defaultPrefs(), JSON.parse(raw))
    // A line style that no longer exists falls back to the default one.
    if (!STROKE_KINDS.some((k) => k.value === p.line.kind)) p.line.kind = defaultPrefs().line.kind
    return p
  } catch {
    return defaultPrefs()
  }
}

export function savePrefs(p: Prefs, storage: Pick<Storage, 'setItem'> | null = safeStorage()): void {
  try {
    storage?.setItem(KEY, JSON.stringify(p))
  } catch {
    /* storage full or blocked: preferences just won't persist */
  }
}

function safeStorage(): Storage | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage
  } catch {
    return null
  }
}

export const prefs = reactive(loadPrefs()) as Prefs

watch(prefs, (p) => savePrefs(p), { deep: true })

/** Restore every preference, tool option, shortcut and the panel layout. */
export function resetPrefs(): void {
  Object.assign(prefs, defaultPrefs())
}

/** Keys of the workspace layout: which panels show and how they are sized. */
export const WORKSPACE_KEYS = ['panels', 'propertiesHeight'] as const

/** Show every panel at its default size again. */
export function resetWorkspace(): void {
  const d = defaultPrefs()
  for (const k of WORKSPACE_KEYS) (prefs as unknown as Record<string, unknown>)[k] = d[k]
}
