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
import { useEditor } from '../stores/editor'
import { SHAPE_KINDS, prefs } from '../stores/prefs'
import FontSelect from './ui/FontSelect.vue'
import { rotateSelection } from '../stores/commands'
import { selectedPill } from '../tools'
import { STROKE_KINDS, hasChannel, hasDashes } from '../core/lineStyles'

const store = useEditor()
const s = store.state
const tool = computed(() => s.tool)

const zoomPct = computed(() => Math.round(s.view.zoom * 100))

/** Text options: the formatting of the text being edited, or the defaults for new text. */
const textOpts = computed(() => {
  void s.revision
  void s.textEdit?.selection
  void s.textEdit?.version
  return (s.textEdit && store.textFormat()) || prefs.text
})
type TextKey = 'font_family' | 'size' | 'color' | 'bold' | 'italic' | 'align' | 'direction'
function setText(key: TextKey, value: unknown): void {
  if (s.textEdit) store.formatText({ [key]: value })
  else (prefs.text as Record<TextKey, unknown>)[key] = value
}
/** Colors and border of the shape kind the shape tool draws. */
const shapePaint = computed(() => prefs.shape[prefs.shape.kind])
const shapeHint = computed(
  () =>
    ({
      line: 'Drag between grid intersections to draw a line in one of 8 directions · Alt: any point and angle',
      rect: 'Drag across cells to draw a rectangle covering them · Shift: square · Alt: off the grid',
      circle: 'Drag out from a cell to draw a circle centered on it · Alt: off the grid, any radius',
    })[prefs.shape.kind],
)
const pillSelected = computed(() => {
  void s.revision
  return selectedPill(store) !== null
})
</script>

