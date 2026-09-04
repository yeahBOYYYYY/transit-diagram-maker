use serde::{Deserialize, Serialize};

use crate::bvh::{detect_text_collisions, SpatialItem};
use crate::coord::{
    project_point_to_segment, snap_to_grid, snap_transit_octolinear_to_grid, DVec2,
};
use crate::graph::{
    DocumentNode, DocumentTree, LineCap, NodeData, NodeId, StationStyle, StrokeStyle,
};
use crate::history::DocumentUndoHistory;
use crate::messages::{
    EditorAction, EditorStateResponse, ExportPayload, NodeSummary, ToolType,
};
use crate::render::export_svg;
use crate::tools::ToolState;
use crate::viewport::ViewportCamera;

/// Full state representation saved in the standalone .transit document format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitDocumentFile {
    pub version: u32,
    pub tree: DocumentTree,
    pub camera: ViewportCamera,
}

pub struct EditorEngine {
    pub tree: DocumentTree,
    pub history: DocumentUndoHistory,
    pub camera: ViewportCamera,
    pub active_tool: ToolType,
    pub tool_state: ToolState,
    pub selected_node_ids: Vec<NodeId>,
    pub shift_held: bool,
    pub ctrl_held: bool,
    pub alt_held: bool,
    pub last_export: Option<ExportPayload>,
}

impl EditorEngine {
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self {
        let mut tree = DocumentTree::new();
        let camera = ViewportCamera::new(viewport_width, viewport_height);

        // Prepopulate with a pristine metro demo where all lines run strictly along grid channels:
        // Central Red Line (Solid)
        let red_points = vec![
            DVec2::new(-240.0, 160.0),
            DVec2::new(-120.0, 160.0),
            DVec2::new(0.0, 40.0),
            DVec2::new(140.0, 40.0),
            DVec2::new(260.0, 160.0),
        ];
        let red_line_id = tree.add_node(
            NodeData::TransitLine {
                name: "Line 1 (Red)".into(),
                points: red_points.clone(),
                corner_radius: 20.0,
                color: "#e63946".into(),
                stroke_width: 8.0,
                style: StrokeStyle::Solid,
                cap: LineCap::Round,
                hidden_segments: Vec::new(),
            },
            10,
        );

        // North-South Blue Line (Hollow with white center)
        let blue_points = vec![
            DVec2::new(0.0, -160.0),
            DVec2::new(0.0, 40.0),
            DVec2::new(0.0, 200.0),
        ];
        let _blue_line_id = tree.add_node(
            NodeData::TransitLine {
                name: "Line 2 (Blue)".into(),
                points: blue_points,
                corner_radius: 20.0,
                color: "#007aff".into(),
                stroke_width: 10.0,
                style: StrokeStyle::Hollow {
                    inner_width: 4.0,
                    inner_color: "#ffffff".into(),
                },
                cap: LineCap::Round,
                hidden_segments: Vec::new(),
            },
            11,
        );

        // Stations along red line
        let stations = vec![
            ("West End", red_points[0], false),
            ("Civic Center", red_points[1], false),
            ("Central Interchange", red_points[2], true), // Slinky stretched dot interchange!
            ("Grand Terminal", red_points[3], false),
            ("East Pier", red_points[4], false),
        ];

        for (name, pos, is_interchange) in stations {
            let text_id = tree.allocate_id();
            let (span_cells, span_angle_deg) = if is_interchange {
                (2, 90.0) // Stretch vertically across the intersection!
            } else {
                (0, 0.0)
            };

            let station_id = tree.add_node(
                NodeData::Station {
                    name: name.into(),
                    pos,
                    associated_line_id: Some(red_line_id),
                    style: if name == "Grand Terminal" {
                        StationStyle::Tick
                    } else {
                        StationStyle::Dot
                    },
                    radius: 6.5,
                    fill_color: "#ffffff".into(),
                    stroke_color: "#151518".into(),
                    stroke_width: 3.0,
                    rotation_rad: 0.0,
                    child_text_id: Some(text_id),
                    span_cells,
                    span_angle_deg,
                },
                20,
            );

            let text_offset = if is_interchange {
                DVec2::new(16.0, 10.0)
            } else {
                DVec2::new(12.0, -14.0)
            };

            let text_node = DocumentNode {
                id: text_id,
                visible: true,
                locked: false,
                z_index: 30,
                data: NodeData::Text {
                    text: name.into(),
                    pos: pos + text_offset,
                    font_size: 13.0,
                    color: "#f1faee".into(),
                    parent_station_id: Some(station_id),
                    offset_from_station: text_offset,
                },
            };
            tree.nodes.insert(text_id, text_node);
            tree.node_order.push(text_id);
        }

        Self {
            tree,
            history: DocumentUndoHistory::new(50),
            camera,
            active_tool: ToolType::Select,
            tool_state: ToolState::new_select(),
            selected_node_ids: Vec::new(),
            shift_held: false,
            ctrl_held: false,
            alt_held: false,
            last_export: None,
        }
    }

