import { describe, expect, it } from 'vitest'
import { defaultPrefs, loadPrefs, prefs, resetPrefs, resetWorkspace, savePrefs } from '../../src/stores/prefs'

const mem = (raw: string | null) => ({ getItem: () => raw })

describe('preferences', () => {
  it('fall back to defaults when storage is empty or corrupt', () => {
    expect(loadPrefs(mem(null))).toEqual(defaultPrefs())
    expect(loadPrefs(mem('{not json'))).toEqual(defaultPrefs())
    expect(loadPrefs(null)).toEqual(defaultPrefs())
  })
  it('merge stored values over defaults, dropping unknown and mistyped keys', () => {
    const p = loadPrefs(
      mem(JSON.stringify({ zoomStep: 1.5, bogus: 1, showGrid: 'yes', line: { color: '#123456' }, shortcuts: { 'tool.pen': 'B' } })),
    )
    expect(p.zoomStep).toBe(1.5)
    expect('bogus' in p).toBe(false)
    expect(p.showGrid).toBe(true)
    expect(p.line.color).toBe('#123456')
    expect(p.line.kind).toBe('plain')
    expect(p.shortcuts['tool.pen']).toBe('B')
    expect(p.shortcuts['tool.select']).toBe('V')
  })
  it('fall back to the default line style for one that no longer exists', () => {
    expect(loadPrefs(mem(JSON.stringify({ line: { kind: 'dashed' } }))).line.kind).toBe('plain')
    expect(loadPrefs(mem(JSON.stringify({ line: { kind: 'inner_dashed' } }))).line.kind).toBe('inner_dashed')
  })
  it('default to the requested line styles', () => {
    const { line } = defaultPrefs()
    expect([line.inner_width_pct, line.dash_width_pct, line.dash_length, line.dash_gap, line.dash_start_offset, line.dash_end_offset]).toEqual([33, 30, 1, 1, 0, 0])
  })
  it('restore defaults: everything, or only the workspace layout', () => {
    prefs.zoomStep = 2
    prefs.panels.layers = false
    prefs.propertiesHeight = 500
    resetWorkspace()
    expect([prefs.panels.layers, prefs.propertiesHeight, prefs.zoomStep]).toEqual([true, defaultPrefs().propertiesHeight, 2])
    resetPrefs()
    expect(prefs.zoomStep).toBe(defaultPrefs().zoomStep)
  })
  it('accept values for nullable defaults', () => {
    const p = loadPrefs(mem(JSON.stringify({ icon: { size: 2 }, station: { tick_color: '#ff0000' } })))
    expect(p.icon.size).toBe(2)
    expect(p.station.tick_color).toBe('#ff0000')
  })
  it('save and reload', () => {
    let stored = ''
    const p = defaultPrefs()
    p.eraserSize = 3
    savePrefs(p, { setItem: (_k: string, v: string) => (stored = v) })
    expect(loadPrefs(mem(stored)).eraserSize).toBe(3)
  })
  it('survive a throwing storage', () => {
    expect(() => savePrefs(defaultPrefs(), { setItem: () => { throw new Error('quota') } })).not.toThrow()
  })
})
