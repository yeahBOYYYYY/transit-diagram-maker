use serde_json::json;
use transit_core::geom::expand;
use transit_core::tree;
use transit_core::*;

fn c(x: i32, y: i32) -> Cell {
    Cell::new(x, y)
}

fn line(ed: &mut Editor, pts: &[Cell]) -> CreatedLine {
    ed.create_line(pts, LineStyle::default()).unwrap()
}

fn segment(ed: &Editor, id: Id) -> &LineSegment {
    match &ed.layer(id).unwrap().content {
        Content::Line(s) => s,
        other => panic!("not a line: {other:?}"),
    }
}

fn station(ed: &Editor, id: Id) -> &Station {
    match &ed.layer(id).unwrap().content {
        Content::Station(s) => s,
        other => panic!("not a station: {other:?}"),
    }
}

fn text(ed: &Editor, id: Id) -> &TextBox {
    match &ed.layer(id).unwrap().content {
        Content::Text(t) => t,
        other => panic!("not text: {other:?}"),
    }
}

fn segments_of(ed: &Editor, line_id: Id) -> Vec<Id> {
    let mut out = Vec::new();
    tree::walk_layers(&ed.document().root, &mut |l, _, _| {
        if matches!(&l.content, Content::Line(s) if s.line_id == line_id) {
            out.push(l.id);
        }
    });
    out
}

// ------------------------------------------------------------------ lines

#[test]
fn create_line_builds_group_with_segment() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(5, 2)]);
    let tree = ed.tree();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].id, l.group_id);
    assert_eq!(tree[0].role.as_deref(), Some("line"));
    assert_eq!(tree[0].children[0].id, l.layer_id);
    // Unaligned input is routed: straight first, then 45°.
    assert_eq!(segment(&ed, l.layer_id).points, vec![c(0, 0), c(3, 0), c(5, 2)]);
}

#[test]
fn create_line_rejects_single_cell() {
    let mut ed = Editor::new();
    assert!(ed.create_line(&[c(1, 1), c(1, 1)], LineStyle::default()).is_err());
    assert!(!ed.can_undo(), "failed operations are not recorded");
}

#[test]
fn lines_are_numbered() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(2, 0)]);
    line(&mut ed, &[c(0, 2), c(2, 2)]);
    let names: Vec<_> = ed.tree().iter().map(|t| t.name.clone()).collect();
    assert_eq!(names, vec!["Line 2", "Line 1"], "top of stack first");
}

#[test]
fn line_hit_and_nearest_cell() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0), c(6, 2)]);
    let hit = ed.line_hit_at(c(5, 1)).unwrap();
    assert_eq!(hit.layer_id, l.layer_id);
    assert_eq!(hit.dir, Dir8::SE);
    assert!(ed.line_hit_at(c(5, 0)).is_none());
    let near = ed.nearest_line_cell(Vec2::new(2.4, 1.3), 1.5).unwrap();
    assert_eq!(near.cell, c(2, 0));
    assert!(ed.nearest_line_cell(Vec2::new(2.5, 9.0), 1.5).is_none());
}

#[test]
fn hidden_or_locked_lines_are_not_hit() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    ed.set_visible(l.group_id, false).unwrap();
    assert!(ed.line_hit_at(c(1, 0)).is_none());
    ed.set_visible(l.group_id, true).unwrap();
    ed.set_locked(l.layer_id, true).unwrap();
    assert!(ed.line_hit_at(c(1, 0)).is_none());
}

// ------------------------------------------------------------------ erase

#[test]
fn erase_middle_splits_into_two_layers_in_same_group() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    assert_eq!(ed.erase(&[c(4, 0)]).unwrap(), 1);
    let segs = segments_of(&ed, l.line_id);
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0], l.layer_id, "first part keeps the original id");
    assert_eq!(segment(&ed, segs[0]).points, vec![c(0, 0), c(3, 0)]);
    assert_eq!(segment(&ed, segs[1]).points, vec![c(5, 0), c(9, 0)]);
    for s in &segs {
        assert_eq!(tree::locate(&ed.document().root, *s).unwrap().parent, Some(l.group_id));
    }
    assert_eq!(ed.layer(segs[1]).unwrap().name, "Segment 2");
}

