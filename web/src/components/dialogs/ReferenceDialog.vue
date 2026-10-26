<script setup lang="ts">
import ModalDialog from './ModalDialog.vue'
import { ACTIONS, ACTION_IDS } from '../../core/actions'
import { useEditor } from '../../stores/editor'
import { prefs } from '../../stores/prefs'

const store = useEditor()
const gestures = [
  ['Ctrl + Scroll', 'Zoom at cursor'],
  ['Scroll / Shift + Scroll', 'Pan vertically / horizontally'],
  ['Space + drag, middle drag', 'Pan with the hand tool'],
  ['Long press pen slot', 'Choose Line Pen / Erase tool'],
  ['Click selected object', 'Toggle rotate handles'],
  ['Shift while rotating', 'Snap to 15°'],
  ['Alt while moving', 'Move off the snap grid'],
  ['Double click text', 'Edit text in place'],
]
</script>

<template>
  <ModalDialog title="Shortcut Reference" :width="520" ok-label="Close" :cancel="false" @ok="store.state.dialog = null">
    <div class="cols">
      <table>
        <tr v-for="id in ACTION_IDS.filter((i) => prefs.shortcuts[i])" :key="id">
          <td>{{ ACTIONS[id].label }}</td>
          <td class="k">{{ prefs.shortcuts[id] }}</td>
        </tr>
      </table>
      <table>
        <tr v-for="[k, v] in gestures" :key="k">
          <td>{{ v }}</td>
          <td class="k">{{ k }}</td>
        </tr>
      </table>
    </div>
  </ModalDialog>
</template>

<style scoped>
.cols {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
table {
  width: 100%;
  border-collapse: collapse;
}
td {
  padding: 3px 4px;
  border-bottom: 1px solid #333;
}
.k {
  text-align: right;
  font-family: ui-monospace, Consolas, monospace;
  color: var(--c-accent);
  font-size: var(--fs-small);
}
</style>
