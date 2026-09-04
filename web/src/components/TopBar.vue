<script setup lang="ts">
import { computed } from 'vue';
import { engineBridge } from '../engineBridge';
import { EditorStateResponse } from '../types';

const props = defineProps<{
  state: EditorStateResponse | null;
}>();

const canUndo = computed(() => props.state?.can_undo ?? false);
const canRedo = computed(() => props.state?.can_redo ?? false);
const collidingCount = computed(() => props.state?.colliding_text_count ?? 0);

function onRecenter() {
  engineBridge.dispatch({ type: 'RecenterView' });
}

function onUndo() {
  engineBridge.dispatch({ type: 'Undo' });
}

function onRedo() {
  engineBridge.dispatch({ type: 'Redo' });
}

function onExportSvg() {
  const resp = engineBridge.dispatch({ type: 'ExportSvg' });
  if (resp.export_payload) {
    downloadFile(resp.export_payload.content, resp.export_payload.filename, 'image/svg+xml');
  }
}

function onSaveDiagram() {
  const resp = engineBridge.dispatch({ type: 'ExportTransitDocument' });
  if (resp.export_payload) {
    downloadFile(resp.export_payload.content, resp.export_payload.filename, 'application/json');
  }
}

function onLoadDiagramClick() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.transit,.json';
  input.onchange = async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (file) {
      const text = await file.text();
      engineBridge.dispatch({ type: 'LoadTransitDocument', payload: text });
    }
  };
  input.click();
}

function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
</script>

<template>
  <header class="top-bar">
    <div class="brand-section">
      <svg class="brand-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="6" cy="18" r="3" />
        <circle cx="18" cy="6" r="3" />
        <path d="M6 15V9a3 3 0 0 1 3-3h6" />
      </svg>
      <span class="brand-title">Transit Diagram Studio</span>
      <span class="brand-badge">Standalone WASM</span>
    </div>

    <div class="actions-section">
      <button class="action-btn" :disabled="!canUndo" @click="onUndo" title="Undo (Ctrl+Z)">
        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none">
          <path d="M3 7v6h6" />
          <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" />
        </svg>
        <span>Undo</span>
      </button>

      <button class="action-btn" :disabled="!canRedo" @click="onRedo" title="Redo (Ctrl+Shift+Z)">
        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none">
          <path d="M21 7v6h-6" />
          <path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3L21 13" />
        </svg>
        <span>Redo</span>
      </button>

      <div class="separator"></div>

      <button class="action-btn" @click="onRecenter" title="Recenter View to Content">
        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none">
          <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" />
        </svg>
        <span>Recenter</span>
      </button>

      <div v-if="collidingCount > 0" class="collision-pill" title="Real-time BVH detected overlapping labels">
        <span class="warning-icon">⚠</span>
        <span>{{ collidingCount }} label collision{{ collidingCount > 1 ? 's' : '' }}</span>
      </div>
    </div>

    <div class="export-section">
      <button class="action-btn secondary" @click="onLoadDiagramClick" title="Open .transit file">
        <span>Open</span>
      </button>
      <button class="action-btn secondary" @click="onSaveDiagram" title="Save lossless .transit state">
        <span>Save (.transit)</span>
      </button>
      <button class="action-btn primary" @click="onExportSvg" title="Export clean optimized SVG vector map">
        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
        <span>Export SVG</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--topbar-height);
  background-color: var(--bg-panel);
  border-bottom: 1px solid var(--border-color);
  padding: 0 16px;
  z-index: 100;
}

.brand-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.brand-icon {
  width: 20px;
  height: 20px;
  color: var(--accent-secondary);
}

.brand-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-main);
  letter-spacing: -0.2px;
}

.brand-badge {
  font-size: 10px;
  font-weight: 600;
  background: var(--bg-subpanel);
  color: var(--accent-primary);
  border: 1px solid var(--border-active);
  padding: 2px 6px;
  border-radius: 10px;
}

.actions-section {
  display: flex;
  align-items: center;
  gap: 6px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background-color: var(--bg-subpanel);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-main);
  font-size: 12px;
}

.action-btn:hover:not(:disabled) {
  background-color: var(--border-color);
  border-color: var(--border-active);
}

.action-btn.primary {
  background-color: var(--accent-primary);
  border-color: var(--accent-primary);
  color: #ffffff;
  font-weight: 500;
}

.action-btn.primary:hover {
  background-color: var(--accent-primary-hover);
}

.separator {
  width: 1px;
  height: 18px;
  background-color: var(--border-color);
  margin: 0 4px;
}

.collision-pill {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  background: rgba(255, 69, 58, 0.15);
  border: 1px solid rgba(255, 69, 58, 0.35);
  border-radius: 12px;
  color: var(--danger-color);
  font-size: 11px;
  font-weight: 500;
  animation: pulse 2s infinite ease-in-out;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.export-section {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
