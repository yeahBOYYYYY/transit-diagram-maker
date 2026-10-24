<script setup lang="ts">
// Every pictogram in a grid: the built-in library, the icons uploaded into
// the document and a tile to upload another SVG. Clicking a pictogram picks
// it (for the Icon tool, and for the selected pictograms); dragging one onto
// the canvas places it there.
import { computed } from 'vue'
import HelpTip from '../ui/HelpTip.vue'
import { Engine } from '../../core/engine'
import { HELP } from '../../core/help'
import type { IconInfo, Id } from '../../core/types'
import { leafIds } from '../../core/tree'
import { useEditor } from '../../stores/editor'
import { prefs } from '../../stores/prefs'
import { uploadIcon } from '../../stores/commands'

const store = useEditor()
const s = store.state
const builtIn = Engine.iconLibrary()

/** Pictogram layers in the selection (directly or inside selected groups). */
const selectedIcons = computed<Id[]>(() => {
  void s.revision
  return leafIds(s.tree, s.selection).filter((id) => {
    const n = store.node(id)
    return n?.type === 'layer' && n.content.kind === 'icon'
  })
})

/** The highlighted symbol: the selected pictogram's, or the Icon tool's. */
const current = computed(() => {
  const ids = selectedIcons.value
  const n = ids.length ? store.node(ids[0]) : null
  return n?.type === 'layer' && n.content.kind === 'icon' ? n.content.icon : prefs.icon.icon
})

function choose(icon: IconInfo): void {
  prefs.icon.icon = icon.name
  const ids = selectedIcons.value
  if (ids.length) store.run((e) => ids.forEach((id) => e.updateContent(id, { icon: icon.name })))
  else store.setTool('icon')
}

function onDragStart(icon: IconInfo, e: DragEvent): void {
  e.dataTransfer?.setData('application/x-transit-icon', icon.name)
  if (e.dataTransfer) e.dataTransfer.effectAllowed = 'copy'
}

async function upload(): Promise<void> {
  const name = await uploadIcon(store)
  if (name) prefs.icon.icon = name
}
</script>

<template>
  <div class="icons" data-testid="icons-panel">
    <p class="dim hint">
      Click a pictogram to use it{{ selectedIcons.length ? ' for the selection' : ' with the Icon tool' }}, or drag it onto the canvas.
      <HelpTip :text="HELP['icon.panel']" />
    </p>
    <div class="section-title">Library</div>
    <div class="grid">
      <button
        v-for="i in builtIn"
        :key="i.name"
        type="button"
        class="cell"
        :class="{ on: i.name === current }"
        :title="i.label"
        :data-icon="i.name"
        draggable="true"
        @click="choose(i)"
        @dragstart="onDragStart(i, $event)"
      >
        <svg viewBox="0 0 24 24" width="32" height="32" v-html="i.svg" />
        <span>{{ i.label }}</span>
      </button>
    </div>
    <div class="section-title">Uploaded<HelpTip :text="HELP['icon.upload']" /></div>
    <div class="grid">
      <button
        v-for="i in s.assets.icons"
        :key="i.name"
        type="button"
        class="cell"
        :class="{ on: i.name === current }"
        :title="i.label"
        :data-icon="i.name"
        draggable="true"
        @click="choose(i)"
        @dragstart="onDragStart(i, $event)"
      >
        <svg viewBox="0 0 24 24" width="32" height="32" v-html="i.svg" />
        <span>{{ i.label }}</span>
      </button>
      <button type="button" class="cell upload" title="Add your own SVG icon to this document" data-testid="upload-icon" @click="upload">
        <svg viewBox="0 0 24 24" width="32" height="32">
          <rect x="1" y="1" width="22" height="22" rx="5" fill="none" stroke="currentColor" stroke-dasharray="3 2" />
          <path d="M12 7v10M7 12h10" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
        <span>Upload SVG…</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.icons {
  padding: 8px 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hint {
  margin: 0;
  line-height: 1.5;
  font-size: var(--fs-small);
}
.section-title {
  display: flex;
  align-items: center;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--c-border-soft);
  font-weight: 600;
  color: var(--c-text-bright);
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(64px, 1fr));
  gap: 4px;
}
.cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  min-width: 0;
  padding: 6px 2px;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: transparent;
  cursor: grab;
  font-size: 10px;
  color: var(--c-text-dim);
  text-align: center;
}
.cell span {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cell:hover {
  background: var(--c-hover);
  color: var(--c-text);
}
.cell.on {
  border-color: var(--c-accent);
  color: var(--c-text-bright);
}
.cell.upload {
  cursor: pointer;
}
</style>
