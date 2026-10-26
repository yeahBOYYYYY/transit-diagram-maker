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
import ExportDialog from '../../src/components/dialogs/ExportDialog.vue'
import PreferencesDialog from '../../src/components/dialogs/PreferencesDialog.vue'
import IconsPanel from '../../src/components/panels/IconsPanel.vue'
import * as files from '../../src/core/files'
import { prefs } from '../../src/stores/prefs'
import { runAction } from '../../src/stores/commands'
import { picking, toHex } from '../../src/core/eyedropper'

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

  it('edits one segment, or the whole line from its group', async () => {
    vi.useFakeTimers()
    const store = freshStore()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 9, y: 0 }], {}))!
    store.run((e) => e.erase([{ x: 4, y: 0 }]))
    store.select([l.layer_id])
    const w = mountIt(PropertiesPanel)
    const dashed = () => store.state.fragments.filter((f) => f.svg.includes('stroke-dasharray')).length
    expect(w.text()).toContain('Segment appearance')
    expect(w.text()).toContain('1 of 2 segments')
    await w.find('[data-testid=prop-line-style]').setValue('hollow')
    vi.advanceTimersByTime(1000)
    await nextTick()
    expect(dashed()).toBe(0)
    expect(w.text()).toContain('Channel')
    expect(w.text()).not.toContain('Gap')
    await w.find('[data-testid=prop-line-style]').setValue('inner_dashed')
    vi.advanceTimersByTime(1000)
    await nextTick()
    expect(dashed()).toBe(1)
    expect(w.text()).not.toContain('Channel')
    expect(w.text()).toContain('Dashes')
    expect(w.text()).toContain('Gap')
    expect(w.text()).toContain('Start offset')
    expect(w.text()).toContain('End offset')
    await w.find('[data-testid=end-cap]').setValue('butt')
    vi.advanceTimersByTime(1000)
    const n = store.node(l.layer_id)
    expect(n?.type === 'layer' && n.content.kind === 'line' && n.content.style.end_cap).toBe('butt')
    // Copy the segment's style to the rest of the line.
    await w.find('[data-testid=apply-style-to-line]').trigger('click')
    expect(dashed()).toBe(2)
    // The line group edits every segment.
    store.select([l.group_id])
    await nextTick()
    expect(w.text()).toContain('Line appearance')
    expect(w.find('[data-testid=apply-style-to-line]').exists()).toBe(false)
    await w.find('[data-testid=prop-line-style]').setValue('plain')
    vi.advanceTimersByTime(1000)
    expect(dashed()).toBe(0)
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

  it('opens the text editor and formats the whole text', async () => {
    const store = freshStore()
    const t = store.run((e) => e.addText({ text: 'Old' }))!
    store.select([t])
    const w = mountIt(PropertiesPanel)
    await w.find('[data-testid=prop-text-bold]').trigger('click')
    const n = store.node(t)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.bold).toBe(true)
    await w.find('[data-testid=prop-edit-text]').trigger('click')
    expect(store.state.textEdit?.id).toBe(t)
    store.finishTextEdit(false)
  })

  it('switches the writing direction of text', async () => {
    const store = freshStore()
    const t = store.run((e) => e.addText({ text: 'תחנה' }))!
    store.select([t])
    const w = mountIt(PropertiesPanel)
    await w.find('[data-testid=prop-text-direction] [title="Right to left"]').trigger('click')
    const n = store.node(t)
    expect(n?.type === 'layer' && n.content.kind === 'text' && n.content.direction).toBe('rtl')
    expect(store.state.fragments.find((f) => f.id === t)!.svg).toContain('direction="rtl"')
  })

  async function setField(w: VueWrapper, label: string, value: string) {
    const input = w.find('[data-testid=prop-position]').findAll('.num-field').find((f) => f.find('.scrub').text() === label)!.find('input')
    await input.setValue(value)
    await input.trigger('blur')
    vi.advanceTimersByTime(1000)
    await nextTick()
  }

  it('shows and edits X / Y for every kind of object', async () => {
    vi.useFakeTimers()
    const store = freshStore()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 9, y: 0 }], {}))!
    const st = store.run((e) => e.addStation('dot', { x: 3, y: 0 }, { label: false }))!
    const t = store.run((e) => e.addText({ pos: { x: 1, y: 4 } }))!
    const w = mountIt(PropertiesPanel)
    const pos = (id: number) => store.engine.value.position([id])

    // Text moves in tenths.
    store.select([t])
    await nextTick()
    await setField(w, 'X', '2.34')
    expect(pos(t)).toEqual({ x: 2.3, y: 4 })

    // A line group moves in whole cells and carries its station along.
    store.select([l.group_id])
    await nextTick()
    expect(w.find('[data-testid=prop-position]').text()).toContain('cells')
    await setField(w, 'Y', '2.5')
    expect(pos(l.layer_id)).toEqual({ x: 0.5, y: 2.5 })
    expect(pos(st.station_id)).toEqual({ x: 3.5, y: 2.5 })

    // A station moved by whole cells stays on its line; by a fraction it leaves it.
    store.select([st.station_id])
    await nextTick()
    await setField(w, 'X', '5.5')
    const on = store.node(st.station_id)
    expect(on?.type === 'layer' && on.content.kind === 'station' && on.content.offset).toEqual({ x: 0, y: 0 })
    await setField(w, 'Y', '2.8')
    expect(pos(st.station_id)).toEqual({ x: 5.5, y: 2.8 })

    // Several objects: the top-left-most anchor.
    store.select([l.group_id, t])
    await nextTick()
    expect(w.find('[data-testid=prop-position]').exists()).toBe(true)
    vi.useRealTimers()
  })
})

