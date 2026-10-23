use transit_core::*;

fn c(x: i32, y: i32) -> Cell {
    Cell::new(x, y)
}

fn sample() -> Editor {
    let mut ed = Editor::new();
    let red = ed.create_line(&[c(0, 0), c(6, 0), c(9, 3)], LineStyle::default()).unwrap();
    let blue = LineStyle {
        color: "#0098d4".into(),
        kind: StrokeKind::Dashed,
        start_cap: Cap::Butt,
        ..LineStyle::default()
    };
    ed.create_line(&[c(3, -4), c(3, 6)], blue).unwrap();
    ed.erase(&[c(3, 3)]).unwrap();
    ed.add_station(StationKind::Dot, c(1, 0), &StationOptions::default()).unwrap();
    ed.add_station(StationKind::Tick, c(3, -2), &StationOptions::default()).unwrap();
    ed.add_pill(&[c(3, 0), c(3, 1)], &StationOptions::default()).unwrap();
    ed.add_text(TextBox { text: "Metro \"Map\" & <Co>".into(), rotation: 45.0, ..TextBox::default() }).unwrap();
    ed.add_icon(Icon { icon: "ferry".into(), pos: Vec2::new(10.0, 5.0), ..Icon::default() }).unwrap();
    let g = ed.new_group(Some("Empty folder")).unwrap();
    ed.set_locked(g, true).unwrap();
    ed.set_visible(red.group_id, true).unwrap();
    ed
}

#[test]
fn transit_roundtrip_is_lossless() {
    let ed = sample();
    let text = io::to_transit(ed.document());
    let back = io::from_transit(&text).unwrap();
    assert_eq!(&back, ed.document());
    // And it is stable: saving again yields the same bytes.
    assert_eq!(io::to_transit(&back), text);
}

#[test]
fn transit_has_envelope() {
    let text = io::to_transit(sample().document());
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["format"], "transit-diagram");
    assert_eq!(v["version"], 1);
    assert!(v["document"]["root"].is_array());
}

