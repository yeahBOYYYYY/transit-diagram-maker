//! SVG generation. The same markup is used for the live canvas and for export,
//! so what you see is exactly what gets exported. Output units are pixels
//! (cell units × `grid_size`).

use crate::geom::{fmt_num, rounded_polyline, Cell, Rect, Vec2};
use crate::icons;
use crate::model::*;
use crate::tree;
use serde::Serialize;
use std::collections::HashSet;
use std::fmt::Write;

pub fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&#39;"),
            c => o.push(c),
        }
    }
    o
}

fn n(v: f64) -> String {
    fmt_num(v)
}

/// Filled shape that closes off a line end. `c` is the last cell center, `d` the
/// outward unit direction, `h` the half width and `rr` the corner radius.
fn cap_path(c: Vec2, d: Vec2, h: f64, rr: f64) -> String {
    let nrm = d.perp();
    // Overlap slightly with the body so antialiasing never shows a seam.
    let ov = h * 0.1;
    let base = c.sub(d.scale(ov));
    let p1 = base.add(nrm.scale(h));
    let p2 = c.add(nrm.scale(h)).add(d.scale(h - rr));
    let p3 = c.add(nrm.scale(h - rr)).add(d.scale(h));
    let p4 = c.sub(nrm.scale(h - rr)).add(d.scale(h));
    let p5 = c.sub(nrm.scale(h)).add(d.scale(h - rr));
    let p6 = base.sub(nrm.scale(h));
    let mut s = format!("M{} {}L{} {}", n(p1.x), n(p1.y), n(p2.x), n(p2.y));
    if rr > 0.0 {
        let _ = write!(s, "A{} {} 0 0 1 {} {}", n(rr), n(rr), n(p3.x), n(p3.y));
    }
    let _ = write!(s, "L{} {}", n(p4.x), n(p4.y));
    if rr > 0.0 {
        let _ = write!(s, "A{} {} 0 0 1 {} {}", n(rr), n(rr), n(p5.x), n(p5.y));
    }
    let _ = write!(s, "L{} {}Z", n(p6.x), n(p6.y));
    s
}

fn cap_roundness(cap: Cap, settings: &DocSettings) -> f64 {
    match cap {
        Cap::Butt => 0.0,
        Cap::Round => settings.cap_roundness,
    }
}

/// Markup for a line built from the given vertices and style.
pub fn line_svg(points: &[Cell], style: &LineStyle, settings: &DocSettings) -> String {
    let g = settings.grid_size;
    if points.len() < 2 {
        return String::new();
    }
    let pts: Vec<Vec2> = points.iter().map(|c| c.center().scale(g)).collect();
    let body = rounded_polyline(&pts, settings.corner_radius * g);
    let w = settings.line_width * g;
    let h = w / 2.0;
    let start_d = pts[0].sub(pts[1]).normalized();
    let end_d = pts[pts.len() - 1].sub(pts[pts.len() - 2]).normalized();
    let rs = cap_roundness(style.start_cap, settings);
    let re = cap_roundness(style.end_cap, settings);
    let caps = |hw: f64| {
        format!(
            "{}{}",
            cap_path(pts[0], start_d, hw, rs * hw),
            cap_path(pts[pts.len() - 1], end_d, hw, re * hw)
        )
    };
    let color = esc(&style.color);
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<path d="{body}" fill="none" stroke="{color}" stroke-width="{}" stroke-linejoin="round"/><path d="{}" fill="{color}"/>"#,
        n(w),
        caps(h)
    );
    if matches!(style.kind, StrokeKind::Hollow | StrokeKind::Dashed) {
        let wi = w * (style.inner_width_pct / 100.0).clamp(0.0, 1.0);
        if wi > 0.0 {
            let inner = esc(&style.inner_color);
            let _ = write!(
                s,
                r#"<path d="{body}" fill="none" stroke="{inner}" stroke-width="{}" stroke-linejoin="round"/><path d="{}" fill="{inner}"/>"#,
                n(wi),
                caps(wi / 2.0)
            );
        }
    }
    if style.kind == StrokeKind::Dashed {
        let wd = w * (style.dash_width_pct / 100.0).clamp(0.0, 1.0);
        let len = (style.dash_length * g).max(0.1);
        let gap = (style.dash_gap * g).max(0.0);
        if wd > 0.0 {
            let _ = write!(
                s,
                r#"<path d="{body}" fill="none" stroke="{color}" stroke-width="{}" stroke-dasharray="{} {}" stroke-dashoffset="{}"/>"#,
                n(wd),
                n(len),
                n(gap),
                n(-style.dash_offset * g)
            );
        }
    }
    s
}

