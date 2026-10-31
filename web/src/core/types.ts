// Mirrors of the Rust `transit-core` serde types. All lengths are in grid cells.

export type Id = number

export interface Cell {
  x: number
  y: number
}

export interface Vec2 {
  x: number
  y: number
}

export interface Rect {
  x: number
  y: number
  w: number
  h: number
}

export type RouteMode = 'straight_first' | 'diagonal_first'
export type Cap = 'butt' | 'round'
export type StrokeKind = 'plain' | 'hollow' | 'inner_dashed'
export type StationKind = 'dot' | 'tick' | 'pill'
export type TextAlign = 'start' | 'middle' | 'end'
export type TextDirection = 'ltr' | 'rtl'

export interface DocSettings {
  line_width: number
  corner_radius: number
  cap_roundness: number
  station_outer_radius: number
  station_inner_radius: number
  tick_width: number
  tick_length: number
  label_offset: number
  label_font_size: number
  label_font_family: string
  icon_size: number | null
  route_mode: RouteMode
}

/** How the drawing is turned into a standalone SVG (not part of the document). */
export interface ExportOptions {
  /** Pixels per grid cell. */
  cell_px: number
  /** Empty space around the drawing, in cells. */
  margin: number
  /** Background color; empty for transparent. */
  background: string
}

export interface LineStyle {
  color: string
  kind: StrokeKind
  inner_color: string
  inner_width_pct: number
  dash_length: number
  dash_gap: number
  dash_width_pct: number
  /** Cells at the start of the line left solid before the dashes begin. */
  dash_start_offset: number
  /** Cells at the end of the line left solid after the dashes stop. */
  dash_end_offset: number
  start_cap: Cap
  end_cap: Cap
}

export interface LineSegment {
  kind: 'line'
  line_id: Id
  points: Cell[]
  style: LineStyle
}

export type StationShape =
  | { shape: 'dot'; cell: Cell; outer_color: string; inner_color: string }
  /** `along`: -1/+1 = corner of the cell along the line direction, 0 = centered. */
  | { shape: 'tick'; cell: Cell; dir: number; side: number; along: number; width: number; length: number; color: string }
  | { shape: 'pill'; cells: Cell[]; outer_color: string; inner_color: string }

/** `offset` is the free displacement from the line cell (Alt-drag), in cells. */
export type StationContent = { kind: 'station'; line_ids: Id[]; offset: Vec2 } & StationShape

/** Character formatting of a run; a missing value uses the text box's. */
export interface RunStyle {
  font_family?: string
  /** Font size in cells. */
  size?: number
  color?: string
  bold?: boolean
  italic?: boolean
}

export interface TextRun extends RunStyle {
  text: string
}

/** One line of a text box; `align` overrides the box alignment. */
export interface Paragraph {
  align?: TextAlign
  runs: TextRun[]
}

/** A text box: `pos` is its top-left corner and `width` its widest line, both in cells; it rotates around `pos`. */
export interface TextBox {
  pos: Vec2
  paragraphs: Paragraph[]
  width: number
  size: number
  color: string
  font_family: string
  bold: boolean
  italic: boolean
  align: TextAlign
  direction: TextDirection
  rotation: number
  station_id: Id | null
}

export interface Icon {
  pos: Vec2
  icon: string
  size: number
  rotation: number
  color: string
  background: string
}

export type ShapeKind = 'line' | 'rect' | 'circle'

/** Outline of a shape, in cells. */
export type ShapeGeometry =
  /** A straight line between two points (grid intersections unless drawn with Alt). */
  | { shape: 'line'; from: Vec2; to: Vec2; thickness: number; cap: Cap; dashed: boolean; dash_length: number; dash_gap: number }
  /** `pos` is the top-left corner; it turns around its center, `rotation` in (-45, 45]. `corner_radius` in cells, 0 for sharp corners. */
  | { shape: 'rect'; pos: Vec2; width: number; height: number; rotation: number; corner_radius: number }
  | { shape: 'circle'; center: Vec2; radius: number }

/** Colors and border shared by every shape. */
export interface ShapePaint {
  /** Color of a line, or the fill of a rectangle or circle. */
  color: string
  /** Rectangles and circles: false leaves them empty inside. */
  filled: boolean
  border_color: string
  /** In cells; 0 for no border. */
  border_width: number
}

export type Shape = ShapeGeometry & ShapePaint

export type Content =
  | LineSegment
  | StationContent
  | ({ kind: 'text' } & TextBox)
  | ({ kind: 'icon' } & Icon)
  | ({ kind: 'shape' } & Shape)

export type ContentKind = Content['kind']

export interface LayerNode {
  type: 'layer'
  id: Id
  name: string
  visible: boolean
  locked: boolean
  content: Content
}

export type GroupKind = { role: 'generic' } | { role: 'line'; line_id: Id } | { role: 'station' }

export interface GroupNode {
  type: 'group'
  id: Id
  name: string
  visible: boolean
  locked: boolean
  expanded: boolean
  kind: GroupKind
  children: DocNode[]
}

export type DocNode = LayerNode | GroupNode

/** Layers panel view; top of the stack first. */
export interface TreeItem {
  id: Id
  name: string
  kind: 'group' | ContentKind
  role?: 'generic' | 'line' | 'station'
  visible: boolean
  locked: boolean
  expanded: boolean
  children: TreeItem[]
}

export interface Fragment {
  id: Id
  kind: ContentKind
  svg: string
  locked: boolean
  bounds: Rect | null
}

export interface LineHit {
  layer_id: Id
  line_id: Id
  cell: Cell
  dir: number
}

export interface StationOptions {
  outer_color: string
  inner_color: string
  tick_color: string | null
  tick_side: number
  tick_width: number | null
  tick_length: number | null
  label: boolean
  label_color: string
  /** How far from a line (cells) a click still places a station on it. */
  snap_distance: number
}

export interface CreatedLine {
  group_id: Id
  layer_id: Id
  line_id: Id
}

export interface CreatedStation {
  group_id: Id
  station_id: Id
  text_id: Id | null
}

export interface IconInfo {
  name: string
  label: string
  svg: string
}

/** Fonts and icons uploaded into the document. */
export interface AssetList {
  /** Font family names. */
  fonts: string[]
  icons: IconInfo[]
}