    pub fn dispatch(&mut self, action: EditorAction) -> EditorStateResponse {
        self.last_export = None;

        match action {
            EditorAction::SelectTool(tool) => {
                self.commit_pen_if_active();
                self.active_tool = tool;
                self.tool_state = match tool {
                    ToolType::Select => ToolState::new_select(),
                    ToolType::Pen => ToolState::new_pen(),
                    ToolType::Erase => ToolState::new_erase(),
                    ToolType::Station => ToolState::new_station(),
                    ToolType::Text => ToolState::new_text(),
                };
            }
            EditorAction::PointerDown { screen_pos, shift, alt, ctrl, button } => {
                self.shift_held = shift;
                self.alt_held = alt;
                self.ctrl_held = ctrl;
                self.handle_pointer_down(screen_pos, button);
            }
            EditorAction::PointerMove { screen_pos, shift, alt, ctrl } => {
                self.shift_held = shift;
                self.alt_held = alt;
                self.ctrl_held = ctrl;
                self.handle_pointer_move(screen_pos);
            }
            EditorAction::PointerUp { screen_pos, shift, alt, ctrl, button } => {
                self.shift_held = shift;
                self.alt_held = alt;
                self.ctrl_held = ctrl;
                self.handle_pointer_up(screen_pos, button);
            }
            EditorAction::Wheel { screen_pos, delta_y } => {
                let zoom_factor = if delta_y < 0.0 { 1.15 } else { 1.0 / 1.15 };
                self.camera.zoom_at_screen_pos(screen_pos, zoom_factor);
            }
            EditorAction::KeyDown { key, shift, ctrl } => {
                self.shift_held = shift;
                self.ctrl_held = ctrl;
                self.handle_key_down(&key);
            }
            EditorAction::KeyUp { key: _, shift, ctrl } => {
                self.shift_held = shift;
                self.ctrl_held = ctrl;
            }
            EditorAction::ResizeViewport { width, height } => {
                self.camera.resize(width, height);
            }
            EditorAction::RecenterView => {
                let mut bounds: Option<crate::coord::Aabb> = None;
                for &id in &self.tree.node_order {
                    if let Some(node) = self.tree.nodes.get(&id) {
                        if node.visible {
                            let b = node.aabb();
                            bounds = Some(match bounds {
                                Some(acc) => acc.union(&b),
                                None => b,
                            });
                        }
                    }
                }
                if let Some(b) = bounds {
                    self.camera.recenter_to_bounds(b);
                }
            }
            EditorAction::Undo => {
                if let Some(prev) = self.history.undo(self.tree.clone()) {
                    self.tree = prev;
                }
            }
            EditorAction::Redo => {
                if let Some(next) = self.history.redo(self.tree.clone()) {
                    self.tree = next;
                }
            }
            EditorAction::SelectNode { node_id, multi } => {
                if multi {
                    if let Some(idx) = self.selected_node_ids.iter().position(|&x| x == node_id) {
                        self.selected_node_ids.remove(idx);
                    } else {
                        self.selected_node_ids.push(node_id);
                    }
                } else {
                    self.selected_node_ids = vec![node_id];
                }
            }
            EditorAction::ClearSelection => {
                self.selected_node_ids.clear();
            }
            EditorAction::DeleteSelected => {
                if !self.selected_node_ids.is_empty() {
                    self.history.push(self.tree.clone());
                    for id in &self.selected_node_ids {
                        self.tree.remove_node(*id);
                    }
                    self.selected_node_ids.clear();
                }
            }
            EditorAction::ToggleNodeVisibility(node_id) => {
                if let Some(node) = self.tree.get_node_mut(node_id) {
                    node.visible = !node.visible;
                }
            }
            EditorAction::ToggleNodeLock(node_id) => {
                if let Some(node) = self.tree.get_node_mut(node_id) {
                    node.locked = !node.locked;
                }
            }
            EditorAction::ReorderNodeZIndex { node_id, new_z_index } => {
                if let Some(node) = self.tree.get_node_mut(node_id) {
                    node.z_index = new_z_index;
                }
            }
            EditorAction::SetNodeParameter { node_id, param_name, value } => {
                self.update_node_parameter(node_id, &param_name, value);
            }
            EditorAction::ExportSvg => {
                let svg_content = export_svg(&self.tree);
                self.last_export = Some(ExportPayload {
                    format: "svg".into(),
                    content: svg_content,
                    filename: "transit_map.svg".into(),
                });
            }
            EditorAction::ExportTransitDocument => {
                let doc = TransitDocumentFile {
                    version: 1,
                    tree: self.tree.clone(),
                    camera: self.camera.clone(),
                };
                if let Ok(json) = serde_json::to_string_pretty(&doc) {
                    self.last_export = Some(ExportPayload {
                        format: "transit".into(),
                        content: json,
                        filename: "transit_map.transit".into(),
                    });
                }
            }
            EditorAction::LoadTransitDocument(json) => {
                if let Ok(doc) = serde_json::from_str::<TransitDocumentFile>(&json) {
                    self.history.push(self.tree.clone());
                    self.tree = doc.tree;
                    self.camera = doc.camera;
                    self.selected_node_ids.clear();
                }
            }
        }

        self.build_response()
    }

