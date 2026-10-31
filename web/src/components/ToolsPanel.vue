<script setup lang="ts">
// Vertical tool strip: one button per tool, like Inkscape and GIMP. Pictograms
// are added from the Icons panel, so they have no tool of their own.
import { computed } from 'vue'
import UiIcon from './ui/UiIcon.vue'
import { ACTIONS } from '../core/actions'
import { displayCombo } from '../core/shortcuts'
import { useEditor, type ToolId } from '../stores/editor'
import { defaultPrefs, prefs } from '../stores/prefs'
import { runAction } from '../stores/commands'
import type { ActionId } from '../core/actions'

const store = useEditor()

const TOOLS: ToolId[] = ['select', 'hand', 'pen', 'eraser', 'station', 'text', 'shape']

function title(id: ActionId): string {
  const key = displayCombo(prefs.shortcuts[id] ?? '')
  return key ? `${ACTIONS[id].label} (${key})` : ACTIONS[id].label
}

const activeColor = computed(() => prefs.line.color)
const defaults = defaultPrefs().line
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
      :title="title(`tool.${t}`)"
      :data-tool="t"
      @click="store.setTool(t)"
    >
      <UiIcon :name="t" :size="20" />
    </button>
    <div class="sep" />
    <!-- Like the foreground and background colors of Photoshop: the line
         color on top, the inner color (hollow and dashed lines) below. -->
    <div class="swatches" data-testid="tool-colors">
      <label class="fg" data-swatch title="Line color (click to choose)" :style="{ background: activeColor }">
        <input type="color" data-testid="tool-color-line" :value="prefs.line.color" @input="prefs.line.color = ($event.target as HTMLInputElement).value" />
      </label>
      <label class="bg" data-swatch title="Inner color of hollow and dashed lines (click to choose)" :style="{ background: prefs.line.inner_color }">
        <input type="color" data-testid="tool-color-inner" :value="prefs.line.inner_color" @input="prefs.line.inner_color = ($event.target as HTMLInputElement).value" />
      </label>
      <button type="button" class="swap" data-testid="tool-colors-swap" :title="title('color.swap')" @click="runAction(store, 'color.swap')">
        <svg viewBox="0 0 10 10" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.2">
          <!-- One arrow points left at the line color, the other down at the inner color. -->
          <path d="M2.5 2H6a2 2 0 0 1 2 2v3.5" />
          <path d="M4 0.5 2.2 2 4 3.5M9.5 6 8 7.8 6.5 6" />
        </svg>
      </button>
      <button type="button" class="reset" data-testid="tool-colors-reset" :title="title('color.reset')" @click="runAction(store, 'color.reset')">
        <span class="mini-bg" :style="{ background: defaults.inner_color }" /><span class="mini-fg" :style="{ background: defaults.color }" />
      </button>
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
  cursor: pointer;
}
.fg input,
.bg input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}
.swap,
.reset {
  position: absolute;
  width: 11px;
  height: 11px;
  padding: 0;
  border: 0;
  background: transparent;
  color: #cfcfcf;
  cursor: pointer;
}
.swap:hover,
.reset:hover {
  color: #fff;
}
.swap {
  right: 0;
  top: 0;
}
.reset {
  left: 0;
  bottom: 0;
}
.mini-fg,
.mini-bg {
  position: absolute;
  width: 6px;
  height: 6px;
  border: 1px solid #cfcfcf;
}
.mini-fg {
  left: 0;
  top: 0;
}
.mini-bg {
  right: 0;
  bottom: 0;
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