#[test]
fn erase_keeps_style_and_line_identity() {
    let mut ed = Editor::new();
    let style = LineStyle { color: "#00ff00".into(), kind: StrokeKind::Hollow, ..LineStyle::default() };
    let l = ed.create_line(&[c(0, 0), c(9, 0)], style.clone()).unwrap();
    ed.erase(&[c(3, 0), c(6, 0)]).unwrap();
    let segs = segments_of(&ed, l.line_id);
    assert_eq!(segs.len(), 3);
    for s in segs {
        assert_eq!(segment(&ed, s).style, style);
    }
}

#[test]
fn erase_drops_single_cell_leftovers_and_empty_groups() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(2, 0)]);
    ed.erase(&[c(1, 0)]).unwrap();
    assert!(segments_of(&ed, l.line_id).is_empty());
    assert!(ed.tree().is_empty(), "empty line group is pruned");
}

#[test]
fn erase_across_corner_and_diagonal() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0), c(8, 4)]);
    ed.erase(&[c(6, 2)]).unwrap();
    let segs = segments_of(&ed, l.line_id);
    assert_eq!(segment(&ed, segs[0]).points, vec![c(0, 0), c(4, 0), c(5, 1)]);
    assert_eq!(segment(&ed, segs[1]).points, vec![c(7, 3), c(8, 4)]);
}

#[test]
fn erase_skips_locked_layers_and_misses() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    assert_eq!(ed.erase(&[c(4, 5)]).unwrap(), 0);
    ed.set_locked(l.group_id, true).unwrap();
    assert_eq!(ed.erase(&[c(4, 0)]).unwrap(), 0);
    assert_eq!(segments_of(&ed, l.line_id).len(), 1);
}

#[test]
fn erase_whole_drag_is_one_undo_step_in_transaction() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    let depth = ed.undo_depth();
    ed.begin();
    ed.erase(&[c(2, 0)]).unwrap();
    ed.erase(&[c(5, 0)]).unwrap();
    ed.commit();
    assert_eq!(ed.undo_depth(), depth + 1);
    assert_eq!(segments_of(&ed, l.line_id).len(), 3);
    ed.undo();
    assert_eq!(segments_of(&ed, l.line_id).len(), 1);
}

// --------------------------------------------------------------- stations

#[test]
fn dot_station_joins_line_group_with_label() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(3, 0), &StationOptions::default()).unwrap();
    let root = &ed.document().root;
    assert_eq!(tree::locate(root, s.group_id).unwrap().parent, Some(l.group_id));
    assert_eq!(tree::locate(root, s.station_id).unwrap().parent, Some(s.group_id));
    let text_id = s.text_id.unwrap();
    assert_eq!(tree::locate(root, text_id).unwrap().parent, Some(s.group_id));
    let st = station(&ed, s.station_id);
    assert_eq!(st.line_ids, vec![l.line_id]);
    assert!(matches!(st.shape, StationShape::Dot { cell, ref outer_color, ref inner_color }
        if cell == c(3, 0) && outer_color == "#000000" && inner_color == "#ffffff"));
    let label = text(&ed, text_id);
    assert_eq!(label.text, "Station 1");
    assert_eq!(label.station_id, Some(s.station_id));
    assert!(label.pos.y < 0.5, "label sits above a horizontal line");
}

#[test]
fn station_snaps_to_nearby_line() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(4, 1), &StationOptions::default()).unwrap();
    assert_eq!(station(&ed, s.station_id).shape.cells(), vec![c(4, 0)]);
}

#[test]
fn station_off_line_is_rejected() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    assert!(ed.add_station(StationKind::Dot, c(4, 6), &StationOptions::default()).is_err());
    let mut empty = Editor::new();
    assert!(empty.add_station(StationKind::Tick, c(0, 0), &StationOptions::default()).is_err());
}