    fn handle_pointer_down(&mut self, screen_pos: DVec2, button: u32) {
        if button == 1 {
            return;
        }

        let world_pos = self.camera.screen_to_world(screen_pos);
        // STRICT grid snapping: lines and stations always lock strictly into grid lines
        let grid_pos = snap_to_grid(world_pos, self.camera.grid_spacing);

        match self.active_tool {
            ToolType::Select => {
                let hit_node_id = self.hit_test(world_pos);
                if let Some(id) = hit_node_id {
                    if self.shift_held {
                        if let Some(pos) = self.selected_node_ids.iter().position(|&x| x == id) {
                            self.selected_node_ids.remove(pos);
                        } else {
                            self.selected_node_ids.push(id);
                        }
                    } else if !self.selected_node_ids.contains(&id) {
                        self.selected_node_ids = vec![id];
                    }
                    self.tool_state = ToolState::Select {
                        drag_start: Some(grid_pos),
                        last_world_pos: Some(grid_pos),
                        is_dragging_nodes: true,
                    };
                } else {
                    if !self.shift_held {
                        self.selected_node_ids.clear();
                    }
                    self.tool_state = ToolState::Select {
                        drag_start: Some(grid_pos),
                        last_world_pos: Some(grid_pos),
                        is_dragging_nodes: false,
                    };
                }
            }
            ToolType::Pen => {
                // Strictly lock to grid channel:
                // Horizontal lines lie strictly on a single grid line (y = y0)
                // Vertical lines lie strictly on a single grid line (x = x0)
                // Diagonal lines connect grid intersections with equal dx and dy
                let current_point = if let ToolState::Pen { points, .. } = &self.tool_state {
                    if let Some(&last) = points.last() {
                        snap_transit_octolinear_to_grid(last, world_pos, self.camera.grid_spacing)
                    } else {
                        grid_pos
                    }
                } else {
                    grid_pos
                };

                if let ToolState::Pen { points, current_preview, .. } = &mut self.tool_state {
                    points.push(current_point);
                    *current_preview = Some(current_point);
                }
            }
            ToolType::Station => {
                self.history.push(self.tree.clone());
                let snapped = self.tree.snap_to_nearest_line(world_pos, 30.0);
                let station_pos = snapped.map(|s| snap_to_grid(s.1, self.camera.grid_spacing)).unwrap_or(grid_pos);
                let line_id = snapped.map(|s| s.0);

                let text_id = self.tree.allocate_id();
                let station_id = self.tree.add_node(
                    NodeData::Station {
                        name: "New Station".into(),
                        pos: station_pos,
                        associated_line_id: line_id,
                        style: StationStyle::Dot,
                        radius: 7.0,
                        fill_color: "#ffffff".into(),
                        stroke_color: "#151518".into(),
                        stroke_width: 3.0,
                        rotation_rad: 0.0,
                        child_text_id: Some(text_id),
                        span_cells: 0,
                        span_angle_deg: 0.0,
                    },
                    20,
                );

                let text_node = DocumentNode {
                    id: text_id,
                    visible: true,
                    locked: false,
                    z_index: 30,
                    data: NodeData::Text {
                        text: "New Station".into(),
                        pos: station_pos + DVec2::new(12.0, -14.0),
                        font_size: 13.0,
                        color: "#f1faee".into(),
                        parent_station_id: Some(station_id),
                        offset_from_station: DVec2::new(12.0, -14.0),
                    },
                };
                self.tree.nodes.insert(text_id, text_node);
                self.tree.node_order.push(text_id);

                self.selected_node_ids = vec![station_id];
            }
            ToolType::Text => {
                self.history.push(self.tree.clone());
                let text_id = self.tree.add_node(
                    NodeData::Text {
                        text: "Text Label".into(),
                        pos: grid_pos,
                        font_size: 14.0,
                        color: "#f1faee".into(),
                        parent_station_id: None,
                        offset_from_station: DVec2::ZERO,
                    },
                    30,
                );
                self.selected_node_ids = vec![text_id];
            }
            ToolType::Erase => {
                // ERASE TOOL WORKS ON STATIONS AS WELL AS LINES!
                let hit = self.hit_test(world_pos);
                if let Some(id) = hit {
                    if let Some(node) = self.tree.get_node(id) {
                        if matches!(node.data, NodeData::Station { .. }) {
                            self.history.push(self.tree.clone());
                            // If station has child text, remove child text too
                            let child_id = match &node.data {
                                NodeData::Station { child_text_id, .. } => *child_text_id,
                                _ => None,
                            };
                            if let Some(cid) = child_id {
                                self.tree.remove_node(cid);
                            }
                            self.tree.remove_node(id);
                            self.selected_node_ids.retain(|&x| x != id);
                            return;
                        }
                    }
                }

                // If not clicking directly on a station, slice nearest transit line
                if let Some((line_id, clicked_pos)) = self.tree.snap_to_nearest_line(world_pos, 25.0) {
                    self.history.push(self.tree.clone());
                    if let Some(DocumentNode { data: NodeData::TransitLine { points, hidden_segments, .. }, .. }) = self.tree.nodes.get_mut(&line_id) {
                        let mut closest_seg = 0;
                        let mut min_d = f64::MAX;
                        for i in 0..points.len().saturating_sub(1) {
                            let (_proj, _t, dist) = project_point_to_segment(clicked_pos, points[i], points[i + 1]);
                            if dist < min_d {
                                min_d = dist;
                                closest_seg = i;
                            }
                        }
                        if !hidden_segments.contains(&closest_seg) {
                            hidden_segments.push(closest_seg);
                        }
                    }
                }
            }
        }
    }

