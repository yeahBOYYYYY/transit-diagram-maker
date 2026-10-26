<script setup lang="ts">
// Shows the chosen pictogram; clicking it opens the Icons panel, where every
// pictogram can be picked or dragged onto the canvas.
import { computed } from 'vue'
import { Engine } from '../../core/engine'
import { useEditor } from '../../stores/editor'
import { prefs } from '../../stores/prefs'

const props = defineProps<{ modelValue: string }>()

const store = useEditor()
const builtIn = Engine.iconLibrary()
const current = computed(() => [...builtIn, ...store.state.assets.icons].find((i) => i.name === props.modelValue) ?? builtIn[0])

function openPanel(): void {
  prefs.panels.properties = true
  store.state.dockTab = 'Icons'
}
</script>

<template>
  <button type="button" class="trigger" :title="`${current.label}. Click to choose another pictogram in the Icons panel`" data-testid="icon-picker" @click="openPanel">
    <svg viewBox="0 0 24 24" width="18" height="18" v-html="current.svg" />
    <span>{{ current.label }}</span>
    <span class="more dim">Choose…</span>
  </button>
</template>

<style scoped>
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
.more {
  font-size: var(--fs-small);
}
</style>
