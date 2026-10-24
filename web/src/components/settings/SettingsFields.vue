<script setup lang="ts">
// Editable document settings, grouped in sections, for the Document Settings dialog.
import NumberField from '../ui/NumberField.vue'
import type { DocSettings } from '../../core/types'
import FontSelect from '../ui/FontSelect.vue'
import { HELP } from '../../core/help'
import HelpTip from '../ui/HelpTip.vue'

export type SettingsSection = 'drawing' | 'lines' | 'stations' | 'ticks' | 'labels' | 'icons'

defineProps<{ draft: DocSettings; sections: SettingsSection[] }>()
</script>

<template>
  <div class="sections">
    <fieldset v-if="sections.includes('drawing')">
      <legend>Drawing</legend>
      <label class="f">
        <span class="dim">Pen routing<HelpTip :text="HELP['settings.route_mode']" /></span>
        <select v-model="draft.route_mode">
          <option value="straight_first">Straight, then diagonal</option>
          <option value="diagonal_first">Diagonal, then straight</option>
        </select>
      </label>
    </fieldset>
    <fieldset v-if="sections.includes('lines')">
      <legend>Lines</legend>
      <NumberField v-model="draft.line_width" :help="HELP['settings.line_width']" label="Width" :min="0.05" :max="4" :step="0.05" unit="cells" />
      <NumberField v-model="draft.corner_radius" :help="HELP['settings.corner_radius']" label="Curve radius" :min="0" :max="50" :step="0.25" unit="cells" />
      <NumberField v-model="draft.cap_roundness" :help="HELP['settings.cap_roundness']" label="Round cap roundness" :min="0" :max="1" :step="0.05" />
    </fieldset>
    <fieldset v-if="sections.includes('stations')">
      <legend>Stations</legend>
      <NumberField v-model="draft.station_outer_radius" :help="HELP['settings.station_outer_radius']" label="Dot / pill outer radius" :min="0.05" :max="10" :step="0.02" unit="cells" />
      <NumberField v-model="draft.station_inner_radius" :help="HELP['settings.station_inner_radius']" label="Dot / pill inner radius" :min="0" :max="10" :step="0.02" unit="cells" />
      <template v-if="sections.includes('ticks')">
        <NumberField v-model="draft.tick_width" :help="HELP['settings.tick_width']" label="New tick width" :min="0.01" :max="10" :step="0.05" unit="cells" />
        <NumberField v-model="draft.tick_length" :help="HELP['settings.tick_length']" label="New tick height" :min="0.01" :max="10" :step="0.05" unit="cells" />
      </template>
    </fieldset>
    <fieldset v-if="sections.includes('labels')">
      <legend>Station labels</legend>
      <NumberField v-model="draft.label_offset" :help="HELP['settings.label_offset']" label="Distance" :min="0" :max="20" :step="0.1" unit="cells" />
      <NumberField v-model="draft.label_font_size" :help="HELP['settings.label_font_size']" label="Font size" :min="0.05" :max="20" :step="0.05" unit="cells" />
      <label class="f">
        <span class="dim">Font<HelpTip :text="HELP['settings.label_font_family']" /></span>
        <FontSelect v-model="draft.label_font_family" />
      </label>
    </fieldset>
    <fieldset v-if="sections.includes('icons')">
      <legend>Pictograms</legend>
      <label class="f">
        <input type="checkbox" :checked="draft.icon_size === null" @change="draft.icon_size = ($event.target as HTMLInputElement).checked ? null : 1.24" />
        <span class="dim">Default size matches dot stations<HelpTip :text="HELP['settings.icon_size']" /></span>
      </label>
      <NumberField v-if="draft.icon_size !== null" v-model="draft.icon_size" label="Default size" :min="0.05" :max="50" :step="0.1" unit="cells" :help="HELP['settings.icon_size']" />
    </fieldset>
  </div>
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
/* label | input | unit columns, so inputs line up with or without a unit */
fieldset :deep(.num-field) {
  display: grid;
  grid-template-columns: 1fr auto 34px;
  column-gap: 4px;
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
