<script setup lang="ts">
// Properties of the current selection (or the document when nothing is selected).
import { computed } from 'vue'
import PanelFrame from './PanelFrame.vue'
import NumberField from '../ui/NumberField.vue'
import ColorField from '../ui/ColorField.vue'
import Segmented from '../ui/Segmented.vue'
import IconPicker from '../ui/IconPicker.vue'
import UiIcon from '../ui/UiIcon.vue'
import type { DocNode, DocSettings, GroupNode, Id, LayerNode, LineSegment, LineStyle, StationKind, StrokeKind } from '../../core/types'
import { STROKE_KINDS, hasChannel, hasDashes } from '../../core/lineStyles'
import type { Engine } from '../../core/engine'
import { findItem } from '../../core/tree'
import { useEditor } from '../../stores/editor'
import { FONT_FAMILIES } from '../../stores/prefs'
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
  idle = setTimeout(done, 600)
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

function setStyle<K extends keyof LineStyle>(key: K, value: LineStyle[K]): void {
  const ls = lineSeg.value
  if (ls) live((e) => e.updateContent(ls.id, { style: { [key]: value } }))
}

function patch(obj: Record<string, unknown>): void {
  const l = layer.value
  if (l) live((e) => e.updateContent(l.id, obj))
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

const kindLabel = computed(() => {
  if (group.value) {
    return { generic: 'Group', line: 'Transit line', station: 'Station group' }[group.value.kind.role]
  }
  const k = layer.value?.content.kind
  return k ? { line: 'Line segment', station: 'Station', text: 'Text', icon: 'Pictogram' }[k] : ''
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
const fontLabel = (f: string) => f.split(',')[0].replace(/"/g, '')
</script>

<template>
  <PanelFrame title="Properties">
    <div class="props" data-testid="properties-panel">
      <!-- nothing selected: document -->
      <template v-if="s.selection.length === 0">
        <div class="section-title">Document</div>
        <div class="grid2">
          <NumberField :model-value="s.settings.line_width" label="Line W" :min="0.05" :max="4" :step="0.05" unit="cells" @update:model-value="(v) => setSetting('line_width', v)" @commit="done" />
          <NumberField :model-value="s.settings.corner_radius" label="Curve" :min="0" :max="50" :step="0.25" unit="cells" @update:model-value="(v) => setSetting('corner_radius', v)" @commit="done" />
          <NumberField :model-value="s.settings.cap_roundness" label="Cap round" :min="0" :max="1" :step="0.05" @update:model-value="(v) => setSetting('cap_roundness', v)" @commit="done" />
          <NumberField :model-value="s.settings.station_outer_radius" label="Stn outer" :min="0.05" :max="10" :step="0.02" unit="cells" @update:model-value="(v) => setSetting('station_outer_radius', v)" @commit="done" />
          <NumberField :model-value="s.settings.station_inner_radius" label="Stn inner" :min="0" :max="10" :step="0.02" unit="cells" @update:model-value="(v) => setSetting('station_inner_radius', v)" @commit="done" />
        </div>
        <button class="ps-btn more" @click="s.dialog = 'document'">All document settings…</button>
        <p class="dim tip">Select a layer on the canvas or in the Layers panel to edit its properties.</p>
      </template>

      <!-- multiple selection -->
      <template v-else-if="s.selection.length > 1">
        <div class="section-title">{{ s.selection.length }} items selected</div>
        <div class="row">
          <button class="ps-btn" @click="rotateSelection(store, -45)"><UiIcon name="rotate-ccw" :size="14" /></button>
          <button class="ps-btn" @click="rotateSelection(store, 45)"><UiIcon name="rotate-cw" :size="14" /></button>
          <button class="ps-btn" @click="store.run((e) => e.group(s.selection))">Group</button>
        </div>
        <div v-if="position" class="grid2" data-testid="prop-position">
          <NumberField :model-value="position.x" label="X" :step="positionStep" :precision="1" unit="cells" title="Horizontal position in cells (an object's anchor; the top-left one for several)" @update:model-value="(v) => setPosition('x', v)" @commit="done" />
          <NumberField :model-value="position.y" label="Y" :step="positionStep" :precision="1" unit="cells" title="Vertical position in cells (an object's anchor; the top-left one for several)" @update:model-value="(v) => setPosition('y', v)" @commit="done" />
        </div>
      </template>

      <!-- single node -->
      <template v-else-if="single">
        <div class="header-row">
          <span class="kind">{{ kindLabel }}</span>
          <span class="name" :title="single.name">{{ single.name }}</span>
          <button class="icon-btn" :title="single.visible ? 'Hide' : 'Show'" @click="toggleVisible">
            <UiIcon :name="single.visible ? 'eye' : 'eye-off'" :size="15" />
          </button>
          <button class="icon-btn" :class="{ on: single.locked }" :title="single.locked ? 'Unlock' : 'Lock'" @click="toggleLocked">
            <UiIcon :name="single.locked ? 'lock' : 'unlock'" :size="14" />
          </button>
        </div>

        <div v-if="group" class="dim small">{{ childCount }} item{{ childCount === 1 ? '' : 's' }} in group</div>
        <div v-if="position" class="grid2" data-testid="prop-position">
          <NumberField :model-value="position.x" label="X" :step="positionStep" :precision="1" unit="cells" title="Horizontal position in cells (an object's anchor; the top-left one for several)" @update:model-value="(v) => setPosition('x', v)" @commit="done" />
          <NumberField :model-value="position.y" label="Y" :step="positionStep" :precision="1" unit="cells" title="Vertical position in cells (an object's anchor; the top-left one for several)" @update:model-value="(v) => setPosition('y', v)" @commit="done" />
        </div>

        <!-- line style (segment or line group) -->
        <template v-if="lineSeg">
          <div class="section-title">Line appearance <span class="dim small">· {{ lineSegmentCount }} segment{{ lineSegmentCount === 1 ? '' : 's' }}</span></div>
          <div class="field">
            <span class="lbl">Color</span>
            <ColorField :model-value="lineSeg.seg.style.color" @update:model-value="(v) => setStyle('color', v)" />
          </div>
          <div class="field">
            <span class="lbl">Style</span>
            <select :value="lineSeg.seg.style.kind" data-testid="prop-line-style" @change="setStyle('kind', ($event.target as HTMLSelectElement).value as StrokeKind); done()">
              <option v-for="k in STROKE_KINDS" :key="k.value" :value="k.value">{{ k.label }}</option>
            </select>
          </div>
          <template v-if="hasChannel(lineSeg.seg.style.kind)">
            <div class="field">
              <span class="lbl">Channel</span>
              <ColorField :model-value="lineSeg.seg.style.inner_color" @update:model-value="(v) => setStyle('inner_color', v)" />
              <NumberField :model-value="lineSeg.seg.style.inner_width_pct" label="Width" :min="0" :max="100" :precision="0" unit="% of line" title="Channel width as a percentage of the line width" @update:model-value="(v) => setStyle('inner_width_pct', v)" @commit="done" />
            </div>
          </template>
          <template v-if="hasDashes(lineSeg.seg.style.kind)">
            <div class="grid2">
              <NumberField :model-value="lineSeg.seg.style.dash_length" label="Dash" :min="0.1" :step="0.1" :precision="1" unit="cells" title="Length of each dash" @update:model-value="(v) => setStyle('dash_length', v)" @commit="done" />
              <NumberField :model-value="lineSeg.seg.style.dash_gap" label="Gap" :min="0" :step="0.1" :precision="1" unit="cells" title="Space between dashes" @update:model-value="(v) => setStyle('dash_gap', v)" @commit="done" />
              <NumberField v-if="lineSeg.seg.style.kind === 'hollow_dashed'" :model-value="lineSeg.seg.style.dash_width_pct" label="Dash W" :min="0" :max="100" :precision="0" unit="%" title="Dash width as a percentage of the line width" @update:model-value="(v) => setStyle('dash_width_pct', v)" @commit="done" />
            </div>
            <div class="dim small">Dash and gap are stretched slightly so the line starts and ends with a full dash.</div>
          </template>
          <div class="field">
            <span class="lbl">Caps</span>
            <select :value="lineSeg.seg.style.start_cap" title="Start cap" data-testid="start-cap" @change="setStyle('start_cap', ($event.target as HTMLSelectElement).value as 'butt' | 'round')">
              <option value="round">Round start</option>
              <option value="butt">Butt start</option>
            </select>
            <select :value="lineSeg.seg.style.end_cap" title="End cap" data-testid="end-cap" @change="setStyle('end_cap', ($event.target as HTMLSelectElement).value as 'butt' | 'round')">
              <option value="round">Round end</option>
              <option value="butt">Butt end</option>
            </select>
          </div>
          <div v-if="layer" class="dim small">
            Points: {{ lineSeg.seg.points.map((p) => `(${p.x}, ${p.y})`).join(' → ') }}
          </div>
        </template>

        <!-- station -->
        <template v-if="layer && layer.content.kind === 'station'">
          <div class="section-title">Station</div>
          <div class="field">
            <span class="lbl">Shape</span>
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
          <template v-if="layer.content.shape === 'dot' || layer.content.shape === 'pill'">
            <div class="field">
              <span class="lbl">Outer</span>
              <ColorField :model-value="layer.content.outer_color" @update:model-value="(v) => patch({ outer_color: v })" />
            </div>
            <div class="field">
              <span class="lbl">Inner</span>
              <ColorField :model-value="layer.content.inner_color" @update:model-value="(v) => patch({ inner_color: v })" />
            </div>
          </template>
          <template v-else-if="layer.content.shape === 'tick'">
            <div class="field">
              <span class="lbl">Color</span>
              <ColorField :model-value="layer.content.color" @update:model-value="(v) => patch({ color: v })" />
            </div>
            <div class="grid2">
              <NumberField :model-value="layer.content.width" label="Width" :min="0.01" :step="0.05" unit="cells" @update:model-value="(v) => patch({ width: v })" @commit="done" />
              <NumberField :model-value="layer.content.length" label="Height" :min="0.01" :step="0.05" unit="cells" @update:model-value="(v) => patch({ length: v })" @commit="done" />
            </div>
            <div class="field">
              <span class="lbl">Side</span>
              <Segmented
                :model-value="layer.content.side"
                :options="[
                  { value: 1, label: 'Left' },
                  { value: -1, label: 'Right' },
                ]"
                @update:model-value="(v) => patch({ side: v })"
              />
            </div>
          </template>
          <div class="dim small">Coupled to: {{ coupledLines }}</div>
        </template>

        <!-- text -->
        <template v-if="layer && layer.content.kind === 'text'">
          <div class="section-title">Character</div>
          <textarea
            class="text-input"
            rows="2"
            :value="layer.content.text"
            data-testid="prop-text"
            @keydown.stop
            @change="patch({ text: ($event.target as HTMLTextAreaElement).value }); done()"
          />
          <div class="field">
            <select :value="layer.content.font_family" @change="patch({ font_family: ($event.target as HTMLSelectElement).value }); done()">
              <option v-for="f in FONT_FAMILIES" :key="f" :value="f">{{ fontLabel(f) }}</option>
            </select>
            <button class="icon-btn" :class="{ on: layer.content.bold }" title="Bold" @click="patch({ bold: !(layer.content as any).bold }); done()">
              <UiIcon name="bold" :size="15" />
            </button>
            <button class="icon-btn" :class="{ on: layer.content.italic }" title="Italic" @click="patch({ italic: !(layer.content as any).italic }); done()">
              <UiIcon name="italic" :size="15" />
            </button>
          </div>
          <div class="field">
            <ColorField :model-value="layer.content.color" @update:model-value="(v) => patch({ color: v })" />
            <Segmented
              :model-value="layer.content.align"
              :options="[
                { value: 'start', icon: 'align-start', title: 'Left' },
                { value: 'middle', icon: 'align-middle', title: 'Center' },
                { value: 'end', icon: 'align-end', title: 'Right' },
              ]"
              @update:model-value="(v) => (patch({ align: v }), done())"
            />
          </div>
          <div class="grid2">
            <NumberField :model-value="layer.content.size" label="Size" :min="0.05" :step="0.05" unit="cells" @update:model-value="(v) => patch({ size: v })" @commit="done" />
            <NumberField :model-value="layer.content.rotation" label="Angle" :min="-180" :max="180" :step="45" :precision="0" unit="°" @update:model-value="(v) => patch({ rotation: v })" @commit="done" />
          </div>
        </template>

        <!-- icon -->
        <template v-if="layer && layer.content.kind === 'icon'">
          <div class="section-title">Pictogram</div>
          <IconPicker :model-value="layer.content.icon" @update:model-value="(v) => (patch({ icon: v }), done())" />
          <div class="field">
            <span class="lbl">Glyph</span>
            <ColorField :model-value="layer.content.color" @update:model-value="(v) => patch({ color: v })" />
          </div>
          <div class="field">
            <span class="lbl">Tile</span>
            <ColorField :model-value="layer.content.background" @update:model-value="(v) => patch({ background: v })" />
          </div>
          <div class="grid2">
            <NumberField :model-value="layer.content.size" label="Size" :min="0.1" :step="0.1" unit="cells" @update:model-value="(v) => patch({ size: v })" @commit="done" />
            <NumberField :model-value="layer.content.rotation" label="Angle" :min="-180" :max="180" :step="45" :precision="0" unit="°" @update:model-value="(v) => patch({ rotation: v })" @commit="done" />
          </div>
        </template>
      </template>
    </div>
  </PanelFrame>
</template>

<style scoped>
.props {
  padding: 8px 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.section-title {
  margin-top: 4px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--c-border-soft);
  font-weight: 600;
  color: var(--c-text-bright);
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
.field {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.lbl {
  width: 44px;
  color: var(--c-text-dim);
  flex: none;
}
.grid2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px 10px;
}
.grid2 :deep(.num-field) {
  justify-content: space-between;
}
.row {
  display: flex;
  gap: 6px;
}
.small {
  font-size: var(--fs-small);
  line-height: 1.5;
  word-break: break-word;
}
.text-input {
  width: 100%;
  resize: vertical;
  padding: 4px 6px;
}
.more {
  align-self: flex-start;
}
.tip {
  margin: 4px 0 0;
  line-height: 1.5;
}
</style>
