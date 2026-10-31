<script setup lang="ts">
// One row of the layers panel (recursive for groups).
import { computed, nextTick, ref } from 'vue'
import UiIcon from '../ui/UiIcon.vue'
import type { Fragment, Id, TreeItem } from '../../core/types'

const props = defineProps<{
  item: TreeItem
  depth: number
  selected: Id[]
  fragments: Map<Id, Fragment>
  thumbnails: boolean
  parentHidden?: boolean
  parentLocked?: boolean
  dropHint: { id: Id; where: 'above' | 'below' | 'inside' } | null
}>()

const emit = defineEmits<{
  select: [id: Id, e: MouseEvent]
  toggleVisible: [item: TreeItem]
  toggleLock: [item: TreeItem]
  toggleExpand: [item: TreeItem]
  rename: [id: Id, name: string]
  dragstart: [id: Id, e: DragEvent]
  dragover: [item: TreeItem, e: DragEvent]
  drop: [item: TreeItem, e: DragEvent]
  dragend: []
}>()

const isGroup = computed(() => props.item.kind === 'group')
const isSelected = computed(() => props.selected.includes(props.item.id))
const editing = ref(false)
const draft = ref('')
const input = ref<HTMLInputElement | null>(null)

const thumb = computed(() => {
  const f = props.fragments.get(props.item.id)
  if (!f || !f.bounds) return null
  const pad = Math.max(f.bounds.w, f.bounds.h) * 0.08 + 1
  return {
    viewBox: `${f.bounds.x - pad} ${f.bounds.y - pad} ${f.bounds.w + pad * 2} ${f.bounds.h + pad * 2}`,
    svg: f.svg,
  }
})

const kindIcon = computed(() => {
  switch (props.item.kind) {
    case 'group':
      return props.item.role === 'line' ? 'line' : props.item.role === 'station' ? 'station' : 'folder'
    case 'line':
      return 'line'
    case 'station':
      return 'station'
    case 'text':
      return 'text'
    case 'shape':
      return 'shape'
    default:
      return 'icon'
  }
})

async function startRename(): Promise<void> {
  draft.value = props.item.name
  editing.value = true
  await nextTick()
  input.value?.select()
}

function finishRename(commit: boolean): void {
  if (!editing.value) return
  editing.value = false
  if (commit && draft.value.trim() && draft.value !== props.item.name) emit('rename', props.item.id, draft.value)
}

const hint = computed(() => (props.dropHint?.id === props.item.id ? props.dropHint.where : null))
</script>

