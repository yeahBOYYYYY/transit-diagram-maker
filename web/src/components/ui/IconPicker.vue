<script setup lang="ts">
// Dropdown grid with the pictogram library: built-in pictograms, the icons
// uploaded into the document, and a button to upload another SVG.
import { computed, onBeforeUnmount, ref } from 'vue'
import { Engine } from '../../core/engine'
import { useEditor } from '../../stores/editor'
import { uploadIcon } from '../../stores/commands'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [string] }>()

const store = useEditor()
const builtIn = Engine.iconLibrary()
const library = computed(() => [...builtIn, ...store.state.assets.icons])
const open = ref(false)
const current = computed(() => library.value.find((i) => i.name === props.modelValue) ?? builtIn[0])

async function upload(): Promise<void> {
  open.value = false
  const name = await uploadIcon(store)
  if (name) emit('update:modelValue', name)
}
const root = ref<HTMLElement | null>(null)

function choose(name: string): void {
  emit('update:modelValue', name)
  open.value = false
}

function onDocDown(e: PointerEvent): void {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false
}
document.addEventListener('pointerdown', onDocDown)
onBeforeUnmount(() => document.removeEventListener('pointerdown', onDocDown))
</script>

<template>
  <div ref="root" class="icon-picker">
    <button type="button" class="trigger" :title="current.label" data-testid="icon-picker" @click="open = !open">
      <svg viewBox="0 0 24 24" width="18" height="18" v-html="current.svg" />
      <span>{{ current.label }}</span>
      <span class="chev">▾</span>
    </button>
    <div v-if="open" class="menu">
      <button
        v-for="i in library"
        :key="i.name"
        type="button"
        class="cell"
        :class="{ on: i.name === modelValue }"
        :title="i.label"
        :data-icon="i.name"
        @click="choose(i.name)"
      >
        <svg viewBox="0 0 24 24" width="28" height="28" v-html="i.svg" />
        <span>{{ i.label }}</span>
      </button>
      <button type="button" class="cell upload" title="Add your own SVG icon to this document" data-testid="upload-icon" @click="upload">
        <svg viewBox="0 0 24 24" width="28" height="28">
          <rect x="1" y="1" width="22" height="22" rx="5" fill="none" stroke="currentColor" stroke-dasharray="3 2" />
          <path d="M12 7v10M7 12h10" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
        <span>Upload SVG…</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.icon-picker {
  position: relative;
}
.trigger {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 6px;
  border: 1px solid var(--c-input-border);
  border-radius: var(--radius);
  background: var(--c-input);
  cursor: pointer;
}
.chev {
  color: var(--c-text-dim);
}
.menu {
  position: absolute;
  top: 28px;
  left: 0;
  z-index: 50;
  display: grid;
  grid-template-columns: repeat(4, 76px);
  gap: 4px;
  padding: 6px;
  background: var(--c-panel);
  border: 1px solid var(--c-border);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  border-radius: 4px;
}
.cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 6px 2px;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: transparent;
  cursor: pointer;
  font-size: 10px;
  color: var(--c-text-dim);
  text-align: center;
}
.cell:hover {
  background: var(--c-hover);
  color: var(--c-text);
}
.cell.on {
  border-color: var(--c-accent);
  color: var(--c-text-bright);
}
</style>
