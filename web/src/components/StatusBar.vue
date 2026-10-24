<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useEditor } from '../stores/editor'
import { APP } from '../core/about'
import UiIcon from './ui/UiIcon.vue'

const store = useEditor()
const s = store.state
const zoomText = ref('100')
watch(
  () => s.view.zoom,
  (z) => (zoomText.value = String(Math.round(z * 1000) / 10)),
  { immediate: true },
)

function commitZoom(): void {
  const v = parseFloat(zoomText.value)
  if (Number.isFinite(v) && v > 0) store.setZoom(v / 100)
  else zoomText.value = String(Math.round(s.view.zoom * 1000) / 10)
}

const cursor = computed(() => {
  if (!s.cursor) return ''
  return `X ${s.cursor.x.toFixed(1)}  Y ${s.cursor.y.toFixed(1)}  · cell ${Math.floor(s.cursor.x)}, ${Math.floor(s.cursor.y)}`
})
const layerCount = computed(() => s.fragments.length)
</script>

<template>
  <footer class="status" data-testid="status-bar">
    <label class="zoom">
      <input v-model="zoomText" data-testid="zoom-input" @keydown.stop.enter="commitZoom" @blur="commitZoom" />%
    </label>
    <span class="sep" />
    <span class="dim">{{ cursor }}</span>
    <span class="sep" />
    <span class="dim">{{ layerCount }} visible layer{{ layerCount === 1 ? '' : 's' }}</span>
    <span class="msg" :class="s.messageKind" data-testid="status-message">{{ s.message }}</span>
    <span class="dim hist">{{ s.canUndo ? 'Undo available' : '' }}</span>
    <span class="sep" />
    <a class="github" :href="APP.repoUrl" target="_blank" rel="noopener" title="Transit Diagram Maker on GitHub" data-testid="github-link">
      <UiIcon name="github" :size="14" />
    </a>
  </footer>
</template>

<style scoped>
.status {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 24px;
  padding: 0 10px;
  background: var(--c-panel-alt);
  border-top: 1px solid var(--c-border);
  font-size: var(--fs-small);
  white-space: nowrap;
}
.zoom input {
  width: 46px;
  height: 18px;
  padding: 0 4px;
  text-align: right;
  font-size: var(--fs-small);
  margin-right: 2px;
}
.sep {
  width: 1px;
  height: 14px;
  background: var(--c-border-soft);
}
.msg {
  flex: 1;
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
}
.msg.error {
  color: var(--c-danger);
}
.hist {
  min-width: 90px;
  text-align: right;
}
.github {
  display: flex;
  align-items: center;
  color: var(--c-text-dim);
}
.github:hover {
  color: #fff;
}
</style>
