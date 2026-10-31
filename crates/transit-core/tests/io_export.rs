use transit_core::*;

fn c(x: i32, y: i32) -> Cell {
    Cell::new(x, y)
}

fn sample() -> Editor {
    let mut ed = Editor::new();
    let red = ed.create_line(&[c(0, 0), c(6, 0), c(9, 3)], LineStyle::default()).unwrap();
    let blue = LineStyle {
        color: "#0098d4".into(),
        kind: StrokeKind::InnerDashed,
        start_cap: Cap::Butt,
        dash_start_offset: 1.5,
        dash_end_offset: 0.5,
        ..LineStyle::default()
    };
    ed.create_line(&[c(3, -4), c(3, 6)], blue).unwrap();
    ed.erase(&[c(3, 3)]).unwrap();
    ed.add_station(StationKind::Dot, c(1, 0), &StationOptions::default()).unwrap();
    ed.add_station(StationKind::Tick, c(3, -2), &StationOptions::default()).unwrap();
    ed.add_pill(&[c(3, 0), c(3, 1)], &StationOptions::default()).unwrap();
    ed.add_text(TextBox { rotation: 45.0, ..TextBox::with_text("Metro \"Map\" & <Co>") }).unwrap();
    ed.add_text(TextBox { direction: TextDirection::Rtl, ..TextBox::with_text("תחנה מרכזית") }).unwrap();
    ed.add_icon(Icon { icon: "ferry".into(), pos: Vec2::new(10.0, 5.0), ..Icon::default() }).unwrap();
    let mut dashed = Shape::line(Vec2::new(-2.0, 8.0), Vec2::new(4.0, 8.0));
    if let ShapeGeometry::Line { cap, dashed: d, .. } = &mut dashed.geometry {
        *cap = Cap::Butt;
        *d = true;
    }
    ed.add_shape(dashed).unwrap();
    let mut rect = Shape::rect(Vec2::new(5.0, 7.0), 3.0, 2.0);
    rect.filled = false;
    if let ShapeGeometry::Rect { corner_radius, .. } = &mut rect.geometry {
        *corner_radius = 0.5;
    }
    ed.add_shape(rect).unwrap();
    let circle = ed.add_shape(Shape::circle(Vec2::new(12.5, 0.5), 1.5)).unwrap();
    ed.rotate(&[circle], 30.0, Vec2::new(0.0, 0.0), true).unwrap();
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
fn saved_file_is_an_svg_picture_with_editor_data() {
    let ed = sample();
    let text = io::to_transit(ed.document());
    let xml = roxmltree::Document::parse(&text).expect("well-formed xml");
    let root = xml.root_element();
    assert_eq!(root.tag_name().name(), "svg");
    assert_eq!(root.tag_name().namespace(), Some("http://www.w3.org/2000/svg"));
    let data = |name: &str| xml.descendants().filter(|n| n.tag_name().namespace() == Some(io::NS) && n.tag_name().name() == name).count();
    assert_eq!(data("document"), 1);
    assert_eq!(data("settings"), 1);
    assert_eq!(data("segment"), 3, "the blue line was erased into two segments");
    assert_eq!(data("station"), 3);
    assert_eq!(data("shape"), 3);
    // The picture is the regular export: same shapes as the SVG export.
    assert!(text.contains("stroke-dasharray"));
    assert!(text.contains(r##"fill="#ffffff""##), "background rect");
    let seg = xml.descendants().find(|n| n.tag_name().name() == "segment").unwrap();
    assert_eq!(seg.attribute("points"), Some("0,0 6,0 9,3"));
    assert_eq!(seg.attribute("kind"), Some("plain"));
    assert_eq!(seg.parent().unwrap().attribute("id").map(|i| i.starts_with("layer-")), Some(true));
}

#[test]
fn hidden_layers_are_saved_but_not_painted() {
    let mut ed = Editor::new();
    let t = ed.add_text(TextBox::with_text("Hidden")).unwrap();
    ed.set_visible(t, false).unwrap();
    let text = io::to_transit(ed.document());
    assert!(text.contains(&format!(r#"<g id="layer-{t}" data-name="Hidden" display="none">"#)), "{text}");
    assert!(!io::from_transit(&text).unwrap().root.iter().any(|n| n.visible()));
}

#[test]
fn multi_line_text_and_special_characters_survive() {
    let mut ed = Editor::new();
    let t = TextBox { font_family: "A \"B\" & C".into(), ..TextBox::with_text("  Line one\nLine \"two\" & <three>\r\n\tend ") };
    ed.add_text(t).unwrap();
    let back = io::from_transit(&io::to_transit(ed.document())).unwrap();
    assert_eq!(&back, ed.document());
}

#[test]
fn rich_text_survives() {
    let mut ed = Editor::new();
    let mut t = TextBox { width: 4.25, ..TextBox::with_text("") };
    t.paragraphs = vec![
        Paragraph {
            align: Some(TextAlign::End),
            runs: vec![
                TextRun::plain("Plain "),
                TextRun {
                    text: "styled <b>".into(),
                    style: RunStyle { font_family: Some("Serif".into()), size: Some(1.5), color: Some("#ff0000".into()), bold: Some(true), italic: Some(true) },
                },
            ],
        },
        Paragraph::default(),
        Paragraph::plain(" last "),
    ];
    let id = ed.add_text(t).unwrap();
    ed.fit_text_widths(&[(id, 4.25)]);
    let saved = io::to_transit(ed.document());
    assert!(saved.contains(r#"<transit:p align="end"><transit:run>Plain </transit:run>"#), "{saved}");
    let back = io::from_transit(&saved).unwrap();
    assert_eq!(&back, ed.document());
}

#[test]
fn rejects_foreign_or_future_files() {
    assert!(io::from_transit("not xml").is_err());
    assert!(io::from_transit("<html/>").unwrap_err().to_string().contains("not an SVG"));
    let plain = render::export_svg(sample().document());
    assert!(io::from_transit(&plain).unwrap_err().to_string().contains("no Transit Diagram Maker data"), "a plain export cannot be opened");
    let future = io::to_transit(&Document::default()).replace(r#"version="1""#, r#"version="99""#);
    assert!(io::from_transit(&future).unwrap_err().to_string().contains("newer"));
}

fn file_with(settings: &str, body: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:transit="{}"><metadata><transit:document version="1" next-id="1">{settings}</transit:document></metadata>{body}</svg>"#,
        io::NS
    )
}

fn file(body: &str) -> String {
    file_with("", body)
}

#[test]
fn rejects_duplicate_ids_and_bad_values() {
    let layer = r#"<g><transit:text id="5" name="a">x</transit:text></g>"#;
    let err = io::from_transit(&file(&format!("{layer}{layer}"))).unwrap_err();
    assert!(err.to_string().contains("two layers share the identifier 5"));
    let bad = io::from_transit(&file(r#"<g><transit:icon id="1" size="big"/></g>"#)).unwrap_err();
    assert!(bad.to_string().contains(r#"invalid size "big""#), "{bad}");
    assert!(io::from_transit(&file(r#"<g><transit:segment id="1" points="0,0 1,0"/></g>"#)).unwrap_err().to_string().contains("missing attribute line"));
    assert!(io::from_transit(&file(r#"<g><transit:station id="1" shape="star" cell="0,0"/></g>"#)).is_err());
}

#[test]
fn repairs_id_counter_settings_and_bad_lines() {
    let text = file_with(
        r#"<transit:settings line-width="0.8" tick-width="-3"/>"#,
        r#"<g><transit:segment id="40" name="a" line="41" points="0,0 3,1"/></g>
           <g><transit:segment id="42" name="b" line="43" points="0,0"/></g>
           <g><transit:text id="44" pos="1.23456,2" rotation="50.04"><transit:p><transit:run>t</transit:run></transit:p></transit:text></g>
           <rect width="5" height="5"/><g><path d="M0 0"/></g>"#,
    );
    let doc = io::from_transit(&text).unwrap();
    assert_eq!(doc.next_id, 45);
    assert_eq!(doc.settings.line_width, 0.8);
    assert_eq!(doc.settings.tick_width, 0.01, "out-of-range settings are clamped");
    assert_eq!(doc.root.len(), 2, "single-point line dropped, foreign markup ignored");
    let Node::Layer(l) = &doc.root[0] else { panic!() };
    let Content::Line(seg) = &l.content else { panic!() };
    assert_eq!(seg.points, vec![c(0, 0), c(2, 0), c(3, 1)], "unaligned points are routed");
    assert_eq!(seg.style, LineStyle::default(), "missing style attributes get defaults");
    let Node::Layer(t) = &doc.root[1] else { panic!() };
    let Content::Text(t) = &t.content else { panic!() };
    assert_eq!((t.pos, t.rotation, t.text()), (Vec2::new(1.235, 2.0), 50.0, "t".to_string()));
    assert!(t.width > 0.0, "a missing width is estimated");
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
    let opts = ExportOptions { background: String::new(), margin: 2.0, ..ExportOptions::default() };
    let svg = render::export_svg_with(ed.document(), &opts);
    assert!(!svg.contains("<rect"));
    assert!(svg.contains(r#"viewBox="-48 -48 216 120""#), "{svg}");
}

#[test]
fn export_of_empty_document_is_valid() {
    let svg = render::export_svg(&Document::default());
    roxmltree::Document::parse(&svg).unwrap();
}

#[test]
fn export_options_scale_the_picture_only() {
    let mut ed = Editor::new();
    ed.create_line(&[c(0, 0), c(4, 0)], LineStyle::default()).unwrap();
    let normal = render::export_svg(ed.document());
    assert!(normal.contains(r#"width="168" height="72""#), "{normal}");
    let big = render::export_svg_with(ed.document(), &ExportOptions { cell_px: 48.0, ..ExportOptions::default() });
    assert!(big.contains(r#"width="336" height="144""#), "twice the pixels per cell: {big}");
    assert!(big.contains(r#"stroke-width="48""#), "one-cell line at 48px per cell: {big}");
    assert_eq!(render::export_svg(ed.document()), normal, "the document is unchanged");
    assert!(!render::export_svg(ed.document()).contains("transit:"), "an export has no editor data");
}

#[test]
fn cell_size_is_not_saved() {
    let ed = Editor::new();
    assert!(!io::to_transit(ed.document()).contains("cell-px"));
    assert!(!io::to_transit(ed.document()).contains("export"));
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

const FONT: &str = "data:font/woff2;base64,d09GMgABAAAAAA==";
const STAR: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M5 0L10 10H0Z" fill="currentColor"/><script>alert(1)</script></svg>"#;

#[test]
fn uploaded_assets_are_saved_in_the_file_and_exported() {
    let mut ed = Editor::new();
    let family = ed.add_font("City Sans", FONT).unwrap();
    assert_eq!(family, "\"City Sans\", sans-serif");
    let name = ed.add_icon_asset("Star", STAR).unwrap();
    assert_eq!(name, "custom-1");
    ed.add_text(TextBox { font_family: family.clone(), ..TextBox::with_text("Hi") }).unwrap();
    let icon = ed.add_icon(Icon { icon: name.clone(), color: "#ff0000".into(), ..Icon::default() }).unwrap();
    assert_eq!(ed.layer(icon).unwrap().name, "Star");

    let text = io::to_transit(ed.document());
    let back = io::from_transit(&text).unwrap();
    assert_eq!(&back, ed.document(), "assets survive a save and load");

    let svg = render::export_svg(ed.document());
    roxmltree::Document::parse(&svg).unwrap();
    assert!(svg.contains(r#"@font-face{font-family:"City Sans";src:url(data:font/woff2;base64,"#), "{svg}");
    assert!(svg.contains(r##"viewBox="0 0 10 10" color="#ff0000""##), "{svg}");
    assert!(svg.contains("M5 0L10 10H0Z"));
    assert!(!svg.contains("script"));
}

#[test]
fn unused_fonts_are_not_embedded_in_exports() {
    let mut ed = Editor::new();
    ed.add_font("Unused", FONT).unwrap();
    ed.add_text(TextBox::default()).unwrap();
    assert!(!render::export_svg(ed.document()).contains("@font-face"));
}

#[test]
fn assets_can_be_removed_only_when_unused_and_changes_are_undoable() {
    let mut ed = Editor::new();
    let family = ed.add_font("Body", FONT).unwrap();
    let name = ed.add_icon_asset("", STAR).unwrap();
    assert_eq!(ed.assets().icons[0].label, "Custom icon");
    let t = ed.add_text(TextBox { font_family: family, ..TextBox::default() }).unwrap();
    let i = ed.add_icon(Icon { icon: name.clone(), ..Icon::default() }).unwrap();
    assert!(ed.remove_font("Body").unwrap_err().to_string().contains("1 text layer"));
    assert!(ed.remove_icon_asset(&name).unwrap_err().to_string().contains("1 pictogram"));
    ed.delete(&[t, i]).unwrap();
    ed.remove_font("Body").unwrap();
    ed.remove_icon_asset(&name).unwrap();
    assert!(ed.assets().fonts.is_empty() && ed.assets().icons.is_empty());
    ed.undo();
    assert_eq!(ed.assets().icons.len(), 1);
    assert!(ed.add_font("Bad", "data:text/html;base64,AAAA").is_err());
    assert!(ed.add_icon_asset("x", "<html/>").is_err());
}
