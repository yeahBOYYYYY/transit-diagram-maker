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

/// How a drawing is turned into a standalone SVG. These only affect the
/// exported picture, not the document, so they are not part of `DocSettings`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportOptions {
    /// Pixels per cell.
    pub cell_px: f64,
    /// Empty space around the drawing, in cells.
    pub margin: f64,
    /// Background color; empty for transparent.
    pub background: String,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions { cell_px: CELL_PX, margin: 1.0, background: "#ffffff".into() }
    }
}

impl ExportOptions {
    pub fn sanitized(mut self) -> Self {
        let d = ExportOptions::default();
        self.cell_px = if self.cell_px.is_finite() { self.cell_px.clamp(1.0, 512.0) } else { d.cell_px };
        self.margin = if self.margin.is_finite() { self.margin.clamp(0.0, 100.0) } else { d.margin };
        self
    }
}

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
    /// A solid line with dashes of `inner_color` running down its middle.
    InnerDashed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LineStyle {
    pub color: String,
    pub kind: StrokeKind,
    /// Inner channel color for hollow lines and dash color for inner dashed lines.
    pub inner_color: String,
    /// Inner channel width as a percentage of the line width.
    pub inner_width_pct: f64,
    /// Requested dash length in cells. Dashes and gaps are stretched slightly
    /// so a line always starts and ends with a full dash.
    pub dash_length: f64,
    /// Requested gap between dashes in cells.
    pub dash_gap: f64,
    /// Width of the dashes inside an inner dashed line, as a percentage of the line width.
    pub dash_width_pct: f64,
    /// Length in cells at the start of the line left solid before the dashes begin.
    pub dash_start_offset: f64,
    /// Length in cells at the end of the line left solid after the dashes stop.
    pub dash_end_offset: f64,
    pub start_cap: Cap,
    pub end_cap: Cap,
}

impl Default for LineStyle {
    fn default() -> Self {
        LineStyle {
            color: "#e03c31".into(),
            kind: StrokeKind::Plain,
            inner_color: "#ffffff".into(),
            inner_width_pct: 33.0,
            dash_length: 1.0,
            dash_gap: 1.0,
            dash_width_pct: 30.0,
            dash_start_offset: 0.0,
            dash_end_offset: 0.0,
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

/// Writing direction of a text. It orders mixed right-to-left and
/// left-to-right runs; the alignment stays visual (left is always left).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
}

/// Height of a line of text as a multiple of its largest font size.
pub const LINE_SPACING: f64 = 1.2;
/// Distance from the top of a line to its baseline, as a multiple of the
/// largest font size (half the extra line spacing plus a typical ascent; the
/// browser places the baseline of a CSS line box at about the same height).
pub const BASELINE: f64 = 0.95;

/// Character formatting of a run of text. Every `None` uses the value of the
/// text box, so changing the box changes every run that doesn't override it.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RunStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Font size in cell units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
}

/// Names of the character properties a run may override.
pub const RUN_PROPERTIES: [&str; 5] = ["font_family", "size", "color", "bold", "italic"];

impl RunStyle {
    pub fn is_empty(&self) -> bool {
        *self == RunStyle::default()
    }

    fn clear(&mut self, key: &str) {
        match key {
            "font_family" => self.font_family = None,
            "size" => self.size = None,
            "color" => self.color = None,
            "bold" => self.bold = None,
            "italic" => self.italic = None,
            _ => {}
        }
    }
}

/// A piece of a paragraph with one formatting.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TextRun {
    #[serde(default)]
    pub text: String,
    #[serde(flatten)]
    pub style: RunStyle,
}

impl TextRun {
    pub fn plain(text: &str) -> Self {
        TextRun { text: text.into(), style: RunStyle::default() }
    }
}

/// One line of a text box.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Paragraph {
    /// Alignment of this line inside the box; `None` uses the box alignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<TextAlign>,
    #[serde(default)]
    pub runs: Vec<TextRun>,
}

impl Paragraph {
    pub fn plain(text: &str) -> Self {
        let runs = if text.is_empty() { Vec::new() } else { vec![TextRun::plain(text)] };
        Paragraph { align: None, runs }
    }

    pub fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
}