#[test]
fn tick_station_follows_line_direction_and_color() {
    let mut ed = Editor::new();
    let style = LineStyle { color: "#0098d4".into(), ..LineStyle::default() };
    ed.create_line(&[c(0, 0), c(0, 6)], style).unwrap();
    let opts = StationOptions { tick_side: -1, ..StationOptions::default() };
    let s = ed.add_station(StationKind::Tick, c(0, 3), &opts).unwrap();
    match &station(&ed, s.station_id).shape {
        StationShape::Tick { dir, side, color, .. } => {
            assert_eq!(*dir, Dir8::S);
            assert_eq!(*side, -1);
            assert_eq!(color, "#0098d4");
        }
        other => panic!("{other:?}"),
    }
    // Line goes down; right-hand side of "south" is west, so the label is left-aligned to the end.
    assert_eq!(text(&ed, s.text_id.unwrap()).align, TextAlign::End);
}

#[test]
fn station_without_label() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    let opts = StationOptions { label: false, ..StationOptions::default() };
    let s = ed.add_station(StationKind::Dot, c(1, 0), &opts).unwrap();
    assert!(s.text_id.is_none());
}

#[test]
fn pill_couples_to_all_touched_lines() {
    let mut ed = Editor::new();
    let a = line(&mut ed, &[c(0, 0), c(9, 0)]);
    let b = line(&mut ed, &[c(0, 1), c(9, 1)]);
    let s = ed.add_pill(&[c(4, 0), c(4, 1), c(4, 1)], &StationOptions::default()).unwrap();
    let st = station(&ed, s.station_id);
    assert_eq!(st.shape.cells(), vec![c(4, 0), c(4, 1)]);
    assert!(st.line_ids.contains(&a.line_id) && st.line_ids.contains(&b.line_id));
    assert!(ed.add_pill(&[c(20, 20)], &StationOptions::default()).is_err());
}

#[test]
fn pill_is_painted_above_all_touched_lines() {
    let mut ed = Editor::new();
    let a = line(&mut ed, &[c(0, 0), c(9, 0)]);
    let b = line(&mut ed, &[c(4, -3), c(4, 3)]);
    // First cell is only on line A (bottom); the pill must still join line B (top).
    let s = ed.add_pill(&[c(3, 0), c(4, 0), c(4, 1)], &StationOptions::default()).unwrap();
    let root = &ed.document().root;
    assert_eq!(tree::ancestors(root, s.station_id).last(), Some(&b.group_id));
    assert!(!tree::ancestors(root, s.station_id).contains(&a.group_id));
    let order: Vec<Id> = render::render_fragments(ed.document()).iter().map(|f| f.id).collect();
    assert_eq!(order.iter().position(|i| *i == s.station_id), Some(order.len() - 2), "only its label is above it");
}

#[test]
fn station_kind_can_change() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(2, 0), &StationOptions::default()).unwrap();
    ed.set_station_kind(s.station_id, StationKind::Tick).unwrap();
    assert!(matches!(station(&ed, s.station_id).shape, StationShape::Tick { dir: Dir8::E, .. }));
    ed.set_station_kind(s.station_id, StationKind::Pill).unwrap();
    assert_eq!(station(&ed, s.station_id).shape.kind(), StationKind::Pill);
    ed.set_station_kind(s.station_id, StationKind::Dot).unwrap();
    assert_eq!(station(&ed, s.station_id).shape.kind(), StationKind::Dot);
}

#[test]
fn moving_station_snaps_to_line_and_drags_label() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(2, 0), &StationOptions::default()).unwrap();
    let before = text(&ed, s.text_id.unwrap()).pos;
    let cell = ed.move_station_to(s.station_id, Vec2::new(6.2, 3.0)).unwrap();
    assert_eq!(cell, c(6, 0));
    let after = text(&ed, s.text_id.unwrap()).pos;
    assert!((after.x - before.x - 4.0).abs() < 1e-9 && (after.y - before.y).abs() < 1e-9);
}

#[test]
fn translating_station_alone_keeps_it_on_line() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(2, 0), &StationOptions::default()).unwrap();
    ed.translate(&[s.station_id], 3.0, 4.0).unwrap();
    assert_eq!(station(&ed, s.station_id).shape.cells(), vec![c(5, 0)]);
}

// ----------------------------------------------------------- text & icons

