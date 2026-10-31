<script setup lang="ts">
// SVG export options: only what changes the exported picture itself (its
// scale, margin and background). The document is never modified; the last
// used options are remembered in the preferences.
import { reactive } from 'vue'
import ModalDialog from './ModalDialog.vue'
import NumberField from '../ui/NumberField.vue'
import ColorField from '../ui/ColorField.vue'
import HelpTip from '../ui/HelpTip.vue'
import type { ExportOptions } from '../../core/types'
import { useEditor } from '../../stores/editor'
import { defaultPrefs, prefs } from '../../stores/prefs'
import { exportSvg } from '../../stores/commands'
import { HELP } from '../../core/help'

const store = useEditor()
const draft = reactive<ExportOptions>({ ...prefs.export })

function ok(): void {
  prefs.export = { ...draft }
  exportSvg(store, { ...draft })
  store.state.dialog = null
}
function reset(): void {
  Object.assign(draft, defaultPrefs().export)
}
</script>

<template>
  <ModalDialog title="Export as SVG" :width="340" ok-label="Export" @ok="ok" @cancel="store.state.dialog = null">
    <div class="fields" data-testid="export-options">
      <NumberField
        v-model="draft.cell_px"
        label="Pixels per cell"
        :min="1"
        :max="512"
        :precision="1"
        unit="pixels"
        :help="HELP['export.cell_px']"
        data-testid="export-cell-px"
      />
      <NumberField v-model="draft.margin" label="Margin" :min="0" :max="100" :step="0.5" unit="cells" :help="HELP['export.margin']" data-testid="export-margin" />
      <label class="f">
        <span class="dim">Transparent background<HelpTip :text="HELP['export.background']" /></span>
        <input type="checkbox" :checked="draft.background === ''" data-testid="export-transparent" @change="draft.background = ($event.target as HTMLInputElement).checked ? '' : '#ffffff'" />
      </label>
      <ColorField v-if="draft.background !== ''" v-model="draft.background" label="Background" :help="HELP['export.background']" />
    </div>
    <p class="dim note">Line widths, station sizes and the other drawing constants are document settings (File ▸ Document Settings).</p>
    <template #extra>
      <button class="ps-btn" @click="reset">Reset to defaults</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.fields {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.fields :deep(.num-field) {
  display: grid;
  grid-template-columns: 1fr auto 40px;
  column-gap: 4px;
}
.fields :deep(.color-field) {
  justify-content: space-between;
}
.f {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}
.note {
  margin: 12px 0 0;
  line-height: 1.5;
  font-size: var(--fs-small);
}
</style>