    fn handle_pointer_move(&mut self, screen_pos: DVec2) {
        let world_pos = self.camera.screen_to_world(screen_pos);
        let grid_pos = snap_to_grid(world_pos, self.camera.grid_spacing);

        let mut drag_delta = None;
        if let ToolState::Select { last_world_pos, is_dragging_nodes: true, .. } = &mut self.tool_state {
            if let Some(last) = *last_world_pos {
                let delta = grid_pos - last;
                if delta.length_squared() > 1e-6 {
                    drag_delta = Some(delta);
                    *last_world_pos = Some(grid_pos);
                }
            }
        }
        if let Some(delta) = drag_delta {
            self.translate_selected(delta);
        }

        let snapped_line = if matches!(self.tool_state, ToolState::Station { .. }) {
            self.tree.snap_to_nearest_line(world_pos, 30.0)
        } else {
            None
        };

        match &mut self.tool_state {
            ToolState::Pen { points, current_preview, .. } => {
                if let Some(&last) = points.last() {
                    let strict_point = snap_transit_octolinear_to_grid(last, world_pos, self.camera.grid_spacing);
                    *current_preview = Some(strict_point);
                } else {
                    *current_preview = Some(grid_pos);
                }
            }
            ToolState::Station { hover_pos, snapped_line_id, .. } => {
                if let Some((line_id, proj)) = snapped_line {
                    *hover_pos = Some(snap_to_grid(proj, self.camera.grid_spacing));
                    *snapped_line_id = Some(line_id);
                } else {
                    *hover_pos = Some(grid_pos);
                    *snapped_line_id = None;
                }
            }
            _ => {}
        }
    }