#[test]
fn rejects_foreign_or_future_files() {
    assert!(io::from_transit("not json").is_err());
    assert!(io::from_transit(r#"{"format":"other","version":1,"document":{}}"#).is_err());
    assert!(io::from_transit(r#"{"format":"transit-diagram","version":99,"document":{}}"#).is_err());
}

#[test]
fn rejects_duplicate_ids() {
    let layer = r#"{"type":"layer","id":5,"name":"a","content":{"kind":"text","text":"x"}}"#;
    let text = format!(r#"{{"format":"transit-diagram","version":1,"document":{{"root":[{layer},{layer}]}}}}"#);
    let err = io::from_transit(&text).unwrap_err();
    assert!(err.to_string().contains("duplicate id 5"));
}

#[test]
fn repairs_id_counter_settings_and_bad_lines() {
    let text = r#"{
        "format": "transit-diagram", "version": 1,
        "document": {
            "next_id": 1,
            "settings": {"grid_size": -4, "line_width": 0.8},
            "root": [
                {"type":"layer","id":40,"name":"a","content":{"kind":"line","line_id":41,
                  "points":[{"x":0,"y":0},{"x":3,"y":1}],"style":{}}},
                {"type":"layer","id":42,"name":"b","content":{"kind":"line","line_id":43,
                  "points":[{"x":0,"y":0}],"style":{}}}
            ]
        }
    }"#;
    let doc = io::from_transit(text).unwrap();
    assert_eq!(doc.next_id, 44);
    assert_eq!(doc.settings.cell_px, CELL_PX, "a stored cell size from older files is ignored");
    assert_eq!(doc.settings.line_width, 0.8);
    assert_eq!(doc.root.len(), 1, "single-point line dropped");
    let Node::Layer(l) = &doc.root[0] else { panic!() };
    let Content::Line(seg) = &l.content else { panic!() };
    assert_eq!(seg.points, vec![c(0, 0), c(2, 0), c(3, 1)], "unaligned points are routed");
    assert_eq!(seg.style, LineStyle::default(), "missing style fields get defaults");
}

#[test]
fn loaded_document_continues_ids() {
    let ed = sample();
    let doc = io::from_transit(&io::to_transit(ed.document())).unwrap();
    let max = doc.next_id;
    let mut ed2 = Editor::from_document(doc);
    let id = ed2.add_text(TextBox::default()).unwrap();
    assert!(id >= max);
}

#[test]
fn svg_export_is_valid_and_complete() {
    let ed = sample();
    let svg = render::export_svg(ed.document());
    let doc = roxmltree::Document::parse(&svg).expect("well-formed xml");
    let root = doc.root_element();
    assert_eq!(root.tag_name().name(), "svg");
    assert_eq!(root.tag_name().namespace(), Some("http://www.w3.org/2000/svg"));
    let vb: Vec<f64> = root.attribute("viewBox").unwrap().split(' ').map(|v| v.parse().unwrap()).collect();
    assert!(vb[2] > 0.0 && vb[3] > 0.0);
    // Every leaf layer that is visible appears once.
    let layer_groups = doc.descendants().filter(|n| n.attribute("id").is_some_and(|i| i.starts_with("layer-"))).count();
    let mut visible = 0;
    tree::walk_layers(&ed.document().root, &mut |_, v, _| visible += v as usize);
    assert_eq!(layer_groups, visible);
    assert!(svg.contains("Metro &quot;Map&quot; &amp; &lt;Co&gt;") || svg.contains("Metro \"Map\" &amp; &lt;Co&gt;"));
    assert!(svg.contains("stroke-dasharray"), "dashed line is exported");
    assert!(svg.contains(r##"fill="#ffffff""##), "background rect");
}

#[test]
fn svg_export_preserves_group_structure() {
    let ed = sample();
    let svg = render::export_svg(ed.document());
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let line_group = doc.descendants().find(|n| n.attribute("data-name") == Some("Line 1")).unwrap();
    assert!(line_group.attribute("id").unwrap().starts_with("group-"));
    let nested = line_group.descendants().filter(|n| n.attribute("data-name") == Some("Station 1")).count();
    assert!(nested >= 1, "station group nested inside line group");
}

#[test]
fn svg_export_transparent_background_and_margin() {
    let mut ed = Editor::new();
    ed.create_line(&[c(0, 0), c(4, 0)], LineStyle::default()).unwrap();
    let s = DocSettings { export_background: String::new(), export_margin: 2.0, ..DocSettings::default() };
    ed.set_settings(s).unwrap();
    let svg = render::export_svg(ed.document());
    assert!(!svg.contains("<rect"));
    assert!(svg.contains(r#"viewBox="-48 -48 216 120""#), "{svg}");
}

#[test]
fn export_of_empty_document_is_valid() {
    let svg = render::export_svg(&Document::default());
    roxmltree::Document::parse(&svg).unwrap();
}

#[test]
fn export_with_one_off_settings_scales_and_restyles() {
    let mut ed = Editor::new();
    ed.create_line(&[c(0, 0), c(4, 0)], LineStyle::default()).unwrap();
    let normal = render::export_svg(ed.document());
    assert!(normal.contains(r#"width="168" height="72""#), "{normal}");
    let custom = DocSettings { cell_px: 48.0, export_background: String::new(), ..ed.settings().clone() };
    let big = render::export_svg_with(ed.document(), &custom);
    assert!(big.contains(r#"width="336" height="144""#), "twice the pixels per cell: {big}");
    assert!(!big.contains("<rect"), "transparent background");
    let thin = render::export_svg_with(ed.document(), &DocSettings { line_width: 0.5, ..custom });
    assert!(thin.contains(r#"stroke-width="24""#), "half-cell line at 48px per cell: {thin}");
    assert_eq!(render::export_svg(ed.document()), normal, "the document is unchanged");
}

#[test]
fn cell_size_is_not_saved() {
    let ed = Editor::new();
    assert!(!io::to_transit(ed.document()).contains("cell_px"));
    assert!(!io::to_transit(ed.document()).contains("grid_size"));
}

#[test]
fn fragments_follow_paint_order() {
    let mut ed = Editor::new();
    let a = ed.add_text(TextBox::default()).unwrap();
    let b = ed.add_text(TextBox::default()).unwrap();
    let ids: Vec<_> = render::render_fragments(ed.document()).iter().map(|f| f.id).collect();
    assert_eq!(ids, vec![a, b], "later = painted on top");
    ed.restack(&[a], 1).unwrap();
    let ids: Vec<_> = render::render_fragments(ed.document()).iter().map(|f| f.id).collect();
    assert_eq!(ids, vec![b, a]);
}