#[test]
fn text_and_icon_layers() {
    let mut ed = Editor::new();
    let t = ed.add_text(TextBox { text: "Central\nStation".into(), ..TextBox::default() }).unwrap();
    assert_eq!(ed.layer(t).unwrap().name, "Central");
    let i = ed.add_icon(Icon { icon: "airport".into(), ..Icon::default() }).unwrap();
    assert_eq!(ed.layer(i).unwrap().name, "Airport");
    let icon_size = match &ed.layer(i).unwrap().content {
        Content::Icon(icon) => icon.size,
        _ => unreachable!(),
    };
    assert!((icon_size - ed.settings().station_outer_radius * 2.0).abs() < 1e-9, "matches dot size");
}

#[test]
fn update_text_renames_layer_that_followed_text() {
    let mut ed = Editor::new();
    let t = ed.add_text(TextBox::default()).unwrap();
    ed.update_content(t, &json!({"text": "Harbour", "size": 1.5})).unwrap();
    assert_eq!(text(&ed, t).text, "Harbour");
    assert_eq!(text(&ed, t).size, 1.5);
    assert_eq!(ed.layer(t).unwrap().name, "Harbour");
    ed.rename(t, "Custom").unwrap();
    ed.update_content(t, &json!({"text": "Other"})).unwrap();
    assert_eq!(ed.layer(t).unwrap().name, "Custom", "custom names are kept");
}

#[test]
fn update_line_style_applies_to_all_segments() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    ed.erase(&[c(4, 0)]).unwrap();
    ed.update_content(l.layer_id, &json!({"style": {"color": "#123456", "kind": "dashed", "end_cap": "butt"}})).unwrap();
    for s in segments_of(&ed, l.line_id) {
        let st = &segment(&ed, s).style;
        assert_eq!(st.color, "#123456");
        assert_eq!(st.kind, StrokeKind::Dashed);
        assert_eq!(st.end_cap, Cap::Butt);
        assert_eq!(st.start_cap, Cap::Round);
    }
}

#[test]
fn update_rejects_bad_patches() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    assert!(ed.update_content(l.layer_id, &json!({"kind": "text"})).is_err());
    assert!(ed.update_content(l.layer_id, &json!({"points": [{"x": 0, "y": 0}, {"x": 3, "y": 1}]})).is_err());
    assert!(ed.update_content(l.layer_id, &json!({"style": {"kind": "zigzag"}})).is_err());
    assert!(ed.update_content(l.layer_id, &json!([1, 2])).is_err());
    assert!(ed.update_content(999, &json!({})).is_err());
    assert!(!ed.can_redo());
}

// ---------------------------------------------------------------- layers

#[test]
fn visibility_lock_and_rename() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    ed.set_visible(l.group_id, false).unwrap();
    assert!(render::render_fragments(ed.document()).is_empty());
    ed.set_visible(l.group_id, true).unwrap();
    ed.set_locked(l.group_id, true).unwrap();
    let frags = render::render_fragments(ed.document());
    assert!(frags[0].locked, "lock is inherited from the group");
    assert!(ed.rename(l.group_id, "  Red Line ").is_ok());
    assert_eq!(ed.node(l.group_id).unwrap().name(), "Red Line");
    assert!(ed.rename(l.group_id, "   ").is_err());
}

#[test]
fn locked_layers_cannot_be_moved_or_deleted() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    ed.set_locked(l.layer_id, true).unwrap();
    ed.translate(&[l.group_id], 2.0, 0.0).unwrap();
    assert_eq!(segment(&ed, l.layer_id).points[0], c(0, 0));
    assert_eq!(ed.delete(&[l.layer_id]).unwrap(), 0);
    assert!(ed.layer(l.layer_id).is_some());
}

#[test]
fn group_and_ungroup() {
    let mut ed = Editor::new();
    let a = ed.add_text(TextBox::default()).unwrap();
    let b = ed.add_text(TextBox::default()).unwrap();
    let c3 = ed.add_text(TextBox::default()).unwrap();
    let g = ed.group(&[a, c3], None).unwrap();
    let tree = ed.tree();
    // Group is placed where the topmost member (c3) was: top of stack.
    assert_eq!(tree[0].id, g);
    assert_eq!(tree[1].id, b);
    let kids: Vec<_> = tree[0].children.iter().map(|t| t.id).collect();
    assert_eq!(kids, vec![c3, a]);
    let released = ed.ungroup(g).unwrap();
    assert_eq!(released, vec![a, c3]);
    let ids: Vec<_> = ed.tree().iter().map(|t| t.id).collect();
    assert_eq!(ids, vec![c3, a, b]);
    assert!(ed.ungroup(a).is_err());
    assert!(ed.group(&[], None).is_err());
}

