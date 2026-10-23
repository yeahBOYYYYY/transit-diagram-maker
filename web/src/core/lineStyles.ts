// Line stroke styles offered in the pen options and the Properties panel.
import type { StrokeKind } from './types'

export const STROKE_KINDS: { value: StrokeKind; label: string }[] = [
  { value: 'plain', label: 'Plain' },
  { value: 'hollow', label: 'Hollow' },
  { value: 'dashed', label: 'Dashed' },
  { value: 'hollow_dashed', label: 'Hollow dashed' },
]

export const hasChannel = (k: StrokeKind) => k === 'hollow' || k === 'hollow_dashed'
export const hasDashes = (k: StrokeKind) => k === 'dashed' || k === 'hollow_dashed'
