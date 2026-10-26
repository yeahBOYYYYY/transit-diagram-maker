<script setup lang="ts">
// Link to the project's GitHub repository: the logo, with a caption unless
// `caption` is empty (the status bar shows only the logo).
import UiIcon from './UiIcon.vue'
import { APP } from '../../core/about'

withDefaults(defineProps<{ size?: number; caption?: string }>(), { size: 18, caption: 'View on GitHub' })
</script>

<template>
  <a class="github-link" :class="{ boxed: caption }" :href="APP.repoUrl" target="_blank" rel="noopener" :title="`${APP.name} on GitHub`">
    <UiIcon name="github" :size="size" />
    <span v-if="caption">{{ caption }}</span>
  </a>
</template>

<style scoped>
.github-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--c-text-dim);
  text-decoration: none;
}
.github-link:hover {
  color: #fff;
}
.boxed {
  padding: 5px 10px;
  border: 1px solid var(--c-border-soft);
  border-radius: 4px;
  background: var(--c-panel-alt);
}
.boxed:hover {
  border-color: #666;
}
</style>
