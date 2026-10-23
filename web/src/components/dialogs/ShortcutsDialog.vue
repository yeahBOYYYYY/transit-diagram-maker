<script setup lang="ts">
// Keyboard shortcut editor: click a binding, press the new combination.
import { computed, reactive, ref } from 'vue'
import ModalDialog from './ModalDialog.vue'
import { ACTIONS, ACTION_IDS, DEFAULT_SHORTCUTS, type ActionId } from '../../core/actions'
import { eventToCombo, findConflicts } from '../../core/shortcuts'
import { useEditor } from '../../stores/editor'
import { prefs } from '../../stores/prefs'

const store = useEditor()
const draft = reactive<Record<ActionId, string>>({ ...prefs.shortcuts })
const recording = ref<ActionId | null>(null)
const filter = ref('')

const categories = computed(() => {
  const q = filter.value.toLowerCase()
  const out = new Map<string, ActionId[]>()
  for (const id of ACTION_IDS) {
    const a = ACTIONS[id]
    if (q && !a.label.toLowerCase().includes(q) && !draft[id].toLowerCase().includes(q)) continue
    out.set(a.category, [...(out.get(a.category) ?? []), id])
  }
  return [...out]
})

const conflicts = computed(() => {
  const set = new Set<ActionId>()
  for (const ids of findConflicts(draft).values()) ids.forEach((i) => set.add(i as ActionId))
  return set
})

function onKey(id: ActionId, e: KeyboardEvent): void {
  e.preventDefault()
  e.stopPropagation()
  if (e.key === 'Escape') {
    recording.value = null
    return
  }
  const combo = eventToCombo(e)
  if (!combo) return
  draft[id] = combo
  recording.value = null
}

function ok(): void {
  Object.assign(prefs.shortcuts, draft)
  store.state.dialog = null
}
</script>

<template>
  <ModalDialog title="Keyboard Shortcuts" :width="560" @ok="ok" @cancel="store.state.dialog = null">
    <div class="no-enter">
      <input v-model="filter" class="filter" placeholder="Search commands…" @keydown.stop />
      <div class="list">
        <template v-for="[cat, ids] in categories" :key="cat">
          <div class="cat">{{ cat }}</div>
          <div v-for="id in ids" :key="id" class="row" :class="{ conflict: conflicts.has(id) }">
            <span class="label">{{ ACTIONS[id].label }}</span>
            <button
              class="binding"
              :class="{ rec: recording === id }"
              :data-action="id"
              @click="recording = id"
              @keydown="recording === id && onKey(id, $event)"
              @blur="recording === id && (recording = null)"
            >
              {{ recording === id ? 'Press keys…' : draft[id] || '—' }}
            </button>
            <button class="icon-btn" title="Clear" @click="draft[id] = ''">✕</button>
            <button class="icon-btn" title="Reset" @click="draft[id] = DEFAULT_SHORTCUTS[id]">↺</button>
          </div>
        </template>
      </div>
      <p v-if="conflicts.size" class="warn">Highlighted shortcuts are assigned to more than one command.</p>
    </div>
    <template #extra>
      <button class="ps-btn" @click="Object.assign(draft, DEFAULT_SHORTCUTS)">Reset all</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.filter {
  width: 100%;
  height: 24px;
  padding: 0 8px;
  margin-bottom: 8px;
}
.list {
  max-height: 52vh;
  overflow: auto;
  border: 1px solid var(--c-border);
  background: var(--c-panel-alt);
}
.cat {
  padding: 6px 10px 4px;
  font-weight: 600;
  color: var(--c-text-bright);
  background: var(--c-panel-header);
  position: sticky;
  top: 0;
}
.row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px 2px 14px;
}
.row:hover {
  background: #3d3d3d;
}
.row.conflict .binding {
  border-color: var(--c-danger);
  color: var(--c-danger);
}
.label {
  flex: 1;
}
.binding {
  min-width: 150px;
  height: 22px;
  border: 1px solid var(--c-input-border);
  border-radius: 3px;
  background: var(--c-input);
  font-family: ui-monospace, Consolas, monospace;
  font-size: var(--fs-small);
  cursor: pointer;
}
.binding.rec {
  border-color: var(--c-accent);
  color: var(--c-accent);
}
.warn {
  color: var(--c-danger);
  margin: 8px 0 0;
}
</style>
