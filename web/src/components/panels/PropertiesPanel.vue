<script setup lang="ts">
// Properties of the current selection (or the document when nothing is selected).
import { computed } from 'vue'
import NumberField from '../ui/NumberField.vue'
import ColorField from '../ui/ColorField.vue'
import Segmented from '../ui/Segmented.vue'
import IconPicker from '../ui/IconPicker.vue'
import UiIcon from '../ui/UiIcon.vue'
import HelpTip from '../ui/HelpTip.vue'
import { HELP } from '../../core/help'
import type { DocNode, DocSettings, GroupNode, Id, LayerNode, LineSegment, LineStyle, StationKind, StrokeKind } from '../../core/types'
import { STROKE_KINDS, hasChannel, hasDashes } from '../../core/lineStyles'
import type { Engine } from '../../core/engine'
import { findItem } from '../../core/tree'
import { useEditor } from '../../stores/editor'
import { prefs } from '../../stores/prefs'
import FontSelect from '../ui/FontSelect.vue'
import { moveSelectionTo, rotateSelection, selectionOnGrid } from '../../stores/commands'

const store = useEditor()
const s = store.state

// Continuous edits (scrubbing, color dragging) are folded into one history
// step: the first change opens a transaction, a pause or an explicit commit
// closes it.
let txnOpen = false
let idle: ReturnType<typeof setTimeout> | null = null
function live(fn: (e: Engine) => void): void {
  const e = store.engine.value
  if (!txnOpen) {
    e.begin()
    txnOpen = true
  }
  store.run((en) => {
    en.rebase()
    fn(en)
  })
  if (idle) clearTimeout(idle)
  idle = setTimeout(done, prefs.panelEditPause)
}
function done(): void {
  if (idle) clearTimeout(idle)
  idle = null
  if (!txnOpen) return
  txnOpen = false
  store.engine.value.commit()
  store.refresh()
}

const single = computed<DocNode | null>(() => {
  void s.revision
  return s.selection.length === 1 ? store.node(s.selection[0]) : null
})

const layer = computed(() => (single.value?.type === 'layer' ? (single.value as LayerNode) : null))
const group = computed(() => (single.value?.type === 'group' ? (single.value as GroupNode) : null))

/** The line segment whose style the panel edits (a segment, or the first segment of a line group). */
const lineSeg = computed<{ id: Id; seg: LineSegment } | null>(() => {
  if (layer.value?.content.kind === 'line') return { id: layer.value.id, seg: layer.value.content }
  const g = group.value
  if (g?.kind.role === 'line') {
    const find = (nodes: DocNode[]): LayerNode | null => {
      for (const n of nodes) {
        if (n.type === 'layer' && n.content.kind === 'line') return n
        if (n.type === 'group') {
          const f = find(n.children)
          if (f) return f
        }
      }
      return null
    }
    const l = find(g.children)
    if (l && l.content.kind === 'line') return { id: l.id, seg: l.content }
  }
  return null
})

const lineSegmentCount = computed(() => {
  const ls = lineSeg.value
  if (!ls) return 0
  let n = 0
  const walk = (items: typeof s.tree) =>
    items.forEach((it) => {
      if (it.kind === 'line') {
        const node = store.node(it.id)
        if (node?.type === 'layer' && node.content.kind === 'line' && node.content.line_id === ls.seg.line_id) n++
      }
      walk(it.children)
    })
  walk(s.tree)
  return n
})

/**
 * A selected segment changes on its own (its color is shared by the whole
 * line); a selected line group changes every one of its segments.
 */
function setStyle<K extends keyof LineStyle>(key: K, value: LineStyle[K]): void {
  const ls = lineSeg.value
  if (!ls) return
  if (layer.value) live((e) => e.updateContent(ls.id, { style: { [key]: value } }))
  else live((e) => e.updateLineStyle(ls.seg.line_id, { [key]: value }))
}

/** Give every segment of the line the selected segment's style. */
function applyStyleToLine(): void {
  const ls = lineSeg.value
  if (ls) store.run((e) => e.updateLineStyle(ls.seg.line_id, ls.seg.style))
}

function patch(obj: Record<string, unknown>): void {
  const l = layer.value
  if (l) live((e) => e.updateContent(l.id, obj))
}

/** Formatting of the selected text (of the selected characters while it is edited). */
const textFmt = computed(() => {
  void s.revision
  void s.textEdit?.selection
  void s.textEdit?.version
  const l = layer.value
  return l?.content.kind === 'text' ? store.textFormat(l.id) : null
})

