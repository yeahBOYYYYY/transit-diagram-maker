//! The document model: a tree of layers and layer groups.
//!
//! `Document::root` is ordered bottom-to-top: later nodes are painted on top of
//! earlier ones, which the layers panel shows reversed (top of the list = top
//! of the stack).

use crate::geom::{Cell, Dir8, RouteMode, Vec2};
use serde::{Deserialize, Serialize};

pub type Id = u64;

/// Pixels per cell on the canvas (at 100% zoom) and the default SVG export scale.
pub const CELL_PX: f64 = 24.0;

/// Document-wide constants that affect rendering. All lengths are in cell units
/// unless the name says otherwise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DocSettings {
    /// Pixels per cell used when rendering. Not part of the document: the
    /// canvas always uses `CELL_PX` and an SVG export may pick its own scale.
    #[serde(skip, default = "cell_px")]
    pub cell_px: f64,
    /// Width of a transit line, as a fraction of a cell.
    pub line_width: f64,
    /// Radius of the curve used where a line turns.
    pub corner_radius: f64,
    /// 0 = square cap corners, 1 = full semicircle for round caps.
    pub cap_roundness: f64,
    /// Outer radius of dot and pill stations.
    pub station_outer_radius: f64,
    /// Inner radius of dot and pill stations.
    pub station_inner_radius: f64,
    pub tick_width: f64,
    pub tick_length: f64,
    /// Distance from a station to its automatically created label.
    pub label_offset: f64,
    pub label_font_size: f64,
    pub label_font_family: String,
    /// Default icon size; `None` means "match the dot station diameter".
    pub icon_size: Option<f64>,
    /// Margin added around the drawing when exporting to SVG.
    pub export_margin: f64,
    /// Background color used for SVG export; empty for transparent.
    pub export_background: String,
    /// Routing used when a path is normalized after editing operations.
    pub route_mode: RouteMode,
}

impl Default for DocSettings {
    fn default() -> Self {
        DocSettings {
            cell_px: CELL_PX,
            line_width: 1.0,
            corner_radius: 1.5,
            cap_roundness: 1.0,
            station_outer_radius: 0.62,
            station_inner_radius: 0.4,
            tick_width: 0.35,
            tick_length: 0.55,
            label_offset: 1.3,
            label_font_size: 0.75,
            label_font_family: "Helvetica, Arial, sans-serif".into(),
            icon_size: None,
            export_margin: 1.0,
            export_background: "#ffffff".into(),
            route_mode: RouteMode::StraightFirst,
        }
    }
}

impl DocSettings {
    pub fn icon_size(&self) -> f64 {
        self.icon_size.unwrap_or(self.station_outer_radius * 2.0)
    }

