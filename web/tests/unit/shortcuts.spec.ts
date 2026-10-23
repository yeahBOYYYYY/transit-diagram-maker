import { describe, expect, it } from 'vitest'
import { displayCombo, eventToCombo, findConflicts, keyName, matches, normalizeCombo, splitCombos } from '../../src/core/shortcuts'
import { ACTION_IDS, DEFAULT_SHORTCUTS } from '../../src/core/actions'

describe('keyName', () => {
  it('uses the physical key for letters and digits', () => {
    expect(keyName({ key: 'z', code: 'KeyZ' })).toBe('Z')
    expect(keyName({ key: 'Z', code: 'KeyZ', shiftKey: true })).toBe('Z')
    expect(keyName({ key: '!', code: 'Digit1', shiftKey: true })).toBe('1')
  })
  it('maps punctuation and special keys', () => {
    expect(keyName({ key: '=', code: 'Equal' })).toBe('=')
    expect(keyName({ key: '+', code: 'NumpadAdd' })).toBe('=')
    expect(keyName({ key: ']', code: 'BracketRight' })).toBe(']')
    expect(keyName({ key: 'Delete', code: 'Delete' })).toBe('Delete')
    expect(keyName({ key: ' ', code: 'Space' })).toBe('Space')
  })
  it('ignores bare modifiers', () => {
    expect(keyName({ key: 'Shift', code: 'ShiftLeft' })).toBeNull()
    expect(eventToCombo({ key: 'Control', code: 'ControlLeft', ctrlKey: true })).toBeNull()
  })
})

describe('combos', () => {
  it('builds combos in canonical modifier order', () => {
    expect(eventToCombo({ key: 'z', code: 'KeyZ', ctrlKey: true, shiftKey: true })).toBe('Ctrl+Shift+Z')
    expect(eventToCombo({ key: 'k', code: 'KeyK', metaKey: true, altKey: true })).toBe('Ctrl+Alt+K')
  })
  it('normalizes user input', () => {
    expect(normalizeCombo('shift+ctrl+z')).toBe('Ctrl+Shift+Z')
    expect(normalizeCombo('Cmd+Option+n')).toBe('Ctrl+Alt+N')
    expect(normalizeCombo('ctrl++')).toBe('Ctrl+=')
    expect(normalizeCombo('delete')).toBe('Delete')
  })
  it('supports several alternatives per binding', () => {
    expect(splitCombos('Ctrl+Shift+Z, Ctrl+Y')).toEqual(['Ctrl+Shift+Z', 'Ctrl+Y'])
    expect(matches('Ctrl+Shift+Z, Ctrl+Y', { key: 'y', code: 'KeyY', ctrlKey: true })).toBe(true)
    expect(matches('Ctrl+Z', { key: 'z', code: 'KeyZ', ctrlKey: true, shiftKey: true })).toBe(false)
    expect(displayCombo('Delete, Backspace')).toBe('Delete')
    expect(displayCombo('')).toBe('')
  })
  it('detects conflicts', () => {
    const c = findConflicts({ a: 'Ctrl+K', b: 'ctrl+k', c: 'X' })
    expect([...c.keys()]).toEqual(['Ctrl+K'])
    expect(c.get('Ctrl+K')).toEqual(['a', 'b'])
  })
  it('ships default shortcuts without conflicts and with single letter tools', () => {
    expect(findConflicts(DEFAULT_SHORTCUTS).size).toBe(0)
    for (const id of ACTION_IDS.filter((a) => a.startsWith('tool.'))) {
      expect(DEFAULT_SHORTCUTS[id]).toMatch(/^[A-Z]$/)
    }
    expect(DEFAULT_SHORTCUTS['edit.undo']).toBe('Ctrl+Z')
    expect(splitCombos(DEFAULT_SHORTCUTS['edit.redo'])).toContain('Ctrl+Shift+Z')
  })
})