#[test]
fn delete_group_removes_children() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(2, 0), &StationOptions::default()).unwrap();
    assert_eq!(ed.delete(&[s.group_id, s.station_id]).unwrap(), 1);
    assert!(ed.layer(s.station_id).is_none());
    assert!(ed.layer(s.text_id.unwrap()).is_none());
}

#[test]
fn deleting_label_only_keeps_station() {
    let mut ed = Editor::new();
    line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(2, 0), &StationOptions::default()).unwrap();
    ed.delete(&[s.text_id.unwrap()]).unwrap();
    assert!(ed.layer(s.station_id).is_some());
}

#[test]
fn move_node_between_groups_and_guard_cycles() {
    let mut ed = Editor::new();
    let t = ed.add_text(TextBox::default()).unwrap();
    let g = ed.new_group(None).unwrap();
    ed.move_node(t, Some(g), 0).unwrap();
    assert_eq!(tree::locate(&ed.document().root, t).unwrap().parent, Some(g));
    let inner = ed.group(&[t], Some("Inner")).unwrap();
    assert!(ed.move_node(g, Some(inner), 0).is_err(), "no cycles");
    assert!(ed.move_node(g, Some(t), 0).is_err(), "layers are not containers");
    ed.move_node(inner, None, 0).unwrap();
    assert_eq!(ed.tree().last().unwrap().id, inner, "index 0 = bottom of stack");
}

#[test]
fn move_node_within_same_parent_accounts_for_removal() {
    let mut ed = Editor::new();
    let a = ed.add_text(TextBox::default()).unwrap();
    let b = ed.add_text(TextBox::default()).unwrap();
    let c3 = ed.add_text(TextBox::default()).unwrap();
    // Move bottom (a) to the very top: index = len.
    ed.move_node(a, None, 3).unwrap();
    let order: Vec<_> = ed.document().root.iter().map(|n| n.id()).collect();
    assert_eq!(order, vec![b, c3, a]);
}

#[test]
fn restack_up_and_down() {
    let mut ed = Editor::new();
    let a = ed.add_text(TextBox::default()).unwrap();
    let b = ed.add_text(TextBox::default()).unwrap();
    let c3 = ed.add_text(TextBox::default()).unwrap();
    ed.restack(&[a], 1).unwrap();
    let order: Vec<_> = ed.document().root.iter().map(|n| n.id()).collect();
    assert_eq!(order, vec![b, a, c3]);
    ed.restack(&[c3], -10).unwrap();
    let order: Vec<_> = ed.document().root.iter().map(|n| n.id()).collect();
    assert_eq!(order, vec![c3, b, a]);
}

#[test]
fn duplicate_line_with_station_gets_fresh_ids_and_references() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(2, 0), &StationOptions::default()).unwrap();
    let copies = ed.duplicate(&[l.group_id]).unwrap();
    assert_eq!(copies.len(), 1);
    let leaves = tree::leaf_ids(&ed.document().root, &copies);
    assert_eq!(leaves.len(), 3);
    let mut new_line_id = None;
    let mut new_station = None;
    for id in &leaves {
        assert!(*id != l.layer_id && *id != s.station_id);
        match &ed.layer(*id).unwrap().content {
            Content::Line(seg) => {
                assert_ne!(seg.line_id, l.line_id);
                assert_eq!(seg.points[0], c(1, 1), "copy is offset by one cell");
                new_line_id = Some(seg.line_id);
            }
            Content::Station(st) => new_station = Some((*id, st.line_ids.clone())),
            Content::Text(t) => assert_ne!(t.station_id, Some(s.station_id)),
            _ => {}
        }
    }
    let (sid, line_ids) = new_station.unwrap();
    assert_eq!(line_ids, vec![new_line_id.unwrap()]);
    assert_eq!(station(&ed, sid).shape.cells(), vec![c(3, 1)]);
}

// ------------------------------------------------------------- transforms

