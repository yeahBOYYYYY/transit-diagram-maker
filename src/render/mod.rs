use crate::coord::Aabb;
use crate::graph::{
    generate_transit_curve_segments, DocumentNode, DocumentTree, LineCap, NodeData,
    PathSegment, StationStyle, StrokeStyle,
};

/// Export the document to an optimized standalone SVG string
pub fn export_svg(tree: &DocumentTree) -> String {
    let mut doc_bounds: Option<Aabb> = None;
    for &id in &tree.node_order {
        if let Some(node) = tree.nodes.get(&id) {
            if node.visible {
                let box_node = node.aabb();
                doc_bounds = Some(match doc_bounds {
                    Some(b) => b.union(&box_node),
                    None => box_node,
                });
            }
        }
    }

    let bounds = doc_bounds
        .unwrap_or_else(|| Aabb::new(crate::coord::DVec2::new(-400.0, -300.0), crate::coord::DVec2::new(400.0, 300.0)))
        .expand_by_margin(40.0);

    let min_x = bounds.min.x.round();
    let min_y = bounds.min.y.round();
    let width = bounds.width().round();
    let height = bounds.height().round();

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}" style="background: #151518; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
"#,
        min_x, min_y, width, height, width, height
    );

    let mut sorted_ids = tree.node_order.clone();
    sorted_ids.sort_by_key(|&id| tree.nodes.get(&id).map(|n| n.z_index).unwrap_or(0));

    // 1. Render Transit Lines
    for &id in &sorted_ids {
        if let Some(DocumentNode { data: NodeData::TransitLine { points, corner_radius, color, stroke_width, style, cap, hidden_segments, .. }, visible: true, .. }) = tree.nodes.get(&id) {
            let segments = generate_transit_curve_segments(points, *corner_radius, hidden_segments);
            if segments.is_empty() {
                continue;
            }

            let mut d = String::new();
            for seg in &segments {
                match seg {
                    PathSegment::MoveTo(p) => d.push_str(&format!("M {:.1} {:.1} ", p.x, p.y)),
                    PathSegment::LineTo(p) => d.push_str(&format!("L {:.1} {:.1} ", p.x, p.y)),
                    PathSegment::QuadTo { ctrl, to } => d.push_str(&format!("Q {:.1} {:.1} {:.1} {:.1} ", ctrl.x, ctrl.y, to.x, to.y)),
                    PathSegment::Close => d.push_str("Z "),
                }
            }

            let cap_str = match cap {
                LineCap::Round => "round",
                LineCap::Square => "square",
                LineCap::Butt => "butt",
            };

            match style {
                StrokeStyle::Solid => {
                    svg.push_str(&format!(
                        r#"  <path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="round" />
"#,
                        d, color, stroke_width, cap_str
                    ));
                }
                StrokeStyle::Hollow { inner_width, inner_color } => {
                    svg.push_str(&format!(
                        r#"  <path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="round" />
  <path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="round" />
"#,
                        d, color, stroke_width, cap_str,
                        d, inner_color, inner_width, cap_str
                    ));
                }
                StrokeStyle::Dashed { dash_len, gap_len } => {
                    svg.push_str(&format!(
                        r#"  <path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="round" stroke-dasharray="{},{}" />
"#,
                        d, color, stroke_width, cap_str, dash_len, gap_len
                    ));
                }
            }
        }
    }

    // 2. Render Stations (Dot or Tick only; Dot can stretch as a slinky capsule)
    for &id in &sorted_ids {
        if let Some(DocumentNode { data: NodeData::Station { pos, radius, style, fill_color, stroke_color, stroke_width, rotation_rad, span_cells, span_angle_deg, .. }, visible: true, .. }) = tree.nodes.get(&id) {
            match style {
                StationStyle::Dot => {
                    if *span_cells == 0 {
                        svg.push_str(&format!(
                            r#"  <circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" stroke="{}" stroke-width="{}" />
"#,
                            pos.x, pos.y, radius, fill_color, stroke_color, stroke_width
                        ));
                    } else {
                        // Stretched dot station: rectangular body with semi-circle caps at the ends
                        let angle_rad = span_angle_deg.to_radians();
                        let dir = crate::coord::DVec2::new(angle_rad.cos(), angle_rad.sin());
                        let normal = crate::coord::DVec2::new(-dir.y, dir.x);
                        let end = *pos + dir * (*span_cells as f64 * 20.0);

                        let p1_top = *pos + normal * *radius;
                        let p2_top = end + normal * *radius;
                        let p2_bot = end - normal * *radius;
                        let p1_bot = *pos - normal * *radius;

                        let path_d = format!(
                            "M {:.1} {:.1} L {:.1} {:.1} A {:.1} {:.1} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {:.1} {:.1} 0 0 1 {:.1} {:.1} Z",
                            p1_top.x, p1_top.y,
                            p2_top.x, p2_top.y,
                            radius, radius, p2_bot.x, p2_bot.y,
                            p1_bot.x, p1_bot.y,
                            radius, radius, p1_top.x, p1_top.y
                        );

                        svg.push_str(&format!(
                            r#"  <path d="{}" fill="{}" stroke="{}" stroke-width="{}" />
"#,
                            path_d, fill_color, stroke_color, stroke_width
                        ));
                    }
                }
                StationStyle::Tick => {
                    let dir = crate::coord::DVec2::new(rotation_rad.cos(), rotation_rad.sin());
                    let p1 = *pos - dir * *radius;
                    let p2 = *pos + dir * *radius;
                    svg.push_str(&format!(
                        r#"  <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{}" stroke-linecap="round" />
"#,
                        p1.x, p1.y, p2.x, p2.y, stroke_color, stroke_width
                    ));
                }
            }
        }
    }

    // 3. Render Text Labels
    for &id in &sorted_ids {
        if let Some(DocumentNode { data: NodeData::Text { text, pos, font_size, color, .. }, visible: true, .. }) = tree.nodes.get(&id) {
            svg.push_str(&format!(
                r#"  <text x="{:.1}" y="{:.1}" fill="{}" font-size="{:.1}" font-weight="600" dominant-baseline="hanging">{}</text>
"#,
                pos.x, pos.y, color, font_size, html_escape(text)
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
