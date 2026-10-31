// Help text for every property the UI shows, keyed by property. Shown by the
// info icon (HelpTip) beside each property label so the same property is
// explained the same way in the Properties panel, the tool options and the
// dialogs.

export const HELP = {
  // Selection
  position:
    'Position in grid cells. Text uses the top-left corner of its box, pictograms their center, stations their center, lines the top-left cell center they pass through, rectangles their top-left corner, circles their center, straight lines their top-left-most end, and several objects the top-left-most of those. Dragged objects snap: text, rectangles and straight lines put a corner or end on a grid intersection, everything else snaps to the cells (hold Alt to move in steps of 0.1 cell). Typed values can use 0.1 steps, except for lines, which move in whole cells; moving a station by a fraction of a cell takes it off its line.',

  // Lines
  'line.target': 'Draw a new line, or add a disconnected segment to an existing line. New segments take that line\'s style.',
  'line.color': 'Main color of the line. It is shared by all segments of the line, so changing it recolors the whole line.',
  'line.kind':
    'Plain: a solid line. Hollow: a line with an inner channel. Inner dashed: a solid line with thinner dashes of another color running down its middle.',
  'line.channel': 'Color of the inner channel of a hollow line.',
  'line.dash_color': 'Color of the dashes running down the middle of an inner dashed line.',
  'line.channel_width': 'Width of the inner channel as a percentage of the line width.',
  'line.dash_length': 'Length of each dash, in cells. Dashes and gaps are stretched slightly so the line starts and ends with a full dash.',
  'line.dash_gap': 'Space between two dashes, in cells. 0 joins the dashes into one inner stripe.',
  'line.dash_width': 'Width of the dashes inside an inner dashed line, as a percentage of the line width.',
  'line.dash_start_offset': 'Length at the start of the line where the dashes do not begin yet, in cells. That part looks like a plain line. 0 starts the dashes right at the line end.',
  'line.dash_end_offset': 'Length at the end of the line where the dashes have already stopped, in cells. That part looks like a plain line. 0 runs the dashes up to the line end.',
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
  'text.edit':
    'Edit the text on the canvas (or double-click it). Select part of it to give that part its own font, size, color, bold or italic; Shift+Enter starts a new line, Enter finishes and Escape cancels.',
  'text.font': 'Font family, bold and italic. While editing on the canvas they apply to the selected characters; otherwise to all of the text.',
  'text.color': 'Text color. While editing on the canvas it applies to the selected characters; otherwise to all of the text.',
  'text.align':
    'How the lines line up inside the text box: left, center or right. The box itself stays where it is. While editing on the canvas it applies to the lines the selection touches.',
  'text.direction': 'Writing direction. Right to left orders the words for languages such as Hebrew and Arabic, including mixed runs with numbers and Latin words. It does not change the alignment.',
  'text.size': 'Font size, in cells. While editing on the canvas it applies to the selected characters; otherwise to all of the text.',
  'text.rotation': 'Rotation in degrees around the top-left corner. The rotate handles turn in steps of 45°; hold Alt to turn freely.',

  // Pictograms
  'icon.icon': 'Which pictogram to show. Pick another one in the Icons panel.',
  'icon.panel':
    'Clicking a pictogram changes the selected pictograms to it, or adds it in the middle of the view when no pictogram is selected. Dragging a pictogram onto the canvas places it in the cell under the pointer (hold Alt to place it exactly under the pointer).',
  'icon.defaults': 'Colors and size given to pictograms you add from this panel.',
  'icon.upload': 'SVG files you uploaded into this document. They are saved inside the file. Remove them in Document ▸ Fonts & Icons.',
  'icon.color': 'Color of the pictogram symbol.',
  'icon.background': 'Color of the rounded tile behind the symbol.',
  'icon.size': 'Edge length of the tile, in cells.',
  'icon.size_default': 'New pictograms match the dot station diameter unless a size is set.',
  'icon.rotation': 'Rotation in degrees. The rotate handles turn in steps of 45°; hold Alt to turn freely.',

  // Shapes
  'shape.kind':
    'Line: a straight line from one grid intersection to another, in one of 8 directions. Rectangle: covers whole cells. Circle: centered on a cell, its radius in steps of 0.1 cell. Hold Alt while drawing to use any point, size and angle.',
  'shape.line_color': 'Color of the line.',
  'shape.thickness': 'Width of the line, in cells.',
  'shape.cap': 'Shape of the line ends. Round ends stick out by half the line width; butt ends stop exactly at the end points.',
  'shape.dashed': 'Draw the line as dashes. Dashes and gaps are stretched slightly so the line starts and ends with a full dash.',
  'shape.dash_length': 'Length of each dash, in cells.',
  'shape.dash_gap': 'Space between two dashes, in cells.',
  'shape.fill': 'Fill the inside of the rectangle or circle. Without a fill only the border shows.',
  'shape.border_color': 'Color of the border.',
  'shape.border_width':
    'Width of the border, in cells. 0 draws no border. A rectangle or circle keeps its size: the border is drawn inside its edge. A line gets the border on both sides.',
  'shape.ends': 'Where the line starts and ends, in cells. Grid intersections have whole numbers.',
  'shape.width': 'Width of the rectangle, in cells.',
  'shape.height': 'Height of the rectangle, in cells.',
  'shape.radius': 'Radius of the circle, in cells. 0.5 fills exactly one cell.',
  'shape.rotation': 'Rotation in degrees around the center of the rectangle. The rotate handles turn in steps of 45°; hold Alt to turn freely. A quarter turn swaps the width and the height.',

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
  'prefs.dragThreshold': 'How far the pointer must move before a click becomes a drag, in screen pixels.',
  'prefs.autoSelect': 'What a click on the canvas selects: the layer under the pointer, or its outermost group.',
  'prefs.zoomStep': 'Zoom factor of one wheel step or zoom command.',
  'prefs.minZoom': 'Smallest zoom level (1 = 100%).',
  'prefs.maxZoom': 'Largest zoom level (1 = 100%).',
  'prefs.showGrid': 'Show the cell grid on the canvas. It never appears in exports.',
  'prefs.showThumbnails': 'Show a small preview of each layer in the Layers panel.',
  'prefs.nudge': 'How far the arrow keys move the selection, in cells.',
  'prefs.bigNudge': 'How far Shift + arrow keys move the selection, in cells.',
  'prefs.eraserSize': 'Default eraser width, in cells.',
  'prefs.canvasBackground': 'Color of the drawing area.',
  'prefs.pasteboardColor': 'Color around the drawing area.',
  'prefs.gridColor': 'Color of the cell grid.',
  'prefs.gridMajorColor': 'Color of the heavier grid lines.',
  'prefs.gridMajorEvery': 'Draw a heavier grid line every this many cells.',
  'prefs.minGridPixels': 'Hide the grid when cells are smaller than this on screen.',
  'prefs.selectionColor': 'Color of selection outlines and handles.',
  'prefs.warningColor': 'Color of the eraser brush and of cells where a station cannot be placed.',
  'prefs.previewOpacity': 'How opaque the previews of the pen, the station tool and dragged pictograms are.',
  'prefs.confirmDiscard': 'Ask before New throws away unsaved changes.',
  'prefs.newText': 'The text a new text box starts with (it is selected, so typing replaces it).',
  'prefs.messageDuration': 'How long messages stay in the status bar, in milliseconds.',
  'prefs.errorDuration': 'How long error messages stay in the status bar, in milliseconds.',
  'prefs.panels': 'Which panels are shown. Window ▸ Reset Workspace shows all of them again.',
  'prefs.propertiesHeight': 'Height of the Properties panel when the Layers panel is shown below it, in pixels. Dragging the border between them changes it too.',
  'prefs.panelEditPause': 'Changes made in the Properties panel (dragging a number, picking colors) become one undo step until they pause for this long, in milliseconds.',
  'prefs.fitMargin': 'Space left around the drawing by Fit on Screen, in screen pixels.',
  'prefs.fitMaxZoom': 'Largest zoom Fit on Screen uses for a small drawing (1 = 100%).',
  'prefs.snapDistance': 'How far from a line a click with the station tool still places a station on it, in cells.',
} as const

export type HelpKey = keyof typeof HELP