/// A text box: paragraphs of formatted runs inside a box whose top-left
/// corner is `pos`. The box is as wide as its widest line (`width`, measured
/// by the browser that renders it) and is rotated around that corner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextBox {
    /// Top-left corner of the box in cell units.
    pub pos: Vec2,
    pub paragraphs: Vec<Paragraph>,
    /// Width of the widest line in cell units.
    pub width: f64,
    /// Font size in cell units; runs may override it, like the other
    /// character properties below.
    pub size: f64,
    pub color: String,
    pub font_family: String,
    pub bold: bool,
    pub italic: bool,
    /// Alignment of the lines inside the box; paragraphs may override it. For
    /// a station label it also picks the edge that stays next to the station
    /// when the width changes.
    pub align: TextAlign,
    pub direction: TextDirection,
    /// Rotation in degrees (clockwise) around `pos`.
    pub rotation: f64,
    /// Station this text names, if any.
    pub station_id: Option<Id>,
}

impl Default for TextBox {
    fn default() -> Self {
        let mut t = TextBox {
            pos: Vec2::default(),
            paragraphs: vec![Paragraph::plain("Text")],
            width: 0.0,
            size: 0.75,
            color: "#1a1a1a".into(),
            font_family: "Helvetica, Arial, sans-serif".into(),
            bold: false,
            italic: false,
            align: TextAlign::Middle,
            direction: TextDirection::Ltr,
            rotation: 0.0,
            station_id: None,
        };
        t.width = t.estimated_width();
        t
    }
}

/// The formatting a run is drawn with.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStyle<'a> {
    pub font_family: &'a str,
    pub size: f64,
    pub color: &'a str,
    pub bold: bool,
    pub italic: bool,
}

impl TextBox {
    /// A box holding `text`, one paragraph per line.
    pub fn with_text(text: &str) -> Self {
        let mut t = TextBox { paragraphs: Self::plain_paragraphs(text), ..TextBox::default() };
        t.width = t.estimated_width();
        t
    }

    pub fn plain_paragraphs(text: &str) -> Vec<Paragraph> {
        text.split('\n').map(|l| Paragraph::plain(l.strip_suffix('\r').unwrap_or(l))).collect()
    }

    /// The text without formatting, paragraphs separated by line breaks.
    pub fn text(&self) -> String {
        self.paragraphs.iter().map(|p| p.text()).collect::<Vec<_>>().join("\n")
    }

    pub fn style_of<'a>(&'a self, r: &'a RunStyle) -> ResolvedStyle<'a> {
        ResolvedStyle {
            font_family: r.font_family.as_deref().unwrap_or(&self.font_family),
            size: r.size.unwrap_or(self.size),
            color: r.color.as_deref().unwrap_or(&self.color),
            bold: r.bold.unwrap_or(self.bold),
            italic: r.italic.unwrap_or(self.italic),
        }
    }

    pub fn align_of(&self, p: &Paragraph) -> TextAlign {
        p.align.unwrap_or(self.align)
    }

    /// Largest font size on a line; the box size counts too, like the strut
    /// of a CSS line box.
    pub fn line_size(&self, p: &Paragraph) -> f64 {
        p.runs.iter().filter_map(|r| r.style.size).fold(self.size, f64::max)
    }

    /// Height of the box in cells.
    pub fn height(&self) -> f64 {
        self.paragraphs.iter().map(|p| self.line_size(p) * LINE_SPACING).sum::<f64>().max(self.size * LINE_SPACING)
    }

    /// Rough width of the widest line, used until the browser measures it.
    pub fn estimated_width(&self) -> f64 {
        let line = |p: &Paragraph| -> f64 {
            p.runs
                .iter()
                .map(|r| {
                    let st = self.style_of(&r.style);
                    r.text.chars().count() as f64 * st.size * if st.bold { 0.62 } else { 0.56 }
                })
                .sum()
        };
        let w = self.paragraphs.iter().map(line).fold(0.0, f64::max);
        (w * 1000.0).round() / 1000.0
    }

    /// Share of a width change that moves the left edge. A text box keeps its
    /// top-left corner (on the grid) and grows to the right; a station label
    /// keeps the edge it is aligned to, so it stays centered under (or beside)
    /// its station.
    fn anchor_share(&self) -> f64 {
        if self.station_id.is_none() {
            return 0.0;
        }
        match self.align {
            TextAlign::Start => 0.0,
            TextAlign::Middle => 0.5,
            TextAlign::End => 1.0,
        }
    }