describe('property help', () => {
  /** Every labeled property row must carry exactly one info icon. */
  function expectHelpEverywhere(w: VueWrapper) {
    const rows = [...w.findAll('.num-field'), ...w.findAll('.color-field').filter((c) => c.find('.dim').exists()), ...w.findAll('.lbl')]
    expect(rows.length).toBeGreaterThan(0)
    for (const r of rows) {
      const isInsideField = r.classes('lbl') && r.element.closest('.num-field, .color-field')
      if (isInsideField) continue
      expect(r.findAll('[data-testid=help-tip]').length, r.text()).toBe(1)
    }
  }

  it('every property in the Properties panel has help, for every kind of object', async () => {
    const store = freshStore()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 9, y: 0 }], { kind: 'inner_dashed' }))!
    const dot = store.run((e) => e.addStation('dot', { x: 2, y: 0 }, { label: false }))!
    const tick = store.run((e) => e.addStation('tick', { x: 5, y: 0 }, { label: false }))!
    const t = store.run((e) => e.addText({}))!
    const i = store.run((e) => e.addIcon({}))!
    const w = mountIt(PropertiesPanel)
    expectHelpEverywhere(w)
    for (const id of [l.layer_id, l.group_id, dot.station_id, tick.station_id, t, i]) {
      store.select([id])
      await nextTick()
      expectHelpEverywhere(w)
    }
    store.select([t, i])
    await nextTick()
    expectHelpEverywhere(w)
  })

  it('every setting in the dialogs and tool options has help', async () => {
    const store = freshStore()
    expectHelpEverywhere(mountIt(DocumentSettingsDialog))
    expectHelpEverywhere(mountIt(ExportDialog))
    const prefsDialog = mountIt(PreferencesDialog)
    for (const tab of prefsDialog.findAll('[role=tab]')) {
      await tab.trigger('click')
      expectHelpEverywhere(prefsDialog)
    }
    for (const tool of ['pen', 'eraser', 'station', 'text'] as const) {
      store.setTool(tool)
      // Mounted per tool: happy-dom can't patch the bar between some tools.
      const w = mount(OptionsBar as never, { attachTo: document.body })
      expectHelpEverywhere(w)
      w.unmount()
    }
  })

  it('shows the help text while hovering the info icon', async () => {
    freshStore()
    const w = mountIt(NumberField, { props: { modelValue: 1, label: 'Width', help: 'How wide it is.' } })
    const tip = w.find('[data-testid=help-tip]')
    expect(tip.attributes('aria-label')).toBe('How wide it is.')
    await tip.trigger('mouseenter')
    expect(document.body.querySelector('[role=tooltip]')?.textContent).toBe('How wide it is.')
    await tip.trigger('mouseleave')
    expect(document.body.querySelector('[role=tooltip]')).toBeNull()
  })
})

