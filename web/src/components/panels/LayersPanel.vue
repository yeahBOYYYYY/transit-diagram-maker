<script setup lang="ts">
// Layers panel: top of the list is painted on top. Supports selection,
// visibility, locking, renaming, grouping and drag & drop re-ordering.
import { computed, nextTick, ref, watch } from 'vue'
import PanelFrame from './PanelFrame.vue'
import LayerRow from './LayerRow.vue'
import UiIcon from '../ui/UiIcon.vue'
import type { Fragment, Id, TreeItem } from '../../core/types'
import { ancestorsOf, findItem, flatten, isEffectivelyLocked, stackIndex } from '../../core/tree'
import { useEditor } from '../../stores/editor'
import { prefs } from '../../stores/prefs'
import { canUngroup, runAction, selectionDeletable } from '../../stores/commands'

const store = useEditor()
const s = store.state

const fragments = computed(() => new Map<Id, Fragment>(s.fragments.map((f) => [f.id, f])))
let anchor: Id | null = null
const list = ref<HTMLElement | null>(null)

// Like Photoshop and GIMP: whatever gets selected (a click on the canvas,
// a marquee, a new object) is shown in the list. The groups around it open
// (pure view state, so neither an undo step nor an unsaved change) and the
// list scrolls to its row.
watch(
  () => s.selection,
  async (sel, old) => {
    const added = sel.filter((id) => !old?.includes(id))
    if (!added.length) return
    const closed = new Set(added.flatMap((id) => ancestorsOf(s.tree, id)).filter((g) => findItem(s.tree, g)?.expanded === false))
    if (closed.size) {
      for (const g of closed) store.engine.value.setExpanded(g, true)
      store.refresh()
    }
    await nextTick()
    list.value?.querySelector(`[data-layer-id="${added[added.length - 1]}"]`)?.scrollIntoView?.({ block: 'nearest' })
  },
)

function onSelect(id: Id, e: MouseEvent): void {
  if (e.shiftKey && anchor !== null) {
    const rows = flatten(s.tree, true).map((r) => r.id)
    const a = rows.indexOf(anchor)
    const b = rows.indexOf(id)
    if (a >= 0 && b >= 0) {
      store.select(rows.slice(Math.min(a, b), Math.max(a, b) + 1))
      return
    }
  }
  if (e.ctrlKey || e.metaKey) store.select([id], 'toggle')
  else store.select([id])
  anchor = id
}

function toggleVisible(item: TreeItem): void {
  store.run((e) => e.setVisible(item.id, !item.visible))
}

function toggleLock(item: TreeItem): void {
  store.run((e) => e.setLocked(item.id, !item.locked))
}

function toggleExpand(item: TreeItem): void {
  store.run((e) => e.setExpanded(item.id, !item.expanded))
}

function rename(id: Id, name: string): void {
  store.run((e) => e.rename(id, name))
}

// ------------------------------------------------------------ drag & drop

const dragId = ref<Id | null>(null)
const dropHint = ref<{ id: Id; where: 'above' | 'below' | 'inside' } | null>(null)

function onDragStart(id: Id, e: DragEvent): void {
  dragId.value = id
  e.dataTransfer?.setData('text/plain', String(id))
  if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move'
}

function onDragOver(item: TreeItem, e: DragEvent): void {
  if (dragId.value === null || dragId.value === item.id) {
    dropHint.value = null
    return
  }
  const el = e.currentTarget as HTMLElement
  const rect = el.getBoundingClientRect()
  const t = (e.clientY - rect.top) / rect.height
  const where = item.kind === 'group' && t > 0.3 && t < 0.7 ? 'inside' : t < 0.5 ? 'above' : 'below'
  dropHint.value = { id: item.id, where }
}

function onDrop(item: TreeItem): void {
  const hint = dropHint.value
  const id = dragId.value
  dropHint.value = null
  dragId.value = null
  if (!hint || id === null || id === item.id) return
  if (hint.where === 'inside') {
    const count = findItem(s.tree, item.id)?.children.length ?? 0
    store.run((e) => e.moveNode(id, item.id, count))
    return
  }
  const pos = stackIndex(s.tree, item.id)
  if (!pos) return
  // "above" in the list = higher in the stack.
  const index = hint.where === 'above' ? pos.index + 1 : pos.index
  store.run((e) => e.moveNode(id, pos.parent, index))
}

function onDragEnd(): void {
  dragId.value = null
  dropHint.value = null
}

const selectionLocked = computed(
  () => s.selection.length > 0 && s.selection.every((id) => isEffectivelyLocked(s.tree, id)),
)
</script>

<template>
  <PanelFrame title="Layers" grow>
    <div ref="list" class="layers" data-testid="layers-panel" @click.self="store.clearSelection()">
      <LayerRow
        v-for="item in s.tree"
        :key="item.id"
        :item="item"
        :depth="0"
        :selected="s.selection"
        :fragments="fragments"
        :thumbnails="prefs.showThumbnails"
        :drop-hint="dropHint"
        @select="onSelect"
        @toggle-visible="toggleVisible"
        @toggle-lock="toggleLock"
        @toggle-expand="toggleExpand"
        @rename="rename"
        @dragstart="onDragStart"
        @dragover="onDragOver"
        @drop="onDrop"
        @dragend="onDragEnd"
      />
      <div v-if="!s.tree.length" class="empty dim">
        No layers yet.<br />Pick the Line Pen (P) and click on the canvas to start drawing.
      </div>
    </div>
    <template #footer>
      <span class="count dim">{{ s.selection.length ? `${s.selection.length} selected` : '' }}</span>
      <button class="icon-btn" :class="{ on: selectionLocked }" title="Lock / unlock selection" :disabled="!s.selection.length" @click="runAction(store, 'layer.toggleLock')">
        <UiIcon name="lock" :size="15" />
      </button>
      <button class="icon-btn" title="Bring forward" :disabled="!s.selection.length" @click="runAction(store, 'layer.bringForward')">
        <UiIcon name="up" :size="15" />
      </button>
      <button class="icon-btn" title="Send backward" :disabled="!s.selection.length" @click="runAction(store, 'layer.sendBackward')">
        <UiIcon name="down" :size="15" />
      </button>
      <button class="icon-btn" title="Group selection (Ctrl+G)" data-testid="group-btn" :disabled="!s.selection.length" @click="runAction(store, 'layer.group')">
        <UiIcon name="group" :size="15" />
      </button>
      <button class="icon-btn" title="Ungroup (Ctrl+Shift+G)" data-testid="ungroup-btn" :disabled="!canUngroup(store)" @click="runAction(store, 'layer.ungroup')">
        <UiIcon name="ungroup" :size="15" />
      </button>
      <button class="icon-btn" title="New group" data-testid="new-group-btn" @click="runAction(store, 'layer.newGroup')">
        <UiIcon name="folder-plus" :size="15" />
      </button>
      <button class="icon-btn" title="Delete" data-testid="delete-btn" :disabled="!selectionDeletable(store)" @click="runAction(store, 'edit.delete')">
        <UiIcon name="trash" :size="15" />
      </button>
    </template>
  </PanelFrame>
</template>

<style scoped>
.layers {
  min-height: 100%;
}
.empty {
  padding: 24px 18px;
  text-align: center;
  line-height: 1.6;
}
.count {
  margin-right: auto;
  font-size: var(--fs-small);
}
</style>
