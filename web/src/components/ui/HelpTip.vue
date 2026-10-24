<script setup lang="ts">
// Info icon placed beside a property label; hovering or focusing it shows the
// property's help text. The bubble is teleported to <body> so scrolling panels
// never clip it.
import { computed, ref } from 'vue'

defineProps<{ text: string }>()

const el = ref<HTMLElement | null>(null)
const at = ref<{ x: number; y: number; above: boolean } | null>(null)
const WIDTH = 260

function show(): void {
  const r = el.value?.getBoundingClientRect()
  if (!r) return
  const above = r.bottom + 120 > window.innerHeight
  at.value = { x: r.left + r.width / 2, y: above ? r.top - 6 : r.bottom + 6, above }
}
function hide(): void {
  at.value = null
}

const bubble = computed(() => {
  const a = at.value
  if (!a) return {}
  const left = Math.min(Math.max(8, a.x - WIDTH / 2), window.innerWidth - WIDTH - 8)
  return { left: `${left}px`, top: `${a.y}px`, width: `${WIDTH}px`, transform: a.above ? 'translateY(-100%)' : 'none' }
})
</script>

<template>
  <span
    ref="el"
    class="help-tip"
    tabindex="0"
    role="img"
    :aria-label="text"
    data-testid="help-tip"
    @mouseenter="show"
    @mouseleave="hide"
    @focus="show"
    @blur="hide"
    @click.prevent.stop
    @pointerdown.stop
  >
    <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
      <circle cx="6" cy="6" r="5.3" fill="none" stroke="currentColor" stroke-width="1.1" />
      <circle cx="6" cy="3.6" r=".8" fill="currentColor" />
      <path d="M6 5.4v3.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    </svg>
    <Teleport to="body">
      <div v-if="at" class="help-bubble" role="tooltip" :style="bubble">{{ text }}</div>
    </Teleport>
  </span>
</template>

<style scoped>
.help-tip {
  display: inline-flex;
  align-items: center;
  flex: none;
  margin: 0 2px;
  color: #7d7d7d;
  cursor: help;
  outline: none;
  vertical-align: middle;
}
.help-tip:hover,
.help-tip:focus-visible {
  color: var(--c-accent);
}
.help-bubble {
  position: fixed;
  z-index: 1000;
  padding: 7px 9px;
  border: 1px solid #555;
  border-radius: 4px;
  background: #1f1f1f;
  color: #e6e6e6;
  font-size: 11.5px;
  line-height: 1.45;
  white-space: normal;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.5);
  pointer-events: none;
}
</style>
