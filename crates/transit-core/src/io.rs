//! `.transit.svg` file format: a standalone SVG picture of the drawing that
//! also carries everything the editor needs to open it again, so any browser
//! or image viewer shows a saved file and the editor edits it.
//!
//! The picture is the SVG export at the default scale, with hidden layers
//! kept as `display="none"`. Every `<g>` of the layer tree starts with a
//! `<transit:…>` element (in the [`NS`] namespace, which SVG viewers ignore)
//! that holds the group or layer as attributes. The document settings and the
//! uploaded icons live in `<metadata>`, uploaded fonts in `<style>` elements
//! (which the picture needs anyway).
//!
//! ```xml
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:transit="…" viewBox="…">
//!   <metadata>
//!     <transit:document version="1" next-id="7">
//!       <transit:settings line-width="1" corner-radius="1.5" … route-mode="straight_first"/>
//!       <transit:icon name="custom-1" label="Star" view-box="0 0 10 10"><path d="…"/></transit:icon>
//!     </transit:document>
//!   </metadata>
//!   <defs><style transit:font="City Sans">@font-face{…}</style></defs>
//!   <g id="group-3" data-name="Line 1">
//!     <transit:group id="3" name="Line 1" role="line" line="1"/>
//!     <g id="layer-2" data-name="Segment 1">
//!       <transit:segment id="2" name="Segment 1" line="1" points="0,0 6,0 9,3" color="#e03c31" kind="plain" …/>
//!       <path …/>
//!     </g>
//!   </g>
//! </svg>
//! ```

use crate::assets::{font_face_css, FontAsset, IconAsset};
use crate::editor::{free_rotation, tidy_shape};
use crate::error::{EditError, Result};
use crate::geom::{self, Cell, Dir8, Vec2};
use crate::model::*;
use crate::render::{self, esc};
use crate::tree;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashSet;
use std::fmt::{Display, Write};

/// Namespace of the editor data inside a saved SVG.
pub const NS: &str = "https://github.com/yeahBOYYYYY/transit-diagram-maker/format";
pub const VERSION: u32 = 1;
/// File name extension of saved documents.
pub const EXTENSION: &str = ".transit.svg";

// ------------------------------------------------------------------ writing

/// Attribute value: XML-escaped, with line breaks and tabs kept as character
/// references (a parser would otherwise turn them into spaces).
fn attr_esc(v: &str) -> String {
    esc(v).replace('\n', "&#10;").replace('\r', "&#13;").replace('\t', "&#9;")
}

/// Shortest text that parses back to exactly the same number.
fn num(v: f64) -> String {
    format!("{v}")
}

fn cell(c: Cell) -> String {
    format!("{},{}", c.x, c.y)
}

fn cells(cs: &[Cell]) -> String {
    cs.iter().map(|c| cell(*c)).collect::<Vec<_>>().join(" ")
}

fn vec2(v: Vec2) -> String {
    format!("{},{}", num(v.x), num(v.y))
}

/// The serde name of a unit enum value (`plain`, `straight_first`, …).
fn enum_name<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default()
}

const DIRS: [&str; 8] = ["e", "se", "s", "sw", "w", "nw", "n", "ne"];

/// A `<transit:…>` element being written.
struct Tag(String);

