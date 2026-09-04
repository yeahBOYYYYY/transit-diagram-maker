<script setup lang="ts">
import { ref } from 'vue';
import { engineBridge } from '../engineBridge';
import { EditorStateResponse, ParameterSchema } from '../types';

const props = defineProps<{
  state: EditorStateResponse | null;
}>();

const activeTab = ref<'properties' | 'layers'>('properties');

function onParamChange(name: string, value: any) {
  const selectedId = props.state?.selected_node_ids[0];
  if (!selectedId) return;

  engineBridge.dispatch({
    type: 'SetNodeParameter',
    payload: {
      node_id: selectedId,
      param_name: name,
      value,
    },
  });
}

function selectNode(id: number, e: MouseEvent) {
  engineBridge.dispatch({
    type: 'SelectNode',
    payload: {
      node_id: id,
      multi: e.shiftKey,
    },
  });
}

function toggleVisibility(id: number, e: MouseEvent) {
  e.stopPropagation();
  engineBridge.dispatch({
    type: 'ToggleNodeVisibility',
    payload: id,
  });
}

function toggleLock(id: number, e: MouseEvent) {
  e.stopPropagation();
  engineBridge.dispatch({
    type: 'ToggleNodeLock',
    payload: id,
  });
}

function deleteSelected() {
  engineBridge.dispatch({ type: 'DeleteSelected' });
}
</script>

<template>
  <aside class="properties-panel">
    <div class="tab-headers">
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'properties' }"
        @click="activeTab = 'properties'"
      >
        <span>Properties</span>
      </button>
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'layers' }"
        @click="activeTab = 'layers'"
      >
        <span>Node Graph / Layers ({{ state?.node_summaries.length ?? 0 }})</span>
      </button>
    </div>

    <!-- Properties Tab -->
    <div v-if="activeTab === 'properties'" class="tab-body">
      <div v-if="state?.selected_parameters && state.selected_parameters.length > 0" class="property-list">
        <div class="panel-section-title">Contextual Properties</div>

        <div v-for="param in state.selected_parameters" :key="param.name" class="property-row">
          <label class="param-label">{{ param.label }}</label>

          <div class="param-control">
            <!-- Text parameter -->
            <input
              v-if="'Text' in param.param_type"
              type="text"
              :value="param.param_type.Text"
              @input="e => onParamChange(param.name, (e.target as HTMLInputElement).value)"
            />

            <!-- Color parameter -->
            <div v-else-if="'Color' in param.param_type" class="color-picker-wrapper">
              <input
                type="color"
                :value="param.param_type.Color"
                @input="e => onParamChange(param.name, (e.target as HTMLInputElement).value)"
              />
              <span class="color-hex">{{ param.param_type.Color }}</span>
            </div>

            <!-- Number parameter -->
            <div v-else-if="'Number' in param.param_type" class="number-control">
              <input
                type="range"
                :min="param.param_type.Number.min"
                :max="param.param_type.Number.max"
                :step="param.param_type.Number.step"
                :value="param.param_type.Number.value"
                @input="e => onParamChange(param.name, parseFloat((e.target as HTMLInputElement).value))"
              />
              <span class="num-val">{{ param.param_type.Number.value }}</span>
            </div>

            <!-- Choice parameter -->
            <select
              v-else-if="'Choice' in param.param_type"
              :value="param.param_type.Choice.current"
              @change="e => onParamChange(param.name, (e.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in param.param_type.Choice.options" :key="opt" :value="opt">
                {{ opt }}
              </option>
            </select>
          </div>
        </div>

        <div class="action-footer">
          <button class="delete-btn" @click="deleteSelected" title="Delete selected node (Del)">
            <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            <span>Delete Selected Node</span>
          </button>
        </div>
      </div>

      <div v-else class="empty-state">
        <svg viewBox="0 0 24 24" width="32" height="32" stroke="currentColor" stroke-width="1.5" fill="none" class="empty-icon">
          <path d="M15 15l6 6m-11-4a7 7 0 1 1 0-14 7 7 0 0 1 0 14z" />
        </svg>
        <p>No node selected</p>
        <span class="subtext">Click on any transit line, station, or text label to inspect and edit procedural parameters.</span>
      </div>
    </div>

    <!-- Node Graph / Layer Hierarchy Tab -->
    <div v-else class="tab-body">
      <div class="panel-section-title">Topological Node Stack</div>
      <div class="node-tree">
        <div
          v-for="node in state?.node_summaries.slice().reverse()"
          :key="node.id"
          class="node-row"
          :class="{ selected: state?.selected_node_ids.includes(node.id) }"
          @click="e => selectNode(node.id, e)"
        >
          <div class="node-info">
            <span class="type-badge">{{ node.type_name }}</span>
            <span class="node-name">{{ node.name }}</span>
          </div>

          <div class="node-controls">
            <button
              class="icon-btn"
              :class="{ inactive: !node.visible }"
              title="Toggle Visibility"
              @click="e => toggleVisibility(node.id, e)"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </button>

            <button
              class="icon-btn"
              :class="{ active: node.locked }"
              title="Toggle Lock"
              @click="e => toggleLock(node.id, e)"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                <path d="M7 11V7a5 5 0 0 1 10 0v4" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.properties-panel {
  width: var(--panel-width);
  background-color: var(--bg-panel);
  border-left: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  z-index: 50;
}

