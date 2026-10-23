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
export type StrokeKind = 'plain' | 'hollow' | 'dashed'
export type StationKind = 'dot' | 'tick' | 'pill'
export type TextAlign = 'start' | 'middle' | 'end'

export interface DocSettings {
  grid_size: number
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
  export_margin: number
  export_background: string
  route_mode: RouteMode
}

export interface LineStyle {
  color: string
  kind: StrokeKind
  inner_color: string
  inner_width_pct: number
  dash_length: number
  dash_gap: number
  dash_width_pct: number
  dash_offset: number
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
  | { shape: 'tick'; cell: Cell; dir: number; side: number; width: number; length: number; color: string }
  | { shape: 'pill'; cells: Cell[]; outer_color: string; inner_color: string }

/** `offset` is the free displacement from the line cell (Alt-drag), in cells. */
export type StationContent = { kind: 'station'; line_ids: Id[]; offset: Vec2 } & StationShape

export interface TextBox {
  pos: Vec2
  text: string
  size: number
  color: string
  font_family: string
  bold: boolean
  italic: boolean
  align: TextAlign
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

export type Content =
  | LineSegment
  | StationContent
  | ({ kind: 'text' } & TextBox)
  | ({ kind: 'icon' } & Icon)

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
