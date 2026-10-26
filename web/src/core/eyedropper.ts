// Color picker for the color fields: after `startPicking`, the next click
// takes the color painted on the canvas under the pointer (or the color of
// any color swatch in the interface) instead of acting on what it hits.
import { shallowRef } from 'vue'

type OnPick = (hex: string) => void

/** The field waiting for a color, or null when not picking. */
export const picking = shallowRef<OnPick | null>(null)

/** `rgb(…)`, `rgba(…)` or `#rgb` / `#rrggbb` as `#rrggbb`; null for anything else (none, gradients, transparent). */
export function toHex(css: string | null | undefined): string | null {
  const v = (css ?? '').trim().toLowerCase()
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/.exec(v)
  if (short) return `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}`
  if (/^#[0-9a-f]{6}$/.test(v)) return v
  const m = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)(?:[\s,/]+([\d.]+%?))?\s*\)$/.exec(v)
  if (!m) return null
  if (m[4] !== undefined && parseFloat(m[4]) === 0) return null
  return '#' + [m[1], m[2], m[3]].map((c) => Math.round(Math.min(255, parseFloat(c))).toString(16).padStart(2, '0')).join('')
}

/** The color painted by an SVG shape at a screen point: its stroke if the point is on it, otherwise its fill. */
function shapeColor(el: SVGElement, x: number, y: number): string | null {
  const style = getComputedStyle(el)
  const stroke = toHex(style.stroke)
  const fill = toHex(style.fill)
  const geom = el as SVGGeometryElement
  const m = typeof geom.getScreenCTM === 'function' ? geom.getScreenCTM() : null
  if (m && typeof geom.isPointInStroke === 'function' && typeof DOMPoint !== 'undefined') {
    const p = new DOMPoint(x, y).matrixTransform(m.inverse())
    // The stroke is painted over the fill.
    if (stroke && geom.isPointInStroke(p)) return stroke
    if (fill && geom.isPointInFill(p)) return fill
    return null
  }
  return fill ?? stroke
}

/** The color under a screen point: a drawn shape, a color swatch, or the canvas paper. */
export function colorAt(x: number, y: number): string | null {
  for (const el of document.elementsFromPoint(x, y)) {
    const swatch = (el as HTMLElement).closest?.('[data-swatch]') as HTMLElement | null
    if (swatch) return toHex(swatch.style.background || getComputedStyle(swatch).backgroundColor)
    if (el instanceof SVGElement && el.closest('g[data-id]') && !(el instanceof SVGGElement || el instanceof SVGSVGElement)) {
      const c = shapeColor(el, x, y)
      if (c) return c
    }
    if (el instanceof SVGElement && el.dataset.paper !== undefined) return toHex(getComputedStyle(el).fill)
  }
  return null
}

function swallow(e: Event): void {
  e.preventDefault()
  e.stopPropagation()
}

/**
 * Keep the release and the click that follow the picking press away from
 * whatever was under the pointer. Both are dropped at the next press, in
 * case the pointer moved off and no click came.
 */
function swallowRestOfClick(): void {
  const done = () => {
    window.removeEventListener('pointerup', swallow, true)
    window.removeEventListener('click', onClick, true)
    window.removeEventListener('pointerdown', done, true)
  }
  const onClick = (e: Event) => {
    swallow(e)
    done()
  }
  window.addEventListener('pointerup', swallow, true)
  window.addEventListener('click', onClick, true)
  // Registered after the current press has finished dispatching.
  setTimeout(() => window.addEventListener('pointerdown', done, true), 0)
}

function onDown(e: PointerEvent): void {
  swallow(e)
  swallowRestOfClick()
  // Clicking the button that started picking again just stops.
  const onPick = (e.target as HTMLElement | null)?.closest?.('[data-eyedropper]') ? null : picking.value
  const color = onPick ? colorAt(e.clientX, e.clientY) : null
  stopPicking()
  if (color && onPick) onPick(color)
}

function onKey(e: KeyboardEvent): void {
  if (e.key !== 'Escape') return
  e.preventDefault()
  e.stopPropagation()
  stopPicking()
}

export function startPicking(onPick: OnPick): void {
  stopPicking()
  picking.value = onPick
  window.addEventListener('pointerdown', onDown, true)
  window.addEventListener('keydown', onKey, true)
  document.body.classList.add('picking-color')
}

export function stopPicking(): void {
  picking.value = null
  window.removeEventListener('pointerdown', onDown, true)
  window.removeEventListener('keydown', onKey, true)
  document.body.classList.remove('picking-color')
}
