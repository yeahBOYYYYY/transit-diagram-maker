<script setup lang="ts">
// Docked panel with a title header and a collapse toggle.
import { ref } from 'vue'

defineProps<{ title: string; grow?: boolean }>()
const collapsed = ref(false)
</script>

<template>
  <section class="panel" :class="{ collapsed, grow }" :data-panel="title">
    <header class="head" @dblclick="collapsed = !collapsed">
      <div class="tab active">{{ title }}</div>
      <div class="spacer" />
      <button class="collapse" :title="collapsed ? 'Expand' : 'Collapse'" @click="collapsed = !collapsed">
        {{ collapsed ? '▸' : '▾' }}
      </button>
    </header>
    <div v-show="!collapsed" class="body">
      <slot />
    </div>
    <footer v-if="$slots.footer && !collapsed" class="foot">
      <slot name="footer" />
    </footer>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  background: var(--c-panel);
  border-bottom: 1px solid var(--c-border);
  min-height: 0;
}
/* A collapsed panel shrinks to its header even when it has a fixed height. */
.panel.collapsed {
  height: auto !important;
}
.panel.grow:not(.collapsed) {
  flex: 1;
}
.head {
  display: flex;
  align-items: stretch;
  height: 26px;
  background: var(--c-panel-header);
  border-bottom: 1px solid var(--c-border);
}
.tab {
  display: flex;
  align-items: center;
  padding: 0 12px;
  color: var(--c-text-dim);
  border-right: 1px solid var(--c-border);
}
.tab.active {
  background: var(--c-panel);
  color: var(--c-text-bright);
  font-weight: 600;
  margin-bottom: -1px;
}
.spacer {
  flex: 1;
}
.collapse {
  border: 0;
  background: transparent;
  color: var(--c-text-dim);
  cursor: pointer;
  width: 26px;
}
.collapse:hover {
  color: #fff;
}
.body {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
.foot {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 2px;
  height: 28px;
  padding: 0 6px;
  border-top: 1px solid var(--c-border);
  background: var(--c-panel);
}
</style>