.tab-headers {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-subpanel);
}

.tab-btn {
  flex: 1;
  padding: 10px 8px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  border-bottom: 2px solid transparent;
  border-radius: 0;
}

.tab-btn:hover {
  color: var(--text-main);
  background: transparent;
}

.tab-btn.active {
  color: var(--accent-primary);
  border-bottom-color: var(--accent-primary);
  background: var(--bg-panel);
}

.tab-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px;
  display: flex;
  flex-direction: column;
}

.panel-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-dim);
  margin-bottom: 14px;
}

.property-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.property-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.param-label {
  font-size: 11px;
  color: var(--text-muted);
}

.param-control input[type="text"],
.param-control select {
  width: 100%;
}

.color-picker-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-picker-wrapper input[type="color"] {
  width: 32px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
}

.color-hex {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-muted);
}

.number-control {
  display: flex;
  align-items: center;
  gap: 10px;
}

.number-control input[type="range"] {
  flex: 1;
  accent-color: var(--accent-primary);
}

.num-val {
  font-family: var(--font-mono);
  font-size: 12px;
  min-width: 32px;
  text-align: right;
  color: var(--text-main);
}

.action-footer {
  margin-top: 20px;
  padding-top: 14px;
  border-top: 1px solid var(--border-color);
}

.delete-btn {
  width: 100%;
  padding: 8px;
  background: rgba(255, 69, 58, 0.1);
  border: 1px solid rgba(255, 69, 58, 0.3);
  color: var(--danger-color);
  font-weight: 500;
  gap: 6px;
}

.delete-btn:hover {
  background: rgba(255, 69, 58, 0.2);
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 30px 16px;
  color: var(--text-dim);
  gap: 8px;
}

.empty-icon {
  margin-bottom: 4px;
  opacity: 0.4;
}

.empty-state p {
  color: var(--text-muted);
  font-weight: 500;
}

.subtext {
  font-size: 11px;
  line-height: 1.4;
}

/* Node Tree */
.node-tree {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.node-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--bg-subpanel);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.15s ease;
}

.node-row:hover {
  border-color: var(--border-color);
}

.node-row.selected {
  border-color: var(--accent-primary);
  background: rgba(0, 122, 255, 0.1);
}

.node-info {
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
}

.type-badge {
  font-size: 9px;
  font-weight: 600;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  padding: 2px 5px;
  border-radius: 4px;
  color: var(--text-muted);
  white-space: nowrap;
}

.node-name {
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-main);
}

.node-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.icon-btn {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  color: var(--text-muted);
}

.icon-btn:hover {
  color: var(--text-main);
}

.icon-btn.inactive {
  opacity: 0.3;
}

.icon-btn.active {
  color: var(--accent-primary);
}
</style>
