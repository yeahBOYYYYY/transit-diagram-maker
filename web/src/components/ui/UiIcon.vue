<script setup lang="ts">
// Monochrome UI glyphs drawn on a 20×20 grid with the current text color.
defineProps<{ name: string; size?: number }>()

const PATHS: Record<string, string> = {
  select: '<path d="M5 2.5v13.2l3.4-3.2 2.3 5.2 2.3-1-2.3-5.1h4.8z" fill="currentColor"/>',
  hand:
    '<path d="M6.2 10.5V5.3a1.3 1.3 0 0 1 2.6 0v4.2V3.9a1.3 1.3 0 0 1 2.6 0v5.6V4.7a1.3 1.3 0 0 1 2.6 0v5.5-3a1.3 1.3 0 0 1 2.6 0v5.6c0 3.4-2.4 5.7-5.6 5.7-2.3 0-3.6-.9-4.8-2.6L3.6 11.7a1.3 1.3 0 0 1 2-1.6z" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/>',
  pen:
    '<path d="M3 17l1.2-4.6L13.6 3l3.4 3.4-9.4 9.4z" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/><path d="M11.8 4.8l3.4 3.4M4.2 12.4l3.4 3.4" stroke="currentColor" stroke-width="1.4"/><path d="M2 18.5h7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>',
  eraser:
    '<path d="M7.2 16.5L2.8 12.1a1.2 1.2 0 0 1 0-1.7l7.6-7.6a1.2 1.2 0 0 1 1.7 0l5.1 5.1a1.2 1.2 0 0 1 0 1.7l-7.1 7.1z" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/><path d="M6.5 6.6l6.8 6.8" stroke="currentColor" stroke-width="1.4"/><path d="M7 16.5h10" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>',
  station:
    '<path d="M1.5 10h17" stroke="currentColor" stroke-width="3.2" stroke-linecap="round"/><circle cx="10" cy="10" r="4.4" fill="#383838" stroke="currentColor" stroke-width="1.8"/>',
  text: '<path d="M4 4.5h12v2.4M10 4.5v11M7.4 15.5h5.2M4 4.5v2.4" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="square"/>',
  icon:
    '<rect x="3" y="3" width="14" height="14" rx="3" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M6.4 13.6V6.4h2.7a2.3 2.3 0 0 1 0 4.6H6.4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"/><circle cx="13.2" cy="12.8" r="1.4" fill="currentColor"/>',
  eye:
    '<path d="M1.8 10S5 4.8 10 4.8 18.2 10 18.2 10 15 15.2 10 15.2 1.8 10 1.8 10z" fill="none" stroke="currentColor" stroke-width="1.4"/><circle cx="10" cy="10" r="2.6" fill="currentColor"/>',
  'eye-off':
    '<path d="M3.2 12.5C2.3 11.4 1.8 10 1.8 10S5 4.8 10 4.8c1.1 0 2 .2 2.9.6M16.6 7.4c1 1.2 1.6 2.6 1.6 2.6S15 15.2 10 15.2c-1 0-2-.2-2.8-.6M3 17L17 3" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>',
  lock:
    '<rect x="4.5" y="9" width="11" height="8" rx="1.5" fill="currentColor"/><path d="M7 9V6.6a3 3 0 0 1 6 0V9" fill="none" stroke="currentColor" stroke-width="1.6"/>',
  unlock:
    '<rect x="4.5" y="9" width="11" height="8" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M7 9V6.6a3 3 0 0 1 5.8-1" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>',
  folder: '<path d="M2.5 5.5a1 1 0 0 1 1-1h4l1.6 1.8h7.4a1 1 0 0 1 1 1v8.2a1 1 0 0 1-1 1h-13a1 1 0 0 1-1-1z" fill="currentColor" opacity=".85"/>',
  'folder-plus':
    '<path d="M2.5 5.5a1 1 0 0 1 1-1h4l1.6 1.8h7.4a1 1 0 0 1 1 1v8.2a1 1 0 0 1-1 1h-13a1 1 0 0 1-1-1z" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M10 8.6v5M7.5 11.1h5" stroke="currentColor" stroke-width="1.4"/>',
  group:
    '<rect x="2.5" y="2.5" width="9" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1.3"/><rect x="8.5" y="8.5" width="9" height="9" rx="1" fill="currentColor" opacity=".8"/>',
  ungroup:
    '<rect x="2.5" y="2.5" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.3"/><rect x="10.5" y="10.5" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.3"/>',
  trash:
    '<path d="M3.5 5.5h13M8 5.5V3.8h4v1.7M5.2 5.5l.9 11h7.8l.9-11" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/><path d="M8.3 8.5v5.5M11.7 8.5v5.5" stroke="currentColor" stroke-width="1.2"/>',
  up: '<path d="M5 12l5-5 5 5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>',
  down: '<path d="M5 8l5 5 5-5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>',
  caret: '<path d="M7 5l6 5-6 5z" fill="currentColor"/>',
  'caret-down': '<path d="M5 7l5 6 5-6z" fill="currentColor"/>',
  'rotate-cw':
    '<path d="M15.5 9.5a5.5 5.5 0 1 1-2.2-4.4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/><path d="M14.4 2.4l.2 3.6-3.6.3" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>',
  'rotate-ccw':
    '<path d="M4.5 9.5a5.5 5.5 0 1 0 2.2-4.4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/><path d="M5.6 2.4l-.2 3.6 3.6.3" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>',
  move:
    '<path d="M10 2v16M2 10h16M10 2l-2.5 2.5M10 2l2.5 2.5M10 18l-2.5-2.5M10 18l2.5-2.5M2 10l2.5-2.5M2 10l2.5 2.5M18 10l-2.5-2.5M18 10l-2.5 2.5" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>',
  line: '<path d="M2.5 14.5h6l5-5h4" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>',
  close: '<path d="M5 5l10 10M15 5L5 15" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>',
  bold: '<path d="M6 4h5a3 3 0 0 1 0 6H6zm0 6h6a3 3 0 0 1 0 6H6z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>',
  italic: '<path d="M8.5 4h6M5.5 16h6M11.5 4l-3 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>',
  'align-start': '<path d="M3 5h14M3 9h9M3 13h14M3 17h9" stroke="currentColor" stroke-width="1.5"/>',
  'align-middle': '<path d="M3 5h14M5.5 9h9M3 13h14M5.5 17h9" stroke="currentColor" stroke-width="1.5"/>',
  'align-end': '<path d="M3 5h14M8 9h9M3 13h14M8 17h9" stroke="currentColor" stroke-width="1.5"/>',
  zoom: '<circle cx="8.5" cy="8.5" r="5.5" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="M12.5 12.5l5 5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>',
}
</script>

<template>
  <svg
    class="ui-icon"
    :width="size ?? 18"
    :height="size ?? 18"
    viewBox="0 0 20 20"
    aria-hidden="true"
    v-html="PATHS[name] ?? ''"
  />
</template>

<style scoped>
.ui-icon {
  display: block;
  flex: none;
}
</style>
