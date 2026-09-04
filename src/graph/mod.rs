use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::coord::{project_point_to_segment, Aabb, DVec2};

pub type NodeId = u64;

/// Styling options for line strokes: Solid, Dashed, or Hollow (with inner color)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StrokeStyle {
    Solid,
    Dashed { dash_len: f64, gap_len: f64 },
    Hollow { inner_width: f64, inner_color: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LineCap {
    Round,
    Square,
    Butt,
}

/// Stations can strictly be only a Dot or a Tick!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationStyle {
    Dot,
    Tick,
}

/// Evaluated path segment primitive for rendering and export
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    MoveTo(DVec2),
    LineTo(DVec2),
    QuadTo { ctrl: DVec2, to: DVec2 },
    Close,
}

/// Evaluated procedural vector primitive ready for canvas/SVG rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPrimitive {
    pub id: NodeId,
    pub z_index: i32,
    pub primitive_type: RenderPrimitiveType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderPrimitiveType {
    Path {
        segments: Vec<PathSegment>,
        stroke_color: String,
        stroke_width: f64,
        fill_color: Option<String>,
        stroke_style: StrokeStyle,
        line_cap: LineCap,
    },
    Station {
        center: DVec2,
        end_pos: DVec2,
        radius: f64,
        style: StationStyle,
        fill_color: String,
        stroke_color: String,
        stroke_width: f64,
        rotation_rad: f64,
        span_cells: u32,
    },
    Text {
        text: String,
        pos: DVec2,
        font_size: f64,
        color: String,
        is_colliding: bool,
        bounds: Aabb,
    },
}

/// Parameter definition for Vue frontend dynamic property inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub name: String,
    pub label: String,
    pub param_type: ParameterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Color(String),
    Number { value: f64, min: f64, max: f64, step: f64 },
    Text(String),
    Choice { current: String, options: Vec<String> },
    Bool(bool),
}

