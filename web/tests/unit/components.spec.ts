// Component level tests: render real components against a real engine.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import { freshStore } from './helpers'
import LayersPanel from '../../src/components/panels/LayersPanel.vue'
import PropertiesPanel from '../../src/components/panels/PropertiesPanel.vue'
import ToolsPanel from '../../src/components/ToolsPanel.vue'
import MenuBar from '../../src/components/MenuBar.vue'
import OptionsBar from '../../src/components/OptionsBar.vue'
import StatusBar from '../../src/components/StatusBar.vue'
import NumberField from '../../src/components/ui/NumberField.vue'
import ColorField from '../../src/components/ui/ColorField.vue'
import ShortcutsDialog from '../../src/components/dialogs/ShortcutsDialog.vue'
import DocumentSettingsDialog from '../../src/components/dialogs/DocumentSettingsDialog.vue'
import AboutDialog from '../../src/components/dialogs/AboutDialog.vue'
import { prefs } from '../../src/stores/prefs'

let wrappers: VueWrapper[] = []
function mountIt<T>(c: T, opts: Record<string, unknown> = {}) {
  const w = mount(c as never, { attachTo: document.body, ...opts })
  wrappers.push(w)
  return w
}
afterEach(() => {
  wrappers.forEach((w) => w.unmount())
  wrappers = []
})

describe('LayersPanel', () => {
  it('lists layers top first with nested groups', async () => {
    const store = freshStore()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 5, y: 0 }], {}))!
    store.run((e) => e.addStation('dot', { x: 2, y: 0 }, {}))
    store.run((e) => e.addText({ text: 'Title' }))
    const w = mountIt(LayersPanel)
    const names = w.findAll('.row .name').map((n) => n.text())
    expect(names[0]).toBe('Title')
    expect(names).toContain('Line 1')
    expect(names).toContain('Segment 1')
    expect(names).toContain('Station 1')
    // station group is collapsed by default, so its children are hidden
    expect(names.filter((n) => n === 'Station 1')).toHaveLength(1)
    await w.find(`[data-layer-id="${l.group_id}"]`).trigger('click')
    expect(store.state.selection).toEqual([l.group_id])
  })

  it('toggles visibility and lock from the row buttons', async () => {
    const store = freshStore()
    const t = store.run((e) => e.addText({}))!
    const w = mountIt(LayersPanel)
    await w.find(`[data-layer-id="${t}"] [data-testid=toggle-visible]`).trigger('click')
    expect(store.state.tree[0].visible).toBe(false)
    expect(store.state.fragments).toHaveLength(0)
    await w.find(`[data-layer-id="${t}"] [data-testid=toggle-lock]`).trigger('click')
    expect(store.state.tree[0].locked).toBe(true)
    store.undo()
    expect(store.state.tree[0].locked).toBe(false)
  })

  it('supports ctrl and shift multi selection', async () => {
    const store = freshStore()
    const ids = [1, 2, 3].map((i) => store.run((e) => e.addText({ text: `T${i}` }))!)
    const w = mountIt(LayersPanel)
    await w.find(`[data-layer-id="${ids[0]}"]`).trigger('click')
    await w.find(`[data-layer-id="${ids[2]}"]`).trigger('click', { shiftKey: true })
    expect(store.state.selection.sort()).toEqual([...ids].sort())
    await w.find(`[data-layer-id="${ids[1]}"]`).trigger('click', { ctrlKey: true })
    expect(store.state.selection.sort()).toEqual([ids[0], ids[2]].sort())
  })

  it('renames on double click', async () => {
    const store = freshStore()
    const t = store.run((e) => e.addText({}))!
    const w = mountIt(LayersPanel)
    await w.find(`[data-layer-id="${t}"] .name`).trigger('dblclick')
    const input = w.find('input.rename')
    await input.setValue('Harbour label')
    await input.trigger('keydown', { key: 'Enter' })
    expect(store.state.tree[0].name).toBe('Harbour label')
  })

  it('footer buttons group, ungroup and delete', async () => {
    const store = freshStore()
    const a = store.run((e) => e.addText({}))!
    const b = store.run((e) => e.addText({}))!
    store.select([a, b])
    const w = mountIt(LayersPanel)
    await w.find('[data-testid=group-btn]').trigger('click')
    expect(store.state.tree).toHaveLength(1)
    expect(store.state.tree[0].kind).toBe('group')
    await w.find('[data-testid=ungroup-btn]').trigger('click')
    expect(store.state.tree).toHaveLength(2)
    await w.find('[data-testid=delete-btn]').trigger('click')
    expect(store.state.tree).toHaveLength(0)
    expect(w.text()).toContain('No layers yet')
  })

  it('drag and drop moves layers into groups and reorders', async () => {
    const store = freshStore()
    const t = store.run((e) => e.addText({ text: 'Mover' }))!
    const g = store.run((e) => e.newGroup())!
    const w = mountIt(LayersPanel)
    const dt = { setData: vi.fn(), effectAllowed: '' }
    await w.find(`[data-layer-id="${t}"]`).trigger('dragstart', { dataTransfer: dt })
    const target = w.find(`[data-layer-id="${g}"]`)
    target.element.getBoundingClientRect = () => ({ top: 0, height: 30, left: 0, width: 100, bottom: 30, right: 100, x: 0, y: 0, toJSON: () => ({}) })
    await target.trigger('dragover', { clientY: 15 })
    await target.trigger('drop', { clientY: 15 })
    expect(store.state.tree).toHaveLength(1)
    expect(store.state.tree[0].children[0].id).toBe(t)
  })
})

