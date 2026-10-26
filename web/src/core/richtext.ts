// Rich text of a text box: paragraphs of runs, each run with its own optional
// formatting (a missing value uses the box's). These helpers edit that model
// by character range and move it in and out of the inline editor's DOM.
import type { Paragraph, RunStyle, TextAlign, TextBox, TextRun } from './types'

/** A caret position: paragraph index and character offset inside it. */
export interface TextPos {
  p: number
  o: number
}

export interface TextRange {
  start: TextPos
  end: TextPos
}

export const RUN_KEYS = ['font_family', 'size', 'color', 'bold', 'italic'] as const
export type RunKey = (typeof RUN_KEYS)[number]

/** Formatting the box applies to runs without their own values. */
export type BoxStyle = Pick<TextBox, RunKey | 'align' | 'direction'>

/** Height of a line as a multiple of its largest font size (`model::LINE_SPACING`). */
export const LINE_SPACING = 1.2

export function plainParagraphs(text: string): Paragraph[] {
  return text.split('\n').map((l) => ({ runs: l ? [{ text: l }] : [] }))
}

export function plainText(ps: Paragraph[]): string {
  return ps.map(paragraphText).join('\n')
}

export function paragraphText(p: Paragraph): string {
  return p.runs.map((r) => r.text).join('')
}

export function styleOf(r: TextRun): RunStyle {
  const out: RunStyle = {}
  for (const k of RUN_KEYS) if (r[k] !== undefined && r[k] !== null) (out as Record<string, unknown>)[k] = r[k]
  return out
}

function sameStyle(a: RunStyle, b: RunStyle): boolean {
  return RUN_KEYS.every((k) => (a[k] ?? null) === (b[k] ?? null))
}

export function cloneParagraphs(ps: Paragraph[]): Paragraph[] {
  return ps.map((p) => ({ ...(p.align ? { align: p.align } : {}), runs: p.runs.map((r) => ({ ...r })) }))
}

/** Merge neighbouring runs with the same formatting and drop empty ones. */
export function normalize(ps: Paragraph[]): Paragraph[] {
  const out = ps.map((p) => {
    const runs: TextRun[] = []
    for (const r of p.runs) {
      if (!r.text) continue
      const last = runs[runs.length - 1]
      if (last && sameStyle(styleOf(last), styleOf(r))) last.text += r.text
      else runs.push({ ...styleOf(r), text: r.text })
    }
    return { ...(p.align ? { align: p.align } : {}), runs }
  })
  return out.length ? out : [{ runs: [] }]
}

export function comparePos(a: TextPos, b: TextPos): number {
  return a.p - b.p || a.o - b.o
}

/** The range with `start` before `end`. */
export function ordered(r: TextRange): TextRange {
  return comparePos(r.start, r.end) <= 0 ? r : { start: r.end, end: r.start }
}

export function isCollapsed(r: TextRange): boolean {
  return comparePos(r.start, r.end) === 0
}

export function wholeRange(ps: Paragraph[]): TextRange {
  const last = ps.length - 1
  return { start: { p: 0, o: 0 }, end: { p: Math.max(0, last), o: last >= 0 ? paragraphText(ps[last]).length : 0 } }
}

/** Whether the range covers all of the text. */
export function coversAll(ps: Paragraph[], r: TextRange): boolean {
  const w = wholeRange(ps)
  const o = ordered(r)
  return comparePos(o.start, w.start) <= 0 && comparePos(o.end, w.end) >= 0
}

/** Split runs so that `o` falls on a run boundary; returns the index of the run starting there. */
function splitRun(p: Paragraph, o: number): number {
  let at = 0
  for (let i = 0; i < p.runs.length; i++) {
    const r = p.runs[i]
    if (o <= at) return i
    if (o < at + r.text.length) {
      const cut = o - at
      p.runs.splice(i, 1, { ...r, text: r.text.slice(0, cut) }, { ...r, text: r.text.slice(cut) })
      return i + 1
    }
    at += r.text.length
  }
  return p.runs.length
}

/** Set (or, with `undefined`, clear) formatting on every character in the range. */
export function applyStyle(ps: Paragraph[], range: TextRange, style: Partial<Record<RunKey, unknown>>): Paragraph[] {
  const out = cloneParagraphs(ps)
  const { start, end } = ordered(range)
  for (let pi = start.p; pi <= end.p && pi < out.length; pi++) {
    const p = out[pi]
    const from = pi === start.p ? start.o : 0
    const to = pi === end.p ? end.o : paragraphText(p).length
    if (to <= from) continue
    const a = splitRun(p, from)
    const b = splitRun(p, to)
    for (let i = a; i < b; i++) {
      const r = p.runs[i] as unknown as Record<string, unknown>
      for (const [k, v] of Object.entries(style)) {
        if (v === undefined || v === null) delete r[k]
        else r[k] = v
      }
    }
  }
  return normalize(out)
}