/** While the text is edited, format the selection; otherwise all of the text (one undo step while scrubbing). */
function setText(key: 'font_family' | 'size' | 'color' | 'bold' | 'italic' | 'align' | 'direction', value: unknown): void {
  const l = layer.value
  if (!l) return
  if (s.textEdit?.id === l.id) store.formatText({ [key]: value })
  else live((e) => e.updateContent(l.id, { [key]: value }))
}

function setStationKind(kind: StationKind): void {
  const l = layer.value
  if (l) store.run((e) => e.setStationKind(l.id, kind))
}

function setSetting<K extends keyof DocSettings>(key: K, value: DocSettings[K]): void {
  live((e) => e.setSettings({ ...s.settings, [key]: value }))
}

/** X / Y of the selection in cells (an object's anchor, see `Engine.position`). */
const position = computed(() => {
  void s.revision
  return s.selection.length ? store.engine.value.position(s.selection) : null
})
const positionStep = computed(() => {
  void s.revision
  return selectionOnGrid(store) ? 1 : 0.1
})
function setPosition(axis: 'x' | 'y', value: number): void {
  live((e) => moveSelectionTo(e, store, axis, value))
}

/** The selected shape, if a shape is selected. */
const shape = computed(() => (layer.value?.content.kind === 'shape' ? layer.value.content : null))

const kindLabel = computed(() => {
  if (group.value) {
    return { generic: 'Group', line: 'Transit line', station: 'Station group' }[group.value.kind.role]
  }
  if (shape.value) return { line: 'Straight line', rect: 'Rectangle', circle: 'Circle' }[shape.value.shape]
  const k = layer.value?.content.kind
  return k ? { line: 'Line segment', station: 'Station', text: 'Text', icon: 'Pictogram', shape: 'Shape' }[k] : ''
})

const coupledLines = computed(() => {
  const c = layer.value?.content
  if (c?.kind !== 'station') return ''
  const names: string[] = []
  const walk = (items: typeof s.tree) =>
    items.forEach((it) => {
      if (it.kind === 'group' && it.role === 'line') {
        const n = store.node(it.id)
        if (n?.type === 'group' && n.kind.role === 'line' && c.line_ids.includes(n.kind.line_id)) names.push(it.name)
      }
      walk(it.children)
    })
  walk(s.tree)
  return names.join(', ') || '—'
})

const toggleLocked = () => single.value && store.run((e) => e.setLocked(single.value!.id, !single.value!.locked))
const toggleVisible = () => single.value && store.run((e) => e.setVisible(single.value!.id, !single.value!.visible))
const childCount = computed(() => (group.value ? (findItem(s.tree, group.value.id)?.children.length ?? 0) : 0))
</script>