describe('PropertiesPanel', () => {
  it('shows document settings without a selection', () => {
    freshStore()
    const w = mountIt(PropertiesPanel)
    expect(w.text()).toContain('Document')
    expect(w.text()).toContain('All document settings')
  })

  it('edits the style of a whole line from a segment', async () => {
    vi.useFakeTimers()
    const store = freshStore()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 9, y: 0 }], {}))!
    store.run((e) => e.erase([{ x: 4, y: 0 }]))
    store.select([l.layer_id])
    const w = mountIt(PropertiesPanel)
    expect(w.text()).toContain('Line segment')
    expect(w.text()).toContain('2 segments')
    await w.find('[data-testid=prop-line-style]').setValue('dashed')
    vi.advanceTimersByTime(1000)
    await nextTick()
    expect(store.state.fragments.filter((f) => f.svg.includes('stroke-dasharray'))).toHaveLength(2)
    expect(w.text()).toContain('Gap')
    expect(w.text()).not.toContain('Dash W')
    await w.find('[data-testid=prop-line-style]').setValue('hollow_dashed')
    await nextTick()
    expect(w.text()).toContain('Channel')
    expect(w.text()).toContain('Dash W')
    await w.find('[data-testid=end-cap]').setValue('butt')
    vi.advanceTimersByTime(1000)
    const n = store.node(l.layer_id)
    expect(n?.type === 'layer' && n.content.kind === 'line' && n.content.style.end_cap).toBe('butt')
    vi.useRealTimers()
  })

  it('switches station shape', async () => {
    const store = freshStore()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 9, y: 0 }], {}))
    const s = store.run((e) => e.addStation('dot', { x: 3, y: 0 }, {}))!
    store.select([s.station_id])
    const w = mountIt(PropertiesPanel)
    expect(w.text()).toContain('Coupled to: Line 1')
    await w.findAll('button[role=radio]').find((b) => b.text() === 'Tick')!.trigger('click')
    const n = store.node(s.station_id)
    expect(n?.type === 'layer' && n.content.kind === 'station' && n.content.shape).toBe('tick')
  })

  it('edits text content', async () => {
    const store = freshStore()
    const t = store.run((e) => e.addText({ text: 'Old' }))!
    store.select([t])
    const w = mountIt(PropertiesPanel)
    const ta = w.find('[data-testid=prop-text]')
    await ta.setValue('Riverside')
    await ta.trigger('change')
    const n = store.node(t)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.text).toBe('Riverside')
  })
})

describe('ToolsPanel', () => {
  it('activates tools and shows the pen / eraser flyout on long press', async () => {
    vi.useFakeTimers()
    const store = freshStore()
    const w = mountIt(ToolsPanel)
    await w.find('[data-tool=hand]').trigger('pointerdown')
    await w.find('[data-tool=hand]').trigger('pointerup')
    expect(store.state.tool).toBe('hand')
    const pen = w.find('[data-tool=pen]')
    await pen.trigger('pointerdown')
    vi.advanceTimersByTime(prefs.longPressMs + 50)
    await nextTick()
    await pen.trigger('pointerup')
    expect(store.state.tool).toBe('hand')
    expect(w.find('[data-testid=tool-flyout]').exists()).toBe(true)
    await w.find('[data-testid=tool-flyout] [data-tool=eraser]').trigger('click')
    expect(store.state.tool).toBe('eraser')
    expect(w.find('.tools > .slot .tool[data-tool=eraser]').exists()).toBe(true)
    vi.useRealTimers()
  })

  it('opens the flyout on double click and picks a tool on release', async () => {
    const store = freshStore()
    const w = mountIt(ToolsPanel)
    await w.find('[data-tool=pen]').trigger('dblclick')
    expect(w.find('[data-testid=tool-flyout]').exists()).toBe(true)
    // Releasing over an item (press-and-drag from the slot) picks it.
    await w.find('[data-testid=tool-flyout] [data-tool=eraser]').trigger('pointerup')
    expect(store.state.tool).toBe('eraser')
    expect(w.find('[data-testid=tool-flyout]').exists()).toBe(false)
  })
})

