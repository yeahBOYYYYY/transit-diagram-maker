//! Property tests: invariants that must hold for any input, not just the
//! hand-picked cases of the other tests. proptest shrinks a failing input to a
//! minimal one and stores it in `proptest-regressions/` so it is replayed.
use proptest::prelude::*;
use transit_core::geom::{self, Dir8};
use transit_core::*;

fn cell() -> impl Strategy<Value = Cell> {
    (-20i32..20, -20i32..20).prop_map(|(x, y)| Cell::new(x, y))
}

fn route_mode() -> impl Strategy<Value = RouteMode> {
    prop_oneof![Just(RouteMode::StraightFirst), Just(RouteMode::DiagonalFirst)]
}

#[derive(Debug, Clone)]
enum Op {
    Line(Vec<Cell>),
    Station(StationKind, Cell),
    Erase(Vec<Cell>),
    Move(f64, f64),
    Rotate(f64),
    Undo,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => prop::collection::vec(cell(), 2..5).prop_map(Op::Line),
        2 => (prop_oneof![Just(StationKind::Dot), Just(StationKind::Tick)], cell()).prop_map(|(k, c)| Op::Station(k, c)),
        1 => prop::collection::vec(cell(), 1..4).prop_map(Op::Erase),
        1 => (-5i32..5, -5i32..5).prop_map(|(x, y)| Op::Move(x as f64, y as f64)),
        1 => (-8i32..8).prop_map(|n| Op::Rotate(n as f64 * 45.0)),
        1 => Just(Op::Undo),
    ]
}

/// Runs the operations, ignoring the ones the editor refuses (a station off any line, ...).
fn run(ops: &[Op]) -> Editor {
    let mut ed = Editor::new();
    for op in ops {
        let all: Vec<Id> = ed.document().root.iter().map(|n| n.id()).collect();
        let _ = match op {
            Op::Line(pts) => ed.create_line(pts, LineStyle::default()).map(drop),
            Op::Station(kind, c) => ed.add_station(*kind, *c, &StationOptions::default()).map(drop),
            Op::Erase(cells) => ed.erase(cells).map(drop),
            Op::Move(dx, dy) => ed.translate(&all, *dx, *dy, false),
            Op::Rotate(deg) => ed.rotate(&all, *deg, Vec2::new(0.0, 0.0), false).map(drop),
            Op::Undo => {
                ed.undo();
                Ok(())
            }
        };
    }
    ed
}