<template>
  <!--
    One grid for the whole panel: label | input | the rest. Number fields
    place their label, input and unit in those columns; other rows (.prop)
    put a label in the first column and their controls across the others,
    so every value starts at the same place.
  -->
  <div class="props" data-testid="properties-panel">
    <!-- nothing selected: document -->
    <template v-if="s.selection.length === 0">
      <div class="section-title">Document</div>
      <NumberField :model-value="s.settings.line_width" label="Line width" :min="0.05" :max="4" :step="0.05" unit="cells" :help="HELP['settings.line_width']" @update:model-value="(v) => setSetting('line_width', v)" @commit="done" />
      <NumberField :model-value="s.settings.corner_radius" label="Curve radius" :min="0" :max="50" :step="0.25" unit="cells" :help="HELP['settings.corner_radius']" @update:model-value="(v) => setSetting('corner_radius', v)" @commit="done" />
      <NumberField :model-value="s.settings.cap_roundness" label="Cap roundness" :min="0" :max="1" :step="0.05" :help="HELP['settings.cap_roundness']" @update:model-value="(v) => setSetting('cap_roundness', v)" @commit="done" />
      <NumberField :model-value="s.settings.station_outer_radius" label="Station radius" :min="0.05" :max="10" :step="0.02" unit="cells" :help="HELP['settings.station_outer_radius']" @update:model-value="(v) => setSetting('station_outer_radius', v)" @commit="done" />
      <NumberField :model-value="s.settings.station_inner_radius" label="Station fill" :min="0" :max="10" :step="0.02" unit="cells" :help="HELP['settings.station_inner_radius']" @update:model-value="(v) => setSetting('station_inner_radius', v)" @commit="done" />
      <button class="ps-btn wide start" @click="s.dialog = 'document'">All document settings…</button>
      <p class="dim tip wide">Select a layer on the canvas or in the Layers panel to edit its properties.</p>
    </template>

    <!-- multiple selection -->
    <template v-else-if="s.selection.length > 1">
      <div class="section-title">{{ s.selection.length }} items selected</div>
      <div class="wide buttons">
        <button class="ps-btn" title="Rotate 45° counter clockwise" @click="rotateSelection(store, -45)"><UiIcon name="rotate-ccw" :size="14" /></button>
        <button class="ps-btn" title="Rotate 45° clockwise" @click="rotateSelection(store, 45)"><UiIcon name="rotate-cw" :size="14" /></button>
        <button class="ps-btn" @click="store.run((e) => e.group(s.selection))">Group</button>
      </div>
      <div v-if="position" class="rows" data-testid="prop-position">
        <NumberField :model-value="position.x" label="X" :step="positionStep" :precision="1" unit="cells" :help="HELP.position" @update:model-value="(v) => setPosition('x', v)" @commit="done" />
        <NumberField :model-value="position.y" label="Y" :step="positionStep" :precision="1" unit="cells" :help="HELP.position" @update:model-value="(v) => setPosition('y', v)" @commit="done" />
      </div>
    </template>

    <!-- single node -->
    <template v-else-if="single">
      <div class="header-row wide">
        <span class="kind">{{ kindLabel }}</span>
        <span class="name" :title="single.name">{{ single.name }}</span>
        <button class="icon-btn" :title="single.visible ? 'Hide' : 'Show'" @click="toggleVisible">
          <UiIcon :name="single.visible ? 'eye' : 'eye-off'" :size="15" />
        </button>
        <button class="icon-btn" :class="{ on: single.locked }" :title="single.locked ? 'Unlock' : 'Lock'" @click="toggleLocked">
          <UiIcon :name="single.locked ? 'lock' : 'unlock'" :size="14" />
        </button>
      </div>

      <div v-if="group" class="dim small wide">{{ childCount }} item{{ childCount === 1 ? '' : 's' }} in the group</div>
      <div v-if="position" class="rows" data-testid="prop-position">
        <NumberField :model-value="position.x" label="X" :step="positionStep" :precision="1" unit="cells" :help="HELP.position" @update:model-value="(v) => setPosition('x', v)" @commit="done" />
        <NumberField :model-value="position.y" label="Y" :step="positionStep" :precision="1" unit="cells" :help="HELP.position" @update:model-value="(v) => setPosition('y', v)" @commit="done" />
      </div>

      <!-- line style (segment or line group) -->
      <template v-if="lineSeg">
        <div class="section-title">
          {{ layer ? 'Segment appearance' : 'Line appearance' }}<HelpTip :text="HELP['line.style_scope']" />
          <span class="count dim">{{ layer ? (lineSegmentCount === 1 ? 'The only segment' : `1 of ${lineSegmentCount} segments`) : `${lineSegmentCount} segment${lineSegmentCount === 1 ? '' : 's'}` }}</span>
        </div>
        <div class="prop">
          <span class="lbl">Color<HelpTip :text="HELP['line.color']" /></span>
          <div class="ctl"><ColorField :model-value="lineSeg.seg.style.color" @update:model-value="(v) => setStyle('color', v)" /></div>
        </div>
        <div class="prop">
          <span class="lbl">Style<HelpTip :text="HELP['line.kind']" /></span>
          <div class="ctl">
            <select :value="lineSeg.seg.style.kind" data-testid="prop-line-style" @change="setStyle('kind', ($event.target as HTMLSelectElement).value as StrokeKind); done()">
              <option v-for="k in STROKE_KINDS" :key="k.value" :value="k.value">{{ k.label }}</option>
            </select>
          </div>
        </div>
        <template v-if="hasChannel(lineSeg.seg.style.kind)">
          <div class="prop">
            <span class="lbl">Channel<HelpTip :text="HELP['line.channel']" /></span>
            <div class="ctl"><ColorField :model-value="lineSeg.seg.style.inner_color" @update:model-value="(v) => setStyle('inner_color', v)" /></div>
          </div>
          <NumberField :model-value="lineSeg.seg.style.inner_width_pct" label="Channel width" :min="0" :max="100" :precision="0" unit="%" :help="HELP['line.channel_width']" @update:model-value="(v) => setStyle('inner_width_pct', v)" @commit="done" />
        </template>
        <template v-if="hasDashes(lineSeg.seg.style.kind)">
          <div class="prop">
            <span class="lbl">Dashes<HelpTip :text="HELP['line.dash_color']" /></span>
            <div class="ctl"><ColorField :model-value="lineSeg.seg.style.inner_color" @update:model-value="(v) => setStyle('inner_color', v)" /></div>
          </div>
          <NumberField :model-value="lineSeg.seg.style.dash_width_pct" label="Dash width" :min="0" :max="100" :precision="0" unit="%" :help="HELP['line.dash_width']" @update:model-value="(v) => setStyle('dash_width_pct', v)" @commit="done" />
          <NumberField :model-value="lineSeg.seg.style.dash_length" label="Dash length" :min="0.1" :step="0.1" :precision="1" unit="cells" :help="HELP['line.dash_length']" @update:model-value="(v) => setStyle('dash_length', v)" @commit="done" />
          <NumberField :model-value="lineSeg.seg.style.dash_gap" label="Gap" :min="0" :step="0.1" :precision="1" unit="cells" :help="HELP['line.dash_gap']" @update:model-value="(v) => setStyle('dash_gap', v)" @commit="done" />
          <NumberField :model-value="lineSeg.seg.style.dash_start_offset" label="Start offset" :min="0" :step="0.5" :precision="1" unit="cells" :help="HELP['line.dash_start_offset']" data-testid="dash-start-offset" @update:model-value="(v) => setStyle('dash_start_offset', v)" @commit="done" />
          <NumberField :model-value="lineSeg.seg.style.dash_end_offset" label="End offset" :min="0" :step="0.5" :precision="1" unit="cells" :help="HELP['line.dash_end_offset']" data-testid="dash-end-offset" @update:model-value="(v) => setStyle('dash_end_offset', v)" @commit="done" />
        </template>
        <div class="prop">
          <span class="lbl">Start cap<HelpTip :text="HELP['line.caps']" /></span>
          <div class="ctl">
            <select :value="lineSeg.seg.style.start_cap" data-testid="start-cap" @change="setStyle('start_cap', ($event.target as HTMLSelectElement).value as 'butt' | 'round')">
              <option value="round">Round</option>
              <option value="butt">Butt</option>
            </select>
          </div>
        </div>
        <div class="prop">
          <span class="lbl">End cap<HelpTip :text="HELP['line.caps']" /></span>
          <div class="ctl">
            <select :value="lineSeg.seg.style.end_cap" data-testid="end-cap" @change="setStyle('end_cap', ($event.target as HTMLSelectElement).value as 'butt' | 'round')">
              <option value="round">Round</option>
              <option value="butt">Butt</option>
            </select>
          </div>
        </div>
        <div v-if="layer" class="prop">
          <span class="lbl">Points<HelpTip :text="HELP['line.points']" /></span>
          <div class="ctl value">{{ lineSeg.seg.points.map((p) => `(${p.x}, ${p.y})`).join(' → ') }}</div>
        </div>
        <button v-if="layer && lineSegmentCount > 1" class="ps-btn wide start apply-line" data-testid="apply-style-to-line" @click="applyStyleToLine">
          Apply this style to the whole line<HelpTip :text="HELP['line.apply_to_line']" />
        </button>
      </template>

      <!-- station -->
      <template v-if="layer && layer.content.kind === 'station'">
        <div class="section-title">Station</div>
        <div class="prop">
          <span class="lbl">Shape<HelpTip :text="HELP['station.shape']" /></span>
          <div class="ctl">
            <Segmented
              :model-value="layer.content.shape"
              :options="[
                { value: 'dot', label: 'Dot' },
                { value: 'tick', label: 'Tick' },
                { value: 'pill', label: 'Pill' },
              ]"
              @update:model-value="setStationKind"
            />
          </div>
        </div>
        <template v-if="layer.content.shape === 'dot' || layer.content.shape === 'pill'">
          <div class="prop">
            <span class="lbl">Border<HelpTip :text="HELP['station.outer_color']" /></span>
            <div class="ctl"><ColorField :model-value="layer.content.outer_color" @update:model-value="(v) => patch({ outer_color: v })" /></div>
          </div>
          <div class="prop">
            <span class="lbl">Fill<HelpTip :text="HELP['station.inner_color']" /></span>
            <div class="ctl"><ColorField :model-value="layer.content.inner_color" @update:model-value="(v) => patch({ inner_color: v })" /></div>
          </div>
        </template>
        <template v-else-if="layer.content.shape === 'tick'">
          <div class="prop">
            <span class="lbl">Color<HelpTip :text="HELP['station.tick_color']" /></span>
            <div class="ctl"><ColorField :model-value="layer.content.color" @update:model-value="(v) => patch({ color: v })" /></div>
          </div>
          <NumberField :model-value="layer.content.width" label="Width" :min="0.01" :step="0.05" unit="cells" :help="HELP['station.tick_width']" @update:model-value="(v) => patch({ width: v })" @commit="done" />
          <NumberField :model-value="layer.content.length" label="Height" :min="0.01" :step="0.05" unit="cells" :help="HELP['station.tick_length']" @update:model-value="(v) => patch({ length: v })" @commit="done" />
          <div class="prop">
            <span class="lbl">Side<HelpTip :text="HELP['station.tick_side']" /></span>
            <div class="ctl">
              <Segmented
                :model-value="layer.content.side"
                :options="[
                  { value: 1, label: 'Left' },
                  { value: -1, label: 'Right' },
                ]"
                @update:model-value="(v) => patch({ side: v })"
              />
            </div>
          </div>
        </template>
        <div class="prop">
          <span class="lbl">Lines<HelpTip :text="HELP['station.coupled']" /></span>
          <div class="ctl value">{{ coupledLines }}</div>
        </div>
      </template>

      <!-- text -->
      <div v-if="layer && layer.content.kind === 'text' && textFmt" class="rows" data-keeps-text-edit>
        <div class="section-title">Character</div>
        <div class="prop">
          <span class="lbl">Text<HelpTip :text="HELP['text.edit']" /></span>
          <div class="ctl">
            <button v-if="s.textEdit?.id !== layer.id" class="ps-btn" data-testid="prop-edit-text" @click="store.startTextEdit(layer.id)">Edit text</button>
            <span v-else class="dim small">Editing on the canvas: changes apply to the selected characters.</span>
          </div>
        </div>
        <div class="prop">
          <span class="lbl">Font<HelpTip :text="HELP['text.font']" /></span>
          <div class="ctl">
            <FontSelect :model-value="textFmt.font_family" @update:model-value="(v) => (setText('font_family', v), done())" />
            <button class="icon-btn" :class="{ on: textFmt.bold }" title="Bold" data-testid="prop-text-bold" @mousedown.prevent @click="setText('bold', !textFmt.bold); done()">
              <UiIcon name="bold" :size="15" />
            </button>
            <button class="icon-btn" :class="{ on: textFmt.italic }" title="Italic" @mousedown.prevent @click="setText('italic', !textFmt.italic); done()">
              <UiIcon name="italic" :size="15" />
            </button>
          </div>
        </div>
        <NumberField :model-value="textFmt.size" label="Size" :min="0.05" :step="0.05" unit="cells" :help="HELP['text.size']" data-testid="prop-text-size" @update:model-value="(v) => setText('size', v)" @commit="done" />
        <div class="prop">
          <span class="lbl">Color<HelpTip :text="HELP['text.color']" /></span>
          <div class="ctl"><ColorField :model-value="textFmt.color" @update:model-value="(v) => setText('color', v)" /></div>
        </div>
        <div class="prop">
          <span class="lbl">Align<HelpTip :text="HELP['text.align']" /></span>
          <div class="ctl">
            <Segmented
              :model-value="textFmt.align"
              data-testid="prop-text-align"
              :options="[
                { value: 'start', icon: 'align-start', title: 'Left' },
                { value: 'middle', icon: 'align-middle', title: 'Center' },
                { value: 'end', icon: 'align-end', title: 'Right' },
              ]"
              @mousedown.prevent
              @update:model-value="(v) => (setText('align', v), done())"
            />
          </div>
        </div>
        <div class="prop">
          <span class="lbl">Direction<HelpTip :text="HELP['text.direction']" /></span>
          <div class="ctl">
            <Segmented
              :model-value="textFmt.direction"
              data-testid="prop-text-direction"
              :options="[
                { value: 'ltr', icon: 'dir-ltr', title: 'Left to right' },
                { value: 'rtl', icon: 'dir-rtl', title: 'Right to left' },
              ]"
              @mousedown.prevent
              @update:model-value="(v) => (setText('direction', v), done())"
            />
          </div>
        </div>
        <NumberField :model-value="layer.content.rotation" label="Angle" :min="-180" :max="180" :step="45" :precision="1" unit="degrees" :help="HELP['text.rotation']" @update:model-value="(v) => patch({ rotation: v })" @commit="done" />
      </div>

      <!-- shape -->
      <template v-if="shape">
        <div class="section-title">Appearance</div>
        <template v-if="shape.shape === 'line'">
          <div class="prop">
            <span class="lbl">Color<HelpTip :text="HELP['shape.line_color']" /></span>
            <div class="ctl"><ColorField :model-value="shape.color" @update:model-value="(v) => patch({ color: v })" /></div>
          </div>
          <NumberField :model-value="shape.thickness" label="Width" :min="0.01" :step="0.05" unit="cells" :help="HELP['shape.thickness']" data-testid="prop-shape-thickness" @update:model-value="(v) => patch({ thickness: v })" @commit="done" />
          <div class="prop">
            <span class="lbl">Caps<HelpTip :text="HELP['shape.cap']" /></span>
            <div class="ctl">
              <select :value="shape.cap" data-testid="prop-shape-cap" @change="patch({ cap: ($event.target as HTMLSelectElement).value }); done()">
                <option value="round">Round</option>
                <option value="butt">Butt</option>
              </select>
            </div>
          </div>
        </template>
        <div v-else class="prop">
          <span class="lbl">Fill<HelpTip :text="HELP['shape.fill']" /></span>
          <div class="ctl">
            <input type="checkbox" :checked="shape.filled" title="Fill the inside" data-testid="prop-shape-filled" @change="patch({ filled: ($event.target as HTMLInputElement).checked }); done()" />
            <ColorField v-if="shape.filled" :model-value="shape.color" @update:model-value="(v) => patch({ color: v })" />
          </div>
        </div>
        <div class="prop">
          <span class="lbl">Border<HelpTip :text="HELP['shape.border_color']" /></span>
          <div class="ctl"><ColorField :model-value="shape.border_color" @update:model-value="(v) => patch({ border_color: v })" /></div>
        </div>
        <NumberField :model-value="shape.border_width" label="Border width" :min="0" :step="0.05" unit="cells" :help="HELP['shape.border_width']" @update:model-value="(v) => patch({ border_width: v })" @commit="done" />
        <template v-if="shape.shape === 'line'">
          <div class="prop">
            <span class="lbl">Dashed<HelpTip :text="HELP['shape.dashed']" /></span>
            <div class="ctl">
              <input type="checkbox" :checked="shape.dashed" title="Draw the line as dashes" data-testid="prop-shape-dashed" @change="patch({ dashed: ($event.target as HTMLInputElement).checked }); done()" />
            </div>
          </div>
          <template v-if="shape.dashed">
            <NumberField :model-value="shape.dash_length" label="Dash length" :min="0.05" :step="0.1" unit="cells" :help="HELP['shape.dash_length']" @update:model-value="(v) => patch({ dash_length: v })" @commit="done" />
            <NumberField :model-value="shape.dash_gap" label="Gap" :min="0" :step="0.1" unit="cells" :help="HELP['shape.dash_gap']" @update:model-value="(v) => patch({ dash_gap: v })" @commit="done" />
          </template>
          <div class="section-title">Ends</div>
          <div class="rows" data-testid="prop-shape-ends">
            <NumberField :model-value="shape.from.x" label="Start X" :step="1" :precision="1" unit="cells" :help="HELP['shape.ends']" @update:model-value="(v) => patch({ from: { x: v } })" @commit="done" />
            <NumberField :model-value="shape.from.y" label="Start Y" :step="1" :precision="1" unit="cells" :help="HELP['shape.ends']" @update:model-value="(v) => patch({ from: { y: v } })" @commit="done" />
            <NumberField :model-value="shape.to.x" label="End X" :step="1" :precision="1" unit="cells" :help="HELP['shape.ends']" @update:model-value="(v) => patch({ to: { x: v } })" @commit="done" />
            <NumberField :model-value="shape.to.y" label="End Y" :step="1" :precision="1" unit="cells" :help="HELP['shape.ends']" @update:model-value="(v) => patch({ to: { y: v } })" @commit="done" />
          </div>
        </template>
        <div v-else-if="shape.shape === 'rect'" class="rows" data-testid="prop-shape-size">
          <div class="section-title">Size</div>
          <NumberField :model-value="shape.width" label="Width" :min="0.1" :step="1" :precision="1" unit="cells" :help="HELP['shape.width']" @update:model-value="(v) => patch({ width: v })" @commit="done" />
          <NumberField :model-value="shape.height" label="Height" :min="0.1" :step="1" :precision="1" unit="cells" :help="HELP['shape.height']" @update:model-value="(v) => patch({ height: v })" @commit="done" />
          <NumberField :model-value="shape.rotation" label="Angle" :min="-180" :max="180" :step="45" :precision="1" unit="degrees" :help="HELP['shape.rotation']" @update:model-value="(v) => patch({ rotation: v })" @commit="done" />
        </div>
        <div v-else class="rows" data-testid="prop-shape-size">
          <div class="section-title">Size</div>
          <NumberField :model-value="shape.radius" label="Radius" :min="0.1" :step="0.1" :precision="1" unit="cells" :help="HELP['shape.radius']" @update:model-value="(v) => patch({ radius: v })" @commit="done" />
        </div>
      </template>

      <!-- icon -->
      <template v-if="layer && layer.content.kind === 'icon'">
        <div class="section-title">Pictogram</div>
        <div class="prop">
          <span class="lbl">Symbol<HelpTip :text="HELP['icon.icon']" /></span>
          <div class="ctl"><IconPicker :model-value="layer.content.icon" /></div>
        </div>
        <div class="prop">
          <span class="lbl">Symbol color<HelpTip :text="HELP['icon.color']" /></span>
          <div class="ctl"><ColorField :model-value="layer.content.color" @update:model-value="(v) => patch({ color: v })" /></div>
        </div>
        <div class="prop">
          <span class="lbl">Tile color<HelpTip :text="HELP['icon.background']" /></span>
          <div class="ctl"><ColorField :model-value="layer.content.background" @update:model-value="(v) => patch({ background: v })" /></div>
        </div>
        <NumberField :model-value="layer.content.size" label="Size" :min="0.1" :step="0.1" unit="cells" :help="HELP['icon.size']" @update:model-value="(v) => patch({ size: v })" @commit="done" />
        <NumberField :model-value="layer.content.rotation" label="Angle" :min="-180" :max="180" :step="45" :precision="1" unit="degrees" :help="HELP['icon.rotation']" @update:model-value="(v) => patch({ rotation: v })" @commit="done" />
      </template>
    </template>
  </div>
