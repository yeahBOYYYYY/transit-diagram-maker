// Application preferences (not part of the document), persisted in localStorage.
import { reactive, watch } from 'vue'
import type { ExportOptions, Icon, LineStyle, StationKind, StationOptions, TextAlign, TextDirection } from '../core/types'
import { DEFAULT_SHORTCUTS, type ActionId } from '../core/actions'
import { FONT_FAMILIES } from '../core/fonts'

export interface Prefs {
  historyLimit: number
  zoomStep: number
  minZoom: number
  maxZoom: number
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
    zoomStep: 1.15,
    minZoom: 0.05,
    maxZoom: 16,
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
      inner_width_pct: 55,
      dash_length: 1,
      dash_gap: 0.5,
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
    // The hollow dashed style was replaced by the inner dashed one.
    if ((p.line.kind as string) === 'hollow_dashed') p.line.kind = 'inner_dashed'
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

export function resetPrefs(): void {
  Object.assign(prefs, defaultPrefs())
}
