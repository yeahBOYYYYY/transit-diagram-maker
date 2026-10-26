import { describe, expect, it } from 'vitest'
import {
  applyStyle,
  clearRunProperty,
  coversAll,
  insertText,
  plainParagraphs,
  plainText,
  readEditor,
  renderEditor,
  resolvedStyleAt,
  setAlign,
  wholeRange,
  type BoxStyle,
} from '../../src/core/richtext'

const box: BoxStyle = { font_family: 'Sans', size: 1, color: '#000000', bold: false, italic: false, align: 'start', direction: 'ltr' }
const range = (p0: number, o0: number, p1: number, o1: number) => ({ start: { p: p0, o: o0 }, end: { p: p1, o: o1 } })

describe('rich text model', () => {
  it('formats a range across runs and paragraphs', () => {
    const ps = plainParagraphs('Hello world\nSecond')
    const out = applyStyle(ps, range(0, 6, 1, 3), { color: '#ff0000' })
    expect(out[0].runs).toEqual([{ text: 'Hello ' }, { text: 'world', color: '#ff0000' }])
    expect(out[1].runs).toEqual([{ text: 'Sec', color: '#ff0000' }, { text: 'ond' }])
    // Backwards ranges work too, and clearing a value merges the runs again.
    const back = applyStyle(out, range(1, 3, 0, 6), { color: undefined })
    expect(back).toEqual(plainParagraphs('Hello world\nSecond'))
  })

  it('reports the formatting of the selection', () => {
    const ps = applyStyle(plainParagraphs('ab'), range(0, 1, 0, 2), { size: 2, bold: true })
    expect(resolvedStyleAt(ps, range(0, 1, 0, 2), box)).toMatchObject({ size: 2, bold: true, font_family: 'Sans' })
    expect(resolvedStyleAt(ps, range(0, 0, 0, 1), box)).toMatchObject({ size: 1, bold: false })
    expect(resolvedStyleAt(ps, range(0, 2, 0, 2), box), 'a caret takes the character before it').toMatchObject({ size: 2 })
  })

  it('inserts text with line breaks, keeping the formatting around it', () => {
    const ps = applyStyle(plainParagraphs('abcd'), range(0, 0, 0, 2), { italic: true })
    const { paragraphs, caret } = insertText(ps, range(0, 1, 0, 3), 'X\nY')
    expect(plainText(paragraphs)).toBe('aX\nYd')
    expect(caret).toEqual({ p: 1, o: 1 })
    expect(paragraphs[0].runs).toEqual([{ text: 'aX', italic: true }])
  })

  it('aligns paragraphs and clears per-character values', () => {
    const ps = setAlign(plainParagraphs('a\nb\nc'), range(0, 0, 1, 0), 'end')
    expect(ps.map((p) => p.align ?? null)).toEqual(['end', 'end', null])
    expect(clearRunProperty(ps, 'align').every((p) => !p.align)).toBe(true)
    expect(coversAll(ps, wholeRange(ps))).toBe(true)
    expect(coversAll(ps, range(0, 0, 2, 0))).toBe(false)
  })
})

describe('rich text editor DOM', () => {
  it('renders runs and paragraphs and reads them back', () => {
    const el = document.createElement('div')
    const ps = applyStyle(plainParagraphs('Big red\n\nEnd'), range(0, 4, 0, 7), { color: '#ff0000', size: 2 })
    ps[2].align = 'middle'
    renderEditor(el, ps, box)
    expect(el.children).toHaveLength(3)
    expect(el.querySelector('span[style*="font-size: 2em"], span[style*="font-size:2em"]')).not.toBeNull()
    expect(readEditor(el).paragraphs).toEqual(ps)
  })

  it('reads what browsers produce while typing', () => {
    const el = document.createElement('div')
    el.innerHTML = 'loose text<div><span data-run=\'{"bold":true}\'>bold</span> after</div><div><br></div>'
    const text = el.querySelector('span')!.firstChild!
    const { paragraphs, positions } = readEditor(el, [{ node: text, offset: 2 }])
    expect(paragraphs).toEqual([{ runs: [{ text: 'loose text' }] }, { runs: [{ text: 'bold', bold: true }, { text: ' after' }] }, { runs: [] }])
    expect(positions[0]).toEqual({ p: 1, o: 2 })
  })
})