/** Remove a property from every run, so the box value applies to all text. */
export function clearRunProperty(ps: Paragraph[], key: RunKey | 'align'): Paragraph[] {
  if (key === 'align') return ps.map((p) => ({ runs: p.runs.map((r) => ({ ...r })) }))
  return normalize(ps.map((p) => ({ ...p, runs: p.runs.map((r) => ({ ...r, [key]: undefined })) })))
}

/** Align the paragraphs the range touches. */
export function setAlign(ps: Paragraph[], range: TextRange, align: TextAlign): Paragraph[] {
  const out = cloneParagraphs(ps)
  const { start, end } = ordered(range)
  for (let pi = start.p; pi <= end.p && pi < out.length; pi++) out[pi].align = align
  return out
}

/** Formatting of the character just before `pos` (or after it at a paragraph start). */
export function styleAt(ps: Paragraph[], pos: TextPos): RunStyle {
  const p = ps[Math.min(pos.p, ps.length - 1)]
  if (!p || !p.runs.length) return {}
  let at = 0
  for (const r of p.runs) {
    if (pos.o <= at + r.text.length && (pos.o > at || at === 0)) return styleOf(r)
    at += r.text.length
  }
  return styleOf(p.runs[p.runs.length - 1])
}

/** Formatting shown for a selection: the first selected character's, resolved against the box. */
export function resolvedStyleAt(ps: Paragraph[], range: TextRange | null, box: BoxStyle): Required<RunStyle> & { align: TextAlign } {
  const r = range ? ordered(range) : wholeRange(ps)
  const pos = isCollapsed(r) ? r.start : { p: r.start.p, o: r.start.o + 1 }
  const st = styleAt(ps, pos)
  return {
    font_family: st.font_family ?? box.font_family,
    size: st.size ?? box.size,
    color: st.color ?? box.color,
    bold: st.bold ?? box.bold,
    italic: st.italic ?? box.italic,
    align: ps[Math.min(r.start.p, ps.length - 1)]?.align ?? box.align,
  }
}

/** Replace the range with `text` (line breaks start new paragraphs). The new text takes the formatting before the caret. */
export function insertText(ps: Paragraph[], range: TextRange, text: string): { paragraphs: Paragraph[]; caret: TextPos } {
  const out = cloneParagraphs(ps)
  const { start, end } = ordered(range)
  const style = styleAt(out, start)
  const first = out[start.p] ?? { runs: [] }
  const last = out[end.p] ?? first
  const a = splitRun(first, start.o)
  const head = first.runs.slice(0, a)
  const b = splitRun(last, end.o)
  const tail = last.runs.slice(b)
  const lines = text.replace(/\r/g, '').split('\n')
  const made: Paragraph[] = lines.map((l, i) => ({
    ...(i === 0 && first.align ? { align: first.align } : i > 0 && last.align ? { align: last.align } : {}),
    runs: l ? [{ ...style, text: l }] : [],
  }))
  made[0].runs = [...head, ...made[0].runs]
  const caret = { p: start.p + made.length - 1, o: paragraphText(made[made.length - 1]).length }
  made[made.length - 1].runs.push(...tail)
  out.splice(start.p, end.p - start.p + 1, ...made)
  return { paragraphs: normalize(out), caret }
}

// ------------------------------------------------------------------- DOM

const CSS_ALIGN: Record<TextAlign, string> = { start: 'left', middle: 'center', end: 'right' }

/** CSS for a run inside the editor; sizes are relative to the box size (em). */
function runCss(r: RunStyle, box: BoxStyle): string {
  const css: string[] = []
  if (r.font_family) css.push(`font-family:${r.font_family}`)
  if (r.size) css.push(`font-size:${r.size / box.size}em`)
  if (r.color) css.push(`color:${r.color}`)
  if (r.bold !== undefined) css.push(`font-weight:${r.bold ? 'bold' : 'normal'}`)
  if (r.italic !== undefined) css.push(`font-style:${r.italic ? 'italic' : 'normal'}`)
  return css.join(';')
}

/** Fill the editor element with one block per paragraph and one span per run. */
export function renderEditor(root: HTMLElement, ps: Paragraph[], box: BoxStyle): void {
  const doc = root.ownerDocument
  root.replaceChildren()
  for (const p of ps) {
    const div = doc.createElement('div')
    div.dataset.p = ''
    if (p.align) div.dataset.align = p.align
    div.style.textAlign = CSS_ALIGN[p.align ?? box.align]
    for (const r of p.runs) {
      const span = doc.createElement('span')
      const st = styleOf(r)
      span.dataset.run = JSON.stringify(st)
      const css = runCss(st, box)
      if (css) span.setAttribute('style', css)
      span.textContent = r.text
      div.appendChild(span)
    }
    if (!p.runs.length) div.appendChild(doc.createElement('br'))
    root.appendChild(div)
  }
}

