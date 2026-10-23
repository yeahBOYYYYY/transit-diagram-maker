import { describe, expect, it } from 'vitest'
import type { TreeItem } from '../../src/core/types'
import { ancestorsOf, findItem, flatten, isEffectivelyLocked, leafIds, parentOf, stackIndex, topLevelOf } from '../../src/core/tree'

const leaf = (id: number, extra: Partial<TreeItem> = {}): TreeItem => ({
  id,
  name: `L${id}`,
  kind: 'text',
  visible: true,
  locked: false,
  expanded: false,
  children: [],
  ...extra,
})
const group = (id: number, children: TreeItem[], extra: Partial<TreeItem> = {}): TreeItem => ({
  ...leaf(id),
  kind: 'group',
  role: 'generic',
  expanded: true,
  children,
  ...extra,
})

// Display order: top first.
const tree: TreeItem[] = [leaf(6), group(2, [group(4, [leaf(5)]), leaf(3)], { locked: true }), leaf(1)]

describe('tree helpers', () => {
  it('finds items and ancestors', () => {
    expect(findItem(tree, 5)?.name).toBe('L5')
    expect(findItem(tree, 99)).toBeNull()
    expect(ancestorsOf(tree, 5)).toEqual([4, 2])
    expect(parentOf(tree, 3)).toBe(2)
    expect(parentOf(tree, 6)).toBeNull()
    expect(topLevelOf(tree, 5)).toBe(2)
    expect(topLevelOf(tree, 1)).toBe(1)
  })
  it('expands groups to leaves', () => {
    expect(leafIds(tree, [2])).toEqual([5, 3])
    expect(leafIds(tree, [1, 4, 5])).toEqual([5, 1])
  })
  it('flattens with optional collapse', () => {
    expect(flatten(tree).map((t) => t.id)).toEqual([6, 2, 4, 5, 3, 1])
    const collapsed = [group(2, [leaf(3)], { expanded: false })]
    expect(flatten(collapsed, true).map((t) => t.id)).toEqual([2])
  })
  it('converts display position to bottom-to-top stack index', () => {
    expect(stackIndex(tree, 6)).toEqual({ parent: null, index: 2, count: 3 })
    expect(stackIndex(tree, 1)).toEqual({ parent: null, index: 0, count: 3 })
    expect(stackIndex(tree, 3)).toEqual({ parent: 2, index: 0, count: 2 })
    expect(stackIndex(tree, 42)).toBeNull()
  })
  it('inherits locks', () => {
    expect(isEffectivelyLocked(tree, 5)).toBe(true)
    expect(isEffectivelyLocked(tree, 1)).toBe(false)
    expect(isEffectivelyLocked(tree, 42)).toBe(false)
  })
})
