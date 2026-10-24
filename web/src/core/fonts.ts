// Fonts: the built-in choices plus fonts uploaded into the document, which
// are registered with the browser so the canvas can draw them.
import type { Engine } from './engine'

export const FONT_FAMILIES = [
  'Helvetica, Arial, sans-serif',
  'Inter, Helvetica, Arial, sans-serif',
  'Georgia, serif',
  '"Trebuchet MS", sans-serif',
  'Verdana, sans-serif',
  '"Courier New", monospace',
]

/** The CSS font-family value text layers use for an uploaded font (as the engine writes it). */
export function uploadedFamily(family: string): string {
  return `"${family}", sans-serif`
}

/** Display name of a CSS font-family value. */
export function fontLabel(css: string): string {
  return css.split(',')[0].replace(/"/g, '')
}

/** Built-in fonts followed by the document's uploaded fonts. */
export function fontChoices(uploaded: string[]): string[] {
  return [...FONT_FAMILIES, ...uploaded.map(uploadedFamily)]
}

const registered = new Set<string>()

/** Make uploaded fonts available to the page (once per family and content). */
export function registerFonts(engine: Engine, families: string[]): void {
  if (typeof FontFace === 'undefined' || typeof document === 'undefined' || !document.fonts) return
  for (const family of families) {
    const data = engine.fontData(family)
    const key = `${family}:${data.length}:${data.slice(-32)}`
    if (!data || registered.has(key)) continue
    registered.add(key)
    const face = new FontFace(family, `url(${data})`)
    face
      .load()
      .then((f) => document.fonts.add(f))
      .catch(() => registered.delete(key))
  }
}
