<script setup lang="ts">
// Context sensitive options for the active tool. Values here are the defaults
// used for newly created elements and are remembered between sessions.
import { computed } from 'vue'
import UiIcon from './ui/UiIcon.vue'
import HelpTip from './ui/HelpTip.vue'
import { HELP } from '../core/help'
import NumberField from './ui/NumberField.vue'
import ColorField from './ui/ColorField.vue'
import Segmented from './ui/Segmented.vue'
import IconPicker from './ui/IconPicker.vue'
import { useEditor } from '../stores/editor'
import { FONT_FAMILIES, prefs } from '../stores/prefs'
import { rotateSelection } from '../stores/commands'
import { selectedPill } from '../tools'
import { STROKE_KINDS, hasChannel, hasDashes } from '../core/lineStyles'

const store = useEditor()
const s = store.state
const tool = computed(() => s.tool)

const zoomPct = computed(() => Math.round(s.view.zoom * 100))
const pillSelected = computed(() => {
  void s.revision
  return selectedPill(store) !== null
})
const fontLabel = (f: string) => f.split(',')[0].replace(/"/g, '')
</script>

<template>
  <div class="options" data-testid="options-bar">
    <div class="tool-badge" :title="tool">
      <UiIcon :name="tool" :size="18" />
    </div>
    <div class="sep-v" />

    <template v-if="tool === 'select'">
      <label class="opt">
        <span class="dim">Auto-Select<HelpTip :text="HELP['prefs.autoSelect']" /></span>
        <select v-model="prefs.autoSelect" data-testid="auto-select">
          <option value="layer">Layer</option>
          <option value="group">Group</option>
        </select>
      </label>
      <div class="sep-v" />
      <Segmented
        v-model="s.transformMode"
        :options="[
          { value: 'move', icon: 'move', title: 'Move mode' },
          { value: 'rotate', icon: 'rotate-cw', title: 'Rotate mode (click a selected object to toggle)' },
        ]"
      />
      <button class="icon-btn" title="Rotate 45° counter clockwise" :disabled="!s.selection.length" @click="rotateSelection(store, -45)">
        <UiIcon name="rotate-ccw" :size="16" />
      </button>
      <button class="icon-btn" title="Rotate 45° clockwise" :disabled="!s.selection.length" @click="rotateSelection(store, 45)">
        <UiIcon name="rotate-cw" :size="16" />
      </button>
      <span v-if="pillSelected" class="hint dim">Double-click an arrow to extend the pill · double-click one of its cells to remove it</span>
      <span v-else class="hint dim">Click a selected object to toggle rotate handles · Shift: add to selection · Alt+drag: free move (stations leave their line)</span>
    </template>

    <template v-else-if="tool === 'hand'">
      <button class="ps-btn" @click="store.actualSize()">100%</button>
      <button class="ps-btn" @click="store.fitToScreen()">Fit Screen</button>
      <span class="dim">Zoom {{ zoomPct }}% · Ctrl + Scroll to zoom, drag to pan</span>
    </template>

    <template v-else-if="tool === 'pen'">
      <label class="opt">
        <span class="dim">Draw into<HelpTip :text="HELP['line.target']" /></span>
        <select v-model="s.penTarget" data-testid="pen-target">
          <option :value="null">New line</option>
          <option v-for="l in store.lines()" :key="l.lineId" :value="l.lineId">{{ l.name }}</option>
        </select>
      </label>
      <div class="sep-v" />
      <template v-if="s.penTarget === null">
        <ColorField v-model="prefs.line.color" label="Color" :help="HELP['line.color']" />
        <label class="opt">
          <span class="dim">Style<HelpTip :text="HELP['line.kind']" /></span>
          <select v-model="prefs.line.kind" data-testid="line-style">
            <option v-for="k in STROKE_KINDS" :key="k.value" :value="k.value">{{ k.label }}</option>
          </select>
        </label>
        <template v-if="hasChannel(prefs.line.kind)">
          <ColorField v-model="prefs.line.inner_color" label="Channel" :help="HELP['line.channel']" />
          <NumberField v-model="prefs.line.inner_width_pct" label="W" :min="0" :max="100" :precision="0" unit="%" :width="36" :help="HELP['line.channel_width']" />
        </template>
        <template v-if="hasDashes(prefs.line.kind)">
          <NumberField v-model="prefs.line.dash_length" label="Dash" :min="0.1" :step="0.1" :precision="1" unit="cells" :width="36" :help="HELP['line.dash_length']" />
          <NumberField v-model="prefs.line.dash_gap" label="Gap" :min="0" :step="0.1" :precision="1" unit="cells" :width="36" :help="HELP['line.dash_gap']" />
          <NumberField v-if="prefs.line.kind === 'hollow_dashed'" v-model="prefs.line.dash_width_pct" label="Dash W" :min="0" :max="100" :precision="0" unit="%" :width="36" :help="HELP['line.dash_width']" />
        </template>
        <div class="sep-v" />
        <label class="opt">
          <span class="dim">Caps<HelpTip :text="HELP['line.caps']" /> Start</span>
          <select v-model="prefs.line.start_cap">
            <option value="round">Round</option>
            <option value="butt">Butt</option>
          </select>
        </label>
        <label class="opt">
          <span class="dim">End</span>
          <select v-model="prefs.line.end_cap">
            <option value="round">Round</option>
            <option value="butt">Butt</option>
          </select>
        </label>
      </template>
      <span v-else class="dim">Uses the line's style</span>
      <span class="hint dim">Click to add points · click last point / Enter to finish · Backspace removes</span>
    </template>

    <template v-else-if="tool === 'eraser'">
      <NumberField v-model="prefs.eraserSize" label="Size" :min="1" :max="15" :step="1" :precision="0" unit="cells" :width="36" :help="HELP['eraser.size']" />
      <span class="hint dim">Drag across a line to cut it into separate segments</span>
    </template>

    <template v-else-if="tool === 'station'">
      <span class="dim opt">Shape<HelpTip :text="HELP['station.shape']" /></span>
      <Segmented
        v-model="prefs.station.kind"
        :options="[
          { value: 'dot', label: 'Dot' },
          { value: 'tick', label: 'Tick' },
          { value: 'pill', label: 'Pill' },
        ]"
      />
      <div class="sep-v" />
      <template v-if="prefs.station.kind !== 'tick'">
        <ColorField v-model="prefs.station.outer_color" label="Outer" :help="HELP['station.outer_color']" />
        <ColorField v-model="prefs.station.inner_color" label="Inner" :help="HELP['station.inner_color']" />
      </template>
      <template v-else>
        <label class="opt">
          <span class="dim">Side<HelpTip :text="HELP['station.tick_side']" /></span>
          <select v-model.number="prefs.station.tick_side">
            <option :value="1">Left</option>
            <option :value="-1">Right</option>
          </select>
        </label>
        <label class="opt">
          <input type="checkbox" :checked="prefs.station.tick_color === null" @change="prefs.station.tick_color = ($event.target as HTMLInputElement).checked ? null : '#000000'" />
          <span class="dim">Line color<HelpTip :text="HELP['station.tick_color']" /></span>
        </label>
        <ColorField v-if="prefs.station.tick_color !== null" v-model="prefs.station.tick_color" />
      </template>
      <label class="opt">
        <input v-model="prefs.station.label" type="checkbox" />
        <span class="dim">Name label<HelpTip :text="HELP['station.label']" /></span>
      </label>
      <span class="hint dim">{{ prefs.station.kind === 'pill' ? 'Drag across lines to paint a pill station' : 'Click on a line to place a station' }}</span>
    </template>

    <template v-else-if="tool === 'text'">
      <span class="dim opt">Font<HelpTip :text="HELP['text.font']" /></span>
      <select v-model="prefs.text.font_family">
        <option v-for="f in FONT_FAMILIES" :key="f" :value="f">{{ fontLabel(f) }}</option>
      </select>
      <NumberField v-model="prefs.text.size" label="Size" :min="0.1" :step="0.05" :width="40" unit="cells" :help="HELP['text.size']" />
      <ColorField v-model="prefs.text.color" label="Color" :help="HELP['text.color']" />
      <button class="icon-btn" :class="{ on: prefs.text.bold }" title="Bold" @click="prefs.text.bold = !prefs.text.bold">
        <UiIcon name="bold" :size="16" />
      </button>
      <button class="icon-btn" :class="{ on: prefs.text.italic }" title="Italic" @click="prefs.text.italic = !prefs.text.italic">
        <UiIcon name="italic" :size="16" />
      </button>
      <span class="dim opt">Align<HelpTip :text="HELP['text.align']" /></span>
      <Segmented
        v-model="prefs.text.align"
        :options="[
          { value: 'start', icon: 'align-start', title: 'Left' },
          { value: 'middle', icon: 'align-middle', title: 'Center' },
          { value: 'end', icon: 'align-end', title: 'Right' },
        ]"
      />
      <span class="hint dim">Click to place text · Enter commits · Shift+Enter new line</span>
    </template>

    <template v-else-if="tool === 'icon'">
      <span class="dim opt">Symbol<HelpTip :text="HELP['icon.icon']" /></span>
      <IconPicker v-model="prefs.icon.icon" />
      <ColorField v-model="prefs.icon.color" label="Glyph" :help="HELP['icon.color']" />
      <ColorField v-model="prefs.icon.background" label="Tile" :help="HELP['icon.background']" />
      <label class="opt">
        <input type="checkbox" :checked="prefs.icon.size === null" @change="prefs.icon.size = ($event.target as HTMLInputElement).checked ? null : 1.24" />
        <span class="dim">Match station size<HelpTip :text="HELP['icon.size_default']" /></span>
      </label>
      <NumberField v-if="prefs.icon.size !== null" v-model="prefs.icon.size" label="Size" :min="0.1" :step="0.1" :width="40" unit="cells" :help="HELP['icon.size']" />
      <span class="hint dim">Click to drop · Alt: place off-grid</span>
    </template>
  </div>
</template>

<style scoped>
.options {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 10px 0 6px;
  background: var(--c-panel);
  border-bottom: 1px solid var(--c-border);
  white-space: nowrap;
  overflow: visible;
  position: relative;
  z-index: 30;
}
.tool-badge {
  width: 30px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  background: #2e2e2e;
  border: 1px solid #262626;
}
.opt {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.hint {
  margin-left: 6px;
  font-size: var(--fs-small);
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
