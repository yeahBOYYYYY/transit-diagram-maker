//! SVG generation. The same markup is used for the live canvas and for export,
//! so what you see is exactly what gets exported. Output units are pixels
//! (cell units × `DocSettings::cell_px`).

use crate::assets::{Assets, FontAsset};
use crate::geom::{dash_array, fmt_num, rounded_polyline, rounded_polyline_len, Cell, Rect, Vec2};
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
    let g = settings.cell_px;
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
    let radius = settings.corner_radius * g;
    let mut s = String::new();
    let gap = (style.dash_gap * g).max(0.0);
    let (lead, trail) = (style.dash_start_offset * g, style.dash_end_offset * g);
    let _ = write!(s, r#"<path d="{body}" fill="none" stroke="{color}" stroke-width="{}" stroke-linejoin="round"/>"#, n(w));
    let _ = write!(s, r#"<path d="{}" fill="{color}"/>"#, caps(h));
    if style.kind == StrokeKind::Hollow {
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
    if style.kind == StrokeKind::InnerDashed {
        // Dashes of the inner color run down the middle of the line, from the
        // first to the last cell center.
        let wd = w * (style.dash_width_pct / 100.0).clamp(0.0, 1.0);
        let pattern = dash_array(rounded_polyline_len(&pts, radius), lead, trail, (style.dash_length * g).max(0.5), gap);
        if let (true, Some(v)) = (wd > 0.0, pattern) {
            let dash_attrs = v.iter().map(|x| n(*x)).collect::<Vec<_>>().join(" ");
            let _ = write!(
                s,
                r#"<path d="{body}" fill="none" stroke="{}" stroke-width="{}" stroke-dasharray="{}"/>"#,
                esc(&style.inner_color),
                n(wd),
                dash_attrs
            );
        }
    }
    s
}

/// Skeleton of a blob of cells: centers, links between orthogonal neighbors,
/// filled squares for every 2×2 block and links between diagonal neighbors
/// that are not already connected through a shared orthogonal neighbor.
/// Stroking (and filling) it with round joins gives the Minkowski sum of the
/// blob with a disk: an outline with the same radius everywhere, with true
/// inside corners where the blob bends.
pub fn blob_skeleton(cells: &[Cell], g: f64) -> String {
    let set: HashSet<Cell> = cells.iter().copied().collect();
    let mut sorted: Vec<Cell> = set.iter().copied().collect();
    sorted.sort();
    let pt = |c: Cell| {
        let p = c.center().scale(g);
        format!("{} {}", n(p.x), n(p.y))
    };
    let mut d = String::new();
    for &c in &sorted {
        let _ = write!(d, "M{}h0.01", pt(c));
        for (dx, dy) in [(1, 0), (0, 1)] {
            let o = c.offset(dx, dy);
            if set.contains(&o) {
                let _ = write!(d, "M{}L{}", pt(c), pt(o));
            }
        }
        for (dx, dy) in [(1, 1), (1, -1)] {
            let o = c.offset(dx, dy);
            let bridged = set.contains(&c.offset(dx, 0)) || set.contains(&c.offset(0, dy));
            if set.contains(&o) && !bridged {
                let _ = write!(d, "M{}L{}", pt(c), pt(o));
            }
        }
        let (r, b, rb) = (c.offset(1, 0), c.offset(0, 1), c.offset(1, 1));
        if set.contains(&r) && set.contains(&b) && set.contains(&rb) {
            let _ = write!(d, "M{}L{}L{}L{}Z", pt(c), pt(r), pt(rb), pt(b));
        }
    }
    d
}

fn blob_svg(cells: &[Cell], outer: &str, inner: &str, settings: &DocSettings) -> String {
    let g = settings.cell_px;
    let d = blob_skeleton(cells, g);
    let ro = settings.station_outer_radius * g;
    let ri = settings.station_inner_radius.min(settings.station_outer_radius) * g;
    let (outer, inner) = (esc(outer), esc(inner));
    let mut s = format!(
        r#"<path d="{d}" fill="{outer}" stroke="{outer}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
        n(ro * 2.0)
    );
    if ri > 0.0 {
        let _ = write!(
            s,
            r#"<path d="{d}" fill="{inner}" stroke="{inner}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
            n(ri * 2.0)
        );
    }
    s
}

fn tick_corners(cell: Cell, dir: crate::geom::Dir8, side: i8, along: i8, width: f64, length: f64, settings: &DocSettings) -> [Vec2; 4] {
    let g = settings.cell_px;
    let c = crate::geom::tick_center(cell, dir, along, width).scale(g);
    let d = dir.unit();
    let nrm = d.perp().scale(if side < 0 { -1.0 } else { 1.0 });
    let h = settings.line_width * g / 2.0;
    let base = c.add(nrm.scale(h - 0.05 * g));
    let tip = c.add(nrm.scale(h + length * g));
    let hw = d.scale(width * g / 2.0);
    [base.sub(hw), tip.sub(hw), tip.add(hw), base.add(hw)]
}

pub fn station_svg(st: &Station, settings: &DocSettings) -> String {
    let body = station_shape_svg(st, settings);
    if st.offset == Vec2::default() {
        return body;
    }
    let o = st.offset.scale(settings.cell_px);
    format!(r#"<g transform="translate({} {})">{body}</g>"#, n(o.x), n(o.y))
}

fn station_shape_svg(st: &Station, settings: &DocSettings) -> String {
    match &st.shape {
        StationShape::Dot { cell, outer_color, inner_color } => blob_svg(&[*cell], outer_color, inner_color, settings),
        StationShape::Pill { cells, outer_color, inner_color } => blob_svg(cells, outer_color, inner_color, settings),
        StationShape::Tick { cell, dir, side, along, width, length, color } => {
            let p = tick_corners(*cell, *dir, *side, *along, *width, *length, settings);
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

/// Formatting attributes of a run that differ from its text box.
fn run_attrs(t: &TextBox, r: &RunStyle, g: f64) -> String {
    let mut a = String::new();
    if let Some(v) = &r.font_family {
        let _ = write!(a, r#" font-family="{}""#, esc(v));
    }
    if let Some(v) = r.size {
        let _ = write!(a, r#" font-size="{}""#, n(v * g));
    }
    if let Some(v) = &r.color {
        let _ = write!(a, r#" fill="{}""#, esc(v));
    }
    if let Some(v) = r.bold.filter(|v| *v != t.bold) {
        a.push_str(if v { r#" font-weight="bold""# } else { r#" font-weight="normal""# });
    }
    if let Some(v) = r.italic.filter(|v| *v != t.italic) {
        a.push_str(if v { r#" font-style="italic""# } else { r#" font-style="normal""# });
    }
    a
}

/// A text box: one `<tspan>` per paragraph (marked `data-line` so the editor
/// can measure it) holding one `<tspan>` per formatted run. Lines are placed
/// from the top-left corner of the box and aligned inside its width.
pub fn text_svg(t: &TextBox, settings: &DocSettings) -> String {
    let g = settings.cell_px;
    let p = t.pos.scale(g);
    let w = t.width * g;
    // `align` is visual (start = left), while SVG anchors follow the writing
    // direction, so right-to-left text swaps them.
    let rtl = t.direction == TextDirection::Rtl;
    let mut s = format!(
        r#"<text font-size="{}" font-family="{}" fill="{}""#,
        n(t.size * g),
        esc(&t.font_family),
        esc(&t.color)
    );
    if rtl {
        s.push_str(r#" direction="rtl""#);
    }
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
    let mut top = p.y;
    for para in &t.paragraphs {
        let size = t.line_size(para) * g;
        let (x, anchor) = match (t.align_of(para), rtl) {
            (TextAlign::Middle, _) => (p.x + w / 2.0, "middle"),
            (TextAlign::Start, false) => (p.x, "start"),
            (TextAlign::Start, true) => (p.x, "end"),
            (TextAlign::End, false) => (p.x + w, "end"),
            (TextAlign::End, true) => (p.x + w, "start"),
        };
        let _ = write!(s, r#"<tspan data-line="" x="{}" y="{}" text-anchor="{anchor}">"#, n(x), n(top + size * BASELINE));
        for r in &para.runs {
            if r.style.is_empty() {
                s.push_str(&esc(&r.text));
            } else {
                let _ = write!(s, "<tspan{}>{}</tspan>", run_attrs(t, &r.style, g), esc(&r.text));
            }
        }
        s.push_str("</tspan>");
        top += size * LINE_SPACING;
    }
    s.push_str("</text>");
    s
}

pub fn icon_svg(i: &Icon, settings: &DocSettings, assets: &Assets) -> String {
    let g = settings.cell_px;
    let p = i.pos.scale(g);
    let size = i.size * g;
    format!(
        r#"<g transform="translate({} {}) rotate({}) scale({}) translate(-12 -12)">{}</g>"#,
        n(p.x),
        n(p.y),
        n(i.rotation),
        n(size / 24.0),
        match assets.icon(&i.icon) {
            Some(custom) => crate::assets::icon_markup(custom, &i.color, &i.background),
            None => icons::markup(&i.icon, &i.color, &i.background),
        }
    )
}

pub fn content_svg(c: &Content, settings: &DocSettings, assets: &Assets) -> String {
    match c {
        Content::Line(seg) => line_svg(&seg.points, &seg.style, settings),
        Content::Station(st) => station_svg(st, settings),
        Content::Text(t) => text_svg(t, settings),
        Content::Icon(i) => icon_svg(i, settings, assets),
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
    let g = settings.cell_px;
    match c {
        Content::Line(seg) => {
            Rect::from_points(seg.points.iter().map(|p| p.center().scale(g))).map(|r| r.inflate(settings.line_width * g / 2.0))
        }
        Content::Station(st) => {
            let r = match &st.shape {
                StationShape::Tick { cell, dir, side, along, width, length, .. } => {
                    Rect::from_points(tick_corners(*cell, *dir, *side, *along, *width, *length, settings))
                }
                shape => Rect::from_points(shape.cells().iter().map(|p| p.center().scale(g)))
                    .map(|r| r.inflate(settings.station_outer_radius * g)),
            };
            r.map(|r| Rect { x: r.x + st.offset.x * g, y: r.y + st.offset.y * g, ..r })
        }
        Content::Text(t) => {
            let p = t.pos.scale(g);
            Some(rotated_rect_bounds(p, Rect { x: p.x, y: p.y, w: t.width * g, h: t.height() * g }, t.rotation))
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
                svg: content_svg(&l.content, &doc.settings, &doc.assets),
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

fn export_nodes(nodes: &[Node], settings: &DocSettings, assets: &Assets, out: &mut String, depth: usize) {
    for node in nodes {
        if !node.visible() {
            continue;
        }
        let pad = "  ".repeat(depth);
        match node {
            Node::Group(g) => {
                let _ = writeln!(out, r#"{pad}<g id="group-{}" data-name="{}">"#, g.id, esc(&g.name));
                export_nodes(&g.children, settings, assets, out, depth + 1);
                let _ = writeln!(out, "{pad}</g>");
            }
            Node::Layer(l) => {
                let _ = writeln!(
                    out,
                    r#"{pad}<g id="layer-{}" data-name="{}">{}</g>"#,
                    l.id,
                    esc(&l.name),
                    content_svg(&l.content, settings, assets)
                );
            }
        }
    }
}

/// Start of a standalone SVG cropped to the visible content: the root element
/// (with `root_attrs` appended), the title and the background.
pub(crate) fn svg_head(doc: &Document, opts: &ExportOptions, root_attrs: &str) -> String {
    let g = opts.cell_px;
    let b = document_bounds(doc).unwrap_or(Rect { x: 0.0, y: 0.0, w: g, h: g }).inflate(opts.margin * g);
    let mut out = String::new();
    let _ = writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg"{root_attrs} viewBox="{} {} {} {}" width="{}" height="{}">"#,
        n(b.x),
        n(b.y),
        n(b.w),
        n(b.h),
        n(b.w),
        n(b.h)
    );
    let _ = writeln!(out, "  <title>Transit diagram</title>");
    if !opts.background.is_empty() {
        let _ = writeln!(
            out,
            r#"  <rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
            n(b.x),
            n(b.y),
            n(b.w),
            n(b.h),
            esc(&opts.background)
        );
    }
    out
}

/// `doc` with its settings rendered at `opts.cell_px` pixels per cell.
pub(crate) fn scaled(doc: &Document, opts: &ExportOptions) -> Document {
    Document { settings: DocSettings { cell_px: opts.cell_px, ..doc.settings.clone() }, ..doc.clone() }
}

/// Standalone SVG document of everything visible, cropped to the content,
/// with the default export options.
pub fn export_svg(doc: &Document) -> String {
    export_svg_with(doc, &ExportOptions::default())
}

/// Standalone SVG document of everything visible, with the given scale,
/// margin and background. The document is not changed.
pub fn export_svg_with(doc: &Document, opts: &ExportOptions) -> String {
    let opts = &opts.clone().sanitized();
    let doc = &scaled(doc, opts);
    let mut out = svg_head(doc, opts, "");
    // Embed the uploaded fonts that visible text actually uses.
    let mut used = Vec::new();
    tree::walk_layers(&doc.root, &mut |l, vis, _| {
        if let (true, Content::Text(t)) = (vis, &l.content) {
            for f in &doc.assets.fonts {
                if t.font_family.contains(&format!("\"{}\"", f.family)) && !used.iter().any(|u: &&FontAsset| u.family == f.family) {
                    used.push(f);
                }
            }
        }
    });
    if !used.is_empty() {
        let _ = writeln!(out, "  <defs><style>{}</style></defs>", crate::assets::font_face_css(&used));
    }
    export_nodes(&doc.root, &doc.settings, &doc.assets, &mut out, 1);
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
    fn hollow_and_inner_dashed_layers() {
        let pts = [Cell::new(0, 0), Cell::new(4, 0), Cell::new(6, 2)];
        let hollow = LineStyle { kind: StrokeKind::Hollow, ..LineStyle::default() };
        let s = line_svg(&pts, &hollow, &settings());
        parse(&s);
        assert_eq!(s.matches("<path").count(), 4);
        assert!(s.contains("#ffffff"));
        assert!(!s.contains("dasharray"));
        // A plain line (body and caps) with dashes of the inner color on top.
        let dashed = LineStyle { kind: StrokeKind::InnerDashed, ..LineStyle::default() };
        let s = line_svg(&pts, &dashed, &settings());
        parse(&s);
        assert_eq!(s.matches("<path").count(), 3);
        let inner = s.split("<path").nth(3).unwrap();
        assert!(inner.contains(r##"stroke="#ffffff""##) && inner.contains("stroke-dasharray"), "{s}");
        assert!(!s.contains("stroke-dashoffset"), "{s}");
    }

    #[test]
    fn dash_offsets_keep_the_line_ends_plain() {
        let s = settings();
        let style = LineStyle { kind: StrokeKind::InnerDashed, dash_start_offset: 2.0, ..LineStyle::default() };
        let m = line_svg(&[Cell::new(0, 0), Cell::new(10, 0)], &style, &s);
        parse(&m);
        assert!(m.contains(r#"stroke-dasharray="0 48 "#), "no inner dashes over the first two cells: {m}");
    }

    #[test]
    fn right_to_left_text_keeps_its_visual_alignment() {
        let t = TextBox { align: TextAlign::Start, direction: TextDirection::Rtl, ..TextBox::with_text("שלום") };
        let m = text_svg(&t, &settings());
        parse(&m);
        assert!(m.contains(r#"direction="rtl""#) && m.contains(r#"text-anchor="end""#), "{m}");
        let m = text_svg(&TextBox { align: TextAlign::Start, ..TextBox::default() }, &settings());
        assert!(!m.contains("direction") && m.contains(r#"text-anchor="start""#), "{m}");
    }

    /// (dash, gap) of the first dashed path in the markup.
    fn dash_pattern(s: &str) -> (f64, f64) {
        let start = s.find(r#"stroke-dasharray=""#).unwrap_or_else(|| panic!("no dashes: {s}")) + 18;
        let arr: Vec<f64> = s[start..].split('"').next().unwrap().split(' ').map(|v| v.parse().unwrap()).collect();
        (arr[0], arr[1])
    }

    #[test]
    fn inner_dashes_start_and_end_with_a_full_dash() {
        let s = settings();
        let style = LineStyle { kind: StrokeKind::InnerDashed, ..LineStyle::default() };
        // Straight line from cell 0 to cell 10: 240px between the centers.
        let m = line_svg(&[Cell::new(0, 0), Cell::new(10, 0)], &style, &s);
        parse(&m);
        let (dash, gap) = dash_pattern(&m);
        let count = ((240.0 + gap) / (dash + gap)).round();
        assert!((count * dash + (count - 1.0) * gap - 240.0).abs() < 0.01, "{dash} {gap}");
        assert!((dash - 24.0).abs() < 3.0 && (gap - 24.0).abs() < 3.0, "close to the requested 1 / 1 cells: {dash} {gap}");
    }

    #[test]
    fn inner_dashes_fit_around_rounded_corners() {
        let s = settings();
        let style = LineStyle { kind: StrokeKind::InnerDashed, dash_length: 0.7, dash_gap: 0.3, ..LineStyle::default() };
        let pts = [Cell::new(0, 0), Cell::new(6, 0), Cell::new(6, 6)];
        let m = line_svg(&pts, &style, &s);
        let (dash, gap) = dash_pattern(&m);
        let centers: Vec<Vec2> = pts.iter().map(|c| c.center().scale(24.0)).collect();
        let len = rounded_polyline_len(&centers, s.corner_radius * 24.0);
        let count = ((len + gap) / (dash + gap)).round();
        assert!((count * dash + (count - 1.0) * gap - len).abs() < 0.01);
    }

    #[test]
    fn degenerate_line_renders_nothing() {
        assert!(line_svg(&[Cell::new(0, 0)], &LineStyle::default(), &settings()).is_empty());
    }

    #[test]
    fn blob_skeleton_links_neighbors() {
        let d = blob_skeleton(&[Cell::new(0, 0), Cell::new(1, 0), Cell::new(1, 1), Cell::new(5, 5)], 10.0);
        assert_eq!(d.matches("h0.01").count(), 4);
        // (0,0)-(1,0) and (1,0)-(1,1); the diagonal (0,0)-(1,1) is bridged by
        // (1,0) so it must not be linked (that would chamfer the inside corner).
        assert_eq!(d.matches('L').count(), 2);
        assert!(!d.contains("M5 5L15 15"), "{d}");
    }

    #[test]
    fn blob_skeleton_links_pure_diagonals() {
        let d = blob_skeleton(&[Cell::new(0, 0), Cell::new(1, 1), Cell::new(2, 0)], 10.0);
        assert!(d.contains("M5 5L15 15"), "{d}");
        assert!(d.contains("M15 15L25 5"), "{d}");
    }

    #[test]
    fn blob_skeleton_fills_square_blocks() {
        let cells = [Cell::new(0, 0), Cell::new(1, 0), Cell::new(0, 1), Cell::new(1, 1)];
        let d = blob_skeleton(&cells, 10.0);
        assert_eq!(d.matches('Z').count(), 1, "one 2x2 block: {d}");
        assert!(d.contains("M5 5L15 5L15 15L5 15Z"), "{d}");
        assert!(!d.contains("M5 5L15 15"), "no diagonals inside a block");
        let m = station_svg(
            &Station::new(
                vec![],
                StationShape::Pill { cells: cells.to_vec(), outer_color: "#000".into(), inner_color: "#fff".into() },
            ),
            &DocSettings::default(),
        );
        assert!(m.contains(r##"fill="#fff""##), "the block interior is filled with the inner color: {m}");
    }

    #[test]
    fn station_shapes_render() {
        let s = settings();
        let dot = Station::new(
            vec![1],
            StationShape::Dot { cell: Cell::new(2, 2), outer_color: "#000".into(), inner_color: "#fff".into() },
        );
        let m = station_svg(&dot, &s);
        parse(&m);
        assert!(m.contains(r#"stroke-width="29.76""#), "outer diameter 2*0.62*24: {m}");
        let tick = Station::new(
            vec![1],
            StationShape::Tick {
                cell: Cell::new(0, 0),
                dir: Dir8::E,
                side: 1,
                along: 0,
                width: 0.5,
                length: 1.0,
                color: "#f00".into(),
            },
        );
        let m = station_svg(&tick, &s);
        parse(&m);
        let b = content_bounds(&Content::Station(tick), &s).unwrap();
        // Horizontal line, tick pokes up (left of east) from the border at y=0.
        assert!((b.y - (12.0 - 12.0 - 24.0)).abs() < 1e-9, "{b:?}");
        assert!((b.w - 12.0).abs() < 1e-9);
    }

    #[test]
    fn tick_in_corner_is_flush_with_grid_line() {
        let s = settings();
        let tick = |along| {
            Content::Station(Station::new(
                vec![1],
                StationShape::Tick { cell: Cell::new(1, 0), dir: Dir8::E, side: 1, along, width: 0.35, length: 0.55, color: "#000".into() },
            ))
        };
        // Cell 1 spans x = 24..48 px; the tick touches its left or right edge.
        let start = content_bounds(&tick(-1), &s).unwrap();
        assert!((start.x - 24.0).abs() < 1e-9, "{start:?}");
        let end = content_bounds(&tick(1), &s).unwrap();
        assert!((end.x + end.w - 48.0).abs() < 1e-9, "{end:?}");
    }

    #[test]
    fn station_offset_translates_markup_and_bounds() {
        let s = settings();
        let mut st = Station::new(
            vec![1],
            StationShape::Dot { cell: Cell::new(0, 0), outer_color: "#000".into(), inner_color: "#fff".into() },
        );
        let base = content_bounds(&Content::Station(st.clone()), &s).unwrap();
        st.offset = Vec2::new(0.5, -0.2);
        let m = station_svg(&st, &s);
        parse(&m);
        assert!(m.starts_with(r#"<g transform="translate(12 -4.8)">"#), "{m}");
        let b = content_bounds(&Content::Station(st), &s).unwrap();
        assert!((b.x - base.x - 12.0).abs() < 1e-9 && (b.y - base.y + 4.8).abs() < 1e-9);
    }

    #[test]
    fn text_renders_multiline_escaped() {
        let t = TextBox { rotation: 30.0, bold: true, ..TextBox::with_text("A & B\n<C>") };
        let m = text_svg(&t, &settings());
        parse(&m);
        assert_eq!(m.matches("<tspan").count(), 2);
        assert!(m.contains("A &amp; B"));
        assert!(m.contains("rotate(30 0 0)"), "rotates around the top-left corner: {m}");
        assert!(m.contains("bold"));
    }

    #[test]
    fn text_lines_are_placed_from_the_top_left_corner() {
        let mut t = TextBox { pos: Vec2::new(1.0, 2.0), align: TextAlign::Start, ..TextBox::with_text("Ab\nC") };
        t.width = 2.0;
        t.paragraphs[1].align = Some(TextAlign::End);
        let m = text_svg(&t, &settings());
        parse(&m);
        // First line: left edge, baseline 0.95 × 18 px below the top.
        assert!(m.contains(r#"x="24" y="65.1" text-anchor="start""#), "{m}");
        // Second line: right-aligned on its own, one line (1.2 × 18 px) lower.
        assert!(m.contains(r#"x="72" y="86.7" text-anchor="end""#), "{m}");
        let b = content_bounds(&Content::Text(t.clone()), &settings()).unwrap();
        assert_eq!((b.x, b.y, b.w), (24.0, 48.0, 48.0));
        assert!((b.h - 2.0 * 0.75 * 1.2 * 24.0).abs() < 1e-9);
    }

    #[test]
    fn text_runs_carry_their_own_formatting() {
        let mut t = TextBox::with_text("");
        t.paragraphs = vec![Paragraph {
            align: None,
            runs: vec![
                TextRun::plain("Big "),
                TextRun { text: "red".into(), style: RunStyle { color: Some("#ff0000".into()), size: Some(1.5), bold: Some(true), ..RunStyle::default() } },
            ],
        }];
        let m = text_svg(&t, &settings());
        parse(&m);
        assert!(m.contains(r##"Big <tspan font-size="36" fill="#ff0000" font-weight="bold">red</tspan>"##), "{m}");
        // The line is as tall as its largest run.
        assert!((t.height() - 1.5 * 1.2).abs() < 1e-9);
    }

    #[test]
    fn icon_renders_with_scale() {
        let i = Icon { pos: Vec2::new(1.0, 1.0), icon: "airport".into(), size: 1.0, ..Icon::default() };
        let m = icon_svg(&i, &settings(), &Assets::default());
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
