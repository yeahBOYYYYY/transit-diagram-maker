<script setup lang="ts">
// Swatch + hex input. The native picker opens when the swatch is clicked;
// the eyedropper button takes a color from the canvas instead.
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import HelpTip from './HelpTip.vue'
import UiIcon from './UiIcon.vue'
import { picking, startPicking, stopPicking } from '../../core/eyedropper'

const props = defineProps<{ modelValue: string; label?: string; title?: string; help?: string }>()
const emit = defineEmits<{ 'update:modelValue': [string] }>()
const text = ref(props.modelValue)
watch(
  () => props.modelValue,
  (v) => (text.value = v),
)

function normalize(v: string): string | null {
  const s = v.trim().replace(/^#?/, '#')
  if (/^#[0-9a-fA-F]{6}$/.test(s)) return s.toLowerCase()
  if (/^#[0-9a-fA-F]{3}$/.test(s)) {
    const [, r, g, b] = s
    return `#${r}${r}${g}${g}${b}${b}`.toLowerCase()
  }
  return null
}

function commitText(): void {
  const v = normalize(text.value)
  if (v) {
    if (v !== props.modelValue) emit('update:modelValue', v)
    text.value = v
  } else text.value = props.modelValue
}

const pickerValue = (v: string) => normalize(v) ?? '#000000'

function pick(hex: string): void {
  if (hex !== props.modelValue) emit('update:modelValue', hex)
  text.value = hex
}
const isPicking = computed(() => picking.value === pick)
function togglePicking(): void {
  if (isPicking.value) stopPicking()
  else startPicking(pick)
}
onBeforeUnmount(() => isPicking.value && stopPicking())
</script>

<template>
  <label class="color-field" :title="help ? undefined : title">
    <span v-if="label" class="dim">{{ label }}</span>
    <HelpTip v-if="help" :text="help" />
    <span class="swatch" data-swatch :style="{ background: modelValue }">
      <input
        type="color"
        :value="pickerValue(modelValue)"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      />
    </span>
    <input
      v-model="text"
      class="hex"
      type="text"
      maxlength="7"
      @blur="commitText"
      @keydown.stop="$event.key === 'Enter' && commitText()"
    />
    <button
      type="button"
      class="dropper"
      :class="{ on: isPicking }"
      data-eyedropper
      data-testid="eyedropper"
      title="Pick a color: click anything on the canvas or any color swatch (Escape cancels)"
      @click.prevent="togglePicking"
    >
      <UiIcon name="eyedropper" :size="14" />
    </button>
  </label>
</template>

<style scoped>
.color-field {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.swatch {
  position: relative;
  width: 22px;
  height: 22px;
  border-radius: var(--radius);
  border: 1px solid #000;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.25);
  overflow: hidden;
  cursor: pointer;
  flex: none;
}
.swatch input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
  width: 100%;
  height: 100%;
  border: 0;
  padding: 0;
}
.dropper {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: transparent;
  color: var(--c-text-dim);
  cursor: pointer;
  flex: none;
}
.dropper:hover {
  background: var(--c-hover);
  color: var(--c-text-bright);
}
.dropper.on {
  border-color: var(--c-accent);
  color: var(--c-text-bright);
}
.hex {
  width: 68px;
  height: 22px;
  padding: 0 5px;
  font-family: ui-monospace, Consolas, monospace;
  font-size: var(--fs-small);
}
</style>