function runStyleOf(el: Element, inherited: RunStyle): RunStyle {
  const raw = (el as HTMLElement).dataset?.run
  if (raw === undefined) return inherited
  try {
    return JSON.parse(raw) as RunStyle
  } catch {
    return inherited
  }
}

const BLOCKS = new Set(['DIV', 'P', 'LI'])

/**
 * Read paragraphs back from the editor, and the text positions of the given
 * DOM points (a selection's anchor and focus). Tolerates the markup browsers
 * produce while typing: stray text nodes, nested blocks and placeholder `<br>`s.
 */
export function readEditor(root: HTMLElement, points: { node: Node; offset: number }[] = []): { paragraphs: Paragraph[]; positions: (TextPos | null)[] } {
  const out: Paragraph[] = []
  let cur: Paragraph | null = null
  const positions: (TextPos | null)[] = points.map(() => null)
  const ensure = (): Paragraph => {
    if (!cur) {
      cur = { runs: [] }
      out.push(cur)
    }
    return cur
  }
  const len = (p: Paragraph) => p.runs.reduce((n, r) => n + r.text.length, 0)
  const mark = (node: Node, offset: number) => {
    points.forEach((pt, i) => {
      if (positions[i] === null && pt.node === node && pt.offset === offset) {
        const p = node === root ? null : ensure()
        positions[i] = p ? { p: out.length - 1, o: len(p) } : { p: out.length, o: 0 }
      }
    })
  }
  const visit = (node: Node, style: RunStyle): void => {
    if (node.nodeType === 3) {
      const text = (node.nodeValue ?? '').replace(/\n/g, ' ')
      const p = ensure()
      const at = len(p)
      points.forEach((pt, i) => {
        if (positions[i] === null && pt.node === node) positions[i] = { p: out.length - 1, o: at + Math.min(pt.offset, text.length) }
      })
      if (text) {
        const last = p.runs[p.runs.length - 1]
        if (last && sameStyle(styleOf(last), style)) last.text += text
        else p.runs.push({ ...style, text })
      }
      return
    }
    if (node.nodeType !== 1) return
    const el = node as HTMLElement
    const block = BLOCKS.has(el.tagName)
    if (block) {
      cur = { ...(el.dataset.align ? { align: el.dataset.align as TextAlign } : {}), runs: [] }
      out.push(cur)
    }
    const st = runStyleOf(el, style)
    el.childNodes.forEach((child, i) => {
      mark(el, i)
      visit(child, st)
    })
    mark(el, el.childNodes.length)
    if (block) cur = null
  }
  root.childNodes.forEach((child, i) => {
    mark(root, i)
    visit(child, {})
  })
  mark(root, root.childNodes.length)
  const paragraphs = normalize(out)
  const last = paragraphs.length - 1
  return {
    paragraphs,
    positions: positions.map((p) => (p && p.p > last ? { p: last, o: paragraphText(paragraphs[last]).length } : p)),
  }
}

/** The editor's selection as a text range, if the selection is inside it. */
export function editorSelection(root: HTMLElement): TextRange | null {
  const sel = root.ownerDocument.getSelection?.()
  if (!sel || !sel.anchorNode || !sel.focusNode || !root.contains(sel.anchorNode) || !root.contains(sel.focusNode)) return null
  const { positions } = readEditor(root, [
    { node: sel.anchorNode, offset: sel.anchorOffset },
    { node: sel.focusNode, offset: sel.focusOffset },
  ])
  const [a, b] = positions
  return a && b ? { start: a, end: b } : null
}

/** DOM point of a text position in markup made by `renderEditor`. */
function domPoint(root: HTMLElement, pos: TextPos): { node: Node; offset: number } {
  const blocks = [...root.children]
  const div = blocks[Math.min(pos.p, blocks.length - 1)]
  if (!div) return { node: root, offset: 0 }
  let at = 0
  const walker = root.ownerDocument.createTreeWalker(div, 4)
  let last: Text | null = null
  for (let n = walker.nextNode() as Text | null; n; n = walker.nextNode() as Text | null) {
    const l = n.data.length
    if (pos.o <= at + l) return { node: n, offset: pos.o - at }
    at += l
    last = n
  }
  return last ? { node: last, offset: last.data.length } : { node: div, offset: 0 }
}

export function setEditorSelection(root: HTMLElement, range: TextRange): void {
  const sel = root.ownerDocument.getSelection?.()
  if (!sel) return
  const a = domPoint(root, range.start)
  const b = domPoint(root, range.end)
  try {
    sel.setBaseAndExtent(a.node, a.offset, b.node, b.offset)
  } catch {
    /* the editor was re-rendered in between */
  }
}
