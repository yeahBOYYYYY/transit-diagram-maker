<script setup lang="ts">
// SVG export options. Starts from the document settings; changes here apply
// to this export only and never modify the document.
import { reactive } from 'vue'
import ModalDialog from './ModalDialog.vue'
import NumberField from '../ui/NumberField.vue'
import SettingsFields from '../settings/SettingsFields.vue'
import type { DocSettings } from '../../core/types'
import { useEditor } from '../../stores/editor'
import { prefs } from '../../stores/prefs'
import { exportSvg } from '../../stores/commands'
import { HELP } from '../../core/help'

const store = useEditor()
const draft = reactive<DocSettings>({ ...store.state.settings })

function ok(): void {
  exportSvg(store, { ...draft }, prefs.exportCellPx)
  store.state.dialog = null
}
function reset(): void {
  Object.assign(draft, store.state.settings)
  prefs.exportCellPx = 24
}
</script>

<template>
  <ModalDialog title="Export as SVG" :width="560" ok-label="Export" @ok="ok" @cancel="store.state.dialog = null">
    <p class="dim intro">These start from the document settings. Changes apply to this export only.</p>
    <SettingsFields :draft="draft" :sections="['lines', 'stations', 'export']">
      <template #export>
        <NumberField
          v-model="prefs.exportCellPx"
          label="Pixels per cell"
          :min="1"
          :max="512"
          :precision="1"
          unit="px"
          :help="HELP['export.cell_px']"
          data-testid="export-cell-px"
        />
      </template>
    </SettingsFields>
    <template #extra>
      <button class="ps-btn" @click="reset">Use document settings</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.intro {
  margin: 0 0 10px;
}
</style>
