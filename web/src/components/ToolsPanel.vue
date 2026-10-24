<script setup lang="ts">
// Vertical tool strip: one button per tool, like Inkscape and GIMP. Pictograms
// are added from the Icons panel, so they have no tool of their own.
import { computed } from 'vue'
import UiIcon from './ui/UiIcon.vue'
import { ACTIONS } from '../core/actions'
import { displayCombo } from '../core/shortcuts'
import { useEditor, type ToolId } from '../stores/editor'
import { prefs } from '../stores/prefs'

const store = useEditor()

const TOOLS: ToolId[] = ['select', 'hand', 'pen', 'eraser', 'station', 'text']

function title(tool: ToolId): string {
  const id = `tool.${tool}` as const
  const key = displayCombo(prefs.shortcuts[id] ?? '')
  return key ? `${ACTIONS[id].label} (${key})` : ACTIONS[id].label
}

const activeColor = computed(() => prefs.line.color)
</script>

<template>
  <aside class="tools" data-testid="tools-panel">
    <div class="grip" />
    <button
      v-for="t in TOOLS"
      :key="t"
      type="button"
      class="tool"
      :class="{ on: store.state.tool === t }"
      :title="title(t)"
      :data-tool="t"
      @click="store.setTool(t)"
    >
      <UiIcon :name="t" :size="20" />
    </button>
    <div class="sep" />
    <div class="swatches" title="Line color and inner color (set in the pen options)">
      <div class="fg" data-swatch :style="{ background: activeColor }" />
      <div class="bg" data-swatch :style="{ background: prefs.line.inner_color }" />
    </div>
  </aside>
</template>

<style scoped>
.tools {
  width: 44px;
  background: var(--c-panel);
  border-right: 1px solid var(--c-border);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2px 0 8px;
  gap: 2px;
}
.grip {
  width: 100%;
  height: 10px;
  background: var(--c-panel-header);
  margin-bottom: 4px;
  border-bottom: 1px solid var(--c-border);
}
.tool {
  position: relative;
  width: 34px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: #cfcfcf;
  cursor: pointer;
}
.tool:hover {
  background: var(--c-hover);
  color: #fff;
}
.tool.on {
  background: #565656;
  color: #fff;
  border-color: #666;
}
.sep {
  width: 26px;
  height: 1px;
  background: var(--c-border-soft);
  margin: 6px 0;
}
.swatches {
  position: relative;
  width: 34px;
  height: 34px;
}
.fg,
.bg {
  position: absolute;
  width: 20px;
  height: 20px;
  border: 1px solid #000;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.4);
}
.fg {
  left: 3px;
  top: 3px;
  z-index: 1;
}
.bg {
  right: 3px;
  bottom: 3px;
}
</style>