/// Skeleton of a blob of cells: centers plus links between all 8-neighbors.
/// Stroking it with round joins gives the Minkowski sum of the blob with a
/// disk, i.e. a shape whose outline has the same radius everywhere.
pub fn blob_skeleton(cells: &[Cell], g: f64) -> String {
    let set: HashSet<Cell> = cells.iter().copied().collect();
    let mut sorted: Vec<Cell> = set.iter().copied().collect();
    sorted.sort();
    let mut d = String::new();
    for c in &sorted {
        let p = c.center().scale(g);
        let _ = write!(d, "M{} {}h0.01", n(p.x), n(p.y));
        for (dx, dy) in [(1, 0), (0, 1), (1, 1), (1, -1)] {
            let o = c.offset(dx, dy);
            if set.contains(&o) {
                let q = o.center().scale(g);
                let _ = write!(d, "M{} {}L{} {}", n(p.x), n(p.y), n(q.x), n(q.y));
            }
        }
    }
    d
}

fn blob_svg(cells: &[Cell], outer: &str, inner: &str, settings: &DocSettings) -> String {
    let g = settings.grid_size;
    let d = blob_skeleton(cells, g);
    let ro = settings.station_outer_radius * g;
    let ri = settings.station_inner_radius.min(settings.station_outer_radius) * g;
    let mut s = format!(
        r#"<path d="{d}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
        esc(outer),
        n(ro * 2.0)
    );
    if ri > 0.0 {
        let _ = write!(
            s,
            r#"<path d="{d}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
            esc(inner),
            n(ri * 2.0)
        );
    }
    s
}

fn tick_corners(cell: Cell, dir: crate::geom::Dir8, side: i8, width: f64, length: f64, settings: &DocSettings) -> [Vec2; 4] {
    let g = settings.grid_size;
    let c = cell.center().scale(g);
    let d = dir.unit();
    let nrm = d.perp().scale(if side < 0 { -1.0 } else { 1.0 });
    let h = settings.line_width * g / 2.0;
    let base = c.add(nrm.scale(h - 0.05 * g));
    let tip = c.add(nrm.scale(h + length * g));
    let hw = d.scale(width * g / 2.0);
    [base.sub(hw), tip.sub(hw), tip.add(hw), base.add(hw)]
}

pub fn station_svg(st: &Station, settings: &DocSettings) -> String {
    match &st.shape {
        StationShape::Dot { cell, outer_color, inner_color } => blob_svg(&[*cell], outer_color, inner_color, settings),
        StationShape::Pill { cells, outer_color, inner_color } => blob_svg(cells, outer_color, inner_color, settings),
        StationShape::Tick { cell, dir, side, width, length, color } => {
            let p = tick_corners(*cell, *dir, *side, *width, *length, settings);
            format!(
                r#"<path d="M{} {}L{} {}L{} {}L{} {}Z" fill="{}"/>"#,
                n(p[0].x),
                n(p[0].y),
                n(p[1].x),
                n(p[1].y),
                n(p[2].x),
                n(p[2].y),
                n(p[3].x),
                n(p[3].y),
                esc(color)
            )
        }
    }
}

fn text_metrics(t: &TextBox, g: f64) -> (Vec<&str>, f64, f64) {
    let lines: Vec<&str> = t.text.split('\n').collect();
    let fs = t.size * g;
    let chars = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f64;
    let w = chars * fs * if t.bold { 0.62 } else { 0.56 };
    let h = lines.len() as f64 * fs * 1.2;
    (lines, w, h)
}