    /// Change the width, keeping the corner (or, for a label, the aligned edge) in place.
    pub fn set_width(&mut self, width: f64) {
        let width = if width.is_finite() { (width.max(0.0) * 1000.0).round() / 1000.0 } else { 0.0 };
        let shift = (self.width - width) * self.anchor_share();
        if shift != 0.0 {
            let along = Vec2::new(shift, 0.0).rotate_around(Vec2::default(), self.rotation);
            self.pos = self.pos.add(along).rounded(1000.0);
        }
        self.width = width;
    }

    /// Put the box so that its alignment edge is at `anchor.x` and its middle
    /// at `anchor.y` (a station label next to its station).
    pub fn place_at(&mut self, anchor: Vec2) {
        self.pos = Vec2::new(anchor.x - self.width * self.anchor_share(), anchor.y - self.height() / 2.0).rounded(1000.0);
    }

    /// The position for a box moved to `pos` that puts whichever of its four
    /// corners is closest to a grid intersection exactly on it.
    pub fn snap_corner(&self, pos: Vec2) -> Vec2 {
        let (w, h) = (self.width, self.height());
        let best = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)]
            .iter()
            .map(|&(x, y)| pos.add(Vec2::new(x, y).rotate_around(Vec2::default(), self.rotation)))
            .map(|c| c.rounded(1.0).sub(c))
            .min_by(|a, b| a.len().total_cmp(&b.len()))
            .unwrap_or_default();
        pos.add(best).rounded(1000.0)
    }

    /// Remove the overrides of a box property from every run (or of the
    /// alignment from every paragraph), so the box value applies to all text.
    pub fn clear_overrides(&mut self, key: &str) {
        for p in &mut self.paragraphs {
            if key == "align" {
                p.align = None;
            }
            for r in &mut p.runs {
                r.style.clear(key);
            }
        }
    }

    /// Tidy the formatting: overrides equal to the box value are dropped,
    /// neighbouring runs with the same formatting are merged and empty runs
    /// are removed. There is always at least one paragraph.
    pub fn normalize(&mut self) {
        if self.paragraphs.is_empty() {
            self.paragraphs.push(Paragraph::default());
        }
        let boxed = RunStyle {
            font_family: Some(self.font_family.clone()),
            size: Some(self.size),
            color: Some(self.color.clone()),
            bold: Some(self.bold),
            italic: Some(self.italic),
        };
        for p in &mut self.paragraphs {
            if p.align == Some(self.align) {
                p.align = None;
            }
            let mut out: Vec<TextRun> = Vec::new();
            for mut r in std::mem::take(&mut p.runs) {
                if r.text.is_empty() {
                    continue;
                }
                let st = &mut r.style;
                if st.font_family == boxed.font_family {
                    st.font_family = None;
                }
                if st.size.is_some_and(|v| !v.is_finite() || v <= 0.0) || st.size == boxed.size {
                    st.size = None;
                }
                if st.color == boxed.color {
                    st.color = None;
                }
                if st.bold == boxed.bold {
                    st.bold = None;
                }
                if st.italic == boxed.italic {
                    st.italic = None;
                }
                match out.last_mut() {
                    Some(last) if last.style == r.style => last.text.push_str(&r.text),
                    _ => out.push(r),
                }
            }
            p.runs = out;
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

/// Outline of a drawn shape, in cell units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum ShapeGeometry {
    /// A straight line between two points, normally grid intersections.
    Line {
        from: Vec2,
        to: Vec2,
        /// Width of the line in cells.
        thickness: f64,
        /// Round ends stick out by half the thickness; butt ends stop at the points.
        cap: Cap,
        dashed: bool,
        /// Requested dash length in cells (fitted so the line starts and ends with a full dash).
        dash_length: f64,
        dash_gap: f64,
    },
    /// A rectangle whose top-left corner is `pos`, normally covering whole
    /// cells. It is turned by `rotation` degrees around its center, kept in
    /// (-45, 45] (a quarter turn swaps the width and the height instead).
    /// `corner_radius` rounds its corners, in cells (0 for sharp corners);
    /// it is drawn at most half the shorter side, so a resize keeps it.
    Rect {
        pos: Vec2,
        width: f64,
        height: f64,
        rotation: f64,
        #[serde(default)]
        corner_radius: f64,
    },
    /// A circle, normally centered on a cell center.
    Circle { center: Vec2, radius: f64 },
}

/// A free-standing shape drawn with the shape tool: a straight line, a
/// rectangle or a circle that belongs to no transit line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    #[serde(flatten)]
    pub geometry: ShapeGeometry,
    /// Color of a line, or the fill of a rectangle or circle.
    pub color: String,
    /// Rectangles and circles: false leaves them empty inside (only the border shows).
    pub filled: bool,
    pub border_color: String,
    /// Border width in cells, 0 for none. A line gets it on both sides; a
    /// rectangle or circle inside its edge, so it keeps its size.
    pub border_width: f64,
}

