<script setup lang="ts">
// Application preferences (stored in this browser, not in documents), one tab
// per area. Every tab can go back to its defaults, and "Restore all defaults"
// also resets the tool options, keyboard shortcuts and panel layout.
import { computed, reactive, ref } from 'vue'
import ModalDialog from './ModalDialog.vue'
import NumberField from '../ui/NumberField.vue'
import ColorField from '../ui/ColorField.vue'
import { useEditor } from '../../stores/editor'
import { defaultPrefs, prefs, resetPrefs, type Prefs } from '../../stores/prefs'
import { HELP } from '../../core/help'
import HelpTip from '../ui/HelpTip.vue'
import GitHubLink from '../ui/GitHubLink.vue'

const store = useEditor()
const s = store.state

type Draft = Omit<Prefs, 'shortcuts' | 'line' | 'station' | 'text' | 'icon'> & { snapDistance: number }
const pick = (p: Prefs): Draft => {
  const { shortcuts: _s, line: _l, station, text: _t, icon: _i, ...rest } = p
  return JSON.parse(JSON.stringify({ ...rest, snapDistance: station.snap_distance }))
}
const draft = reactive<Draft>(pick(prefs))

const TABS = [
  { id: 'general', label: 'General', keys: ['historyLimit', 'autoSelect', 'confirmDiscard', 'newText', 'messageDuration', 'errorDuration'] },
  { id: 'interface', label: 'Interface', keys: ['panels', 'propertiesHeight', 'showThumbnails', 'panelEditPause'] },
  {
    id: 'canvas',
    label: 'Canvas and grid',
    keys: ['canvasBackground', 'pasteboardColor', 'showGrid', 'gridColor', 'gridMajorColor', 'gridMajorEvery', 'minGridPixels', 'selectionColor', 'warningColor', 'previewOpacity'],
  },
  { id: 'navigation', label: 'Navigation', keys: ['zoomStep', 'minZoom', 'maxZoom', 'fitMargin', 'fitMaxZoom'] },
  { id: 'editing', label: 'Editing', keys: ['nudge', 'bigNudge', 'dragThreshold', 'eraserSize', 'snapDistance'] },
  { id: 'export', label: 'Export', keys: ['export'] },
] as const satisfies readonly { id: string; label: string; keys: readonly (keyof Draft)[] }[]

const tab = ref<(typeof TABS)[number]['id']>('general')
const current = computed(() => TABS.find((t) => t.id === tab.value)!)

/** Asking to confirm "Restore all defaults" (also opened from Edit ▸ Restore All Defaults). */
const confirming = ref(s.restorePrompt)
s.restorePrompt = false

function apply(): void {
  const { snapDistance, ...rest } = draft
  Object.assign(prefs, JSON.parse(JSON.stringify(rest)))
  prefs.station.snap_distance = snapDistance
  store.engine.value.setHistoryLimit(prefs.historyLimit)
}

function ok(): void {
  apply()
  s.dialog = null
}

function restoreTab(): void {
  const d = pick(defaultPrefs()) as unknown as Record<string, unknown>
  const out = draft as unknown as Record<string, unknown>
  for (const k of current.value.keys) out[k] = d[k]
}

function restoreAll(): void {
  resetPrefs()
  store.engine.value.setHistoryLimit(prefs.historyLimit)
  Object.assign(draft, pick(prefs))
  confirming.value = false
  store.notify('Every preference, tool option, keyboard shortcut and panel is back to its default')
}

const PANELS = [
  { key: 'tools', label: 'Tools' },
  { key: 'options', label: 'Options' },
  { key: 'properties', label: 'Properties' },
  { key: 'layers', label: 'Layers' },
] as const
</script>

