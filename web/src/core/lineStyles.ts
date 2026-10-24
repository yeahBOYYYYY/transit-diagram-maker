// Line stroke styles offered in the pen options and the Properties panel.
import type { StrokeKind } from './types'

export const STROKE_KINDS: { value: StrokeKind; label: string }[] = [
  { value: 'plain', label: 'Plain' },
  { value: 'hollow', label: 'Hollow' },
  { value: 'dashed', label: 'Dashed' },
  { value: 'inner_dashed', label: 'Inner dashed' },
]

export const hasChannel = (k: StrokeKind) => k === 'hollow'
export const hasDashes = (k: StrokeKind) => k === 'dashed' || k === 'inner_dashed'