impl Tag {
    fn new(name: &str) -> Self {
        Tag(format!("<transit:{name}"))
    }
    fn attr(mut self, key: &str, value: impl Display) -> Self {
        let _ = write!(self.0, r#" {key}="{}""#, attr_esc(&value.to_string()));
        self
    }
    fn flag(self, key: &str, value: bool, default: bool) -> Self {
        if value == default {
            self
        } else {
            self.attr(key, value)
        }
    }
    fn empty(self) -> String {
        self.0 + "/>"
    }
    fn with_text(self, name: &str, text: &str) -> String {
        self.with_children(name, &esc(text).replace('\r', "&#13;"))
    }
    fn with_children(self, name: &str, inner: &str) -> String {
        format!("{}>{inner}</transit:{name}>", self.0)
    }
}

fn settings_tag(s: &DocSettings) -> String {
    let mut t = Tag::new("settings")
        .attr("line-width", num(s.line_width))
        .attr("corner-radius", num(s.corner_radius))
        .attr("cap-roundness", num(s.cap_roundness))
        .attr("station-outer-radius", num(s.station_outer_radius))
        .attr("station-inner-radius", num(s.station_inner_radius))
        .attr("tick-width", num(s.tick_width))
        .attr("tick-length", num(s.tick_length))
        .attr("label-offset", num(s.label_offset))
        .attr("label-font-size", num(s.label_font_size))
        .attr("label-font-family", &s.label_font_family);
    if let Some(size) = s.icon_size {
        t = t.attr("icon-size", num(size));
    }
    t.attr("route-mode", enum_name(&s.route_mode)).empty()
}

fn style_attrs(t: Tag, s: &LineStyle) -> Tag {
    t.attr("color", &s.color)
        .attr("kind", enum_name(&s.kind))
        .attr("inner-color", &s.inner_color)
        .attr("inner-width", num(s.inner_width_pct))
        .attr("dash-length", num(s.dash_length))
        .attr("dash-gap", num(s.dash_gap))
        .attr("dash-width", num(s.dash_width_pct))
        .attr("dash-start", num(s.dash_start_offset))
        .attr("dash-end", num(s.dash_end_offset))
        .attr("start-cap", enum_name(&s.start_cap))
        .attr("end-cap", enum_name(&s.end_cap))
}

fn layer_tag(l: &Layer) -> String {
    let base = |kind: &str| Tag::new(kind).attr("id", l.id).attr("name", &l.name).flag("visible", l.visible, true).flag("locked", l.locked, false);
    match &l.content {
        Content::Line(seg) => style_attrs(base("segment").attr("line", seg.line_id).attr("points", cells(&seg.points)), &seg.style).empty(),
        Content::Station(st) => {
            let ids = st.line_ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(" ");
            let t = base("station").attr("lines", ids).attr("offset", vec2(st.offset));
            match &st.shape {
                StationShape::Dot { cell: c, outer_color, inner_color } => {
                    t.attr("shape", "dot").attr("cell", cell(*c)).attr("outer-color", outer_color).attr("inner-color", inner_color).empty()
                }
                StationShape::Tick { cell: c, dir, side, along, width, length, color } => t
                    .attr("shape", "tick")
                    .attr("cell", cell(*c))
                    .attr("dir", DIRS[(dir.0 % 8) as usize])
                    .attr("side", side)
                    .attr("along", along)
                    .attr("width", num(*width))
                    .attr("length", num(*length))
                    .attr("color", color)
                    .empty(),
                StationShape::Pill { cells: cs, outer_color, inner_color } => {
                    t.attr("shape", "pill").attr("cells", cells(cs)).attr("outer-color", outer_color).attr("inner-color", inner_color).empty()
                }
            }
        }
        Content::Text(t) => {
            let mut tag = base("text")
                .attr("pos", vec2(t.pos))
                .attr("width", num(t.width))
                .attr("size", num(t.size))
                .attr("color", &t.color)
                .attr("font-family", &t.font_family)
                .flag("bold", t.bold, false)
                .flag("italic", t.italic, false)
                .attr("align", enum_name(&t.align))
                .attr("direction", enum_name(&t.direction))
                .attr("rotation", num(t.rotation));
            if let Some(id) = t.station_id {
                tag = tag.attr("station", id);
            }
            // One <transit:p> per paragraph holding its <transit:run>s,
            // written without whitespace in between (it would be text).
            let mut inner = String::new();
            for p in &t.paragraphs {
                let mut pt = Tag::new("p");
                if let Some(a) = p.align {
                    pt = pt.attr("align", enum_name(&a));
                }
                let mut runs = String::new();
                for r in &p.runs {
                    let mut rt = Tag::new("run");
                    if let Some(v) = &r.style.font_family {
                        rt = rt.attr("font-family", v);
                    }
                    if let Some(v) = r.style.size {
                        rt = rt.attr("size", num(v));
                    }
                    if let Some(v) = &r.style.color {
                        rt = rt.attr("color", v);
                    }
                    if let Some(v) = r.style.bold {
                        rt = rt.attr("bold", v);
                    }
                    if let Some(v) = r.style.italic {
                        rt = rt.attr("italic", v);
                    }
                    runs.push_str(&rt.with_text("run", &r.text));
                }
                inner.push_str(&pt.with_children("p", &runs));
            }
            tag.with_children("text", &inner)
        }
        Content::Icon(i) => base("icon")
            .attr("pos", vec2(i.pos))
            .attr("icon", &i.icon)
            .attr("size", num(i.size))
            .attr("rotation", num(i.rotation))
            .attr("color", &i.color)
            .attr("background", &i.background)
            .empty(),
        Content::Shape(sh) => {
            let t = base("shape")
                .attr("color", &sh.color)
                .flag("filled", sh.filled, true)
                .attr("border-color", &sh.border_color)
                .attr("border-width", num(sh.border_width));
            match &sh.geometry {
                ShapeGeometry::Line { from, to, thickness, cap, dashed, dash_length, dash_gap } => t
                    .attr("shape", "line")
                    .attr("from", vec2(*from))
                    .attr("to", vec2(*to))
                    .attr("thickness", num(*thickness))
                    .attr("cap", enum_name(cap))
                    .flag("dashed", *dashed, false)
                    .attr("dash-length", num(*dash_length))
                    .attr("dash-gap", num(*dash_gap)),
                ShapeGeometry::Rect { pos, width, height, rotation, corner_radius } => t
                    .attr("shape", "rect")
                    .attr("pos", vec2(*pos))
                    .attr("width", num(*width))
                    .attr("height", num(*height))
                    .attr("rotation", num(*rotation))
                    .attr("corner-radius", num(*corner_radius)),
                ShapeGeometry::Circle { center, radius } => t.attr("shape", "circle").attr("center", vec2(*center)).attr("radius", num(*radius)),
            }
            .empty()
        }
    }
}

fn group_tag(g: &Group) -> String {
    let t = Tag::new("group").attr("id", g.id).attr("name", &g.name).flag("visible", g.visible, true).flag("locked", g.locked, false).flag("expanded", g.expanded, true);
    match g.kind {
        GroupKind::Generic => t.attr("role", "generic"),
        GroupKind::Line { line_id } => t.attr("role", "line").attr("line", line_id),
        GroupKind::Station => t.attr("role", "station"),
    }
    .empty()
}

fn write_nodes(nodes: &[Node], doc: &Document, out: &mut String, depth: usize) {
    for node in nodes {
        let pad = "  ".repeat(depth);
        let hidden = if node.visible() { "" } else { r#" display="none""# };
        match node {
            Node::Group(g) => {
                let _ = writeln!(out, r#"{pad}<g id="group-{}" data-name="{}"{hidden}>"#, g.id, esc(&g.name));
                let _ = writeln!(out, "{pad}  {}", group_tag(g));
                write_nodes(&g.children, doc, out, depth + 1);
                let _ = writeln!(out, "{pad}</g>");
            }
            Node::Layer(l) => {
                let _ = writeln!(
                    out,
                    r#"{pad}<g id="layer-{}" data-name="{}"{hidden}>{}{}</g>"#,
                    l.id,
                    esc(&l.name),
                    layer_tag(l),
                    render::content_svg(&l.content, &doc.settings, &doc.assets)
                );
            }
        }
    }
}

/// The document as a `.transit.svg` file.
pub fn to_transit(doc: &Document) -> String {
    let opts = ExportOptions::default();
    let doc = &render::scaled(doc, &opts);
    let mut out = render::svg_head(doc, &opts, &format!(r#" xmlns:transit="{NS}""#));
    out.push_str("  <metadata>\n");
    let _ = writeln!(out, r#"    <transit:document version="{VERSION}" next-id="{}">"#, doc.next_id);
    let _ = writeln!(out, "      {}", settings_tag(&doc.settings));
    for i in &doc.assets.icons {
        let tag = Tag::new("icon").attr("name", &i.name).attr("label", &i.label).attr("view-box", &i.view_box);
        let _ = writeln!(out, "      {}>{}</transit:icon>", tag.0, i.body);
    }
    out.push_str("    </transit:document>\n  </metadata>\n");
    if !doc.assets.fonts.is_empty() {
        out.push_str("  <defs>\n");
        for f in &doc.assets.fonts {
            let _ = writeln!(out, r#"    <style transit:font="{}">{}</style>"#, attr_esc(&f.family), font_face_css(&[f]));
        }
        out.push_str("  </defs>\n");
    }
    write_nodes(&doc.root, doc, &mut out, 1);
    out.push_str("</svg>\n");
    out
}

// ------------------------------------------------------------------ reading

fn perr(msg: impl Into<String>) -> EditError {
    EditError::Parse(msg.into())
}

fn is_transit(n: &roxmltree::Node, name: &str) -> bool {
    n.is_element() && n.tag_name().namespace() == Some(NS) && n.tag_name().name() == name
}

/// Typed access to the attributes of a `<transit:…>` element.
struct El<'a, 'i>(roxmltree::Node<'a, 'i>);

impl<'a, 'i> El<'a, 'i> {
    fn what(&self) -> String {
        match self.0.attribute("id") {
            Some(id) => format!("<{} id=\"{id}\">", self.0.tag_name().name()),
            None => format!("<{}>", self.0.tag_name().name()),
        }
    }
    fn bad(&self, key: &str, v: &str) -> EditError {
        perr(format!("{}: invalid {key} \"{v}\"", self.what()))
    }
    fn get(&self, key: &str) -> Option<&'a str> {
        self.0.attribute(key)
    }
    fn req(&self, key: &str) -> Result<&'a str> {
        self.get(key).ok_or_else(|| perr(format!("{}: missing attribute {key}", self.what())))
    }
    fn parse<T: std::str::FromStr>(&self, key: &str, v: &str) -> Result<T> {
        v.trim().parse().map_err(|_| self.bad(key, v))
    }
    fn string(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or(default).to_string()
    }
    fn id(&self, key: &str) -> Result<Id> {
        self.parse(key, self.req(key)?)
    }
    fn num(&self, key: &str, default: f64) -> Result<f64> {
        let v = self.get(key).map_or(Ok(default), |v| self.parse::<f64>(key, v))?;
        if v.is_finite() {
            Ok(v)
        } else {
            Err(self.bad(key, &v.to_string()))
        }
    }
    fn int<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        self.get(key).map_or(Ok(default), |v| self.parse(key, v))
    }
    fn bool(&self, key: &str, default: bool) -> Result<bool> {
        match self.get(key) {
            None => Ok(default),
            Some("true") => Ok(true),
            Some("false") => Ok(false),
            Some(v) => Err(self.bad(key, v)),
        }
    }
    fn opt_num(&self, key: &str) -> Result<Option<f64>> {
        self.get(key).map(|_| self.num(key, 0.0)).transpose()
    }
    fn opt_bool(&self, key: &str) -> Result<Option<bool>> {
        self.get(key).map(|_| self.bool(key, false)).transpose()
    }
    fn enumeration<T: DeserializeOwned + Default>(&self, key: &str) -> Result<T> {
        match self.get(key) {
            None => Ok(T::default()),
            Some(v) => serde_json::from_value(serde_json::Value::String(v.into())).map_err(|_| self.bad(key, v)),
        }
    }
    fn split_pair<'v>(&self, key: &str, v: &'v str) -> Result<(&'v str, &'v str)> {
        v.split_once(',').ok_or_else(|| self.bad(key, v))
    }
    fn cell_of(&self, key: &str, v: &str) -> Result<Cell> {
        let (x, y) = self.split_pair(key, v)?;
        Ok(Cell::new(self.parse(key, x)?, self.parse(key, y)?))
    }
    fn cell(&self, key: &str) -> Result<Cell> {
        self.cell_of(key, self.req(key)?)
    }
    fn cells(&self, key: &str) -> Result<Vec<Cell>> {
        self.req(key)?.split_whitespace().map(|c| self.cell_of(key, c)).collect()
    }
    fn vec2(&self, key: &str) -> Result<Vec2> {
        let Some(v) = self.get(key) else { return Ok(Vec2::default()) };
        let (x, y) = self.split_pair(key, v)?;
        let p = Vec2::new(self.parse(key, x)?, self.parse(key, y)?);
        if p.x.is_finite() && p.y.is_finite() {
            Ok(p)
        } else {
            Err(self.bad(key, v))
        }
    }
}

fn read_settings(e: &El) -> Result<DocSettings> {
    let d = DocSettings::default();
    Ok(DocSettings {
        line_width: e.num("line-width", d.line_width)?,
        corner_radius: e.num("corner-radius", d.corner_radius)?,
        cap_roundness: e.num("cap-roundness", d.cap_roundness)?,
        station_outer_radius: e.num("station-outer-radius", d.station_outer_radius)?,
        station_inner_radius: e.num("station-inner-radius", d.station_inner_radius)?,
        tick_width: e.num("tick-width", d.tick_width)?,
        tick_length: e.num("tick-length", d.tick_length)?,
        label_offset: e.num("label-offset", d.label_offset)?,
        label_font_size: e.num("label-font-size", d.label_font_size)?,
        label_font_family: e.string("label-font-family", &d.label_font_family),
        icon_size: e.get("icon-size").map(|_| e.num("icon-size", 0.0)).transpose()?,
        route_mode: e.enumeration("route-mode")?,
        ..d
    })
}

fn read_style(e: &El) -> Result<LineStyle> {
    let d = LineStyle::default();
    Ok(LineStyle {
        color: e.string("color", &d.color),
        kind: e.enumeration("kind")?,
        inner_color: e.string("inner-color", &d.inner_color),
        inner_width_pct: e.num("inner-width", d.inner_width_pct)?,
        dash_length: e.num("dash-length", d.dash_length)?,
        dash_gap: e.num("dash-gap", d.dash_gap)?,
        dash_width_pct: e.num("dash-width", d.dash_width_pct)?,
        dash_start_offset: e.num("dash-start", d.dash_start_offset)?,
        dash_end_offset: e.num("dash-end", d.dash_end_offset)?,
        start_cap: e.enumeration("start-cap")?,
        end_cap: e.enumeration("end-cap")?,
    })
}

fn read_station(e: &El) -> Result<Station> {
    let line_ids = e.get("lines").unwrap_or("").split_whitespace().map(|v| e.parse("lines", v)).collect::<Result<Vec<Id>>>()?;
    let color = |key: &str, default: &str| e.string(key, default);
    let shape = match e.req("shape")? {
        "dot" => StationShape::Dot { cell: e.cell("cell")?, outer_color: color("outer-color", "#000000"), inner_color: color("inner-color", "#ffffff") },
        "tick" => {
            let dir = e.req("dir")?;
            let d = DocSettings::default();
            StationShape::Tick {
                cell: e.cell("cell")?,
                dir: Dir8(DIRS.iter().position(|n| *n == dir).ok_or_else(|| e.bad("dir", dir))? as u8),
                side: e.int("side", 1)?,
                along: e.int("along", -1)?,
                width: e.num("width", d.tick_width)?,
                length: e.num("length", d.tick_length)?,
                color: color("color", "#000000"),
            }
        }
        "pill" => StationShape::Pill { cells: e.cells("cells")?, outer_color: color("outer-color", "#000000"), inner_color: color("inner-color", "#ffffff") },
        other => return Err(e.bad("shape", other)),
    };
    Ok(Station { line_ids, shape, offset: e.vec2("offset")? })
}

fn read_text(e: &El) -> Result<TextBox> {
    let d = TextBox::default();
    let mut paragraphs = Vec::new();
    for p in e.0.children().filter(|n| is_transit(n, "p")) {
        let pe = El(p);
        let align = if pe.get("align").is_some() { Some(pe.enumeration("align")?) } else { None };
        let mut runs = Vec::new();
        for r in p.children().filter(|n| is_transit(n, "run")) {
            let re = El(r);
            let style = RunStyle {
                font_family: re.get("font-family").map(String::from),
                size: re.opt_num("size")?,
                color: re.get("color").map(String::from),
                bold: re.opt_bool("bold")?,
                italic: re.opt_bool("italic")?,
            };
            runs.push(TextRun { text: r.text().unwrap_or("").to_string(), style });
        }
        paragraphs.push(Paragraph { align, runs });
    }
    let mut t = TextBox {
        pos: e.vec2("pos")?,
        paragraphs,
        width: e.num("width", 0.0)?.max(0.0),
        size: e.num("size", d.size)?,
        color: e.string("color", &d.color),
        font_family: e.string("font-family", &d.font_family),
        bold: e.bool("bold", false)?,
        italic: e.bool("italic", false)?,
        align: e.enumeration("align")?,
        direction: e.enumeration("direction")?,
        rotation: e.num("rotation", 0.0)?,
        station_id: e.get("station").map(|_| e.id("station")).transpose()?,
    };
    t.normalize();
    if e.get("width").is_none() {
        t.width = t.estimated_width();
    }
    Ok(t)
}

fn read_icon(e: &El) -> Result<Icon> {
    let d = Icon::default();
    Ok(Icon {
        pos: e.vec2("pos")?,
        icon: e.string("icon", &d.icon),
        size: e.num("size", d.size)?,
        rotation: e.num("rotation", 0.0)?,
        color: e.string("color", &d.color),
        background: e.string("background", &d.background),
    })
}

fn read_shape(e: &El) -> Result<Shape> {
    let geometry = match e.req("shape")? {
        "line" => ShapeGeometry::Line {
            from: e.vec2("from")?,
            to: e.vec2("to")?,
            thickness: e.num("thickness", 0.1)?,
            cap: if e.get("cap").is_some() { e.enumeration("cap")? } else { Cap::Butt },
            dashed: e.bool("dashed", false)?,
            dash_length: e.num("dash-length", 0.5)?,
            dash_gap: e.num("dash-gap", 0.3)?,
        },
        "rect" => ShapeGeometry::Rect {
            pos: e.vec2("pos")?,
            width: e.num("width", 1.0)?,
            height: e.num("height", 1.0)?,
            rotation: e.num("rotation", 0.0)?,
            corner_radius: e.num("corner-radius", 0.0)?,
        },
        "circle" => ShapeGeometry::Circle { center: e.vec2("center")?, radius: e.num("radius", 0.5)? },
        other => return Err(e.bad("shape", other)),
    };
    Ok(Shape {
        geometry,
        color: e.string("color", "#1a1a1a"),
        filled: e.bool("filled", true)?,
        border_color: e.string("border-color", "#000000"),
        border_width: e.num("border-width", 0.0)?,
    })
}

/// The layer tree under `parent`: every `<g>` whose first element is a
/// `<transit:…>` description. Anything else (the rendered shapes, the
/// background, foreign markup) is ignored.
fn read_nodes(parent: roxmltree::Node) -> Result<Vec<Node>> {
    let mut out = Vec::new();
    for g in parent.children().filter(|n| n.is_element() && n.tag_name().name() == "g") {
        let Some(data) = g.children().find(|n| n.is_element()).filter(|n| n.tag_name().namespace() == Some(NS)) else { continue };
        let e = El(data);
        let id = e.id("id")?;
        let name = e.string("name", "");
        let visible = e.bool("visible", true)?;
        let locked = e.bool("locked", false)?;
        let content = match data.tag_name().name() {
            "group" => {
                let kind = match e.req("role")? {
                    "generic" => GroupKind::Generic,
                    "line" => GroupKind::Line { line_id: e.id("line")? },
                    "station" => GroupKind::Station,
                    other => return Err(e.bad("role", other)),
                };
                let expanded = e.bool("expanded", true)?;
                out.push(Node::Group(Group { id, name, visible, locked, expanded, kind, children: read_nodes(g)? }));
                continue;
            }
            "segment" => Content::Line(LineSegment { line_id: e.id("line")?, points: e.cells("points")?, style: read_style(&e)? }),
            "station" => Content::Station(read_station(&e)?),
            "text" => Content::Text(read_text(&e)?),
            "icon" => Content::Icon(read_icon(&e)?),
            "shape" => Content::Shape(read_shape(&e)?),
            other => return Err(perr(format!("unknown element <transit:{other}>"))),
        };
        out.push(Node::Layer(Layer { id, name, visible, locked, content }));
    }
    Ok(out)
}

/// The markup between the first and the last child of an element, as written.
fn inner_markup<'i>(text: &'i str, n: roxmltree::Node) -> &'i str {
    match (n.first_child(), n.last_child()) {
        (Some(a), Some(b)) => &text[a.range().start..b.range().end],
        _ => "",
    }
}

/// Read a `.transit.svg` file.
pub fn from_transit(text: &str) -> Result<Document> {
    let opt = roxmltree::ParsingOptions { allow_dtd: false, ..Default::default() };
    let xml = roxmltree::Document::parse_with_options(text, opt).map_err(|e| perr(format!("not a valid SVG file: {e}")))?;
    let root = xml.root_element();
    if root.tag_name().name() != "svg" {
        return Err(perr("not an SVG file"));
    }
    let meta = root
        .descendants()
        .find(|n| is_transit(n, "document"))
        .ok_or_else(|| perr("this SVG file has no Transit Diagram Maker data (it was not saved by this app)"))?;
    let m = El(meta);
    let version: u32 = m.int("version", 0)?;
    if version > VERSION {
        return Err(perr(format!("file version {version} is newer than supported ({VERSION})")));
    }
    let mut doc = Document { next_id: m.int("next-id", 1)?, ..Document::default() };
    if let Some(s) = meta.children().find(|n| is_transit(n, "settings")) {
        doc.settings = read_settings(&El(s))?;
    }
    for i in meta.children().filter(|n| is_transit(n, "icon")) {
        let e = El(i);
        doc.assets.icons.push(IconAsset {
            name: e.req("name")?.to_string(),
            label: e.string("label", ""),
            view_box: e.string("view-box", "0 0 24 24"),
            body: inner_markup(text, i).to_string(),
        });
    }
    for style in root.descendants().filter(|n| n.is_element() && n.tag_name().name() == "style") {
        let Some(family) = style.attribute((NS, "font")) else { continue };
        let css = style.text().unwrap_or("");
        let data = css.split_once("url(").and_then(|(_, rest)| rest.split_once(')')).map(|(d, _)| d.trim_matches(['"', '\'']));
        let data = data.ok_or_else(|| perr(format!("font \"{family}\" has no data")))?;
        doc.assets.fonts.push(FontAsset { family: family.to_string(), data: data.into() });
    }
    doc.root = read_nodes(root)?;
    validate(doc)
}

/// Repair or reject anything that would break editing: duplicate ids, stale
/// id counters, malformed lines and out-of-range settings.
pub fn validate(mut doc: Document) -> Result<Document> {
    let mut seen = HashSet::new();
    let mut max_id = 0;
    let mut dup = None;
    tree::walk_nodes(&doc.root, &mut |n| {
        max_id = max_id.max(n.id());
        if !seen.insert(n.id()) {
            dup = Some(n.id());
        }
    });
    tree::walk_layers(&doc.root, &mut |l, _, _| {
        if let Content::Line(seg) = &l.content {
            max_id = max_id.max(seg.line_id);
        }
    });
    if let Some(id) = dup {
        return Err(EditError::Parse(format!("two layers share the identifier {id}")));
    }
    doc.next_id = doc.next_id.max(max_id + 1);
    doc.settings = doc.settings.sanitized();
    doc.assets = std::mem::take(&mut doc.assets).sanitized();
    let route = doc.settings.route_mode;
    tree::walk_layers_mut(&mut doc.root, &mut |l| {
        match &mut l.content {
            Content::Line(seg) => seg.points = geom::normalize_path(&seg.points, route),
            Content::Text(t) => {
                t.rotation = free_rotation(t.rotation);
                t.pos = t.pos.rounded(1000.0);
            }
            Content::Icon(i) => {
                i.rotation = free_rotation(i.rotation);
                i.pos = i.pos.snapped();
            }
            Content::Shape(sh) => tidy_shape(sh),
            Content::Station(_) => {}
        }
    });
    remove_degenerate(&mut doc.root);
    Ok(doc)
}

fn remove_degenerate(nodes: &mut Vec<Node>) {
    nodes.retain_mut(|n| match n {
        Node::Group(g) => {
            remove_degenerate(&mut g.children);
            true
        }
        Node::Layer(l) => !matches!(&l.content, Content::Line(s) if s.points.len() < 2)
            && !matches!(&l.content, Content::Station(st) if st.shape.cells().is_empty()),
    });
}
