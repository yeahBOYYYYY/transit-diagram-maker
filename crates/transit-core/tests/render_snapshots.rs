//! Snapshots of the SVG markup the renderer produces. They replace screenshot
//! comparisons: the markup is what the canvas and the export both draw, it is
//! the same on every machine, and a change shows up as a readable diff.
//! Review changes with `cargo insta review` (or `INSTA_UPDATE=always cargo test`).
use transit_core::*;

fn c(x: i32, y: i32) -> Cell {
    Cell::new(x, y)
}

/// The same kind of map the end-to-end tests draw: two crossing lines, every
/// station kind, a label and a pictogram.
fn sample_map() -> Editor {
    let mut ed = Editor::new();
    ed.create_line(&[c(0, 0), c(6, 0), c(9, 3)], LineStyle { color: "#e32017".into(), ..LineStyle::default() }).unwrap();
    let blue = LineStyle { color: "#0098d4".into(), kind: StrokeKind::Hollow, inner_color: "#ffffff".into(), ..LineStyle::default() };
    ed.create_line(&[c(3, -4), c(3, 6)], blue).unwrap();
    ed.add_station(StationKind::Dot, c(1, 0), &StationOptions::default()).unwrap();
    ed.add_station(StationKind::Tick, c(3, -2), &StationOptions::default()).unwrap();
    ed.add_pill(&[c(3, 0), c(3, 1)], &StationOptions::default()).unwrap();
    ed.add_text(TextBox { pos: Vec2::new(4.0, -3.0), ..TextBox::with_text("Central") }).unwrap();
    ed.add_icon(Icon { icon: "ferry".into(), pos: Vec2::new(10.0, 5.0), ..Icon::default() }).unwrap();
    ed
}

fn line(style: LineStyle) -> String {
    render::line_svg(&[c(0, 0), c(4, 0), c(6, 2)], &style, &DocSettings::default())
}

#[test]
fn sample_map_export() {
    insta::assert_snapshot!(render::export_svg(sample_map().document()));
}

#[test]
fn sample_map_export_scaled_transparent() {
    let opts = ExportOptions { cell_px: 12.0, margin: 0.0, background: String::new() };
    insta::assert_snapshot!(render::export_svg_with(sample_map().document(), &opts));
}

#[test]
fn plain_line() {
    insta::assert_snapshot!(line(LineStyle::default()));
}

#[test]
fn butt_capped_line() {
    insta::assert_snapshot!(line(LineStyle { start_cap: Cap::Butt, end_cap: Cap::Butt, ..LineStyle::default() }));
}

#[test]
fn hollow_line() {
    insta::assert_snapshot!(line(LineStyle { kind: StrokeKind::Hollow, inner_color: "#ffffff".into(), ..LineStyle::default() }));
}

#[test]
fn inner_dashed_line_with_plain_ends() {
    insta::assert_snapshot!(line(LineStyle {
        kind: StrokeKind::InnerDashed,
        inner_color: "#ffffff".into(),
        dash_start_offset: 1.5,
        dash_end_offset: 0.5,
        ..LineStyle::default()
    }));
}

#[test]
fn every_station_kind() {
    let mut ed = Editor::new();
    ed.create_line(&[c(0, 0), c(8, 0), c(12, 4)], LineStyle::default()).unwrap();
    ed.add_station(StationKind::Dot, c(1, 0), &StationOptions::default()).unwrap();
    ed.add_station(StationKind::Tick, c(4, 0), &StationOptions::default()).unwrap();
    ed.add_station(StationKind::Tick, c(10, 2), &StationOptions { tick_side: -1, ..StationOptions::default() }).unwrap();
    ed.add_pill(&[c(6, 0), c(7, 0), c(6, 1)], &StationOptions::default()).unwrap();
    let stations: Vec<String> = render::render_fragments(ed.document()).into_iter().filter(|f| f.kind == "station").map(|f| f.svg).collect();
    insta::assert_snapshot!(stations.join("\n"));
}

#[test]
fn rich_and_right_to_left_text() {
    let mut t = TextBox { pos: Vec2::new(1.0, 1.0), rotation: 45.0, ..TextBox::with_text("") };
    t.paragraphs = vec![
        Paragraph {
            align: Some(TextAlign::End),
            runs: vec![
                TextRun::plain("Plain "),
                TextRun {
                    text: "styled <b> & \"quoted\"".into(),
                    style: RunStyle { font_family: Some("Serif".into()), size: Some(1.5), color: Some("#ff0000".into()), bold: Some(true), italic: Some(true) },
                },
            ],
        },
        Paragraph::plain("second line"),
    ];
    let rtl = TextBox { pos: Vec2::new(1.0, 5.0), direction: TextDirection::Rtl, align: TextAlign::Start, ..TextBox::with_text("תחנה מרכזית") };
    let s = DocSettings::default();
    insta::assert_snapshot!(format!("{}\n{}", render::text_svg(&t, &s), render::text_svg(&rtl, &s)));
}

#[test]
fn canvas_fragments_are_exactly_what_the_export_draws() {
    // The live canvas draws `render_fragments`; the export must contain the very
    // same markup for each of them, in the same order.
    let ed = sample_map();
    let export = render::export_svg(ed.document());
    let mut from = 0;
    for f in render::render_fragments(ed.document()) {
        let at = export[from..].find(&f.svg).unwrap_or_else(|| panic!("fragment {} ({}) missing from the export, or out of order", f.id, f.kind));
        from += at + f.svg.len();
    }
}