    fn handle_pointer_up(&mut self, _screen_pos: DVec2, button: u32) {
        if button == 1 {
            return;
        }

        if let ToolState::Select { is_dragging_nodes, drag_start, .. } = &mut self.tool_state {
            if *is_dragging_nodes {
                self.history.push(self.tree.clone());
            }
            *is_dragging_nodes = false;
            *drag_start = None;
        }
    }

    fn handle_key_down(&mut self, key: &str) {
        match key {
            "Enter" | "Escape" => {
                self.commit_pen_if_active();
            }
            "Delete" | "Backspace" => {
                if !self.selected_node_ids.is_empty() {
                    self.history.push(self.tree.clone());
                    for id in &self.selected_node_ids {
                        self.tree.remove_node(*id);
                    }
                    self.selected_node_ids.clear();
                }
            }
            "z" | "Z" if self.ctrl_held => {
                if self.shift_held {
                    if let Some(next) = self.history.redo(self.tree.clone()) {
                        self.tree = next;
                    }
                } else if let Some(prev) = self.history.undo(self.tree.clone()) {
                    self.tree = prev;
                }
            }
            "v" | "V" => {
                self.dispatch(EditorAction::SelectTool(ToolType::Select));
            }
            "p" | "P" => {
                self.dispatch(EditorAction::SelectTool(ToolType::Pen));
            }
            "e" | "E" => {
                self.dispatch(EditorAction::SelectTool(ToolType::Erase));
            }
            "s" | "S" => {
                self.dispatch(EditorAction::SelectTool(ToolType::Station));
            }
            "t" | "T" => {
                self.dispatch(EditorAction::SelectTool(ToolType::Text));
            }
            _ => {}
        }
    }

    fn commit_pen_if_active(&mut self) {
        if let ToolState::Pen { points, active_color, active_width, corner_radius, .. } = &self.tool_state {
            if points.len() >= 2 {
                self.history.push(self.tree.clone());
                let line_id = self.tree.add_node(
                    NodeData::TransitLine {
                        name: format!("Line {}", self.tree.node_order.len() + 1),
                        points: points.clone(),
                        corner_radius: *corner_radius,
                        color: active_color.clone(),
                        stroke_width: *active_width,
                        style: StrokeStyle::Solid,
                        cap: LineCap::Round,
                        hidden_segments: Vec::new(),
                    },
                    10,
                );
                self.selected_node_ids = vec![line_id];
            }
        }
        if matches!(self.tool_state, ToolState::Pen { .. }) {
            self.tool_state = ToolState::new_pen();
        }
    }

