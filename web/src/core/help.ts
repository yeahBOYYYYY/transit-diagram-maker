// Help text for every property the UI shows, keyed by property. Shown by the
// info icon (HelpTip) beside each property label so the same property is
// explained the same way in the Properties panel, the tool options and the
// dialogs.

export const HELP = {
  // Selection
  position:
    'Position in grid cells. Text and pictograms use their anchor point, stations their center, lines the top-left cell center they pass through, and several objects the top-left-most of those. Lines move in whole cells; moving a station by a fraction of a cell takes it off its line.',

  // Lines
  'line.target': 'Draw a new line, or add a disconnected segment to an existing line. New segments take that line\'s style.',
  'line.color': 'Main color of the line. It is shared by all segments of the line, so changing it recolors the whole line.',
  'line.kind':
    'Plain: a solid line. Hollow: a line with an inner channel. Dashed: the line is broken into dashes with empty gaps. Hollow dashed: dashes run down the channel of a hollow line.',
  'line.channel': 'Color of the inner channel of a hollow line.',
  'line.channel_width': 'Width of the inner channel as a percentage of the line width.',
  'line.dash_length': 'Length of each dash, in cells. Dashes and gaps are stretched slightly so the line starts and ends with a full dash.',
  'line.dash_gap': 'Space between two dashes, in cells. 0 makes the line solid.',
  'line.dash_width': 'Width of the dashes inside a hollow dashed line, as a percentage of the line width.',
  'line.caps': 'Shape of the line ends. Round ends follow the document\'s cap roundness; butt ends are square and fill the end cell.',
  'line.points': 'The cells where the segment starts, turns and ends.',
  'line.style_scope':
    'A selected segment has its own style, so a line can change style along its way (for example dashed where it is under construction). Select the line group to change all of its segments at once. The color is always shared by the whole line.',
  'line.apply_to_line': 'Copy the style of this segment (stroke, channel, dashes and caps) to every segment of its line.',

  // Stations
  'station.shape':
    'Dot: a circle on one cell. Tick: a short bar sticking out of the line, sitting in a corner of its cell. Pill: a dot stretched over several cells, for example across parallel lines.',
  'station.outer_color': 'Color of the station border.',
  'station.inner_color': 'Fill color inside the station border.',
  'station.tick_color': 'Color of the tick. New ticks use the line color unless another color is set.',
  'station.tick_width': 'Width of the tick along the line, in cells.',
  'station.tick_length': 'How far the tick sticks out of the line, in cells.',
  'station.tick_side': 'Which side of the line the tick points to, looking along the direction the line was drawn.',
  'station.label': 'Create a name label next to each new station. The label follows the station when it moves.',
  'station.coupled': 'The lines this station belongs to. It snaps back onto them when moved without Alt.',

  // Text
  'text.text': 'The text. Press Shift+Enter for a new line when editing on the canvas.',
  'text.font': 'Font family, bold and italic.',
  'text.color': 'Text color.',
  'text.align': 'How the text lines up with its position: left edge, center or right edge.',
  'text.size': 'Font size, in cells.',
  'text.rotation': 'Rotation in degrees, in steps of 45°.',

  // Pictograms
  'icon.icon': 'Which pictogram to show. Pick another one in the Icons panel.',
  'icon.panel':
    'Clicking a pictogram picks it for the Icon tool, or changes the selected pictograms to it. Dragging a pictogram onto the canvas places it in the cell under the pointer (hold Alt to place it exactly under the pointer).',
  'icon.upload': 'SVG files you uploaded into this document. They are saved inside the file. Remove them in Document ▸ Fonts & Icons.',
  'icon.color': 'Color of the pictogram symbol.',
  'icon.background': 'Color of the rounded tile behind the symbol.',
  'icon.size': 'Edge length of the tile, in cells.',
  'icon.size_default': 'New pictograms match the dot station diameter unless a size is set.',
  'icon.rotation': 'Rotation in degrees, in steps of 45°.',

  // Eraser
  'eraser.size': 'Width of the eraser brush, in cells.',

  // Document settings
  'settings.route_mode': 'How the pen joins two clicked cells that are not on a straight or 45° line: straight first and then diagonal, or the other way round.',
  'settings.line_width': 'Width of every transit line, in cells. 1 fills a whole row of cells.',
  'settings.corner_radius': 'Radius of the curve where a line turns, in cells. 0 gives sharp corners.',
  'settings.cap_roundness': 'How round the "Round" line ends are: 0 is square, 1 is a full semicircle.',
  'settings.station_outer_radius': 'Outer radius of dot and pill stations, including their border, in cells.',
  'settings.station_inner_radius': 'Radius of the fill inside dot and pill stations, in cells. The border is the outer radius minus this.',
  'settings.tick_width': 'Width of newly placed tick stations, in cells.',
  'settings.tick_length': 'How far newly placed tick stations stick out of the line, in cells.',
  'settings.label_offset': 'Distance between a new station and its name label, in cells.',
  'settings.label_font_size': 'Font size of new station labels, in cells.',
  'settings.label_font_family': 'Font of new station labels.',
  'settings.icon_size': 'Size of newly placed pictograms. By default they match the dot station diameter.',

  // SVG export
  'export.cell_px': 'Scale of the exported SVG: how many pixels one grid cell becomes. 24 matches the canvas at 100% zoom.',
  'export.margin': 'Empty space around the drawing in the exported SVG, in cells.',
  'export.background': 'Background color of the exported SVG. Choose transparent to export without a background.',

  // Preferences
  'prefs.historyLimit': 'How many steps Undo can go back.',
  'prefs.longPressMs': 'How long to hold the pen tool button before its hidden tools appear.',
  'prefs.dragThreshold': 'How far the pointer must move before a click becomes a drag, in screen pixels.',
  'prefs.autoSelect': 'What a click on the canvas selects: the layer under the pointer, or its outermost group.',
  'prefs.zoomStep': 'Zoom factor of one wheel step or zoom command.',
  'prefs.minZoom': 'Smallest zoom level (1 = 100%).',
  'prefs.maxZoom': 'Largest zoom level (1 = 100%).',
  'prefs.showGrid': 'Show the cell grid on the canvas. It never appears in exports.',
  'prefs.showThumbnails': 'Show a small preview of each layer in the Layers panel.',
  'prefs.nudge': 'How far the arrow keys move the selection, in cells.',
  'prefs.bigNudge': 'How far Shift + arrow keys move the selection, in cells.',
  'prefs.textSnap': 'Text and pictograms snap to this step while dragged, in cells. Hold Alt to move in 0.1 steps.',
  'prefs.eraserSize': 'Default eraser width, in cells.',
  'prefs.canvasBackground': 'Color of the drawing area.',
  'prefs.pasteboardColor': 'Color around the drawing area.',
  'prefs.gridColor': 'Color of the cell grid.',
  'prefs.gridMajorColor': 'Color of the heavier grid lines.',
  'prefs.gridMajorEvery': 'Draw a heavier grid line every this many cells.',
  'prefs.minGridPixels': 'Hide the grid when cells are smaller than this on screen.',
  'prefs.selectionColor': 'Color of selection outlines and handles.',
} as const

export type HelpKey = keyof typeof HELP