<template>
  <div class="options" data-testid="options-bar">
    <div class="tool-badge" :title="tool">
      <UiIcon :name="tool" :size="18" />
    </div>
    <div class="sep-v" />

    <template v-if="s.textEdit || tool === 'text'">
      <div class="opt" data-keeps-text-edit data-testid="text-options">
        <span class="dim label">Font<HelpTip :text="HELP['text.font']" /></span>
        <FontSelect :model-value="textOpts.font_family" @update:model-value="(v) => setText('font_family', v)" />
        <NumberField :model-value="textOpts.size" label="Size" :min="0.1" :step="0.05" :width="40" unit="cells" :help="HELP['text.size']" data-testid="text-size" @update:model-value="(v) => setText('size', v)" />
        <ColorField :model-value="textOpts.color" label="Color" :help="HELP['text.color']" @update:model-value="(v) => setText('color', v)" />
        <button class="icon-btn" :class="{ on: textOpts.bold }" title="Bold (Ctrl+B while editing)" data-testid="text-bold" @mousedown.prevent @click="setText('bold', !textOpts.bold)">
          <UiIcon name="bold" :size="16" />
        </button>
        <button class="icon-btn" :class="{ on: textOpts.italic }" title="Italic (Ctrl+I while editing)" data-testid="text-italic" @mousedown.prevent @click="setText('italic', !textOpts.italic)">
          <UiIcon name="italic" :size="16" />
        </button>
        <span class="dim label">Align<HelpTip :text="HELP['text.align']" /></span>
        <Segmented
          :model-value="textOpts.align"
          data-testid="text-align"
          :options="[
            { value: 'start', icon: 'align-start', title: 'Left' },
            { value: 'middle', icon: 'align-middle', title: 'Center' },
            { value: 'end', icon: 'align-end', title: 'Right' },
          ]"
          @mousedown.prevent
          @update:model-value="(v) => setText('align', v)"
        />
        <span class="dim label">Direction<HelpTip :text="HELP['text.direction']" /></span>
        <Segmented
          :model-value="textOpts.direction"
          :options="[
            { value: 'ltr', icon: 'dir-ltr', title: 'Left to right' },
            { value: 'rtl', icon: 'dir-rtl', title: 'Right to left' },
          ]"
          @mousedown.prevent
          @update:model-value="(v) => setText('direction', v)"
        />
      </div>
      <span v-if="s.textEdit" class="hint dim">Select part of the text to format it · Shift+Enter: new line · Enter: done · Escape: cancel</span>
      <span v-else class="hint dim">Click to place text on a grid intersection · Alt: place it off the grid · Double-click a text to edit it</span>
    </template>

    <template v-else-if="tool === 'select'">
      <label class="opt">
        <span class="dim label">Auto-select<HelpTip :text="HELP['prefs.autoSelect']" /></span>
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
      <span v-else class="hint dim">Click a selected object to toggle rotate handles · Shift: add to selection · Dragged objects snap to the grid · Alt+drag: free move and free rotation (stations leave their line)</span>
    </template>

    <template v-else-if="tool === 'hand'">
      <button class="ps-btn" @click="store.actualSize()">100%</button>
      <button class="ps-btn" @click="store.fitToScreen()">Fit on Screen</button>
      <span class="hint dim">Zoom: {{ zoomPct }}% · Drag to pan · Ctrl+scroll to zoom</span>
    </template>

    <template v-else-if="tool === 'pen'">
      <label class="opt">
        <span class="dim label">Draw into<HelpTip :text="HELP['line.target']" /></span>
        <select v-model="s.penTarget" data-testid="pen-target">
          <option :value="null">New line</option>
          <option v-for="l in store.lines()" :key="l.lineId" :value="l.lineId">{{ l.name }}</option>
        </select>
      </label>
      <div class="sep-v" />
      <template v-if="s.penTarget === null">
        <ColorField v-model="prefs.line.color" label="Color" :help="HELP['line.color']" />
        <label class="opt">
          <span class="dim label">Style<HelpTip :text="HELP['line.kind']" /></span>
          <select v-model="prefs.line.kind" data-testid="line-style">
            <option v-for="k in STROKE_KINDS" :key="k.value" :value="k.value">{{ k.label }}</option>
          </select>
        </label>
        <label class="opt">
          <span class="dim label">Start cap<HelpTip :text="HELP['line.caps']" /></span>
          <select v-model="prefs.line.start_cap">
            <option value="round">Round</option>
            <option value="butt">Butt</option>
          </select>
        </label>
        <label class="opt">
          <span class="dim">End cap</span>
          <select v-model="prefs.line.end_cap">
            <option value="round">Round</option>
            <option value="butt">Butt</option>
          </select>
        </label>
        <div v-if="prefs.line.kind !== 'plain'" class="sep-v" />
        <template v-if="hasChannel(prefs.line.kind)">
          <ColorField v-model="prefs.line.inner_color" label="Channel" :help="HELP['line.channel']" />
          <NumberField v-model="prefs.line.inner_width_pct" label="Channel width" :min="0" :max="100" :precision="0" unit="%" :width="36" :help="HELP['line.channel_width']" />
        </template>
        <template v-if="hasDashes(prefs.line.kind)">
          <ColorField v-model="prefs.line.inner_color" label="Dashes" :help="HELP['line.dash_color']" />
          <NumberField v-model="prefs.line.dash_width_pct" label="Dash width" :min="0" :max="100" :precision="0" unit="%" :width="36" :help="HELP['line.dash_width']" />
          <NumberField v-model="prefs.line.dash_length" label="Dash length" :min="0.1" :step="0.1" :precision="1" unit="cells" :width="36" :help="HELP['line.dash_length']" />
          <NumberField v-model="prefs.line.dash_gap" label="Gap" :min="0" :step="0.1" :precision="1" unit="cells" :width="36" :help="HELP['line.dash_gap']" />
          <NumberField v-model="prefs.line.dash_start_offset" label="Start offset" :min="0" :step="0.5" :precision="1" unit="cells" :width="36" :help="HELP['line.dash_start_offset']" />
          <NumberField v-model="prefs.line.dash_end_offset" label="End offset" :min="0" :step="0.5" :precision="1" unit="cells" :width="36" :help="HELP['line.dash_end_offset']" />
        </template>
      </template>
      <span v-else class="dim">New segments use the line's style</span>
      <span class="hint dim">Click to add points · Click the last point again or press Enter to finish · Backspace: remove the last point</span>
    </template>

    <template v-else-if="tool === 'eraser'">
      <NumberField v-model="prefs.eraserSize" label="Size" :min="1" :max="15" :step="1" :precision="0" unit="cells" :width="36" :help="HELP['eraser.size']" />
      <span class="hint dim">Drag across a line to cut it into separate segments</span>
    </template>

    <template v-else-if="tool === 'station'">
      <span class="dim label">Shape<HelpTip :text="HELP['station.shape']" /></span>
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
        <ColorField v-model="prefs.station.outer_color" label="Border" :help="HELP['station.outer_color']" />
        <ColorField v-model="prefs.station.inner_color" label="Fill" :help="HELP['station.inner_color']" />
      </template>
      <template v-else>
        <label class="opt">
          <span class="dim label">Side<HelpTip :text="HELP['station.tick_side']" /></span>
          <select v-model.number="prefs.station.tick_side">
            <option :value="1">Left</option>
            <option :value="-1">Right</option>
          </select>
        </label>
        <label class="opt">
          <input type="checkbox" :checked="prefs.station.tick_color === null" @change="prefs.station.tick_color = ($event.target as HTMLInputElement).checked ? null : '#000000'" />
          <span class="dim label">Line color<HelpTip :text="HELP['station.tick_color']" /></span>
        </label>
        <ColorField v-if="prefs.station.tick_color !== null" v-model="prefs.station.tick_color" />
      </template>
      <label class="opt">
        <input v-model="prefs.station.label" type="checkbox" />
        <span class="dim label">Name label<HelpTip :text="HELP['station.label']" /></span>
      </label>
      <span class="hint dim">{{ prefs.station.kind === 'pill' ? 'Drag across lines to paint a pill station' : 'Click on a line to place a station' }}</span>
    </template>

    <template v-else-if="tool === 'shape'">
      <span class="dim label">Shape<HelpTip :text="HELP['shape.kind']" /></span>
      <Segmented v-model="prefs.shape.kind" data-testid="shape-kind" :options="SHAPE_KINDS" />
      <div class="sep-v" />
      <template v-if="prefs.shape.kind === 'line'">
        <ColorField v-model="prefs.shape.line.color" label="Color" :help="HELP['shape.line_color']" />
        <NumberField v-model="prefs.shape.line.thickness" label="Width" :min="0.01" :step="0.05" unit="cells" :width="40" :help="HELP['shape.thickness']" data-testid="shape-thickness" />
        <label class="opt">
          <span class="dim label">Caps<HelpTip :text="HELP['shape.cap']" /></span>
          <select v-model="prefs.shape.line.cap">
            <option value="round">Round</option>
            <option value="butt">Butt</option>
          </select>
        </label>
      </template>
      <template v-else>
        <label class="opt">
          <input v-model="shapePaint.filled" type="checkbox" data-testid="shape-filled" />
          <span class="dim label">Fill<HelpTip :text="HELP['shape.fill']" /></span>
        </label>
        <ColorField v-if="shapePaint.filled" v-model="shapePaint.color" />
      </template>
      <ColorField v-model="shapePaint.border_color" label="Border" :help="HELP['shape.border_color']" />
      <NumberField v-model="shapePaint.border_width" label="Border width" :min="0" :step="0.05" unit="cells" :width="40" :help="HELP['shape.border_width']" />
      <template v-if="prefs.shape.kind === 'line'">
        <label class="opt">
          <input v-model="prefs.shape.line.dashed" type="checkbox" data-testid="shape-dashed" />
          <span class="dim label">Dashed<HelpTip :text="HELP['shape.dashed']" /></span>
        </label>
        <template v-if="prefs.shape.line.dashed">
          <NumberField v-model="prefs.shape.line.dash_length" label="Dash length" :min="0.05" :step="0.1" unit="cells" :width="36" :help="HELP['shape.dash_length']" />
          <NumberField v-model="prefs.shape.line.dash_gap" label="Gap" :min="0" :step="0.1" unit="cells" :width="36" :help="HELP['shape.dash_gap']" />
        </template>
      </template>
      <span class="hint dim">{{ shapeHint }}</span>
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
  /* Long tool options (a dashed pen) scroll sideways in narrow windows. */
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: thin;
  position: relative;
  z-index: 30;
}
.options > * {
  flex: none;
}
/* Hints give way first: they shorten with an ellipsis before the bar scrolls. */
.options > .hint {
  flex: 0 1 auto;
  min-width: 60px;
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
  gap: 8px;
}
/* A label and its help icon. */
.label {
  display: inline-flex;
  align-items: center;
}
.hint {
  margin-left: 6px;
  font-size: var(--fs-small);
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