#[test]
fn translate_rounds_grid_content_but_not_text() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(3, 0)]);
    let t = ed.add_text(TextBox { pos: Vec2::new(1.0, 1.0), ..TextBox::default() }).unwrap();
    ed.translate(&[l.group_id, t], 1.6, -0.4).unwrap();
    assert_eq!(segment(&ed, l.layer_id).points, vec![c(2, 0), c(5, 0)]);
    let p = text(&ed, t).pos;
    assert!((p.x - 2.6).abs() < 1e-9 && (p.y - 0.6).abs() < 1e-9);
}

#[test]
fn whole_line_group_moves_with_stations() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(9, 0)]);
    let s = ed.add_station(StationKind::Dot, c(2, 0), &StationOptions::default()).unwrap();
    ed.translate(&[l.group_id], 0.0, 5.0).unwrap();
    assert_eq!(station(&ed, s.station_id).shape.cells(), vec![c(2, 5)]);
}

#[test]
fn drag_with_rebase_does_not_accumulate() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(3, 0)]);
    let depth = ed.undo_depth();
    ed.begin();
    for step in 1..=5 {
        ed.rebase();
        ed.translate(&[l.group_id], step as f64, 0.0).unwrap();
    }
    ed.commit();
    assert_eq!(segment(&ed, l.layer_id).points, vec![c(5, 0), c(8, 0)]);
    assert_eq!(ed.undo_depth(), depth + 1);
}

#[test]
fn cancel_transaction_restores() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(3, 0)]);
    ed.begin();
    ed.translate(&[l.group_id], 4.0, 0.0).unwrap();
    ed.cancel();
    assert_eq!(segment(&ed, l.layer_id).points, vec![c(0, 0), c(3, 0)]);
}

#[test]
fn rotate_line_quarter_turn() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    let applied = ed.rotate(&[l.group_id], 88.0, Vec2::new(0.5, 0.5)).unwrap();
    assert_eq!(applied, 90.0, "grid content snaps to 45° steps");
    assert_eq!(segment(&ed, l.layer_id).points, vec![c(0, 0), c(0, 4)]);
}

#[test]
fn rotate_line_45_degrees_stays_on_grid() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    ed.rotate(&[l.layer_id], 45.0, Vec2::new(0.5, 0.5)).unwrap();
    let pts = &segment(&ed, l.layer_id).points;
    assert_eq!(pts[0], c(0, 0));
    assert_eq!(Dir8::between(pts[0], *pts.last().unwrap()), Some(Dir8::SE));
}

#[test]
fn rotate_tick_station_with_its_line() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(6, 0)]);
    let s = ed.add_station(StationKind::Tick, c(3, 0), &StationOptions::default()).unwrap();
    ed.rotate(&[l.group_id], 90.0, Vec2::new(3.5, 0.5)).unwrap();
    match &station(&ed, s.station_id).shape {
        StationShape::Tick { cell, dir, .. } => {
            assert_eq!(*cell, c(3, 0));
            assert_eq!(*dir, Dir8::S);
        }
        other => panic!("{other:?}"),
    }
    let cells = expand(&segment(&ed, l.layer_id).points);
    assert!(cells.contains(&c(3, 0)));
}

#[test]
fn text_and_icons_rotate_in_45_degree_steps() {
    let mut ed = Editor::new();
    let t = ed.add_text(TextBox { pos: Vec2::new(2.0, 0.0), ..TextBox::default() }).unwrap();
    let applied = ed.rotate(&[t], 30.0, Vec2::new(2.0, 0.0)).unwrap();
    assert_eq!(applied, 45.0);
    assert_eq!(text(&ed, t).rotation, 45.0);
    ed.rotate(&[t], 170.0, Vec2::new(2.0, 0.0)).unwrap();
    assert_eq!(text(&ed, t).rotation, -135.0);
    assert_eq!(ed.rotate(&[t], 20.0, Vec2::new(2.0, 0.0)).unwrap(), 0.0);
}

#[test]
fn text_and_icon_positions_have_one_decimal() {
    let mut ed = Editor::new();
    let t = ed.add_text(TextBox { pos: Vec2::new(1.234, 5.678), ..TextBox::default() }).unwrap();
    assert_eq!(text(&ed, t).pos, Vec2::new(1.2, 5.7));
    ed.translate(&[t], 0.33, -0.01).unwrap();
    assert_eq!(text(&ed, t).pos, Vec2::new(1.5, 5.7));
    ed.update_content(t, &json!({"pos": {"x": 0.05}})).unwrap();
    assert_eq!(text(&ed, t).pos.x, 0.1);
}

