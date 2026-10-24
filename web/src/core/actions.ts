// Catalogue of user actions: used by the menu bar, keyboard handler and the
// shortcut editor.

export const ACTIONS = {
  'file.new': { label: 'New', category: 'File', key: 'Ctrl+Alt+N' },
  'file.open': { label: 'Open…', category: 'File', key: 'Ctrl+O' },
  'file.save': { label: 'Save', category: 'File', key: 'Ctrl+S' },
  'file.saveAs': { label: 'Save As…', category: 'File', key: 'Ctrl+Shift+S' },
  'file.exportSvg': { label: 'Export as SVG…', category: 'File', key: 'Ctrl+Shift+E' },

  'edit.undo': { label: 'Undo', category: 'Edit', key: 'Ctrl+Z' },
  'edit.redo': { label: 'Redo', category: 'Edit', key: 'Ctrl+Shift+Z, Ctrl+Y' },
  'edit.delete': { label: 'Delete', category: 'Edit', key: 'Delete, Backspace' },
  'edit.duplicate': { label: 'Duplicate', category: 'Edit', key: 'Ctrl+J' },
  'edit.selectAll': { label: 'Select All', category: 'Edit', key: 'Ctrl+A' },
  'edit.deselect': { label: 'Deselect', category: 'Edit', key: 'Ctrl+D' },
  'edit.rotateCw': { label: 'Rotate 45° Clockwise', category: 'Edit', key: 'R' },
  'edit.rotateCcw': { label: 'Rotate 45° Counter Clockwise', category: 'Edit', key: 'Shift+R' },
  'edit.preferences': { label: 'Preferences…', category: 'Edit', key: 'Ctrl+K' },
  'edit.shortcuts': { label: 'Keyboard Shortcuts…', category: 'Edit', key: 'Ctrl+Alt+Shift+K' },

  'document.settings': { label: 'Document Settings…', category: 'Document', key: 'Ctrl+Alt+D' },
  'document.assets': { label: 'Fonts & Icons…', category: 'Document', key: '' },

  'layer.newGroup': { label: 'New Group', category: 'Layer', key: 'Ctrl+Shift+N' },
  'layer.group': { label: 'Group Layers', category: 'Layer', key: 'Ctrl+G' },
  'layer.ungroup': { label: 'Ungroup Layers', category: 'Layer', key: 'Ctrl+Shift+G' },
  'layer.toggleLock': { label: 'Lock / Unlock', category: 'Layer', key: 'Ctrl+/' },
  'layer.toggleVisible': { label: 'Hide / Show', category: 'Layer', key: 'Ctrl+,' },
  'layer.bringForward': { label: 'Bring Forward', category: 'Layer', key: 'Ctrl+]' },
  'layer.sendBackward': { label: 'Send Backward', category: 'Layer', key: 'Ctrl+[' },
  'layer.bringToFront': { label: 'Bring to Front', category: 'Layer', key: 'Ctrl+Shift+]' },
  'layer.sendToBack': { label: 'Send to Back', category: 'Layer', key: 'Ctrl+Shift+[' },

  'view.zoomIn': { label: 'Zoom In', category: 'View', key: 'Ctrl+=' },
  'view.zoomOut': { label: 'Zoom Out', category: 'View', key: 'Ctrl+-' },
  'view.fit': { label: 'Fit on Screen', category: 'View', key: 'Ctrl+0' },
  'view.actual': { label: '100%', category: 'View', key: 'Ctrl+1' },
  'view.toggleGrid': { label: 'Show Grid', category: 'View', key: "Ctrl+'" },

  'tool.select': { label: 'Selection Tool', category: 'Tools', key: 'V' },
  'tool.hand': { label: 'Hand Tool', category: 'Tools', key: 'H' },
  'tool.pen': { label: 'Line Pen Tool', category: 'Tools', key: 'P' },
  'tool.eraser': { label: 'Erase Tool', category: 'Tools', key: 'E' },
  'tool.station': { label: 'Station Tool', category: 'Tools', key: 'S' },
  'tool.text': { label: 'Text Tool', category: 'Tools', key: 'T' },
  'tool.icon': { label: 'Icon Tool', category: 'Tools', key: 'I' },

  'help.shortcuts': { label: 'Shortcut Reference', category: 'Help', key: 'F1' },
  'help.about': { label: 'About Transit Diagram Maker', category: 'Help', key: '' },
} as const

export type ActionId = keyof typeof ACTIONS

export const ACTION_IDS = Object.keys(ACTIONS) as ActionId[]

export const DEFAULT_SHORTCUTS = Object.fromEntries(ACTION_IDS.map((id) => [id, ACTIONS[id].key])) as Record<
  ActionId,
  string
>