    fn translate_selected(&mut self, delta: DVec2) {
        for &id in &self.selected_node_ids {
            if let Some(node) = self.tree.nodes.get_mut(&id) {
                if node.locked {
                    continue;
                }
                match &mut node.data {
                    NodeData::TransitLine { points, .. } => {
                        for p in points {
                            *p = *p + delta;
                        }
                    }
                    NodeData::Station { pos, child_text_id, .. } => {
                        *pos = *pos + delta;
                        if let Some(child_id) = *child_text_id {
                            if let Some(child) = self.tree.nodes.get_mut(&child_id) {
                                if let NodeData::Text { pos: text_pos, .. } = &mut child.data {
                                    *text_pos = *text_pos + delta;
                                }
                            }
                        }
                    }
                    NodeData::Text { pos, .. } => {
                        *pos = *pos + delta;
                    }
                }
            }
        }
    }

    fn hit_test(&self, world_pos: DVec2) -> Option<NodeId> {
        for &id in self.tree.node_order.iter().rev() {
            if let Some(node) = self.tree.nodes.get(&id) {
                if !node.visible || node.locked {
                    continue;
                }
                match &node.data {
                    NodeData::Station { pos, radius, span_cells, span_angle_deg, .. } => {
                        let angle_rad = span_angle_deg.to_radians();
                        let dir = DVec2::new(angle_rad.cos(), angle_rad.sin());
                        let end_pos = *pos + dir * (*span_cells as f64 * 20.0);
                        let (_proj, _t, dist) = project_point_to_segment(world_pos, *pos, end_pos);
                        if dist <= *radius + 8.0 {
                            return Some(id);
                        }
                    }
                    NodeData::Text { .. } => {
                        if node.aabb().contains_point(world_pos) {
                            return Some(id);
                        }
                    }
                    NodeData::TransitLine { points, stroke_width, .. } => {
                        for i in 0..points.len().saturating_sub(1) {
                            let (_proj, _t, dist) = project_point_to_segment(world_pos, points[i], points[i + 1]);
                            if dist <= (*stroke_width * 0.5 + 8.0) {
                                return Some(id);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn update_node_parameter(&mut self, node_id: NodeId, param_name: &str, value: serde_json::Value) {
        if let Some(node) = self.tree.get_node_mut(node_id) {
            match &mut node.data {
                NodeData::TransitLine { name, color, stroke_width, corner_radius, style, cap, .. } => {
                    match param_name {
                        "name" => if let Some(s) = value.as_str() { *name = s.to_string(); },
                        "color" => if let Some(s) = value.as_str() { *color = s.to_string(); },
                        "stroke_width" => if let Some(v) = value.as_f64() { *stroke_width = v; },
                        "corner_radius" => if let Some(v) = value.as_f64() { *corner_radius = v; },
                        "pattern" => if let Some(s) = value.as_str() {
                            *style = match s {
                                "Dashed" => StrokeStyle::Dashed { dash_len: 12.0, gap_len: 6.0 },
                                "Hollow" => StrokeStyle::Hollow {
                                    inner_width: (*stroke_width * 0.45).max(3.0),
                                    inner_color: "#ffffff".into(),
                                },
                                _ => StrokeStyle::Solid,
                            };
                        },
                        "inner_color" => if let Some(s) = value.as_str() {
                            if let StrokeStyle::Hollow { inner_color, .. } = style {
                                *inner_color = s.to_string();
                            } else {
                                *style = StrokeStyle::Hollow {
                                    inner_width: (*stroke_width * 0.45).max(3.0),
                                    inner_color: s.to_string(),
                                };
                            }
                        },
                        "cap" => if let Some(s) = value.as_str() {
                            *cap = match s {
                                "Square" => LineCap::Square,
                                "Butt" => LineCap::Butt,
                                _ => LineCap::Round,
                            };
                        },
                        _ => {}
                    }
                }
                NodeData::Station { name, style, radius, fill_color, stroke_color, rotation_rad, span_cells, span_angle_deg, .. } => {
                    match param_name {
                        "name" => if let Some(s) = value.as_str() { *name = s.to_string(); },
                        "style" => if let Some(s) = value.as_str() {
                            *style = match s {
                                "Tick" => StationStyle::Tick,
                                _ => StationStyle::Dot,
                            };
                        },
                        "radius" => if let Some(v) = value.as_f64() { *radius = v; },
                        "span_cells" => if let Some(v) = value.as_f64() { *span_cells = v.round().max(0.0) as u32; },
                        "span_direction" => if let Some(s) = value.as_str() {
                            *span_angle_deg = if s.starts_with("90") {
                                90.0
                            } else if s.starts_with("180") {
                                180.0
                            } else if s.starts_with("270") {
                                270.0
                            } else if s.starts_with("45") {
                                45.0
                            } else if s.starts_with("135") {
                                135.0
                            } else {
                                0.0
                            };
                        },
                        "fill_color" => if let Some(s) = value.as_str() { *fill_color = s.to_string(); },
                        "stroke_color" => if let Some(s) = value.as_str() { *stroke_color = s.to_string(); },
                        "rotation_deg" => if let Some(v) = value.as_f64() { *rotation_rad = v.to_radians(); },
                        _ => {}
                    }
                }
                NodeData::Text { text, font_size, color, .. } => {
                    match param_name {
                        "text" => if let Some(s) = value.as_str() { *text = s.to_string(); },
                        "font_size" => if let Some(v) = value.as_f64() { *font_size = v; },
                        "color" => if let Some(s) = value.as_str() { *color = s.to_string(); },
                        _ => {}
                    }
                }
            }
        }
    }

    fn build_response(&self) -> EditorStateResponse {
        let mut text_items = Vec::new();
        for &id in &self.tree.node_order {
            if let Some(node) = self.tree.nodes.get(&id) {
                if node.visible && matches!(node.data, NodeData::Text { .. }) {
                    text_items.push(SpatialItem {
                        id,
                        aabb: node.aabb(),
                    });
                }
            }
        }

        let colliding_text_ids = detect_text_collisions(text_items);
        let colliding_count = colliding_text_ids.len();

        let mut primitives = self.tree.evaluate(&colliding_text_ids);

        match &self.tool_state {
            ToolState::Pen { points, current_preview, active_color, active_width, corner_radius } => {
                let mut full_pts = points.clone();
                if let Some(prev) = current_preview {
                    full_pts.push(*prev);
                }
                if full_pts.len() >= 2 {
                    let segs = crate::graph::generate_transit_curve_segments(&full_pts, *corner_radius, &[]);
                    primitives.push(crate::graph::RenderPrimitive {
                        id: 999_999,
                        z_index: 999,
                        primitive_type: crate::graph::RenderPrimitiveType::Path {
                            segments: segs,
                            stroke_color: active_color.clone(),
                            stroke_width: *active_width,
                            fill_color: None,
                            stroke_style: StrokeStyle::Dashed { dash_len: 6.0, gap_len: 4.0 },
                            line_cap: LineCap::Round,
                        },
                    });
                }
            }
            ToolState::Station { hover_pos: Some(pos), active_style, active_radius, .. } => {
                primitives.push(crate::graph::RenderPrimitive {
                    id: 999_998,
                    z_index: 999,
                    primitive_type: crate::graph::RenderPrimitiveType::Station {
                        center: *pos,
                        end_pos: *pos,
                        radius: *active_radius,
                        style: *active_style,
                        fill_color: "#ffffff".into(),
                        stroke_color: "#007aff".into(),
                        stroke_width: 2.5,
                        rotation_rad: 0.0,
                        span_cells: 0,
                    },
                });
            }
            _ => {}
        }

        let selected_parameters = self
            .selected_node_ids
            .first()
            .and_then(|id| self.tree.get_node(*id))
            .map(|node| node.get_parameters());

        let node_summaries = self
            .tree
            .node_order
            .iter()
            .filter_map(|&id| {
                self.tree.get_node(id).map(|n| NodeSummary {
                    id: n.id,
                    name: n.name().to_string(),
                    type_name: n.type_name().to_string(),
                    visible: n.visible,
                    locked: n.locked,
                    z_index: n.z_index,
                })
            })
            .collect();

        EditorStateResponse {
            primitives,
            selected_node_ids: self.selected_node_ids.clone(),
            selected_parameters,
            active_tool: self.active_tool,
            camera: self.camera.clone(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            colliding_text_count: colliding_count,
            node_summaries,
            export_payload: self.last_export.clone(),
        }
    }
}