</template>

<style scoped>
/* label | input | the rest (units, or controls wider than an input) */
.props {
  padding: 8px 10px 12px;
  display: grid;
  grid-template-columns: max-content auto minmax(0, 1fr);
  column-gap: 8px;
  row-gap: 7px;
  align-items: center;
}
/* Groups of rows only exist for tests and focus handling; their rows join the grid. */
.rows,
.prop,
.props > :deep(.num-field),
.rows > :deep(.num-field) {
  display: contents;
}
.props :deep(.num-field .lead),
.lbl {
  grid-column: 1;
  display: inline-flex;
  align-items: center;
  color: var(--c-text-dim);
}
.props :deep(.num-field input) {
  grid-column: 2;
  width: 56px !important;
}
.props :deep(.num-field .unit) {
  grid-column: 3;
}
.ctl {
  grid-column: 2 / -1;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  min-width: 0;
}
.value {
  font-size: var(--fs-small);
  line-height: 1.5;
  word-break: break-word;
  color: var(--c-text);
}
.section-title,
.wide,
.header-row {
  grid-column: 1 / -1;
}
.section-title {
  display: flex;
  align-items: center;
  margin-top: 6px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--c-border-soft);
  font-weight: 600;
  color: var(--c-text-bright);
}
.count {
  margin-left: auto;
  font-weight: normal;
  font-size: var(--fs-small);
}
.header-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.kind {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 2px 6px;
  border-radius: 3px;
  background: #2a2a2a;
  color: var(--c-accent);
  flex: none;
}
.name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.buttons {
  display: flex;
  gap: 6px;
}
.small {
  font-size: var(--fs-small);
  line-height: 1.5;
}
.start {
  justify-self: start;
}
.apply-line {
  display: inline-flex;
  align-items: center;
}
.tip {
  margin: 4px 0 0;
  line-height: 1.5;
}
</style>
