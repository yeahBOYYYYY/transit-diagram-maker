<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue'
import MenuBar from './components/MenuBar.vue'
import OptionsBar from './components/OptionsBar.vue'
import ToolsPanel from './components/ToolsPanel.vue'
import CanvasView from './components/CanvasView.vue'
import StatusBar from './components/StatusBar.vue'
import LayersPanel from './components/panels/LayersPanel.vue'
import PropertiesPanel from './components/panels/PropertiesPanel.vue'
import DocumentSettingsDialog from './components/dialogs/DocumentSettingsDialog.vue'
import PreferencesDialog from './components/dialogs/PreferencesDialog.vue'
import ShortcutsDialog from './components/dialogs/ShortcutsDialog.vue'
import ReferenceDialog from './components/dialogs/ReferenceDialog.vue'
import AboutDialog from './components/dialogs/AboutDialog.vue'
import { useEditor } from './stores/editor'
import { prefs } from './stores/prefs'

const store = useEditor()
const s = store.state
const zoom = computed(() => Math.round(s.view.zoom * 100))
const showDock = computed(() => prefs.panels.properties || prefs.panels.layers)

function beforeUnload(e: BeforeUnloadEvent): void {
  if (s.dirty) e.preventDefault()
}
onMounted(() => window.addEventListener('beforeunload', beforeUnload))
onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))
</script>

<template>
  <div class="app" data-testid="app">
    <MenuBar />
    <OptionsBar v-if="prefs.panels.options" />
    <div class="workspace">
      <ToolsPanel v-if="prefs.panels.tools" />
      <main class="center">
        <div class="doc-tabs">
          <div class="doc-tab active" data-testid="doc-tab">
            {{ s.docName }}.transit @ {{ zoom }}%{{ s.dirty ? '*' : '' }}
          </div>
        </div>
        <CanvasView />
      </main>
      <aside v-if="showDock" class="dock">
        <PropertiesPanel v-if="prefs.panels.properties" class="props-panel" />
        <LayersPanel v-if="prefs.panels.layers" />
      </aside>
    </div>
    <StatusBar />
    <DocumentSettingsDialog v-if="s.dialog === 'document'" />
    <PreferencesDialog v-if="s.dialog === 'preferences'" />
    <ShortcutsDialog v-if="s.dialog === 'shortcuts'" />
    <ReferenceDialog v-if="s.dialog === 'reference'" />
    <AboutDialog v-if="s.dialog === 'about'" />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.workspace {
  flex: 1;
  display: flex;
  min-height: 0;
}
.center {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--c-pasteboard);
}
.doc-tabs {
  display: flex;
  height: 26px;
  background: var(--c-panel-header);
  border-bottom: 1px solid var(--c-border);
}
.doc-tab {
  display: flex;
  align-items: center;
  padding: 0 14px;
  font-size: var(--fs-small);
  color: var(--c-text-dim);
  border-right: 1px solid var(--c-border);
}
.doc-tab.active {
  background: var(--c-panel);
  color: var(--c-text-bright);
  box-shadow: inset 0 2px 0 var(--c-accent);
}
.dock {
  width: 300px;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--c-border);
  background: var(--c-panel-alt);
  min-height: 0;
}
.props-panel {
  max-height: 55%;
}
</style>