/// Procedural document nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeData {
    TransitLine {
        name: String,
        points: Vec<DVec2>,
        corner_radius: f64,
        color: String,
        stroke_width: f64,
        style: StrokeStyle,
        cap: LineCap,
        hidden_segments: Vec<usize>,
    },
    Station {
        name: String,
        pos: DVec2,
        associated_line_id: Option<NodeId>,
        style: StationStyle,
        radius: f64,
        fill_color: String,
        stroke_color: String,
        stroke_width: f64,
        rotation_rad: f64,
        child_text_id: Option<NodeId>,
        /// Number of grid units to stretch across parallel lines (0 = standard dot)
        span_cells: u32,
        /// Stretch direction angle in degrees (0, 45, 90, 135, 180, 270)
        span_angle_deg: f64,
    },
    Text {
        text: String,
        pos: DVec2,
        font_size: f64,
        color: String,
        parent_station_id: Option<NodeId>,
        offset_from_station: DVec2,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentNode {
    pub id: NodeId,
    pub visible: bool,
    pub locked: bool,
    pub z_index: i32,
    pub data: NodeData,
}

impl DocumentNode {
    pub fn name(&self) -> &str {
        match &self.data {
            NodeData::TransitLine { name, .. } => name,
            NodeData::Station { name, .. } => name,
            NodeData::Text { text, .. } => text,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match &self.data {
            NodeData::TransitLine { .. } => "Transit Line",
            NodeData::Station { .. } => "Station",
            NodeData::Text { .. } => "Text Label",
        }
    }

    /// Calculate bounding box for hit-testing and BVH
    pub fn aabb(&self) -> Aabb {
        match &self.data {
            NodeData::TransitLine { points, stroke_width, .. } => {
                let margin = *stroke_width * 0.5 + 4.0;
                Aabb::from_points(points.iter().copied())
                    .unwrap_or_else(|| Aabb::from_point(DVec2::ZERO))
                    .expand_by_margin(margin)
            }
            NodeData::Station { pos, radius, span_cells, span_angle_deg, .. } => {
                let r = *radius + 4.0;
                let start = *pos;
                let angle_rad = span_angle_deg.to_radians();
                let dir = DVec2::new(angle_rad.cos(), angle_rad.sin());
                let end = start + dir * (*span_cells as f64 * 20.0);
                Aabb::new(start, end).expand_by_margin(r)
            }
            NodeData::Text { text, pos, font_size, .. } => {
                let char_width = *font_size * 0.6;
                let width = (text.len() as f64) * char_width;
                let height = *font_size;
                Aabb::new(*pos, *pos + DVec2::new(width.max(10.0), height.max(10.0)))
            }
        }
    }

    /// Expose editable parameters to Vue frontend property inspector
    pub fn get_parameters(&self) -> Vec<ParameterSchema> {
        match &self.data {
            NodeData::TransitLine { name, corner_radius, color, stroke_width, style, cap, .. } => {
                let current_pattern = match style {
                    StrokeStyle::Solid => "Solid",
                    StrokeStyle::Dashed { .. } => "Dashed",
                    StrokeStyle::Hollow { .. } => "Hollow",
                };
                let inner_col = match style {
                    StrokeStyle::Hollow { inner_color, .. } => inner_color.clone(),
                    _ => "#ffffff".into(),
                };
                vec![
                    ParameterSchema {
                        name: "name".into(),
                        label: "Line Name".into(),
                        param_type: ParameterType::Text(name.clone()),
                    },
                    ParameterSchema {
                        name: "color".into(),
                        label: "Line Color".into(),
                        param_type: ParameterType::Color(color.clone()),
                    },
                    ParameterSchema {
                        name: "pattern".into(),
                        label: "Pattern Style".into(),
                        param_type: ParameterType::Choice {
                            current: current_pattern.into(),
                            options: vec!["Solid".into(), "Dashed".into(), "Hollow".into()],
                        },
                    },
                    ParameterSchema {
                        name: "inner_color".into(),
                        label: "Hollow Inner Color".into(),
                        param_type: ParameterType::Color(inner_col),
                    },
                    ParameterSchema {
                        name: "stroke_width".into(),
                        label: "Stroke Width".into(),
                        param_type: ParameterType::Number {
                            value: *stroke_width,
                            min: 2.0,
                            max: 30.0,
                            step: 1.0,
                        },
                    },
                    ParameterSchema {
                        name: "corner_radius".into(),
                        label: "Fillet Corner Radius".into(),
                        param_type: ParameterType::Number {
                            value: *corner_radius,
                            min: 0.0,
                            max: 50.0,
                            step: 1.0,
                        },
                    },
                    ParameterSchema {
                        name: "cap".into(),
                        label: "End Cap".into(),
                        param_type: ParameterType::Choice {
                            current: match cap {
                                LineCap::Round => "Round",
                                LineCap::Square => "Square",
                                LineCap::Butt => "Butt",
                            }.into(),
                            options: vec!["Round".into(), "Square".into(), "Butt".into()],
                        },
                    },
                ]
            }
            NodeData::Station { name, style, radius, fill_color, stroke_color, rotation_rad, span_cells, span_angle_deg, .. } => {
                let current_dir = match *span_angle_deg as i64 {
                    90 => "90° (Vertical Down)",
                    180 => "180° (Horizontal Left)",
                    270 => "270° (Vertical Up)",
                    45 => "45° (Diagonal Down-Right)",
                    135 => "135° (Diagonal Down-Left)",
                    _ => "0° (Horizontal Right)",
                };
                vec![
                    ParameterSchema {
                        name: "name".into(),
                        label: "Station Name".into(),
                        param_type: ParameterType::Text(name.clone()),
                    },
                    ParameterSchema {
                        name: "style".into(),
                        label: "Station Style (Dot or Tick)".into(),
                        param_type: ParameterType::Choice {
                            current: match style {
                                StationStyle::Dot => "Dot",
                                StationStyle::Tick => "Tick",
                            }.into(),
                            options: vec!["Dot".into(), "Tick".into()],
                        },
                    },
                    ParameterSchema {
                        name: "radius".into(),
                        label: "Marker Radius".into(),
                        param_type: ParameterType::Number {
                            value: *radius,
                            min: 3.0,
                            max: 20.0,
                            step: 0.5,
                        },
                    },
                    ParameterSchema {
                        name: "span_cells".into(),
                        label: "Stretch Span (grid cells)".into(),
                        param_type: ParameterType::Number {
                            value: *span_cells as f64,
                            min: 0.0,
                            max: 8.0,
                            step: 1.0,
                        },
                    },
                    ParameterSchema {
                        name: "span_direction".into(),
                        label: "Stretch Direction".into(),
                        param_type: ParameterType::Choice {
                            current: current_dir.into(),
                            options: vec![
                                "0° (Horizontal Right)".into(),
                                "90° (Vertical Down)".into(),
                                "180° (Horizontal Left)".into(),
                                "270° (Vertical Up)".into(),
                                "45° (Diagonal Down-Right)".into(),
                                "135° (Diagonal Down-Left)".into(),
                            ],
                        },
                    },
                    ParameterSchema {
                        name: "fill_color".into(),
                        label: "Fill Color".into(),
                        param_type: ParameterType::Color(fill_color.clone()),
                    },
                    ParameterSchema {
                        name: "stroke_color".into(),
                        label: "Stroke Color".into(),
                        param_type: ParameterType::Color(stroke_color.clone()),
                    },
                    ParameterSchema {
                        name: "rotation_deg".into(),
                        label: "Tick Rotation Angle (deg)".into(),
                        param_type: ParameterType::Number {
                            value: rotation_rad.to_degrees(),
                            min: 0.0,
                            max: 360.0,
                            step: 15.0,
                        },
                    },
                ]
            }
            NodeData::Text { text, font_size, color, .. } => {
                vec![
                    ParameterSchema {
                        name: "text".into(),
                        label: "Label Text".into(),
                        param_type: ParameterType::Text(text.clone()),
                    },
                    ParameterSchema {
                        name: "font_size".into(),
                        label: "Font Size".into(),
                        param_type: ParameterType::Number {
                            value: *font_size,
                            min: 6.0,
                            max: 72.0,
                            step: 1.0,
                        },
                    },
                    ParameterSchema {
                        name: "color".into(),
                        label: "Text Color".into(),
                        param_type: ParameterType::Color(color.clone()),
                    },
                ]
            }
        }
    }
}

/// Evaluates quadratic Bezier corner filleting for a polyline.
/// Converts jagged octolinear vertices into smooth transit curves with radius `r`.
pub fn generate_transit_curve_segments(points: &[DVec2], radius: f64, hidden_segments: &[usize]) -> Vec<PathSegment> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 {
        return vec![PathSegment::MoveTo(points[0])];
    }

    let mut segments = Vec::new();
    let count = points.len();

    // Start of the path
    segments.push(PathSegment::MoveTo(points[0]));

    for i in 1..count {
        if hidden_segments.contains(&(i - 1)) {
            segments.push(PathSegment::MoveTo(points[i]));
            continue;
        }

        let is_last = i == count - 1;
        if is_last || radius <= 0.0 {
            segments.push(PathSegment::LineTo(points[i]));
            continue;
        }

        let p_prev = points[i - 1];
        let p_curr = points[i];
        let p_next = points[i + 1];

        let v_in = p_prev - p_curr;
        let v_out = p_next - p_curr;

        let len_in = v_in.length();
        let len_out = v_out.length();

        if len_in < 1e-4 || len_out < 1e-4 {
            segments.push(PathSegment::LineTo(p_curr));
            continue;
        }

        let d_in = v_in / len_in;
        let d_out = v_out / len_out;

        let dot = d_in.dot(d_out);
        if dot < -0.999 || dot > 0.999 {
            segments.push(PathSegment::LineTo(p_curr));
            continue;
        }

        let max_cut = (len_in * 0.5).min(len_out * 0.5).min(radius);

        let fillet_start = p_curr + d_in * max_cut;
        let fillet_end = p_curr + d_out * max_cut;

        segments.push(PathSegment::LineTo(fillet_start));
        segments.push(PathSegment::QuadTo {
            ctrl: p_curr,
            to: fillet_end,
        });
    }

    segments
}

/// The procedural document tree that stores all nodes and handles deterministic evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTree {
    pub nodes: HashMap<NodeId, DocumentNode>,
    pub node_order: Vec<NodeId>,
    next_id: NodeId,
}

