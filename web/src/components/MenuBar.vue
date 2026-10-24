<script setup lang="ts">
// Application menu bar. Menus open on click and follow the pointer while open.
import { computed, onBeforeUnmount, ref } from 'vue'
import { ACTIONS, type ActionId } from '../core/actions'
import { displayCombo } from '../core/shortcuts'
import { useEditor } from '../stores/editor'
import { isEnabled, runAction } from '../stores/commands'
import { prefs } from '../stores/prefs'
import { TRANSIT_EXT } from '../core/files'
import UiIcon from './ui/UiIcon.vue'

// The same artwork as the browser tab icon.
const logoUrl = `${import.meta.env.BASE_URL}favicon.svg`

type Item =
  | { action: ActionId; label?: string }
  | { toggle: () => boolean; set: () => void; label: string }
  | { sep: true }

const store = useEditor()
const open = ref<string | null>(null)
const bar = ref<HTMLElement | null>(null)

const menus = computed<{ name: string; items: Item[] }[]>(() => [
  {
    name: 'File',
    items: [{ action: 'file.new' }, { action: 'file.open' }, { sep: true }, { action: 'file.save' }, { action: 'file.saveAs' }, { action: 'file.exportSvg' }],
  },
  {
    name: 'Edit',
    items: [
      { action: 'edit.undo' },
      { action: 'edit.redo' },
      { sep: true },
      { action: 'edit.duplicate' },
      { action: 'edit.delete' },
      { sep: true },
      { action: 'edit.rotateCw' },
      { action: 'edit.rotateCcw' },
      { sep: true },
      { action: 'edit.shortcuts' },
      { action: 'edit.preferences' },
    ],
  },
  {
    name: 'Document',
    items: [
      { action: 'document.settings' },
      { action: 'document.assets' },
      { sep: true },
      {
        label: 'Route: Straight then Diagonal',
        toggle: () => store.state.settings.route_mode === 'straight_first',
        set: () => store.run((e) => e.setSettings({ ...store.state.settings, route_mode: 'straight_first' })),
      },
      {
        label: 'Route: Diagonal then Straight',
        toggle: () => store.state.settings.route_mode === 'diagonal_first',
        set: () => store.run((e) => e.setSettings({ ...store.state.settings, route_mode: 'diagonal_first' })),
      },
    ],
  },
  {
    name: 'Layer',
    items: [
      { action: 'layer.newGroup' },
      { action: 'layer.group' },
      { action: 'layer.ungroup' },
      { sep: true },
      { action: 'layer.toggleVisible' },
      { action: 'layer.toggleLock' },
      { sep: true },
      { action: 'layer.bringToFront' },
      { action: 'layer.bringForward' },
      { action: 'layer.sendBackward' },
      { action: 'layer.sendToBack' },
    ],
  },
  {
    name: 'Select',
    items: [
      { action: 'edit.selectAll', label: 'All' },
      { action: 'edit.deselect' },
      { sep: true },
      {
        label: 'Auto-Select Layer',
        toggle: () => prefs.autoSelect === 'layer',
        set: () => (prefs.autoSelect = 'layer'),
      },
      {
        label: 'Auto-Select Group',
        toggle: () => prefs.autoSelect === 'group',
        set: () => (prefs.autoSelect = 'group'),
      },
    ],
  },
  {
    name: 'View',
    items: [
      { action: 'view.zoomIn' },
      { action: 'view.zoomOut' },
      { action: 'view.fit' },
      { action: 'view.actual' },
      { sep: true },
      { label: 'Show Grid', toggle: () => prefs.showGrid, set: () => (prefs.showGrid = !prefs.showGrid) },
      {
        label: 'Layer Thumbnails',
        toggle: () => prefs.showThumbnails,
        set: () => (prefs.showThumbnails = !prefs.showThumbnails),
      },
    ],
  },
  {
    name: 'Window',
    items: (['tools', 'options', 'properties', 'layers'] as const).map((k) => ({
      label: { tools: 'Tools', options: 'Options', properties: 'Properties', layers: 'Layers' }[k],
      toggle: () => prefs.panels[k],
      set: () => (prefs.panels[k] = !prefs.panels[k]),
    })),
  },
  { name: 'Help', items: [{ action: 'help.shortcuts' }, { sep: true }, { action: 'help.about' }] },
])

function label(item: Item): string {
  if ('sep' in item) return ''
  if ('action' in item) return item.label ?? ACTIONS[item.action].label
  return item.label
}

