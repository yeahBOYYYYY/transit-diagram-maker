<script setup lang="ts" generic="T extends string | number | boolean">
// Row of mutually exclusive buttons (text or icon).
import UiIcon from './UiIcon.vue'

defineProps<{ modelValue: T; options: { value: T; label?: string; icon?: string; title?: string }[] }>()
const emit = defineEmits<{ 'update:modelValue': [T] }>()
</script>

<template>
  <div class="segmented" role="radiogroup">
    <button
      v-for="o in options"
      :key="String(o.value)"
      type="button"
      role="radio"
      :aria-checked="o.value === modelValue"
      :class="{ on: o.value === modelValue }"
      :title="o.title ?? o.label"
      @click="emit('update:modelValue', o.value)"
    >
      <UiIcon v-if="o.icon" :name="o.icon" :size="16" />
      <span v-if="o.label && !o.icon">{{ o.label }}</span>
    </button>
  </div>
</template>

<style scoped>
.segmented {
  display: inline-flex;
  border: 1px solid var(--c-input-border);
  border-radius: var(--radius);
  overflow: hidden;
}
button {
  height: 20px;
  min-width: 24px;
  padding: 0 8px;
  border: 0;
  background: #3e3e3e;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
button + button {
  border-left: 1px solid var(--c-input-border);
}
button:hover {
  background: #4a4a4a;
}
button.on {
  background: #5a5a5a;
  color: #fff;
  box-shadow: inset 0 -2px 0 var(--c-accent);
}
</style>