impl DocumentTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            node_order: Vec::new(),
            next_id: 1,
        }
    }

    pub fn allocate_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_node(&mut self, data: NodeData, z_index: i32) -> NodeId {
        let id = self.allocate_id();
        let node = DocumentNode {
            id,
            visible: true,
            locked: false,
            z_index,
            data,
        };
        self.nodes.insert(id, node);
        self.node_order.push(id);
        id
    }

    pub fn remove_node(&mut self, id: NodeId) -> Option<DocumentNode> {
        self.node_order.retain(|&i| i != id);
        self.nodes.remove(&id)
    }

    pub fn get_node(&self, id: NodeId) -> Option<&DocumentNode> {
        self.nodes.get(&id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut DocumentNode> {
        self.nodes.get_mut(&id)
    }

    /// Find the nearest transit line and the projected snapping point on it
    pub fn snap_to_nearest_line(&self, coord: DVec2, threshold: f64) -> Option<(NodeId, DVec2)> {
        let mut closest = None;
        let mut min_dist = threshold;

        for &id in &self.node_order {
            if let Some(DocumentNode { data: NodeData::TransitLine { points, .. }, visible: true, .. }) = self.nodes.get(&id) {
                for i in 0..points.len().saturating_sub(1) {
                    let a = points[i];
                    let b = points[i + 1];
                    let (proj, _t, dist) = project_point_to_segment(coord, a, b);
                    if dist < min_dist {
                        min_dist = dist;
                        closest = Some((id, proj));
                    }
                }
            }
        }

        closest
    }

    /// Evaluates the node graph into a flat sequence of renderable primitives,
    /// topologically sorted by z_index and evaluation order.
    pub fn evaluate(&self, colliding_text_ids: &[NodeId]) -> Vec<RenderPrimitive> {
        let mut primitives = Vec::new();

        let mut sorted_ids = self.node_order.clone();
        sorted_ids.sort_by_key(|&id| {
            self.nodes.get(&id).map(|n| n.z_index).unwrap_or(0)
        });

        for id in sorted_ids {
            let node = match self.nodes.get(&id) {
                Some(n) if n.visible => n,
                _ => continue,
            };

            match &node.data {
                NodeData::TransitLine { points, corner_radius, color, stroke_width, style, cap, hidden_segments, .. } => {
                    let segments = generate_transit_curve_segments(points, *corner_radius, hidden_segments);
                    primitives.push(RenderPrimitive {
                        id,
                        z_index: node.z_index,
                        primitive_type: RenderPrimitiveType::Path {
                            segments,
                            stroke_color: color.clone(),
                            stroke_width: *stroke_width,
                            fill_color: None,
                            stroke_style: style.clone(),
                            line_cap: *cap,
                        },
                    });
                }
                NodeData::Station { pos, radius, style, fill_color, stroke_color, stroke_width, rotation_rad, span_cells, span_angle_deg, .. } => {
                    let angle_rad = span_angle_deg.to_radians();
                    let dir = DVec2::new(angle_rad.cos(), angle_rad.sin());
                    let end_pos = *pos + dir * (*span_cells as f64 * 20.0);

                    primitives.push(RenderPrimitive {
                        id,
                        z_index: node.z_index,
                        primitive_type: RenderPrimitiveType::Station {
                            center: *pos,
                            end_pos,
                            radius: *radius,
                            style: *style,
                            fill_color: fill_color.clone(),
                            stroke_color: stroke_color.clone(),
                            stroke_width: *stroke_width,
                            rotation_rad: *rotation_rad,
                            span_cells: *span_cells,
                        },
                    });
                }
                NodeData::Text { text, pos, font_size, color, .. } => {
                    let is_colliding = colliding_text_ids.contains(&id);
                    let bounds = node.aabb();
                    primitives.push(RenderPrimitive {
                        id,
                        z_index: node.z_index,
                        primitive_type: RenderPrimitiveType::Text {
                            text: text.clone(),
                            pos: *pos,
                            font_size: *font_size,
                            color: if is_colliding { "#ff3b30".to_string() } else { color.clone() },
                            is_colliding,
                            bounds,
                        },
                    });
                }
            }
        }

        primitives
    }
}