proptest! {
    #[test]
    fn normalized_paths_are_valid_polylines(points in prop::collection::vec(cell(), 1..8), mode in route_mode()) {
        let path = geom::normalize_path(&points, mode);
        for w in path.windows(2) {
            prop_assert!(Dir8::between(w[0], w[1]).is_some(), "{:?} → {:?} is not 8-way aligned", w[0], w[1]);
        }
        for w in path.windows(3) {
            prop_assert_ne!(Dir8::between(w[0], w[1]), Dir8::between(w[1], w[2]), "redundant vertex {:?}", w[1]);
        }
        // Every point the user clicked is on the line.
        let cells = geom::expand(&path);
        for p in &points {
            prop_assert!(cells.contains(p), "{:?} is not on {:?}", p, path);
        }
        prop_assert_eq!(geom::normalize_path(&path, mode), path.clone(), "normalizing is idempotent");
    }

    #[test]
    fn expand_steps_one_cell_at_a_time(points in prop::collection::vec(cell(), 1..8)) {
        let path = geom::normalize_path(&points, RouteMode::StraightFirst);
        let cells = geom::expand(&path);
        for w in cells.windows(2) {
            prop_assert!((w[0].x - w[1].x).abs() <= 1 && (w[0].y - w[1].y).abs() <= 1 && w[0] != w[1]);
        }
        prop_assert_eq!(geom::compress(&cells), path);
    }

    #[test]
    fn cells_between_has_no_gaps(a in cell(), b in cell()) {
        let cells = geom::cells_between(a, b);
        prop_assert_eq!(cells.first(), Some(&a));
        prop_assert_eq!(cells.last(), Some(&b));
        prop_assert!(geom::is_connected(&cells));
    }

    #[test]
    fn fitted_dashes_fill_the_run(len in 1.0f64..500.0, dash in 0.5f64..20.0, gap in 0.0f64..20.0) {
        let (d, g) = geom::fit_dashes(len, dash, gap);
        if len > dash {
            let n = ((len + g) / (d + g)).round();
            prop_assert!((n * d + (n - 1.0) * g - len).abs() < 1e-6);
            if gap > 0.0 {
                prop_assert!((d / g - dash / gap).abs() < 1e-6, "proportions are kept");
            }
        }
    }

    #[test]
    fn dash_array_leaves_the_ends_plain(len in 10.0f64..500.0, lead in 0.0f64..5.0, trail in 0.0f64..5.0, dash in 0.5f64..10.0, gap in 0.5f64..10.0) {
        let Some(v) = geom::dash_array(len, lead, trail, dash, gap) else { return Ok(()) };
        prop_assert!(v.iter().all(|x| *x >= 0.0));
        if lead > 0.0 || trail > 0.0 {
            // Everything but the closing gap covers the run up to the plain end.
            let covered: f64 = v[..v.len() - 1].iter().sum();
            prop_assert!((covered - (len - trail)).abs() < 1e-6, "{covered} vs {}", len - trail);
        }
    }

    #[test]
    fn rotations_are_normalized(a in -10_000.0f64..10_000.0) {
        let s = editor::snap_rotation(a);
        prop_assert!(s > -180.0 && s <= 180.0);
        prop_assert_eq!(s % 45.0, 0.0);
        let f = editor::free_rotation(a);
        prop_assert!(f > -180.0 && f <= 180.0);
        prop_assert_eq!(editor::free_rotation(f), f);
    }

    #[test]
    fn tenth_snapping_is_idempotent(v in -1_000.0f64..1_000.0) {
        let s = geom::snap_tenth(v);
        prop_assert!((s - v).abs() <= 0.05 + 1e-9);
        prop_assert_eq!(geom::snap_tenth(s), s);
    }

    #[test]
    fn any_document_survives_saving_and_loading(ops in prop::collection::vec(op(), 1..12)) {
        let ed = run(&ops);
        let saved = io::to_transit(ed.document());
        let back = io::from_transit(&saved).unwrap();
        prop_assert_eq!(&back, ed.document());
        prop_assert_eq!(io::to_transit(&back), saved);
    }

    #[test]
    fn undo_all_then_redo_all_restores_every_state(ops in prop::collection::vec(op(), 1..12)) {
        // Without undos in between, so the redo stack ends at the last state.
        let ops: Vec<Op> = ops.into_iter().filter(|o| !matches!(o, Op::Undo)).collect();
        let mut ed = run(&ops);
        let end = ed.document().clone();
        while ed.undo() {}
        let empty = Editor::new();
        prop_assert_eq!(ed.document(), empty.document());
        while ed.redo() {}
        prop_assert_eq!(ed.document(), &end);
    }

    #[test]
    fn every_fragment_parses_as_svg(ops in prop::collection::vec(op(), 1..12)) {
        let ed = run(&ops);
        for f in render::render_fragments(ed.document()) {
            let doc = format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{}</svg>"#, f.svg);
            prop_assert!(roxmltree::Document::parse(&doc).is_ok(), "{}", f.svg);
            prop_assert!(!f.svg.contains("NaN") && !f.svg.contains("inf"), "{}", f.svg);
        }
    }

    #[test]
    fn sanitized_svg_never_keeps_active_content(
        elements in prop::collection::vec((
            prop::sample::select(vec!["path", "rect", "script", "foreignObject", "a", "use", "style", "g", "image", "circle"]),
            prop::sample::select(vec!["fill", "onclick", "onload", "href", "xlink:href", "style", "d", "transform"]),
            prop::sample::select(vec!["red", "javascript:alert(1)", "url(http://x/y)", "M0 0L1 1", "#a"]),
        ), 1..8),
    ) {
        let body: String = elements.iter().map(|(e, a, v)| format!(r#"<{e} {a}="{v}">t</{e}>"#)).collect();
        let svg = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 24 24">{body}</svg>"#);
        if let Ok((_, clean)) = assets::sanitize_svg(&svg) {
            let lower = clean.to_lowercase();
            for bad in ["<script", "<foreignobject", "<style", "<image", "<use", "<a ", " on", "href", "javascript:", "url(http"] {
                prop_assert!(!lower.contains(bad), "{bad} survived in {clean}");
            }
        }
    }
}