function shortcut(item: Item): string {
  return 'action' in item ? displayCombo(prefs.shortcuts[item.action] ?? '') : ''
}

function enabled(item: Item): boolean {
  return 'action' in item ? isEnabled(store, item.action) : true
}

function activate(item: Item): void {
  if ('sep' in item || !enabled(item)) return
  open.value = null
  if ('action' in item) runAction(store, item.action)
  else item.set()
}

function onDocDown(e: PointerEvent): void {
  if (bar.value && !bar.value.contains(e.target as Node)) open.value = null
}
document.addEventListener('pointerdown', onDocDown)
onBeforeUnmount(() => document.removeEventListener('pointerdown', onDocDown))
</script>

<template>
  <nav ref="bar" class="menubar" data-testid="menubar">
    <div class="brand" title="Transit Diagram Maker">
      <img :src="logoUrl" width="20" height="20" alt="Transit Diagram Maker" data-testid="app-logo" />
    </div>
    <button
      type="button"
      class="bar-btn"
      :title="`Save (${displayCombo(prefs.shortcuts['file.save'] ?? '')})`"
      data-testid="quick-save"
      @click="runAction(store, 'file.save')"
    >
      <UiIcon name="save" :size="16" />
    </button>
    <div
      v-for="m in menus"
      :key="m.name"
      class="menu"
      :class="{ open: open === m.name }"
      @mouseenter="open !== null && (open = m.name)"
    >
      <button type="button" class="menu-title" :data-menu="m.name" @click="open = open === m.name ? null : m.name">
        {{ m.name }}
      </button>
      <div v-if="open === m.name" class="dropdown" role="menu">
        <template v-for="(item, i) in m.items" :key="i">
          <div v-if="'sep' in item" class="sep" />
          <button
            v-else
            type="button"
            role="menuitem"
            class="item"
            :disabled="!enabled(item)"
            @click="activate(item)"
          >
            <span class="check">{{ 'toggle' in item && item.toggle() ? '✓' : '' }}</span>
            <span class="label">{{ label(item) }}</span>
            <span class="kbd">{{ shortcut(item) }}</span>
          </button>
        </template>
      </div>
    </div>
    <div class="spacer" />
    <div class="doc-title dim" data-testid="doc-title">{{ store.state.docName }}{{ TRANSIT_EXT }}{{ store.state.dirty ? ' •' : '' }} — Transit Diagram Maker</div>
  </nav>
</template>

<style scoped>
.menubar {
  display: flex;
  align-items: stretch;
  height: 28px;
  background: var(--c-menubar);
  border-bottom: 1px solid var(--c-border);
  padding-left: 6px;
  position: relative;
  z-index: 40;
}
.brand {
  display: flex;
  align-items: center;
  padding: 0 4px 0 2px;
}
.brand + .bar-btn {
  margin-right: 4px;
}
.menu {
  position: relative;
}
.menu-title {
  height: 100%;
  padding: 0 9px;
  border: 0;
  background: transparent;
  cursor: default;
  color: var(--c-text);
}
.menu-title:hover,
.menu.open .menu-title {
  background: #454545;
  color: #fff;
}
.dropdown {
  position: absolute;
  top: 28px;
  left: 0;
  min-width: 250px;
  padding: 4px 0;
  white-space: nowrap;
  background: #3a3a3a;
  border: 1px solid var(--c-border);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.55);
}
.item {
  display: flex;
  width: 100%;
  align-items: center;
  height: 24px;
  padding: 0 14px 0 4px;
  border: 0;
  background: transparent;
  text-align: left;
  cursor: default;
}
.item:hover:not(:disabled) {
  background: var(--c-accent-dim);
  color: #fff;
}
.item:disabled {
  color: #6c6c6c;
}
.check {
  width: 20px;
  text-align: center;
  color: var(--c-accent);
}
.label {
  flex: 1;
}
.kbd {
  color: var(--c-text-dim);
  margin-left: 24px;
  font-size: var(--fs-small);
}
.item:hover:not(:disabled) .kbd {
  color: #dceeff;
}
.sep {
  height: 1px;
  margin: 4px 0;
  background: #4c4c4c;
}
.spacer {
  flex: 1;
}
.doc-title {
  display: flex;
  align-items: center;
  padding: 0 10px;
  font-size: var(--fs-small);
}
.bar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  border: 0;
  background: transparent;
  color: var(--c-text);
  cursor: pointer;
}
.bar-btn:hover {
  background: #454545;
  color: #fff;
}
</style>
