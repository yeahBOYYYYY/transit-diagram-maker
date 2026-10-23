<script setup lang="ts">
// Modal window with a title bar, OK / Cancel buttons and Escape / Enter keys.
import { onBeforeUnmount, onMounted } from 'vue'

const props = withDefaults(defineProps<{ title: string; width?: number; okLabel?: string; cancel?: boolean }>(), {
  width: 460,
  okLabel: 'OK',
  cancel: true,
})
const emit = defineEmits<{ ok: []; cancel: [] }>()

function close(): void {
  if (props.cancel) emit('cancel')
  else emit('ok')
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape') {
    e.preventDefault()
    close()
  } else if (e.key === 'Enter' && (e.target as HTMLElement).tagName !== 'TEXTAREA' && !(e.target as HTMLElement).closest('.no-enter')) {
    e.preventDefault()
    emit('ok')
  }
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="backdrop" @pointerdown.self="close()">
    <div class="dialog" role="dialog" :aria-label="title" :style="{ width: width + 'px' }" data-testid="dialog">
      <header class="title">
        <span>{{ title }}</span>
        <button class="x" title="Close" @click="close()">✕</button>
      </header>
      <div class="content">
        <slot />
      </div>
      <footer class="actions">
        <slot name="extra" />
        <span class="spacer" />
        <button v-if="cancel" class="ps-btn" data-testid="dialog-cancel" @click="emit('cancel')">Cancel</button>
        <button class="ps-btn primary" data-testid="dialog-ok" @click="emit('ok')">{{ okLabel }}</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
}
.dialog {
  max-height: 86vh;
  display: flex;
  flex-direction: column;
  background: var(--c-panel);
  border: 1px solid #151515;
  border-radius: 4px;
  box-shadow: 0 18px 50px rgba(0, 0, 0, 0.6);
}
.title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 30px;
  padding: 0 4px 0 12px;
  background: var(--c-panel-header);
  border-bottom: 1px solid var(--c-border);
  font-weight: 600;
  color: var(--c-text-bright);
  border-radius: 4px 4px 0 0;
}
.x {
  width: 26px;
  height: 24px;
  border: 0;
  background: transparent;
  color: var(--c-text-dim);
  cursor: pointer;
}
.x:hover {
  color: #fff;
}
.content {
  padding: 14px 16px;
  overflow: auto;
}
.actions {
  display: flex;
  gap: 8px;
  padding: 10px 16px 14px;
}
.spacer {
  flex: 1;
}
</style>
