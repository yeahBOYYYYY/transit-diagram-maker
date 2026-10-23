import { describe, expect, it } from 'vitest'
import { freshStore, key } from './helpers'
import { actionForEvent, isEnabled, nudgeSelection, rotateSelection, runAction } from '../../src/stores/commands'
import { prefs } from '../../src/stores/prefs'

describe('commands', () => {
  it('maps keyboard events to actions using the current bindings', () => {
    expect(actionForEvent(key('z', { ctrlKey: true }))).toBe('edit.undo')
    expect(actionForEvent(key('z', { ctrlKey: true, shiftKey: true }))).toBe('edit.redo')
    expect(actionForEvent(key('y', { ctrlKey: true }))).toBe('edit.redo')
    expect(actionForEvent(key('p'))).toBe('tool.pen')
    expect(actionForEvent(key('e'))).toBe('tool.eraser')
    prefs.shortcuts['tool.pen'] = 'B'
    expect(actionForEvent(key('p'))).toBeNull()
    expect(actionForEvent(key('b'))).toBe('tool.pen')
  })

  it('switches tools', () => {
    const store = freshStore()
    runAction(store, 'tool.station')
    expect(store.state.tool).toBe('station')
    runAction(store, 'tool.eraser')
    expect(store.state.penSlot).toBe('eraser')
  })

  it('groups, ungroups and deletes the selection', () => {
    const store = freshStore()
    const a = store.run((e) => e.addText({}))!
    const b = store.run((e) => e.addText({}))!
    store.select([a, b])
    runAction(store, 'layer.group')
    const g = store.state.selection[0]
    expect(store.state.tree[0]).toMatchObject({ id: g, kind: 'group' })
    expect(isEnabled(store, 'layer.ungroup')).toBe(true)
    runAction(store, 'layer.ungroup')
    expect(store.state.selection.sort()).toEqual([a, b].sort())
    runAction(store, 'edit.delete')
    expect(store.state.fragments).toHaveLength(0)
  })

  it('toggles lock and visibility for the selection', () => {
    const store = freshStore()
    const a = store.run((e) => e.addText({}))!
    store.select([a])
    runAction(store, 'layer.toggleLock')
    expect(store.state.tree[0].locked).toBe(true)
    expect(isEnabled(store, 'edit.delete')).toBe(false)
    runAction(store, 'layer.toggleLock')
    runAction(store, 'layer.toggleVisible')
    expect(store.state.tree[0].visible).toBe(false)
    expect(store.state.fragments).toHaveLength(0)
  })

  it('restacks the selection', () => {
    const store = freshStore()
    const a = store.run((e) => e.addText({}))!
    const b = store.run((e) => e.addText({}))!
    store.select([a])
    runAction(store, 'layer.bringToFront')
    expect(store.state.tree.map((t) => t.id)).toEqual([a, b])
    runAction(store, 'layer.sendBackward')
    expect(store.state.tree.map((t) => t.id)).toEqual([b, a])
  })

  it('duplicates, nudges and rotates the selection', () => {
    const store = freshStore()
    const l = store.run((e) => e.createLine([{ x: 0, y: 0 }, { x: 4, y: 0 }], {}))!
    store.select([l.group_id])
    runAction(store, 'edit.duplicate')
    expect(store.state.tree).toHaveLength(2)
    const copy = store.state.selection[0]
    expect(copy).not.toBe(l.group_id)
    store.select([l.group_id])
    nudgeSelection(store, 0, 2)
    const n = store.node(l.layer_id)
    expect(n?.type === 'layer' && n.content.kind === 'line' && n.content.points[0]).toEqual({ x: 0, y: 2 })
    rotateSelection(store, 90)
    const r = store.node(l.layer_id)
    const pts = r?.type === 'layer' && r.content.kind === 'line' ? r.content.points : []
    expect(pts[0].x).toBe(pts[1].x)
  })

  it('selection and history driven enablement', () => {
    const store = freshStore()
    expect(isEnabled(store, 'edit.undo')).toBe(false)
    expect(isEnabled(store, 'edit.duplicate')).toBe(false)
    expect(isEnabled(store, 'layer.group')).toBe(false)
    expect(isEnabled(store, 'file.save')).toBe(true)
    store.run((e) => e.addText({}))
    expect(isEnabled(store, 'edit.undo')).toBe(true)
  })

  it('opens dialogs and toggles the grid', () => {
    const store = freshStore()
    runAction(store, 'document.settings')
    expect(store.state.dialog).toBe('document')
    runAction(store, 'edit.shortcuts')
    expect(store.state.dialog).toBe('shortcuts')
    runAction(store, 'edit.preferences')
    expect(store.state.dialog).toBe('preferences')
    const grid = prefs.showGrid
    runAction(store, 'view.toggleGrid')
    expect(prefs.showGrid).toBe(!grid)
    const z = store.state.view.zoom
    runAction(store, 'view.zoomIn')
    expect(store.state.view.zoom).toBeGreaterThan(z)
  })
})
