<script setup lang="ts">
import { ToolType } from '../types';
import { engineBridge } from '../engineBridge';

const props = defineProps<{
  activeTool: ToolType;
}>();

const tools: Array<{ id: ToolType; name: string; shortcut: string; icon: string }> = [
  {
    id: 'Select',
    name: 'Selection Tool',
    shortcut: 'V',
    icon: 'M4 4l7.07 17 2.51-7.39L21 11.07 4 4z',
  },
  {
    id: 'Pen',
    name: 'Transit Line Pen Tool (Strict Grid Channel Locking & Auto-Rounding)',
    shortcut: 'P',
    icon: 'M12 19l7-7 3 3-7 7-3-3z M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z',
  },
  {
    id: 'Station',
    name: 'Station Tool (Dot or Tick; Dot stretches across parallel lines)',
    shortcut: 'S',
    icon: 'M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 14a4 4 0 1 1 4-4 4 4 0 0 1-4 4z',
  },
  {
    id: 'Erase',
    name: 'Erase Tool (Deletes Stations or Cuts Line Segments)',
    shortcut: 'E',
    icon: 'M18 20H6a2 2 0 0 1-2-2V9l7-7 11 11-4 7z M12 6l-6 6',
  },
  {
    id: 'Text',
    name: 'Text Label Tool',
    shortcut: 'T',
    icon: 'M4 7V4h16v3 M9 20h6 M12 4v16',
  },
];

function setTool(tool: ToolType) {
  engineBridge.dispatch({ type: 'SelectTool', payload: tool });
}
</script>

<template>
  <aside class="toolbar-pane">
    <div class="tool-list">
      <button
        v-for="t in tools"
        :key="t.id"
        class="tool-btn"
        :class="{ active: activeTool === t.id }"
        :title="`${t.name} [${t.shortcut}]`"
        @click="setTool(t.id)"
      >
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="t.icon" />
        </svg>
        <span class="hotkey-badge">{{ t.shortcut }}</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.toolbar-pane {
  width: var(--toolbar-width);
  background-color: var(--bg-panel);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0;
  z-index: 50;
}

.tool-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tool-btn {
  width: 38px;
  height: 38px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  color: var(--text-muted);
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.tool-btn:hover {
  background-color: var(--bg-subpanel);
  color: var(--text-main);
  border-color: var(--border-color);
}

.tool-btn.active {
  background-color: rgba(0, 122, 255, 0.15);
  color: var(--accent-primary);
  border-color: rgba(0, 122, 255, 0.4);
}

.hotkey-badge {
  position: absolute;
  bottom: 2px;
  right: 3px;
  font-size: 8px;
  font-family: var(--font-mono);
  font-weight: 600;
  opacity: 0.6;
}
</style>