<template>
  <ModalDialog title="Preferences" :width="600" @ok="ok" @cancel="s.dialog = null">
    <div class="prefs">
      <nav class="tabs" role="tablist" data-testid="prefs-tabs">
        <button
          v-for="t in TABS"
          :key="t.id"
          role="tab"
          :aria-selected="tab === t.id"
          :class="{ on: tab === t.id }"
          :data-testid="`prefs-tab-${t.id}`"
          @click="tab = t.id"
        >
          {{ t.label }}
        </button>
        <GitHubLink class="repo" :size="20" caption="GitHub" data-testid="prefs-github" />
      </nav>

      <section class="page" role="tabpanel" :data-testid="`prefs-page-${tab}`">
        <template v-if="tab === 'general'">
          <NumberField v-model="draft.historyLimit" :help="HELP['prefs.historyLimit']" label="History states" :min="1" :max="5000" :precision="0" />
          <label class="f">
            <span class="dim">Auto-select<HelpTip :text="HELP['prefs.autoSelect']" /></span>
            <select v-model="draft.autoSelect">
              <option value="layer">Layer</option>
              <option value="group">Group</option>
            </select>
          </label>
          <label class="f">
            <span class="dim">Ask before discarding changes<HelpTip :text="HELP['prefs.confirmDiscard']" /></span>
            <input v-model="draft.confirmDiscard" type="checkbox" />
          </label>
          <label class="f">
            <span class="dim">Text of new text boxes<HelpTip :text="HELP['prefs.newText']" /></span>
            <input v-model="draft.newText" type="text" class="text" data-testid="prefs-new-text" @keydown.stop />
          </label>
          <NumberField v-model="draft.messageDuration" :help="HELP['prefs.messageDuration']" label="Messages stay for" :min="500" :max="60000" :step="500" :precision="0" unit="milliseconds" />
          <NumberField v-model="draft.errorDuration" :help="HELP['prefs.errorDuration']" label="Errors stay for" :min="500" :max="60000" :step="500" :precision="0" unit="milliseconds" />
        </template>

        <template v-else-if="tab === 'interface'">
          <div class="f">
            <span class="dim">Panels<HelpTip :text="HELP['prefs.panels']" /></span>
            <span class="checks">
              <label v-for="p in PANELS" :key="p.key"><input v-model="draft.panels[p.key]" type="checkbox" /> {{ p.label }}</label>
            </span>
          </div>
          <NumberField v-model="draft.propertiesHeight" :help="HELP['prefs.propertiesHeight']" label="Properties panel height" :min="80" :max="2000" :step="10" :precision="0" unit="pixels" />
          <label class="f"><span class="dim">Layer thumbnails<HelpTip :text="HELP['prefs.showThumbnails']" /></span><input v-model="draft.showThumbnails" type="checkbox" /></label>
          <NumberField v-model="draft.panelEditPause" :help="HELP['prefs.panelEditPause']" label="Panel edit pause" :min="100" :max="10000" :step="100" :precision="0" unit="milliseconds" />
        </template>

        <template v-else-if="tab === 'canvas'">
          <label class="f"><span class="dim">Show grid<HelpTip :text="HELP['prefs.showGrid']" /></span><input v-model="draft.showGrid" type="checkbox" /></label>
          <ColorField v-model="draft.canvasBackground" :help="HELP['prefs.canvasBackground']" label="Canvas" />
          <ColorField v-model="draft.pasteboardColor" :help="HELP['prefs.pasteboardColor']" label="Around the canvas" />
          <ColorField v-model="draft.gridColor" :help="HELP['prefs.gridColor']" label="Grid" />
          <ColorField v-model="draft.gridMajorColor" :help="HELP['prefs.gridMajorColor']" label="Major grid" />
          <NumberField v-model="draft.gridMajorEvery" :help="HELP['prefs.gridMajorEvery']" label="Major every" :min="1" :max="100" :precision="0" unit="cells" />
          <NumberField v-model="draft.minGridPixels" :help="HELP['prefs.minGridPixels']" label="Hide grid below" :min="1" :max="50" :precision="0" unit="pixels" />
          <ColorField v-model="draft.selectionColor" :help="HELP['prefs.selectionColor']" label="Selection" />
          <ColorField v-model="draft.warningColor" :help="HELP['prefs.warningColor']" label="Eraser and warnings" />
          <NumberField
            :model-value="Math.round(draft.previewOpacity * 100)"
            :help="HELP['prefs.previewOpacity']"
            label="Preview opacity"
            :min="5"
            :max="100"
            :step="5"
            :precision="0"
            unit="%"
            @update:model-value="(v) => (draft.previewOpacity = v / 100)"
          />
        </template>

        <template v-else-if="tab === 'navigation'">
          <NumberField v-model="draft.zoomStep" :help="HELP['prefs.zoomStep']" label="Zoom step" :min="1.01" :max="3" :step="0.01" />
          <NumberField v-model="draft.minZoom" :help="HELP['prefs.minZoom']" label="Smallest zoom" :min="0.01" :max="1" :step="0.01" />
          <NumberField v-model="draft.maxZoom" :help="HELP['prefs.maxZoom']" label="Largest zoom" :min="1" :max="100" :step="1" />
          <NumberField v-model="draft.fitMargin" :help="HELP['prefs.fitMargin']" label="Fit on screen margin" :min="0" :max="500" :step="10" :precision="0" unit="pixels" />
          <NumberField v-model="draft.fitMaxZoom" :help="HELP['prefs.fitMaxZoom']" label="Fit on screen largest zoom" :min="0.1" :max="100" :step="0.5" />
        </template>

        <template v-else-if="tab === 'editing'">
          <NumberField v-model="draft.nudge" :help="HELP['prefs.nudge']" label="Arrow nudge" :min="0.05" :max="50" :step="0.25" unit="cells" />
          <NumberField v-model="draft.bigNudge" :help="HELP['prefs.bigNudge']" label="Shift + arrow" :min="0.05" :max="100" :step="1" unit="cells" />
          <NumberField v-model="draft.dragThreshold" :help="HELP['prefs.dragThreshold']" label="Drag threshold" :min="0" :max="30" :precision="0" unit="pixels" />
          <NumberField v-model="draft.eraserSize" :help="HELP['prefs.eraserSize']" label="Eraser size" :min="1" :max="15" :precision="0" unit="cells" />
          <NumberField v-model="draft.snapDistance" :help="HELP['prefs.snapDistance']" label="Station reach" :min="0" :max="10" :step="0.5" unit="cells" />
        </template>

        <template v-else-if="tab === 'export'">
          <p class="dim note">Starting values of the Export as SVG dialog; it remembers the last export.</p>
          <NumberField v-model="draft.export.cell_px" :help="HELP['export.cell_px']" label="Pixels per cell" :min="1" :max="512" :step="1" unit="pixels" />
          <NumberField v-model="draft.export.margin" :help="HELP['export.margin']" label="Margin" :min="0" :max="100" :step="0.5" unit="cells" />
          <label class="f">
            <span class="dim">Transparent background<HelpTip :text="HELP['export.background']" /></span>
            <input type="checkbox" :checked="draft.export.background === ''" @change="draft.export.background = ($event.target as HTMLInputElement).checked ? '' : '#ffffff'" />
          </label>
          <ColorField v-if="draft.export.background !== ''" v-model="draft.export.background" :help="HELP['export.background']" label="Background" />
        </template>
      </section>
    </div>

    <div v-if="confirming" class="confirm no-enter" data-testid="restore-confirm">
      <span>Restore every preference, tool option, keyboard shortcut and the panel layout to its default? Your documents are not changed.</span>
      <button class="ps-btn primary" data-testid="restore-all-confirm" @click="restoreAll">Restore all</button>
      <button class="ps-btn" @click="confirming = false">Keep</button>
    </div>

    <template #extra>
      <button class="ps-btn" data-testid="restore-tab" :title="`Put the ${current.label} settings back to their defaults`" @click="restoreTab">Restore tab defaults</button>
      <button class="ps-btn" data-testid="restore-all" @click="confirming = true">Restore all defaults…</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.prefs {
  display: grid;
  grid-template-columns: 140px 1fr;
  gap: 12px;
  min-height: 300px;
}
.tabs {
  display: flex;
  flex-direction: column;
  gap: 2px;
  border-right: 1px solid var(--c-border-soft);
  padding-right: 8px;
}
.tabs button {
  text-align: left;
  padding: 6px 8px;
  border: 0;
  border-radius: 3px;
  background: transparent;
  color: var(--c-text-dim);
  cursor: pointer;
}
.tabs button:hover {
  color: var(--c-text-bright);
  background: var(--c-hover);
}
.repo {
  margin-top: auto;
  align-self: flex-start;
}
.tabs button.on {
  color: var(--c-text-bright);
  background: var(--c-row-selected);
}
.page {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
/* Swatch, hex value and eyedropper line up on the right like the number inputs. */
.page :deep(.color-field .swatch) {
  margin-left: auto;
}
/* label | input | unit columns, so inputs line up with or without a unit */
.page :deep(.num-field) {
  display: grid;
  grid-template-columns: 1fr auto 90px;
  column-gap: 4px;
}
.f {
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: space-between;
}
.checks {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.checks label {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}
.text {
  width: 140px;
}
.note {
  margin: 0 0 4px;
  line-height: 1.5;
}
.confirm {
  margin-top: 12px;
  padding: 8px 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  border: 1px solid var(--c-border-soft);
  border-radius: 4px;
  background: #3a2e1e;
  line-height: 1.4;
}
</style>
