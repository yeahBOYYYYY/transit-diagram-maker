<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { engineBridge } from './engineBridge';
import { EditorStateResponse } from './types';
import TopBar from './components/TopBar.vue';
import Toolbar from './components/Toolbar.vue';
import Workspace from './components/Workspace.vue';
import PropertiesPanel from './components/PropertiesPanel.vue';

const isLoaded = ref(false);
const loadError = ref<string | null>(null);
const editorState = ref<EditorStateResponse | null>(null);

onMounted(async () => {
  try {
    engineBridge.subscribe(state => {
      editorState.value = state;
    });

    await engineBridge.init(window.innerWidth, window.innerHeight);
    isLoaded.value = true;
  } catch (err: any) {
    loadError.value = err?.message || 'Failed to initialize WASM engine';
  }
});
</script>

<template>
  <div class="app-container">
    <div v-if="!isLoaded" class="loading-overlay">
      <div v-if="loadError" class="error-box">
        <h3>Initialization Failed</h3>
        <p>{{ loadError }}</p>
      </div>
      <div v-else class="spinner-box">
        <div class="spinner"></div>
        <p>Initializing Transit Diagram Engine (WASM)...</p>
      </div>
    </div>

    <template v-else>
      <TopBar :state="editorState" />
      <div class="main-content">
        <Toolbar :active-tool="editorState?.active_tool ?? 'Select'" />
        <Workspace :state="editorState" />
        <PropertiesPanel :state="editorState" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.loading-overlay {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100vw;
  height: 100vh;
  background-color: var(--bg-app);
}

.spinner-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  color: var(--text-muted);
}

.spinner {
  width: 36px;
  height: 36px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-box {
  background: rgba(255, 69, 58, 0.1);
  border: 1px solid var(--danger-color);
  padding: 24px;
  border-radius: 8px;
  color: var(--danger-color);
  max-width: 400px;
}
</style>
