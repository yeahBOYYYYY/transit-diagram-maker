export type ToolType = 'Select' | 'Pen' | 'Erase' | 'Station' | 'Text';

export interface DVec2 {
  x: number;
  y: number;
}

export interface Aabb {
  min: DVec2;
  max: DVec2;
}

export type StrokeStyle =
  | 'Solid'
  | { Hollow: { inner_width: number; inner_color: string } }
  | { Dashed: { dash_len: number; gap_len: number } };

export type LineCap = 'Round' | 'Square' | 'Butt';

export type StationStyle = 'Dot' | 'Tick';

export type PathSegment =
  | { MoveTo: DVec2 }
  | { LineTo: DVec2 }
  | { QuadTo: { ctrl: DVec2; to: DVec2 } }
  | 'Close';

export interface RenderPrimitive {
  id: number;
  z_index: number;
  primitive_type:
    | {
        Path: {
          segments: PathSegment[];
          stroke_color: string;
          stroke_width: number;
          fill_color?: string | null;
          stroke_style: StrokeStyle;
          line_cap: LineCap;
        };
      }
    | {
        Station: {
          center: DVec2;
          end_pos: DVec2;
          radius: number;
          style: StationStyle;
          fill_color: string;
          stroke_color: string;
          stroke_width: number;
          rotation_rad: number;
          span_cells: number;
        };
      }
    | {
        Text: {
          text: string;
          pos: DVec2;
          font_size: number;
          color: string;
          is_colliding: boolean;
          bounds: Aabb;
        };
      };
}

export type ParameterType =
  | { Color: string }
  | { Number: { value: number; min: number; max: number; step: number } }
  | { Text: string }
  | { Choice: { current: string; options: string[] } }
  | { Bool: boolean };

export interface ParameterSchema {
  name: string;
  label: string;
  param_type: ParameterType;
}

export interface NodeSummary {
  id: number;
  name: string;
  type_name: string;
  visible: boolean;
  locked: boolean;
  z_index: number;
}

export interface ViewportCamera {
  pan: DVec2;
  zoom: number;
  width: number;
  height: number;
  grid_spacing: number;
}

export interface ExportPayload {
  format: string;
  content: string;
  filename: string;
}

export interface EditorStateResponse {
  primitives: RenderPrimitive[];
  selected_node_ids: number[];
  selected_parameters: ParameterSchema[] | null;
  active_tool: ToolType;
  camera: ViewportCamera;
  can_undo: boolean;
  can_redo: boolean;
  colliding_text_count: number;
  node_summaries: NodeSummary[];
  export_payload: ExportPayload | null;
}

export type EditorAction =
  | { type: 'SelectTool'; payload: ToolType }
  | { type: 'PointerDown'; payload: { screen_pos: DVec2; shift: boolean; alt: boolean; ctrl: boolean; button: number } }
  | { type: 'PointerMove'; payload: { screen_pos: DVec2; shift: boolean; alt: boolean; ctrl: boolean } }
  | { type: 'PointerUp'; payload: { screen_pos: DVec2; shift: boolean; alt: boolean; ctrl: boolean; button: number } }
  | { type: 'Wheel'; payload: { screen_pos: DVec2; delta_y: number } }
  | { type: 'KeyDown'; payload: { key: string; shift: boolean; ctrl: boolean } }
  | { type: 'KeyUp'; payload: { key: string; shift: boolean; ctrl: boolean } }
  | { type: 'ResizeViewport'; payload: { width: number; height: number } }
  | { type: 'RecenterView' }
  | { type: 'Undo' }
  | { type: 'Redo' }
  | { type: 'SetNodeParameter'; payload: { node_id: number; param_name: string; value: any } }
  | { type: 'SelectNode'; payload: { node_id: number; multi: boolean } }
  | { type: 'ClearSelection' }
  | { type: 'DeleteSelected' }
  | { type: 'ToggleNodeVisibility'; payload: number }
  | { type: 'ToggleNodeLock'; payload: number }
  | { type: 'ReorderNodeZIndex'; payload: { node_id: number; new_z_index: number } }
  | { type: 'ExportSvg' }
  | { type: 'ExportTransitDocument' }
  | { type: 'LoadTransitDocument'; payload: string };
