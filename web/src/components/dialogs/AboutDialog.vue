<script setup lang="ts">
import ModalDialog from './ModalDialog.vue'
import { useEditor } from '../../stores/editor'

const store = useEditor()
</script>

<template>
  <ModalDialog title="About Transit Diagram Maker" :width="420" ok-label="Close" :cancel="false" @ok="store.state.dialog = null">
    <div class="about">
      <svg viewBox="0 0 32 32" width="64" height="64">
        <rect width="32" height="32" rx="6" fill="#0d2238" />
        <text x="16" y="21.5" text-anchor="middle" font-size="14" font-weight="700" fill="#31a8ff" font-family="Segoe UI, Arial">Td</text>
      </svg>
      <div>
        <h2>Transit Diagram Maker</h2>
        <p class="dim">Version 0.1.0</p>
        <p>A grid based vector editor for topologically sound metro and transit maps. Geometry, history and SVG rendering run in Rust compiled to WebAssembly.</p>
      </div>
    </div>
  </ModalDialog>
</template>

<style scoped>
.about {
  display: flex;
  gap: 16px;
  line-height: 1.5;
}
h2 {
  margin: 0;
  font-size: 16px;
  color: var(--c-text-bright);
}
p {
  margin: 4px 0;
}
</style>