pub fn text_svg(t: &TextBox, settings: &DocSettings) -> String {
    let g = settings.grid_size;
    let p = t.pos.scale(g);
    let fs = t.size * g;
    let (lines, _, _) = text_metrics(t, g);
    let anchor = match t.align {
        TextAlign::Start => "start",
        TextAlign::Middle => "middle",
        TextAlign::End => "end",
    };
    let mut s = format!(
        r#"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{}" text-anchor="{anchor}" dominant-baseline="central""#,
        n(p.x),
        n(p.y),
        n(fs),
        esc(&t.font_family),
        esc(&t.color)
    );
    if t.bold {
        s.push_str(r#" font-weight="bold""#);
    }
    if t.italic {
        s.push_str(r#" font-style="italic""#);
    }
    if t.rotation != 0.0 {
        let _ = write!(s, r#" transform="rotate({} {} {})""#, n(t.rotation), n(p.x), n(p.y));
    }
    s.push_str(r#" style="white-space:pre">"#);
    let first_dy = -((lines.len() as f64 - 1.0) / 2.0) * 1.2;
    for (i, line) in lines.iter().enumerate() {
        let dy = if i == 0 { first_dy } else { 1.2 };
        let _ = write!(s, r#"<tspan x="{}" dy="{}em">{}</tspan>"#, n(p.x), n(dy), esc(line));
    }
    s.push_str("</text>");
    s
}

pub fn icon_svg(i: &Icon, settings: &DocSettings) -> String {
    let g = settings.grid_size;
    let p = i.pos.scale(g);
    let size = i.size * g;
    format!(
        r#"<g transform="translate({} {}) rotate({}) scale({}) translate(-12 -12)">{}</g>"#,
        n(p.x),
        n(p.y),
        n(i.rotation),
        n(size / 24.0),
        icons::markup(&i.icon, &i.color, &i.background)
    )
}

pub fn content_svg(c: &Content, settings: &DocSettings) -> String {
    match c {
        Content::Line(seg) => line_svg(&seg.points, &seg.style, settings),
        Content::Station(st) => station_svg(st, settings),
        Content::Text(t) => text_svg(t, settings),
        Content::Icon(i) => icon_svg(i, settings),
    }
}

fn rotated_rect_bounds(center: Vec2, rect: Rect, deg: f64) -> Rect {
    let corners = [
        Vec2::new(rect.x, rect.y),
        Vec2::new(rect.x + rect.w, rect.y),
        Vec2::new(rect.x + rect.w, rect.y + rect.h),
        Vec2::new(rect.x, rect.y + rect.h),
    ];
    Rect::from_points(corners.iter().map(|p| p.rotate_around(center, deg))).unwrap()
}

/// Bounding box of content in pixels (approximate for text).
pub fn content_bounds(c: &Content, settings: &DocSettings) -> Option<Rect> {
    let g = settings.grid_size;
    match c {
        Content::Line(seg) => {
            Rect::from_points(seg.points.iter().map(|p| p.center().scale(g))).map(|r| r.inflate(settings.line_width * g / 2.0))
        }
        Content::Station(st) => match &st.shape {
            StationShape::Tick { cell, dir, side, width, length, .. } => {
                Rect::from_points(tick_corners(*cell, *dir, *side, *width, *length, settings))
            }
            shape => Rect::from_points(shape.cells().iter().map(|p| p.center().scale(g)))
                .map(|r| r.inflate(settings.station_outer_radius * g)),
        },
        Content::Text(t) => {
            let (_, w, h) = text_metrics(t, g);
            let p = t.pos.scale(g);
            let x = match t.align {
                TextAlign::Start => p.x,
                TextAlign::Middle => p.x - w / 2.0,
                TextAlign::End => p.x - w,
            };
            Some(rotated_rect_bounds(p, Rect { x, y: p.y - h / 2.0, w, h }, t.rotation))
        }
        Content::Icon(i) => {
            let p = i.pos.scale(g);
            let s = i.size * g;
            Some(rotated_rect_bounds(p, Rect { x: p.x - s / 2.0, y: p.y - s / 2.0, w: s, h: s }, i.rotation))
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Fragment {
    pub id: Id,
    pub kind: &'static str,
    pub svg: String,
    pub locked: bool,
    pub bounds: Option<Rect>,
}

/// Visible layers in paint order, ready for the canvas.
pub fn render_fragments(doc: &Document) -> Vec<Fragment> {
    let mut out = Vec::new();
    tree::walk_layers(&doc.root, &mut |l, vis, locked| {
        if vis {
            out.push(Fragment {
                id: l.id,
                kind: l.content.kind_name(),
                svg: content_svg(&l.content, &doc.settings),
                locked,
                bounds: content_bounds(&l.content, &doc.settings),
            });
        }
    });
    out
}

/// Bounds of all visible content in pixels.
pub fn document_bounds(doc: &Document) -> Option<Rect> {
    let mut acc: Option<Rect> = None;
    tree::walk_layers(&doc.root, &mut |l, vis, _| {
        if vis {
            if let Some(b) = content_bounds(&l.content, &doc.settings) {
                acc = Some(acc.map_or(b, |a| a.union(b)));
            }
        }
    });
    acc
}

fn export_nodes(nodes: &[Node], settings: &DocSettings, out: &mut String, depth: usize) {
    for node in nodes {
        if !node.visible() {
            continue;
        }
        let pad = "  ".repeat(depth);
        match node {
            Node::Group(g) => {
                let _ = writeln!(out, r#"{pad}<g id="group-{}" data-name="{}">"#, g.id, esc(&g.name));
                export_nodes(&g.children, settings, out, depth + 1);
                let _ = writeln!(out, "{pad}</g>");
            }
            Node::Layer(l) => {
                let _ = writeln!(
                    out,
                    r#"{pad}<g id="layer-{}" data-name="{}">{}</g>"#,
                    l.id,
                    esc(&l.name),
                    content_svg(&l.content, settings)
                );
            }
        }
    }
}

/// Standalone SVG document of everything visible, cropped to the content.
pub fn export_svg(doc: &Document) -> String {
    let s = &doc.settings;
    let g = s.grid_size;
    let b = document_bounds(doc).unwrap_or(Rect { x: 0.0, y: 0.0, w: g, h: g }).inflate(s.export_margin * g);
    let mut out = String::new();
    let _ = writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}">"#,
        n(b.x),
        n(b.y),
        n(b.w),
        n(b.h),
        n(b.w),
        n(b.h)
    );
    let _ = writeln!(out, "  <title>Transit diagram</title>");
    if !s.export_background.is_empty() {
        let _ = writeln!(
            out,
            r#"  <rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
            n(b.x),
            n(b.y),
            n(b.w),
            n(b.h),
            esc(&s.export_background)
        );
    }
    export_nodes(&doc.root, s, &mut out, 1);
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Dir8;

    fn settings() -> DocSettings {
        DocSettings::default()
    }

    fn parse(markup: &str) {
        let doc = format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{markup}</svg>"#);
        roxmltree::Document::parse(&doc).unwrap_or_else(|e| panic!("bad svg {e}: {markup}"));
    }

    #[test]
    fn escape_special_characters() {
        assert_eq!(esc(r#"<a href="x">&'"#), "&lt;a href=&quot;x&quot;&gt;&amp;&#39;");
    }

    #[test]
    fn plain_line_has_body_and_caps() {
        let s = line_svg(&[Cell::new(0, 0), Cell::new(4, 0)], &LineStyle::default(), &settings());
        parse(&s);
        assert_eq!(s.matches("<path").count(), 2);
        assert!(s.contains(r#"stroke-width="24""#));
        assert!(s.contains("A12 12"), "round caps use arcs: {s}");
    }

    #[test]
    fn butt_caps_have_no_arcs() {
        let style = LineStyle { start_cap: Cap::Butt, end_cap: Cap::Butt, ..LineStyle::default() };
        let s = line_svg(&[Cell::new(0, 0), Cell::new(4, 0)], &style, &settings());
        let caps = s.split("<path").nth(2).unwrap();
        assert!(!caps.contains('A'), "{caps}");
        // The butt cap reaches the far edge of the last cell: x = 5 cells = 120px.
        assert!(caps.contains("L120 0") || caps.contains("L120 24"), "{caps}");
    }

    #[test]
    fn hollow_and_dashed_layers() {
        let pts = [Cell::new(0, 0), Cell::new(4, 0), Cell::new(6, 2)];
        let hollow = LineStyle { kind: StrokeKind::Hollow, ..LineStyle::default() };
        let s = line_svg(&pts, &hollow, &settings());
        parse(&s);
        assert_eq!(s.matches("<path").count(), 4);
        assert!(s.contains("#ffffff"));
        let dashed = LineStyle { kind: StrokeKind::Dashed, dash_offset: 0.5, ..LineStyle::default() };
        let s = line_svg(&pts, &dashed, &settings());
        parse(&s);
        assert_eq!(s.matches("<path").count(), 5);
        assert!(s.contains(r#"stroke-dasharray="14.4 9.6""#), "{s}");
        assert!(s.contains(r#"stroke-dashoffset="-12""#), "{s}");
    }

    #[test]
    fn degenerate_line_renders_nothing() {
        assert!(line_svg(&[Cell::new(0, 0)], &LineStyle::default(), &settings()).is_empty());
    }

    #[test]
    fn blob_skeleton_links_neighbors() {
        let d = blob_skeleton(&[Cell::new(0, 0), Cell::new(1, 0), Cell::new(1, 1), Cell::new(5, 5)], 10.0);
        assert_eq!(d.matches("h0.01").count(), 4);
        // (0,0)-(1,0), (0,0)-(1,1) diagonal, (1,0)-(1,1)
        assert_eq!(d.matches('L').count(), 3);
    }

    #[test]
    fn station_shapes_render() {
        let s = settings();
        let dot = Station {
            line_ids: vec![1],
            shape: StationShape::Dot { cell: Cell::new(2, 2), outer_color: "#000".into(), inner_color: "#fff".into() },
        };
        let m = station_svg(&dot, &s);
        parse(&m);
        assert!(m.contains(r#"stroke-width="29.76""#), "outer diameter 2*0.62*24: {m}");
        let tick = Station {
            line_ids: vec![1],
            shape: StationShape::Tick {
                cell: Cell::new(0, 0),
                dir: Dir8::E,
                side: 1,
                width: 0.5,
                length: 1.0,
                color: "#f00".into(),
            },
        };
        let m = station_svg(&tick, &s);
        parse(&m);
        let b = content_bounds(&Content::Station(tick), &s).unwrap();
        // Horizontal line, tick pokes up (left of east) from the border at y=0.
        assert!((b.y - (12.0 - 12.0 - 24.0)).abs() < 1e-9, "{b:?}");
        assert!((b.w - 12.0).abs() < 1e-9);
    }

    #[test]
    fn text_renders_multiline_escaped() {
        let t = TextBox { text: "A & B\n<C>".into(), rotation: 30.0, bold: true, ..TextBox::default() };
        let m = text_svg(&t, &settings());
        parse(&m);
        assert_eq!(m.matches("<tspan").count(), 2);
        assert!(m.contains("A &amp; B"));
        assert!(m.contains("rotate(30"));
        assert!(m.contains("bold"));
    }

    #[test]
    fn icon_renders_with_scale() {
        let i = Icon { pos: Vec2::new(1.0, 1.0), icon: "airport".into(), size: 1.0, ..Icon::default() };
        let m = icon_svg(&i, &settings());
        parse(&m);
        assert!(m.contains("scale(1)"));
        let b = content_bounds(&Content::Icon(i), &settings()).unwrap();
        assert_eq!(b, Rect { x: 12.0, y: 12.0, w: 24.0, h: 24.0 });
    }

    #[test]
    fn export_is_well_formed_and_skips_hidden() {
        let mut doc = Document::default();
        doc.root.push(Node::Layer(Layer {
            id: 1,
            name: "Visible <1>".into(),
            visible: true,
            locked: false,
            content: Content::Text(TextBox::default()),
        }));
        doc.root.push(Node::Layer(Layer {
            id: 2,
            name: "Hidden".into(),
            visible: false,
            locked: false,
            content: Content::Text(TextBox::default()),
        }));
        let svg = export_svg(&doc);
        let parsed = roxmltree::Document::parse(&svg).unwrap();
        assert_eq!(parsed.root_element().tag_name().name(), "svg");
        assert!(svg.contains("layer-1"));
        assert!(!svg.contains("layer-2"));
        assert!(svg.contains("Visible &lt;1&gt;"));
    }
}