describe('ToolsPanel', () => {
  it('has one button per tool, including the eraser, and none for pictograms', async () => {
    const store = freshStore()
    const w = mountIt(ToolsPanel)
    expect(w.findAll('.tool').map((b) => b.attributes('data-tool'))).toEqual(['select', 'hand', 'pen', 'eraser', 'station', 'text'])
    await w.find('[data-tool=eraser]').trigger('click')
    expect(store.state.tool).toBe('eraser')
    expect(w.find('[data-tool=eraser]').classes()).toContain('on')
    await w.find('[data-tool=hand]').trigger('click')
    expect(store.state.tool).toBe('hand')
  })

  it('edits, swaps and resets the line and inner colors like foreground and background colors', async () => {
    freshStore()
    const w = mountIt(ToolsPanel)
    await w.find('[data-testid=tool-color-line]').setValue('#0098d4')
    await w.find('[data-testid=tool-color-inner]').setValue('#ffd300')
    expect([prefs.line.color, prefs.line.inner_color]).toEqual(['#0098d4', '#ffd300'])
    await w.find('[data-testid=tool-colors-swap]').trigger('click')
    expect([prefs.line.color, prefs.line.inner_color]).toEqual(['#ffd300', '#0098d4'])
    expect(w.find('.fg').attributes('style')).toContain('#ffd300')
    expect(w.find('[data-testid=tool-colors-swap]').attributes('title')).toContain('(X)')
    await w.find('[data-testid=tool-colors-reset]').trigger('click')
    expect([prefs.line.color, prefs.line.inner_color]).toEqual(['#e03c31', '#ffffff'])
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

  it('links the GitHub repository from the About and Preferences dialogs', () => {
    freshStore()
    const repo = 'https://github.com/yeahBOYYYYY/transit-diagram-maker'
    expect(mountIt(AboutDialog).find('[data-testid=about-github]').attributes('href')).toBe(repo)
    expect(mountIt(PreferencesDialog).find('[data-testid=prefs-github]').attributes('href')).toBe(repo)
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
    expect(w.findAll('[data-testid=line-style] option').map((o) => o.text())).toEqual(['Plain', 'Hollow', 'Inner dashed'])
    await w.find('[data-testid=line-style]').setValue('inner_dashed')
    expect(prefs.line.kind).toBe('inner_dashed')
    expect(w.text()).toContain('Gap')
    store.setTool('station')
    await nextTick()
    expect(w.text()).toContain('Pill')
  })
})

describe('IconsPanel', () => {
  it('adds a pictogram in the middle of the view, or changes the selected pictograms', async () => {
    const store = freshStore()
    store.state.viewport = { width: 480, height: 240 }
    store.state.view = { x: 0, y: 0, zoom: 1 }
    const w = mountIt(IconsPanel)
    await w.find('[data-icon=ferry]').trigger('click')
    expect(prefs.icon.icon).toBe('ferry')
    expect(store.state.tool).toBe('select')
    const added = store.node(store.state.selection[0])
    expect(added?.type === 'layer' && added.content.kind === 'icon' && added.content).toMatchObject({ icon: 'ferry', pos: { x: 10.5, y: 5.5 } })
    const id = store.run((e) => e.addIcon({ icon: 'wheelchair' }))!
    store.select([id])
    await nextTick()
    expect(w.find('.cell.on').attributes('data-icon')).toBe('wheelchair')
    await w.find('[data-icon=bus]').trigger('click')
    const n = store.node(id)
    expect(n?.type === 'layer' && n.content.kind === 'icon' && n.content.icon).toBe('bus')
  })

  it('pictograms can be dragged', () => {
    freshStore()
    const w = mountIt(IconsPanel)
    const cell = w.find('[data-icon=ferry]')
    expect(cell.attributes('draggable')).toBe('true')
    const data = new Map<string, string>()
    const dataTransfer = { setData: (k: string, v: string) => data.set(k, v), effectAllowed: '' }
    cell.element.dispatchEvent(Object.assign(new Event('dragstart'), { dataTransfer }))
    expect(data.get('application/x-transit-icon')).toBe('ferry')
  })
})

describe('ExportDialog', () => {
  it('shows only export options and remembers them', async () => {
    const store = freshStore()
    store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))
    const download = vi.spyOn(files, 'downloadText').mockImplementation(() => {})
    const w = mountIt(ExportDialog)
    expect(w.findAll('.num-field').map((f) => f.text())).toEqual(['Pixels per cellpixels', 'Margincells'])
    await w.find('[data-testid=export-transparent]').setValue(true)
    await w.find('[data-testid=export-cell-px] input').setValue('48')
    await w.find('[data-testid=export-cell-px] input').trigger('blur')
    await w.find('[data-testid=dialog-ok]').trigger('click')
    expect(prefs.export).toEqual({ cell_px: 48, margin: 1, background: '' })
    const svg = download.mock.calls[0][1]
    expect(svg).toContain('width="336" height="144"')
    expect(svg).not.toContain('<rect')
    expect(svg).not.toContain('transit:')
    download.mockRestore()
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

  it('ColorField picks a color from the screen with its eyedropper', async () => {
    const w = mountIt(ColorField, { props: { modelValue: '#000000' } })
    const other = document.createElement('div')
    other.dataset.swatch = ''
    other.style.background = 'rgb(224, 60, 49)'
    document.body.appendChild(other)
    const spy = vi.fn(() => [other])
    ;(document as unknown as { elementsFromPoint: unknown }).elementsFromPoint = spy
    await w.find('[data-testid=eyedropper]').trigger('click')
    expect(picking.value).not.toBeNull()
    expect(document.body.classList.contains('picking-color')).toBe(true)
    other.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, clientX: 5, clientY: 5 }))
    expect(w.emitted('update:modelValue')!.at(-1)).toEqual(['#e03c31'])
    // The click that ends the press doesn't reach what was under the pointer.
    const clicked = vi.fn()
    other.addEventListener('click', clicked)
    other.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(clicked).not.toHaveBeenCalled()
    other.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(clicked).toHaveBeenCalledTimes(1)
    expect(picking.value).toBeNull()
    expect(document.body.classList.contains('picking-color')).toBe(false)
    // Escape stops picking without a color.
    await w.find('[data-testid=eyedropper]').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(picking.value).toBeNull()
    other.remove()
  })

  it('reads the colors the eyedropper finds', () => {
    expect(toHex('rgb(0, 152, 212)')).toBe('#0098d4')
    expect(toHex('rgba(255, 255, 255, 0.5)')).toBe('#ffffff')
    expect(toHex('#F0A')).toBe('#ff00aa')
    expect(toHex('rgba(0, 0, 0, 0)')).toBeNull()
    expect(toHex('none')).toBeNull()
    expect(toHex('url(#grid-minor)')).toBeNull()
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

  it('preferences are organized in tabs and restore their defaults', async () => {
    const store = freshStore()
    store.state.dialog = 'preferences'
    const w = mountIt(PreferencesDialog)
    expect(w.findAll('[role=tab]').map((t) => t.text())).toEqual(['General', 'Interface', 'Canvas and grid', 'Navigation', 'Editing', 'Export'])
    await w.find('[data-testid=prefs-tab-navigation]').trigger('click')
    expect(w.find('[data-testid=prefs-page-navigation]').text()).toContain('Fit on screen margin')
    // Restoring a tab only touches that tab.
    prefs.zoomStep = 2
    prefs.nudge = 3
    const w2 = mountIt(PreferencesDialog)
    await w2.find('[data-testid=prefs-tab-navigation]').trigger('click')
    await w2.find('[data-testid=restore-tab]').trigger('click')
    await w2.find('[data-testid=dialog-ok]').trigger('click')
    expect([prefs.zoomStep, prefs.nudge]).toEqual([1.15, 3])
    // Restoring everything asks first, then also resets tool options and shortcuts.
    prefs.line.color = '#123456'
    prefs.shortcuts['tool.text'] = 'Alt+Y'
    const w3 = mountIt(PreferencesDialog)
    await w3.find('[data-testid=restore-all]').trigger('click')
    expect(prefs.nudge).toBe(3)
    await w3.find('[data-testid=restore-all-confirm]').trigger('click')
    expect([prefs.nudge, prefs.line.color, prefs.shortcuts['tool.text']]).toEqual([1, '#e03c31', 'T'])
  })

  it('Edit ▸ Restore All Defaults opens the preferences asking to confirm', async () => {
    const store = freshStore()
    runAction(store, 'edit.restoreDefaults')
    expect(store.state.dialog).toBe('preferences')
    const w = mountIt(PreferencesDialog)
    expect(w.find('[data-testid=restore-confirm]').exists()).toBe(true)
  })

  it('document settings apply as one undoable change', async () => {
    const store = freshStore()
    store.state.dialog = 'document'
    const w = mountIt(DocumentSettingsDialog)
    expect(w.text()).not.toContain('Cell size')
    const width = w.findAll('.num-field').find((n) => n.text().includes('Width'))!.find('input')
    await width.setValue('0.5')
    await width.trigger('blur')
    await w.find('[data-testid=dialog-ok]').trigger('click')
    expect(store.state.settings.line_width).toBe(0.5)
    store.undo()
    expect(store.state.settings.line_width).toBe(1)
  })
})
