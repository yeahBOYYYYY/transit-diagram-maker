<script setup lang="ts">
// Font family dropdown: built-in fonts, the document's uploaded fonts and an
// entry to upload another font file.
import { computed } from 'vue'
import { fontChoices, fontLabel } from '../../core/fonts'
import { useEditor } from '../../stores/editor'
import { uploadFont } from '../../stores/commands'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [string] }>()
const store = useEditor()
const UPLOAD = '\u0000upload'

const choices = computed(() => {
  const list = fontChoices(store.state.assets.fonts)
  // Keep an unknown current value selectable (e.g. a font removed meanwhile).
  return list.includes(props.modelValue) ? list : [props.modelValue, ...list]
})

async function onChange(e: Event): Promise<void> {
  const el = e.target as HTMLSelectElement
  if (el.value !== UPLOAD) {
    emit('update:modelValue', el.value)
    return
  }
  el.value = props.modelValue
  const family = await uploadFont(store)
  if (family) emit('update:modelValue', family)
}
</script>

<template>
  <select :value="modelValue" data-testid="font-select" @change="onChange">
    <option v-for="f in choices" :key="f" :value="f" :style="{ fontFamily: f }">{{ fontLabel(f) }}</option>
    <option :value="UPLOAD">Upload font…</option>
  </select>
</template>