#[test]
fn rotation_properties_snap_to_45_degrees() {
    let mut ed = Editor::new();
    let t = ed.add_text(TextBox { rotation: 30.0, ..TextBox::default() }).unwrap();
    assert_eq!(text(&ed, t).rotation, 45.0);
    ed.update_content(t, &json!({"rotation": 100.0})).unwrap();
    assert_eq!(text(&ed, t).rotation, 90.0);
}

#[test]
fn rotation_below_half_step_is_noop() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    assert_eq!(ed.rotate(&[l.group_id], 10.0, Vec2::new(0.5, 0.5)).unwrap(), 0.0);
    assert_eq!(ed.undo_depth(), 1);
}

#[test]
fn bounds_cover_selection() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    let b = ed.bounds(&[l.group_id]).unwrap();
    assert_eq!(b, Rect { x: 0.0, y: 0.0, w: 120.0, h: 24.0 });
    assert!(ed.bounds(&[]).is_none());
}

// ---------------------------------------------------------------- history

#[test]
fn undo_redo_walks_history() {
    let mut ed = Editor::new();
    assert!(!ed.can_undo() && !ed.can_redo());
    let a = line(&mut ed, &[c(0, 0), c(4, 0)]);
    let b = line(&mut ed, &[c(0, 2), c(4, 2)]);
    assert_eq!(ed.tree().len(), 2);
    assert!(ed.undo());
    assert_eq!(ed.tree().len(), 1);
    assert!(ed.node(b.group_id).is_none());
    assert!(ed.redo());
    assert!(ed.node(b.group_id).is_some());
    assert!(ed.undo() && ed.undo());
    assert!(ed.tree().is_empty());
    assert!(!ed.undo());
    assert!(ed.redo());
    assert!(ed.node(a.group_id).is_some());
    // New action after undo discards redo.
    line(&mut ed, &[c(0, 5), c(4, 5)]);
    assert!(!ed.can_redo());
}

#[test]
fn history_limit_is_enforced() {
    let mut ed = Editor::new();
    ed.set_history_limit(3);
    for i in 0..10 {
        line(&mut ed, &[c(0, i), c(4, i)]);
    }
    assert_eq!(ed.undo_depth(), 3);
    while ed.undo() {}
    assert_eq!(ed.tree().len(), 7);
}

#[test]
fn no_op_changes_are_not_recorded() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    let depth = ed.undo_depth();
    ed.set_visible(l.group_id, true).unwrap();
    ed.translate(&[l.group_id], 0.2, 0.2).unwrap();
    assert_eq!(ed.undo_depth(), depth);
}

#[test]
fn settings_changes_are_undoable_and_sanitized() {
    let mut ed = Editor::new();
    let s = DocSettings { grid_size: 1e9, station_inner_radius: 5.0, ..DocSettings::default() };
    ed.set_settings(s).unwrap();
    assert_eq!(ed.settings().grid_size, 512.0);
    assert!(ed.settings().station_inner_radius <= ed.settings().station_outer_radius);
    ed.undo();
    assert_eq!(ed.settings().grid_size, 24.0);
}

#[test]
fn expanding_groups_is_not_history() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    let depth = ed.undo_depth();
    ed.set_expanded(l.group_id, false).unwrap();
    assert_eq!(ed.undo_depth(), depth);
    assert!(!ed.tree()[0].expanded);
    ed.undo();
    ed.redo();
    assert!(!ed.tree()[0].expanded, "collapse state survives history navigation");
}

#[test]
fn undo_inside_transaction_commits_first() {
    let mut ed = Editor::new();
    let l = line(&mut ed, &[c(0, 0), c(4, 0)]);
    ed.begin();
    ed.translate(&[l.group_id], 2.0, 0.0).unwrap();
    assert!(ed.can_undo());
    ed.undo();
    assert!(!ed.in_transaction());
    assert_eq!(segment(&ed, l.layer_id).points[0], c(0, 0));
}