impl Shape {
    pub fn line(from: Vec2, to: Vec2) -> Self {
        Shape {
            geometry: ShapeGeometry::Line { from, to, thickness: 0.1, cap: Cap::Butt, dashed: false, dash_length: 0.5, dash_gap: 0.3 },
            color: "#1a1a1a".into(),
            filled: true,
            border_color: "#ffffff".into(),
            border_width: 0.0,
        }
    }

    pub fn rect(pos: Vec2, width: f64, height: f64) -> Self {
        Shape {
            geometry: ShapeGeometry::Rect { pos, width, height, rotation: 0.0, corner_radius: 0.0 },
            color: "#e6e6e6".into(),
            filled: true,
            border_color: "#1a1a1a".into(),
            border_width: 0.1,
        }
    }

    pub fn circle(center: Vec2, radius: f64) -> Self {
        Shape {
            geometry: ShapeGeometry::Circle { center, radius },
            color: "#ffffff".into(),
            filled: true,
            border_color: "#000000".into(),
            border_width: 0.22,
        }
    }

    /// What the layers panel calls a new shape of this kind.
    pub fn label(&self) -> &'static str {
        match self.geometry {
            ShapeGeometry::Line { .. } => "Straight line",
            ShapeGeometry::Rect { .. } => "Rectangle",
            ShapeGeometry::Circle { .. } => "Circle",
        }
    }

    /// Position shown as X / Y: the top-left-most end of a line, the corner
    /// of a rectangle, the center of a circle.
    pub fn anchor(&self) -> Vec2 {
        match &self.geometry {
            ShapeGeometry::Line { from, to, .. } => Vec2::new(from.x.min(to.x), from.y.min(to.y)),
            ShapeGeometry::Rect { pos, .. } => *pos,
            ShapeGeometry::Circle { center, .. } => *center,
        }
    }

    /// Points that land on the grid when the shape snaps: the ends of a line
    /// and the corners of a rectangle (grid intersections), the center of a
    /// circle (a cell center).
    pub fn snap_points(&self) -> Vec<Vec2> {
        match &self.geometry {
            ShapeGeometry::Line { from, to, .. } => vec![*from, *to],
            ShapeGeometry::Rect { pos, width, height, rotation, .. } => {
                let c = pos.add(Vec2::new(width / 2.0, height / 2.0));
                [(0.0, 0.0), (*width, 0.0), (0.0, *height), (*width, *height)]
                    .iter()
                    .map(|&(x, y)| pos.add(Vec2::new(x, y)).rotate_around(c, *rotation))
                    .collect()
            }
            ShapeGeometry::Circle { center, .. } => vec![*center],
        }
    }

    pub fn translate(&mut self, d: Vec2) {
        match &mut self.geometry {
            ShapeGeometry::Line { from, to, .. } => {
                *from = from.add(d);
                *to = to.add(d);
            }
            ShapeGeometry::Rect { pos, .. } => *pos = pos.add(d),
            ShapeGeometry::Circle { center, .. } => *center = center.add(d),
        }
    }

    /// The delta that moves the shape by about `d` and puts it back on the
    /// grid: whichever line end or rectangle corner is closest to a grid
    /// intersection lands on it, and a circle centers on a cell.
    pub fn snap_delta(&self, d: Vec2) -> Vec2 {
        if let ShapeGeometry::Circle { center, .. } = &self.geometry {
            let c = center.add(d);
            return crate::geom::Cell::containing(c).center().sub(*center);
        }
        let best = self
            .snap_points()
            .into_iter()
            .map(|p| p.add(d))
            .map(|p| p.rounded(1.0).sub(p))
            .min_by(|a, b| a.len().total_cmp(&b.len()))
            .unwrap_or_default();
        d.add(best)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Content {
    Line(LineSegment),
    Station(Station),
    Text(TextBox),
    Icon(Icon),
    Shape(Shape),
}

impl Content {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Content::Line(_) => "line",
            Content::Station(_) => "station",
            Content::Text(_) => "text",
            Content::Icon(_) => "icon",
            Content::Shape(_) => "shape",
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