describe('AboutDialog', () => {
  it('describes the program, its author and repository rather than the frameworks', () => {
    freshStore()
    const w = mountIt(AboutDialog)
    const text = w.find('[data-testid=about]').text()
    expect(text).toContain('yeahBOYYYYY')
    expect(text).toContain('github.com/yeahBOYYYYY/transit-diagram-maker')
    expect(text).toMatch(/Version \d+\.\d+\.\d+/)
    expect(text).not.toMatch(/Rust|WebAssembly|Vue/)
  })
})

describe('MenuBar', () => {
  it('opens menus, shows shortcuts and runs actions', async () => {
    const store = freshStore()
    store.run((e) => e.addText({}))
    const w = mountIt(MenuBar)
    await w.find('[data-menu=Edit]').trigger('click')
    const items = w.findAll('.item')
    const undo = items.find((i) => i.text().includes('Undo'))!
    expect(undo.text()).toContain('Ctrl+Z')
    expect(items.find((i) => i.text().includes('Duplicate'))!.attributes('disabled')).toBeDefined()
    await undo.trigger('click')
    expect(store.state.fragments).toHaveLength(0)
    expect(w.find('.dropdown').exists()).toBe(false)
  })

  it('toggles window panels and the grid', async () => {
    freshStore()
    const w = mountIt(MenuBar)
    await w.find('[data-menu=Window]').trigger('click')
    await w.findAll('.item').find((i) => i.text().includes('Layers'))!.trigger('click')
    expect(prefs.panels.layers).toBe(false)
    await w.find('[data-menu=View]').trigger('click')
    await w.findAll('.item').find((i) => i.text().includes('Show Grid'))!.trigger('click')
    expect(prefs.showGrid).toBe(false)
  })
})

describe('OptionsBar', () => {
  it('shows options for the active tool', async () => {
    const store = freshStore()
    const w = mountIt(OptionsBar)
    expect(w.find('[data-testid=auto-select]').exists()).toBe(true)
    store.setTool('pen')
    await nextTick()
    expect(w.find('[data-testid=line-style]').exists()).toBe(true)
    await w.find('[data-testid=line-style]').setValue('dashed')
    expect(prefs.line.kind).toBe('dashed')
    expect(w.text()).toContain('Gap')
    store.setTool('station')
    await nextTick()
    expect(w.text()).toContain('Pill')
  })
})

describe('StatusBar', () => {
  it('sets the zoom from the input', async () => {
    const store = freshStore()
    const w = mountIt(StatusBar)
    const input = w.find('[data-testid=zoom-input]')
    await input.setValue('250')
    await input.trigger('keydown', { key: 'Enter' })
    expect(store.state.view.zoom).toBeCloseTo(2.5)
  })
})

describe('form controls', () => {
  it('NumberField clamps and commits typed values', async () => {
    const w = mountIt(NumberField, { props: { modelValue: 5, min: 0, max: 10, label: 'W' } })
    const input = w.find('input')
    await input.setValue('42')
    await input.trigger('blur')
    expect(w.emitted('update:modelValue')![0]).toEqual([10])
    await input.setValue('abc')
    await input.trigger('blur')
    expect((input.element as HTMLInputElement).value).toBe('5')
    await input.trigger('keydown', { key: 'ArrowUp' })
    expect(w.emitted('commit')!.at(-1)).toEqual([6])
  })

  it('ColorField accepts short hex and rejects junk', async () => {
    const w = mountIt(ColorField, { props: { modelValue: '#000000' } })
    const hex = w.find('input.hex')
    await hex.setValue('f0a')
    await hex.trigger('blur')
    expect(w.emitted('update:modelValue')![0]).toEqual(['#ff00aa'])
    await hex.setValue('nope')
    await hex.trigger('blur')
    expect((hex.element as HTMLInputElement).value).toBe('#000000')
  })
})

describe('dialogs', () => {
  it('shortcut editor records a new binding', async () => {
    const store = freshStore()
    store.state.dialog = 'shortcuts'
    const w = mountIt(ShortcutsDialog)
    const btn = w.find('[data-action="tool.text"]')
    await btn.trigger('click')
    expect(btn.text()).toContain('Press keys')
    await btn.trigger('keydown', { key: 'y', code: 'KeyY', altKey: true })
    expect(btn.text()).toBe('Alt+Y')
    await w.find('[data-testid=dialog-ok]').trigger('click')
    expect(prefs.shortcuts['tool.text']).toBe('Alt+Y')
    expect(store.state.dialog).toBeNull()
  })

  it('document settings apply as one undoable change', async () => {
    const store = freshStore()
    store.state.dialog = 'document'
    const w = mountIt(DocumentSettingsDialog)
    const grid = w.findAll('.num-field').find((n) => n.text().includes('Cell size'))!.find('input')
    await grid.setValue('32')
    await grid.trigger('blur')
    await w.find('[data-testid=dialog-ok]').trigger('click')
    expect(store.state.settings.grid_size).toBe(32)
    store.undo()
    expect(store.state.settings.grid_size).toBe(24)
  })
})
