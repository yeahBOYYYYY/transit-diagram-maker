<script setup lang="ts">
// Every document constant: drawing, line geometry, stations, labels, icons, export.
import { reactive } from 'vue'
import ModalDialog from './ModalDialog.vue'
import SettingsFields from '../settings/SettingsFields.vue'
import type { DocSettings } from '../../core/types'
import { useEditor } from '../../stores/editor'

const store = useEditor()
const draft = reactive<DocSettings>({ ...store.state.settings })
const DEFAULTS: DocSettings = {
  line_width: 1,
  corner_radius: 1.5,
  cap_roundness: 1,
  station_outer_radius: 0.62,
  station_inner_radius: 0.4,
  tick_width: 0.35,
  tick_length: 0.55,
  label_offset: 1.3,
  label_font_size: 0.75,
  label_font_family: 'Helvetica, Arial, sans-serif',
  icon_size: null,
  export_margin: 1,
  export_background: '#ffffff',
  route_mode: 'straight_first',
}

function ok(): void {
  store.run((e) => e.setSettings({ ...draft }))
  store.state.dialog = null
}
function reset(): void {
  Object.assign(draft, DEFAULTS)
}
</script>

<template>
  <ModalDialog title="Document Settings" :width="560" @ok="ok" @cancel="store.state.dialog = null">
    <SettingsFields :draft="draft" :sections="['drawing', 'lines', 'stations', 'ticks', 'labels', 'icons', 'export']" />
    <template #extra>
      <button class="ps-btn" @click="reset">Reset to defaults</button>
    </template>
  </ModalDialog>
</template>
