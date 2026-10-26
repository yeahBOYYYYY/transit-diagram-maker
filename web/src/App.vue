<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import MenuBar from './components/MenuBar.vue'
import OptionsBar from './components/OptionsBar.vue'
import ToolsPanel from './components/ToolsPanel.vue'
import CanvasView from './components/CanvasView.vue'
import StatusBar from './components/StatusBar.vue'
import LayersPanel from './components/panels/LayersPanel.vue'
import PropertiesPanel from './components/panels/PropertiesPanel.vue'
import IconsPanel from './components/panels/IconsPanel.vue'
import PanelFrame from './components/panels/PanelFrame.vue'
import DocumentSettingsDialog from './components/dialogs/DocumentSettingsDialog.vue'
import ExportDialog from './components/dialogs/ExportDialog.vue'
import AssetsDialog from './components/dialogs/AssetsDialog.vue'
import PreferencesDialog from './components/dialogs/PreferencesDialog.vue'
import ShortcutsDialog from './components/dialogs/ShortcutsDialog.vue'
import ReferenceDialog from './components/dialogs/ReferenceDialog.vue'
import AboutDialog from './components/dialogs/AboutDialog.vue'
import { useEditor } from './stores/editor'
import { prefs } from './stores/prefs'

const store = useEditor()
const s = store.state
const showDock = computed(() => prefs.panels.properties || prefs.panels.layers)
const bothPanels = computed(() => prefs.panels.properties && prefs.panels.layers)

// The splitter between Properties and Layers: the Properties panel keeps its
// height whatever it shows, and only dragging the splitter changes it.
const dock = ref<HTMLElement | null>(null)
const MIN_PANEL = 90
let drag: { y: number; h: number } | null = null
function clampHeight(h: number): number {
  const total = dock.value?.clientHeight ?? 800
  return Math.round(Math.min(Math.max(h, MIN_PANEL), Math.max(MIN_PANEL, total - MIN_PANEL)))
}
function onSplitDown(e: PointerEvent): void {
  drag = { y: e.clientY, h: prefs.propertiesHeight }
  ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
}
function onSplitMove(e: PointerEvent): void {
  if (drag) prefs.propertiesHeight = clampHeight(drag.h + e.clientY - drag.y)
}
function onSplitUp(): void {
  drag = null
}

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
        <CanvasView />
      </main>
      <aside v-if="showDock" ref="dock" class="dock" data-testid="dock">
        <PanelFrame
          v-if="prefs.panels.properties"
          v-model:active="s.dockTab"
          title="Properties"
          :tabs="['Properties', 'Icons']"
          class="props-panel"
          :class="{ fill: !prefs.panels.layers }"
          :style="bothPanels ? { height: prefs.propertiesHeight + 'px' } : undefined"
        >
          <PropertiesPanel v-if="s.dockTab === 'Properties'" />
          <IconsPanel v-else />
        </PanelFrame>
        <div
          v-if="bothPanels"
          class="splitter"
          title="Drag to resize the panels"
          data-testid="dock-splitter"
          @pointerdown.prevent="onSplitDown"
          @pointermove="onSplitMove"
          @pointerup="onSplitUp"
          @pointercancel="onSplitUp"
        />
        <LayersPanel v-if="prefs.panels.layers" />
      </aside>
    </div>
    <StatusBar />
    <DocumentSettingsDialog v-if="s.dialog === 'document'" />
    <ExportDialog v-if="s.dialog === 'export'" />
    <AssetsDialog v-if="s.dialog === 'assets'" />
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
.dock {
  width: 300px;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--c-border);
  background: var(--c-panel-alt);
  min-height: 0;
}
.props-panel {
  flex: none;
  max-height: calc(100% - 90px);
}
.props-panel.fill {
  flex: 1;
  max-height: none;
}
.splitter {
  flex: none;
  height: 5px;
  margin: -2px 0;
  position: relative;
  z-index: 5;
  cursor: row-resize;
}
.splitter:hover {
  background: var(--c-accent);
}
</style>
