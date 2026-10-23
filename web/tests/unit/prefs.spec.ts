import { describe, expect, it } from 'vitest'
import { defaultPrefs, loadPrefs, savePrefs } from '../../src/stores/prefs'

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
  it('round a stored snap step to the 0.1 cell precision', () => {
    expect(loadPrefs(mem(JSON.stringify({ textSnap: 0.25 }))).textSnap).toBe(0.3)
    expect(loadPrefs(mem(JSON.stringify({ textSnap: 0 }))).textSnap).toBe(0.1)
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
