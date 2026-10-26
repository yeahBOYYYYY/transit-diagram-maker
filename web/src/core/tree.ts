// Helpers over the layers-panel tree (top of stack first).
import type { Id, TreeItem } from './types'

export function findItem(items: TreeItem[], id: Id): TreeItem | null {
  for (const it of items) {
    if (it.id === id) return it
    const f = findItem(it.children, id)
    if (f) return f
  }
  return null
}

/** Ancestor ids of `id`, nearest first. */
export function ancestorsOf(items: TreeItem[], id: Id): Id[] {
  const path: Id[] = []
  const go = (list: TreeItem[], trail: Id[]): boolean => {
    for (const it of list) {
      if (it.id === id) {
        path.push(...[...trail].reverse())
        return true
      }
      if (go(it.children, [...trail, it.id])) return true
    }
    return false
  }
  go(items, [])
  return path
}

export function parentOf(items: TreeItem[], id: Id): Id | null {
  return ancestorsOf(items, id)[0] ?? null
}

/** All leaf ids below the given nodes (including leaves themselves). */
export function leafIds(items: TreeItem[], ids: Id[]): Id[] {
  const out: Id[] = []
  const want = new Set(ids)
  const go = (list: TreeItem[], inside: boolean) => {
    for (const it of list) {
      const hit = inside || want.has(it.id)
      if (it.kind === 'group') go(it.children, hit)
      else if (hit) out.push(it.id)
    }
  }
  go(items, false)
  return [...new Set(out)]
}

/** Flattened display order (top first), only through expanded groups when `visibleOnly`. */
export function flatten(items: TreeItem[], visibleOnly = false): TreeItem[] {
  const out: TreeItem[] = []
  const go = (list: TreeItem[]) => {
    for (const it of list) {
      out.push(it)
      if (it.kind === 'group' && (!visibleOnly || it.expanded)) go(it.children)
    }
  }
  go(items)
  return out
}

/** Bottom-to-top index of a node within its parent, as the engine expects. */
export function stackIndex(items: TreeItem[], id: Id): { parent: Id | null; index: number; count: number } | null {
  const parent = parentOf(items, id)
  const siblings = parent === null ? items : findItem(items, parent)?.children ?? []
  const displayIndex = siblings.findIndex((s) => s.id === id)
  if (displayIndex < 0) return null
  return { parent, index: siblings.length - 1 - displayIndex, count: siblings.length }
}

/** Outermost group containing `id`, or `id` itself. */
export function topLevelOf(items: TreeItem[], id: Id): Id {
  const anc = ancestorsOf(items, id)
  return anc.length ? anc[anc.length - 1] : id
}

export function isEffectivelyLocked(items: TreeItem[], id: Id): boolean {
  const self = findItem(items, id)
  if (!self) return false
  return self.locked || ancestorsOf(items, id).some((a) => findItem(items, a)?.locked)
}