    /// Clamp values into sane ranges so a bad settings file cannot break rendering.
    pub fn sanitized(mut self) -> Self {
        let fix = |v: f64, lo: f64, hi: f64, def: f64| if v.is_finite() { v.clamp(lo, hi) } else { def };
        let d = DocSettings::default();
        self.cell_px = fix(self.cell_px, 1.0, 512.0, d.cell_px);
        self.line_width = fix(self.line_width, 0.05, 4.0, d.line_width);
        self.corner_radius = fix(self.corner_radius, 0.0, 50.0, d.corner_radius);
        self.cap_roundness = fix(self.cap_roundness, 0.0, 1.0, d.cap_roundness);
        self.station_outer_radius = fix(self.station_outer_radius, 0.05, 10.0, d.station_outer_radius);
        self.station_inner_radius = fix(self.station_inner_radius, 0.0, self.station_outer_radius, d.station_inner_radius);
        self.tick_width = fix(self.tick_width, 0.01, 10.0, d.tick_width);
        self.tick_length = fix(self.tick_length, 0.01, 10.0, d.tick_length);
        self.label_offset = fix(self.label_offset, 0.0, 20.0, d.label_offset);
        self.label_font_size = fix(self.label_font_size, 0.05, 20.0, d.label_font_size);
        self.icon_size = self.icon_size.map(|v| fix(v, 0.05, 50.0, d.icon_size()));
        self.export_margin = fix(self.export_margin, 0.0, 100.0, d.export_margin);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cap {
    /// Flat ending that fills the whole last cell.
    Butt,
    /// Rounded ending; the corner radius comes from `DocSettings::cap_roundness`.
    #[default]
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrokeKind {
    #[default]
    Plain,
    /// A line with an inner channel of `inner_color`.
    Hollow,
    /// A broken line: full-width dashes of the line color with empty gaps.
    Dashed,
    /// A hollow line with dashes of the line color running down its channel.
    HollowDashed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LineStyle {
    pub color: String,
    pub kind: StrokeKind,
    /// Inner channel color for hollow lines.
    pub inner_color: String,
    /// Inner channel width as a percentage of the line width.
    pub inner_width_pct: f64,
    /// Requested dash length in cells. Dashes and gaps are stretched slightly
    /// so a line always starts and ends with a full dash.
    pub dash_length: f64,
    /// Requested gap between dashes in cells.
    pub dash_gap: f64,
    /// Width of the dashes inside a hollow dashed line, as a percentage of the line width.
    pub dash_width_pct: f64,
    pub start_cap: Cap,
    pub end_cap: Cap,
}

impl Default for LineStyle {
    fn default() -> Self {
        LineStyle {
            color: "#e03c31".into(),
            kind: StrokeKind::Plain,
            inner_color: "#ffffff".into(),
            inner_width_pct: 55.0,
            dash_length: 1.0,
            dash_gap: 0.5,
            dash_width_pct: 30.0,
            start_cap: Cap::Round,
            end_cap: Cap::Round,
        }
    }
}

/// One continuous portion of a transit line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineSegment {
    /// Identifier shared by all segments of the same line.
    pub line_id: Id,
    /// Turning vertices; consecutive points are always 8-way aligned.
    pub points: Vec<Cell>,
    pub style: LineStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum StationShape {
    Dot {
        cell: Cell,
        outer_color: String,
        inner_color: String,
    },
    Tick {
        cell: Cell,
        /// Direction of the line at the station.
        dir: Dir8,
        /// +1 pokes out to the left of the line direction, -1 to the right.
        side: i8,
        /// Corner of the cell along the line: -1 = the edge the line enters
        /// through, +1 = the edge it leaves through, 0 = centered. Ignored on
        /// diagonal lines.
        #[serde(default = "start_corner")]
        along: i8,
        width: f64,
        length: f64,
        color: String,
    },
    Pill {
        cells: Vec<Cell>,
        outer_color: String,
        inner_color: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StationKind {
    Dot,
    Tick,
    Pill,
}

impl StationShape {
    pub fn kind(&self) -> StationKind {
        match self {
            StationShape::Dot { .. } => StationKind::Dot,
            StationShape::Tick { .. } => StationKind::Tick,
            StationShape::Pill { .. } => StationKind::Pill,
        }
    }

    pub fn cells(&self) -> Vec<Cell> {
        match self {
            StationShape::Dot { cell, .. } | StationShape::Tick { cell, .. } => vec![*cell],
            StationShape::Pill { cells, .. } => cells.clone(),
        }
    }

    pub fn anchor(&self) -> Vec2 {
        if let StationShape::Tick { cell, dir, along, width, .. } = self {
            return crate::geom::tick_center(*cell, *dir, *along, *width);
        }
        let cells = self.cells();
        let n = cells.len().max(1) as f64;
        let sum = cells.iter().fold(Vec2::default(), |acc, c| acc.add(c.center()));
        sum.scale(1.0 / n)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Station {
    /// Lines this station is coupled to (first one is the primary line).
    pub line_ids: Vec<Id>,
    #[serde(flatten)]
    pub shape: StationShape,
    /// Free displacement from the line cell in cells (0.1 precision), set by
    /// moving the station with Alt held. Zero means it sits on its line.
    #[serde(default)]
    pub offset: Vec2,
}

impl Station {
    pub fn new(line_ids: Vec<Id>, shape: StationShape) -> Self {
        Station { line_ids, shape, offset: Vec2::default() }
    }

    /// Visual center: the shape anchor plus the free offset.
    pub fn anchor(&self) -> Vec2 {
        self.shape.anchor().add(self.offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    Start,
    #[default]
    Middle,
    End,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextBox {
    /// Anchor position in cell units.
    pub pos: Vec2,
    pub text: String,
    /// Font size in cell units.
    pub size: f64,
    pub color: String,
    pub font_family: String,
    pub bold: bool,
    pub italic: bool,
    pub align: TextAlign,
    /// Rotation in degrees (clockwise).
    pub rotation: f64,
    /// Station this text names, if any.
    pub station_id: Option<Id>,
}

impl Default for TextBox {
    fn default() -> Self {
        TextBox {
            pos: Vec2::default(),
            text: "Text".into(),
            size: 0.75,
            color: "#1a1a1a".into(),
            font_family: "Helvetica, Arial, sans-serif".into(),
            bold: false,
            italic: false,
            align: TextAlign::Middle,
            rotation: 0.0,
            station_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Icon {
    /// Center position in cell units.
    pub pos: Vec2,
    /// Pictogram name from the icon library.
    pub icon: String,
    /// Edge length in cell units.
    pub size: f64,
    pub rotation: f64,
    pub color: String,
    pub background: String,
}

impl Default for Icon {
    fn default() -> Self {
        Icon {
            pos: Vec2::default(),
            icon: "wheelchair".into(),
            size: 1.24,
            rotation: 0.0,
            color: "#ffffff".into(),
            background: "#1d4f91".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Content {
    Line(LineSegment),
    Station(Station),
    Text(TextBox),
    Icon(Icon),
}

impl Content {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Content::Line(_) => "line",
            Content::Station(_) => "station",
            Content::Text(_) => "text",
            Content::Icon(_) => "icon",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: Id,
    pub name: String,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default)]
    pub locked: bool,
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum GroupKind {
    Generic,
    /// All segments and stations of one transit line.
    Line { line_id: Id },
    /// A station together with its label.
    Station,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: Id,
    pub name: String,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default = "yes")]
    pub expanded: bool,
    pub kind: GroupKind,
    /// Bottom-to-top.
    pub children: Vec<Node>,
}

fn cell_px() -> f64 {
    CELL_PX
}

fn yes() -> bool {
    true
}

fn start_corner() -> i8 {
    -1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
    Group(Group),
    Layer(Layer),
}

impl Node {
    pub fn id(&self) -> Id {
        match self {
            Node::Group(g) => g.id,
            Node::Layer(l) => l.id,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Node::Group(g) => &g.name,
            Node::Layer(l) => &l.name,
        }
    }
    pub fn visible(&self) -> bool {
        match self {
            Node::Group(g) => g.visible,
            Node::Layer(l) => l.visible,
        }
    }
    pub fn locked(&self) -> bool {
        match self {
            Node::Group(g) => g.locked,
            Node::Layer(l) => l.locked,
        }
    }
    pub fn set_visible(&mut self, v: bool) {
        match self {
            Node::Group(g) => g.visible = v,
            Node::Layer(l) => l.visible = v,
        }
    }
    pub fn set_locked(&mut self, v: bool) {
        match self {
            Node::Group(g) => g.locked = v,
            Node::Layer(l) => l.locked = v,
        }
    }
    pub fn set_name(&mut self, name: String) {
        match self {
            Node::Group(g) => g.name = name,
            Node::Layer(l) => l.name = name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    #[serde(default)]
    pub settings: DocSettings,
    /// Fonts and icons uploaded by the user.
    #[serde(default)]
    pub assets: crate::assets::Assets,
    /// Bottom-to-top.
    #[serde(default)]
    pub root: Vec<Node>,
    #[serde(default = "first_id")]
    pub next_id: Id,
}

fn first_id() -> Id {
    1
}

impl Default for Document {
    fn default() -> Self {
        Document { settings: DocSettings::default(), assets: Default::default(), root: Vec::new(), next_id: 1 }
    }
}

impl Document {
    pub fn alloc_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
