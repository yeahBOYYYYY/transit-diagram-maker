<script setup lang="ts">
// Vertical tool strip. The pen and the eraser share a slot: a long press (or
// right click) on the slot opens a flyout to choose between them.
import { computed, onBeforeUnmount, ref } from 'vue'
import UiIcon from './ui/UiIcon.vue'
import { ACTIONS } from '../core/actions'
import { displayCombo } from '../core/shortcuts'
import { useEditor, type ToolId } from '../stores/editor'
import { prefs } from '../stores/prefs'

const store = useEditor()

interface Slot {
  tools: ToolId[]
}

const slots: Slot[] = [
  { tools: ['select'] },
  { tools: ['hand'] },
  { tools: ['pen', 'eraser'] },
  { tools: ['station'] },
  { tools: ['text'] },
  { tools: ['icon'] },
]

const LABELS: Record<ToolId, string> = {
  select: ACTIONS['tool.select'].label,
  hand: ACTIONS['tool.hand'].label,
  pen: ACTIONS['tool.pen'].label,
  eraser: ACTIONS['tool.eraser'].label,
  station: ACTIONS['tool.station'].label,
  text: ACTIONS['tool.text'].label,
  icon: ACTIONS['tool.icon'].label,
}

const flyout = ref<number | null>(null)
let pressTimer: ReturnType<typeof setTimeout> | null = null
let longPressed = false

function shown(slot: Slot): ToolId {
  return slot.tools.length > 1 ? store.state.penSlot : slot.tools[0]
}

function key(tool: ToolId): string {
  return displayCombo(prefs.shortcuts[`tool.${tool}` as keyof typeof prefs.shortcuts] ?? '')
}

function onDown(i: number, slot: Slot): void {
  longPressed = false
  if (slot.tools.length < 2) return
  pressTimer = setTimeout(() => {
    longPressed = true
    flyout.value = i
  }, prefs.longPressMs)
}

function onUp(slot: Slot): void {
  if (pressTimer) clearTimeout(pressTimer)
  pressTimer = null
  if (!longPressed) store.setTool(shown(slot))
}

function cancelPress(): void {
  if (pressTimer) clearTimeout(pressTimer)
  pressTimer = null
}

function choose(tool: ToolId): void {
  store.setTool(tool)
  flyout.value = null
}

function onDocDown(e: PointerEvent): void {
  if (!(e.target as HTMLElement).closest?.('.flyout')) flyout.value = null
}
document.addEventListener('pointerdown', onDocDown)
onBeforeUnmount(() => document.removeEventListener('pointerdown', onDocDown))

const activeColor = computed(() => prefs.line.color)
</script>

<template>
  <aside class="tools" data-testid="tools-panel">
    <div class="grip" />
    <div v-for="(slot, i) in slots" :key="i" class="slot">
      <button
        type="button"
        class="tool"
        :class="{ on: slot.tools.includes(store.state.tool) }"
        :title="`${LABELS[shown(slot)]} (${key(shown(slot))})`"
        :data-tool="shown(slot)"
        @pointerdown.stop="onDown(i, slot)"
        @pointerup="onUp(slot)"
        @pointerleave="cancelPress"
        @contextmenu.prevent="slot.tools.length > 1 && (flyout = i)"
      >
        <UiIcon :name="shown(slot)" :size="20" />
        <span v-if="slot.tools.length > 1" class="corner" />
      </button>
      <div v-if="flyout === i" class="flyout" data-testid="tool-flyout">
        <button
          v-for="t in slot.tools"
          :key="t"
          type="button"
          class="fly-item"
          :class="{ on: store.state.tool === t }"
          :data-tool="t"
          @click="choose(t)"
        >
          <span class="mark">{{ store.state.tool === t ? '■' : '' }}</span>
          <UiIcon :name="t" :size="18" />
          <span class="name">{{ LABELS[t] }}</span>
          <span class="dim">{{ key(t) }}</span>
        </button>
      </div>
    </div>
    <div class="sep" />
    <div class="swatches" title="Line color (set in the pen options)">
      <div class="fg" :style="{ background: activeColor }" />
      <div class="bg" :style="{ background: prefs.line.inner_color }" />
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
.slot {
  position: relative;
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
.corner {
  position: absolute;
  right: 3px;
  bottom: 3px;
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-bottom: 4px solid #bbb;
}
.flyout {
  position: absolute;
  left: 40px;
  top: 0;
  z-index: 60;
  min-width: 200px;
  background: #3a3a3a;
  border: 1px solid var(--c-border);
  box-shadow: 0 8px 22px rgba(0, 0, 0, 0.5);
  padding: 3px 0;
}
.fly-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 28px;
  padding: 0 10px 0 6px;
  border: 0;
  background: transparent;
  cursor: pointer;
  text-align: left;
}
.fly-item:hover {
  background: var(--c-accent-dim);
  color: #fff;
}
.mark {
  width: 10px;
  font-size: 8px;
  color: var(--c-text);
}
.name {
  flex: 1;
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
