// Keyboard shortcut parsing and matching. Combos are written as
// "Ctrl+Shift+Z"; several alternatives are separated by ", ".

const CODE_KEYS: Record<string, string> = {
  Space: 'Space',
  Equal: '=',
  Minus: '-',
  BracketLeft: '[',
  BracketRight: ']',
  Backslash: '\\',
  Semicolon: ';',
  Quote: "'",
  Comma: ',',
  Period: '.',
  Slash: '/',
  Backquote: '`',
  NumpadAdd: '=',
  NumpadSubtract: '-',
}

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta', 'AltGraph', 'CapsLock'])

export interface KeyLike {
  key: string
  code?: string
  ctrlKey?: boolean
  metaKey?: boolean
  altKey?: boolean
  shiftKey?: boolean
}

/** Layout independent main key name for an event, or null for bare modifiers. */
export function keyName(e: KeyLike): string | null {
  if (MODIFIER_KEYS.has(e.key)) return null
  const code = e.code ?? ''
  if (/^Key[A-Z]$/.test(code)) return code.slice(3)
  if (/^Digit[0-9]$/.test(code)) return code.slice(5)
  if (/^Numpad[0-9]$/.test(code)) return code.slice(6)
  if (CODE_KEYS[code]) return CODE_KEYS[code]
  if (e.key === ' ') return 'Space'
  if (e.key === '+') return '='
  if (e.key.length === 1) return e.key.toUpperCase()
  return e.key // Enter, Escape, Delete, ArrowUp, F1 ...
}

export function eventToCombo(e: KeyLike): string | null {
  const key = keyName(e)
  if (!key) return null
  const parts: string[] = []
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl')
  if (e.altKey) parts.push('Alt')
  if (e.shiftKey) parts.push('Shift')
  parts.push(key)
  return parts.join('+')
}

/** Normalize user written combos ("shift+ctrl+z" -> "Ctrl+Shift+Z"). */
export function normalizeCombo(combo: string): string {
  const raw = combo
    .split('+')
    .map((p) => p.trim())
    .filter(Boolean)
  // "Ctrl++" splits into an empty tail; treat as "=".
  if (combo.trim().endsWith('++')) raw.push('=')
  let ctrl = false
  let alt = false
  let shift = false
  let key = ''
  for (const p of raw) {
    const l = p.toLowerCase()
    if (l === 'ctrl' || l === 'control' || l === 'cmd' || l === 'meta') ctrl = true
    else if (l === 'alt' || l === 'option') alt = true
    else if (l === 'shift') shift = true
    else key = p.length === 1 ? p.toUpperCase() : p[0].toUpperCase() + p.slice(1)
  }
  if (key === '+') key = '='
  return [ctrl && 'Ctrl', alt && 'Alt', shift && 'Shift', key].filter(Boolean).join('+')
}

export function splitCombos(binding: string): string[] {
  // A comma right after "+" is the comma key ("Ctrl+,"), not a separator.
  return binding
    .split(/(?<!\+),\s*/)
    .map((c) => c.trim())
    .filter(Boolean)
    .map(normalizeCombo)
}

export function matches(binding: string, e: KeyLike): boolean {
  const combo = eventToCombo(e)
  return combo !== null && splitCombos(binding).includes(combo)
}

/** Find bindings that share a combo, for the shortcut editor's conflict warnings. */
export function findConflicts(bindings: Record<string, string>): Map<string, string[]> {
  const byCombo = new Map<string, string[]>()
  for (const [action, binding] of Object.entries(bindings)) {
    for (const combo of splitCombos(binding)) {
      byCombo.set(combo, [...(byCombo.get(combo) ?? []), action])
    }
  }
  return new Map([...byCombo].filter(([, list]) => list.length > 1))
}

/** Pretty label for menus. */
export function displayCombo(binding: string): string {
  return splitCombos(binding)[0] ?? ''
}
