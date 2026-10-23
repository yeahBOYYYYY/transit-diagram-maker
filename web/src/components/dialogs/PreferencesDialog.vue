<script setup lang="ts">
// Application preferences (stored in this browser, not in documents).
import { reactive } from 'vue'
import ModalDialog from './ModalDialog.vue'
import NumberField from '../ui/NumberField.vue'
import ColorField from '../ui/ColorField.vue'
import { useEditor } from '../../stores/editor'
import { defaultPrefs, prefs, type Prefs } from '../../stores/prefs'

const store = useEditor()
type Editable = Omit<Prefs, 'shortcuts' | 'line' | 'station' | 'text' | 'icon' | 'panels'>
const pick = (p: Prefs): Editable => {
  const { shortcuts: _s, line: _l, station: _st, text: _t, icon: _i, panels: _p, ...rest } = p
  return { ...rest }
}
const draft = reactive<Editable>(pick(prefs))

function ok(): void {
  Object.assign(prefs, draft)
  store.engine.value.setHistoryLimit(prefs.historyLimit)
  store.state.dialog = null
}
function reset(): void {
  Object.assign(draft, pick(defaultPrefs()))
}
</script>

<template>
  <ModalDialog title="Preferences" :width="560" @ok="ok" @cancel="store.state.dialog = null">
    <div class="sections">
      <fieldset>
        <legend>General</legend>
        <NumberField v-model="draft.historyLimit" label="History states" :min="1" :max="5000" :precision="0" />
        <NumberField v-model="draft.longPressMs" label="Long press" :min="100" :max="3000" :step="50" :precision="0" unit="ms" />
        <NumberField v-model="draft.dragThreshold" label="Drag threshold" :min="0" :max="30" :precision="0" unit="px" />
        <label class="f">
          <span class="dim">Auto-select</span>
          <select v-model="draft.autoSelect">
            <option value="layer">Layer</option>
            <option value="group">Group</option>
          </select>
        </label>
      </fieldset>
      <fieldset>
        <legend>Navigation</legend>
        <NumberField v-model="draft.zoomStep" label="Zoom step" :min="1.01" :max="3" :step="0.01" />
        <NumberField v-model="draft.minZoom" label="Min zoom" :min="0.01" :max="1" :step="0.01" />
        <NumberField v-model="draft.maxZoom" label="Max zoom" :min="1" :max="100" :step="1" />
      </fieldset>
      <fieldset>
        <legend>Editing</legend>
        <NumberField v-model="draft.nudge" label="Arrow nudge" :min="0.05" :max="50" :step="0.25" unit="cells" />
        <NumberField v-model="draft.bigNudge" label="Shift + arrow" :min="0.05" :max="100" :step="1" unit="cells" />
        <NumberField v-model="draft.textSnap" label="Text / icon snap" :min="0.1" :max="5" :step="0.1" :precision="1" unit="cells" />
        <NumberField v-model="draft.eraserSize" label="Eraser size" :min="1" :max="15" :precision="0" unit="cells" />
      </fieldset>
      <fieldset>
        <legend>Canvas &amp; grid</legend>
        <label class="f"><span class="dim">Show grid</span><input v-model="draft.showGrid" type="checkbox" /></label>
        <ColorField v-model="draft.canvasBackground" label="Canvas" />
        <ColorField v-model="draft.gridColor" label="Grid" />
        <ColorField v-model="draft.gridMajorColor" label="Major grid" />
        <NumberField v-model="draft.gridMajorEvery" label="Major every" :min="1" :max="100" :precision="0" unit="cells" />
        <NumberField v-model="draft.minGridPixels" label="Hide grid below" :min="1" :max="50" :precision="0" unit="px" />
        <ColorField v-model="draft.selectionColor" label="Selection" />
        <label class="f"><span class="dim">Layer thumbnails</span><input v-model="draft.showThumbnails" type="checkbox" /></label>
      </fieldset>
    </div>
    <template #extra>
      <button class="ps-btn" @click="reset">Reset to defaults</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.sections {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
fieldset {
  border: 1px solid var(--c-border-soft);
  border-radius: 4px;
  padding: 8px 10px 10px;
  display: flex;
  flex-direction: column;
  gap: 7px;
  margin: 0;
}
fieldset :deep(.num-field),
fieldset :deep(.color-field) {
  justify-content: space-between;
}
legend {
  padding: 0 4px;
  color: var(--c-text-bright);
  font-weight: 600;
}
.f {
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: space-between;
}
</style>
