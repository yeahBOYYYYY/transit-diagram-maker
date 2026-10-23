<script setup lang="ts">
// Numeric input with a "scrubby" label: drag the label horizontally to change
// the value, like in professional design tools.
import { ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue: number
    label?: string
    min?: number
    max?: number
    step?: number
    unit?: string
    width?: number
    precision?: number
    title?: string
  }>(),
  { step: 1, width: 52, precision: 2 },
)
const emit = defineEmits<{ 'update:modelValue': [number]; commit: [number] }>()

const text = ref(format(props.modelValue))
watch(
  () => props.modelValue,
  (v) => (text.value = format(v)),
)

function format(v: number): string {
  if (!Number.isFinite(v)) return ''
  return String(Math.round(v * 10 ** props.precision) / 10 ** props.precision)
}

function clamp(v: number): number {
  let out = v
  if (props.min !== undefined) out = Math.max(props.min, out)
  if (props.max !== undefined) out = Math.min(props.max, out)
  return out
}

function commitText(): void {
  const v = Number(text.value)
  if (text.value.trim() === '' || !Number.isFinite(v)) {
    text.value = format(props.modelValue)
    return
  }
  const c = clamp(v)
  text.value = format(c)
  if (c !== props.modelValue) {
    emit('update:modelValue', c)
    emit('commit', c)
  }
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Enter') {
    commitText()
    ;(e.target as HTMLInputElement).blur()
  } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
    e.preventDefault()
    const mult = e.shiftKey ? 10 : 1
    const v = clamp(props.modelValue + (e.key === 'ArrowUp' ? 1 : -1) * props.step * mult)
    emit('update:modelValue', v)
    emit('commit', v)
  } else if (e.key === 'Escape') {
    text.value = format(props.modelValue)
    ;(e.target as HTMLInputElement).blur()
  }
}

let scrub: { x: number; v: number } | null = null
function onScrubStart(e: PointerEvent): void {
  scrub = { x: e.clientX, v: props.modelValue }
  ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
}
function onScrubMove(e: PointerEvent): void {
  if (!scrub) return
  const dx = e.clientX - scrub.x
  const mult = e.shiftKey ? 10 : 1
  const v = clamp(scrub.v + Math.round(dx / 3) * props.step * mult)
  if (v !== props.modelValue) emit('update:modelValue', Number(v.toFixed(6)))
}
function onScrubEnd(): void {
  if (scrub && scrub.v !== props.modelValue) emit('commit', props.modelValue)
  scrub = null
}
</script>

<template>
  <label class="num-field" :title="title">
    <span
      v-if="label"
      class="scrub"
      @pointerdown.prevent="onScrubStart"
      @pointermove="onScrubMove"
      @pointerup="onScrubEnd"
      @pointercancel="onScrubEnd"
      >{{ label }}</span
    >
    <input
      v-model="text"
      type="text"
      inputmode="decimal"
      :style="{ width: width + 'px' }"
      @blur="commitText"
      @keydown.stop="onKey"
    />
    <span v-if="unit" class="unit">{{ unit }}</span>
  </label>
</template>

<style scoped>
.num-field {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}
.scrub {
  cursor: ew-resize;
  color: var(--c-text-dim);
}
.scrub:hover {
  color: var(--c-text);
}
input {
  height: 22px;
  padding: 0 5px;
  text-align: right;
}
.unit {
  color: var(--c-text-dim);
  font-size: var(--fs-small);
}
</style>