<template>
  <div class="layer-wrap">
    <div
      class="row"
      :class="{
        selected: isSelected,
        hidden: !item.visible || parentHidden,
        ['drop-' + hint]: hint,
      }"
      :data-layer-id="item.id"
      :data-kind="item.kind"
      draggable="true"
      @click="emit('select', item.id, $event)"
      @dragstart="emit('dragstart', item.id, $event)"
      @dragover.prevent="emit('dragover', item, $event)"
      @drop.prevent="emit('drop', item, $event)"
      @dragend="emit('dragend')"
    >
      <button
        class="col-eye"
        :title="item.visible ? 'Hide layer' : 'Show layer'"
        data-testid="toggle-visible"
        @click.stop="emit('toggleVisible', item)"
      >
        <UiIcon v-if="item.visible" name="eye" :size="15" />
        <span v-else class="eye-off" />
      </button>
      <div class="indent" :style="{ width: depth * 14 + 'px' }" />
      <button
        v-if="isGroup"
        class="caret"
        :title="item.expanded ? 'Collapse' : 'Expand'"
        @click.stop="emit('toggleExpand', item)"
      >
        <UiIcon :name="item.expanded ? 'caret-down' : 'caret'" :size="10" />
      </button>
      <span v-else class="caret-space" />
      <div class="thumb" :class="{ group: isGroup }">
        <svg v-if="thumbnails && thumb && !isGroup" :viewBox="thumb.viewBox" preserveAspectRatio="xMidYMid meet" v-html="thumb.svg" />
        <UiIcon v-else :name="kindIcon" :size="16" />
      </div>
      <input
        v-if="editing"
        ref="input"
        v-model="draft"
        class="rename"
        @click.stop
        @keydown.stop.enter="finishRename(true)"
        @keydown.stop.esc="finishRename(false)"
        @blur="finishRename(true)"
      />
      <span v-else class="name" :title="item.name" @dblclick.stop="startRename">{{ item.name }}</span>
      <button
        class="col-lock"
        :class="{ on: item.locked, inherited: !item.locked && parentLocked }"
        :title="item.locked ? 'Unlock' : 'Lock'"
        data-testid="toggle-lock"
        @click.stop="emit('toggleLock', item)"
      >
        <UiIcon :name="item.locked || parentLocked ? 'lock' : 'unlock'" :size="13" />
      </button>
    </div>
    <template v-if="isGroup && item.expanded">
      <LayerRow
        v-for="child in item.children"
        :key="child.id"
        :item="child"
        :depth="depth + 1"
        :selected="selected"
        :fragments="fragments"
        :thumbnails="thumbnails"
        :parent-hidden="parentHidden || !item.visible"
        :parent-locked="parentLocked || item.locked"
        :drop-hint="dropHint"
        @select="(id, e) => emit('select', id, e)"
        @toggle-visible="(i) => emit('toggleVisible', i)"
        @toggle-lock="(i) => emit('toggleLock', i)"
        @toggle-expand="(i) => emit('toggleExpand', i)"
        @rename="(id, n) => emit('rename', id, n)"
        @dragstart="(id, e) => emit('dragstart', id, e)"
        @dragover="(i, e) => emit('dragover', i, e)"
        @drop="(i, e) => emit('drop', i, e)"
        @dragend="emit('dragend')"
      />
    </template>
  </div>
</template>

<style scoped>
.row {
  position: relative;
  display: flex;
  align-items: center;
  height: 34px;
  border-bottom: 1px solid #2c2c2c;
  cursor: default;
  background: var(--c-panel);
}
.row:hover {
  background: #404040;
}
.row.selected {
  background: var(--c-row-selected);
  color: #fff;
}
.row.hidden .name,
.row.hidden .thumb {
  opacity: 0.45;
}
.row.drop-above::before,
.row.drop-below::after {
  content: '';
  position: absolute;
  left: 30px;
  right: 0;
  height: 2px;
  background: var(--c-accent);
  z-index: 2;
}
.row.drop-above::before {
  top: -1px;
}
.row.drop-below::after {
  bottom: -1px;
}
.row.drop-inside {
  box-shadow: inset 0 0 0 2px var(--c-accent);
}
.col-eye {
  width: 30px;
  height: 100%;
  border: 0;
  border-right: 1px solid #2c2c2c;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--c-text);
  flex: none;
}
.eye-off {
  width: 15px;
  height: 15px;
}
.indent {
  flex: none;
}
.caret,
.caret-space {
  width: 16px;
  flex: none;
}
.caret {
  height: 20px;
  border: 0;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--c-text-dim);
  cursor: pointer;
  padding: 0;
}
.thumb {
  width: 36px;
  height: 26px;
  margin: 0 8px 0 2px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fff;
  border: 1px solid #1c1c1c;
  color: #444;
}
.thumb.group {
  background: transparent;
  border-color: transparent;
  color: var(--c-text);
}
.thumb svg {
  width: 100%;
  height: 100%;
}
.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rename {
  flex: 1;
  min-width: 0;
  height: 22px;
  padding: 0 4px;
}
.col-lock {
  width: 26px;
  height: 100%;
  border: 0;
  background: transparent;
  color: var(--c-text-dim);
  opacity: 0;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
}
.row:hover .col-lock,
.col-lock.on,
.col-lock.inherited {
  opacity: 1;
}
.col-lock.inherited {
  color: #666;
}
.col-lock.on {
  color: var(--c-text);
}
</style>
