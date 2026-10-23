<script setup lang="ts">
// Swatch + hex input. The native picker opens when the swatch is clicked.
import { ref, watch } from 'vue'

const props = defineProps<{ modelValue: string; label?: string; title?: string }>()
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
</script>

<template>
  <label class="color-field" :title="title">
    <span v-if="label" class="dim">{{ label }}</span>
    <span class="swatch" :style="{ background: modelValue }">
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
.hex {
  width: 68px;
  height: 22px;
  padding: 0 5px;
  font-family: ui-monospace, Consolas, monospace;
  font-size: var(--fs-small);
}
</style>
