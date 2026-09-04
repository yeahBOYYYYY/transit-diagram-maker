pub mod bvh;
pub mod coord;
pub mod engine;
pub mod graph;
pub mod history;
pub mod messages;
pub mod render;
pub mod tools;
pub mod viewport;
pub mod wasm;

pub use engine::EditorEngine;
pub use messages::{EditorAction, EditorStateResponse, ToolType};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bvh::{detect_text_collisions, SpatialItem};
    use crate::coord::{snap_transit_octolinear_to_grid, Aabb, DVec2};
    use crate::graph::{
        generate_transit_curve_segments, DocumentTree, NodeData, PathSegment, StationStyle,
    };
    use crate::messages::EditorAction;

    #[test]
    fn test_strict_grid_locking() {
        let grid = 20.0;
        let from = DVec2::new(40.0, 60.0);

        // Movement with slight cursor noise mostly horizontal
        let target_horiz = DVec2::new(143.0, 63.5);
        let snapped_h = snap_transit_octolinear_to_grid(from, target_horiz, grid);
        // Vertical coordinate MUST be strictly locked on the original grid line y = 60.0!
        assert_eq!(snapped_h.y, 60.0, "Horizontal transit line must stay strictly inside horizontal grid line!");
        assert_eq!(snapped_h.x, 140.0);

        // Movement mostly vertical
        let target_vert = DVec2::new(42.0, 185.0);
        let snapped_v = snap_transit_octolinear_to_grid(from, target_vert, grid);
        // Horizontal coordinate MUST be strictly locked on the original grid line x = 40.0!
        assert_eq!(snapped_v.x, 40.0, "Vertical transit line must stay strictly inside vertical grid line!");
        assert_eq!(snapped_v.y, 180.0);

        // Movement diagonal
        let target_diag = DVec2::new(103.0, 122.0);
        let snapped_d = snap_transit_octolinear_to_grid(from, target_diag, grid);
        assert_eq!((snapped_d.x - from.x).abs(), (snapped_d.y - from.y).abs(), "Diagonal lines must have equal dx and dy on grid!");
    }

    #[test]
    fn test_transit_fillet() {
        let points = vec![
            DVec2::new(0.0, 0.0),
            DVec2::new(100.0, 0.0),
            DVec2::new(100.0, 100.0),
        ];
        let segments = generate_transit_curve_segments(&points, 20.0, &[]);
        assert!(segments.len() >= 3);
        let has_quad = segments.iter().any(|s| matches!(s, PathSegment::QuadTo { .. }));
        assert!(has_quad, "Corner filleting must produce a QuadTo quadratic Bezier curve");
    }

    #[test]
    fn test_station_erase() {
        let mut engine = EditorEngine::new(800.0, 600.0);
        let initial_node_count = engine.tree.nodes.len();

        // Switch to Erase Tool
        engine.dispatch(EditorAction::SelectTool(ToolType::Erase));

        // West End station is at (-240.0, 160.0). Let's erase it:
        let screen_pos = engine.camera.world_to_screen(DVec2::new(-240.0, 160.0));
        engine.dispatch(EditorAction::PointerDown {
            screen_pos,
            shift: false,
            alt: false,
            ctrl: false,
            button: 0,
        });

        // The station AND its child text label should be removed
        assert!(engine.tree.nodes.len() < initial_node_count, "Erase tool must delete station and child text");
    }

    #[test]
    fn test_bvh_collision() {
        let item1 = SpatialItem {
            id: 10,
            aabb: Aabb::new(DVec2::new(10.0, 10.0), DVec2::new(60.0, 30.0)),
        };
        let item2 = SpatialItem {
            id: 20,
            aabb: Aabb::new(DVec2::new(40.0, 20.0), DVec2::new(90.0, 40.0)),
        };
        let item3 = SpatialItem {
            id: 30,
            aabb: Aabb::new(DVec2::new(200.0, 200.0), DVec2::new(250.0, 220.0)),
        };

        let colliding = detect_text_collisions(vec![item1, item2, item3]);
        assert!(colliding.contains(&10));
        assert!(colliding.contains(&20));
        assert!(!colliding.contains(&30));
    }

    #[test]
    fn test_undo_redo_stack() {
        let mut engine = EditorEngine::new(800.0, 600.0);
        let initial_count = engine.tree.nodes.len();

        engine.dispatch(EditorAction::SelectTool(ToolType::Station));
        engine.dispatch(EditorAction::PointerDown {
            screen_pos: DVec2::new(400.0, 300.0),
            shift: false,
            alt: false,
            ctrl: false,
            button: 0,
        });

        assert!(engine.tree.nodes.len() > initial_count);
        assert!(engine.history.can_undo());

        engine.dispatch(EditorAction::Undo);
        assert_eq!(engine.tree.nodes.len(), initial_count);
        assert!(engine.history.can_redo());

        engine.dispatch(EditorAction::Redo);
        assert!(engine.tree.nodes.len() > initial_count);
    }

    #[test]
    fn test_transit_serialization() {
        let mut engine = EditorEngine::new(1000.0, 800.0);
        let state = engine.dispatch(EditorAction::ExportTransitDocument);

        assert!(state.export_payload.is_some());
        let payload = state.export_payload.unwrap();
        assert_eq!(payload.format, "transit");

        let json = payload.content;
        assert!(json.contains("Line 1 (Red)"));

        let mut new_engine = EditorEngine::new(500.0, 500.0);
        new_engine.dispatch(EditorAction::LoadTransitDocument(json));
        assert_eq!(new_engine.tree.nodes.len(), engine.tree.nodes.len());
    }

    #[test]
    fn test_slinky_dot_station() {
        let mut tree = DocumentTree::new();
        let _sid = tree.add_node(
            NodeData::Station {
                name: "Multi-Track Hub".into(),
                pos: DVec2::new(100.0, 100.0),
                associated_line_id: None,
                style: StationStyle::Dot,
                radius: 7.0,
                fill_color: "#ffffff".into(),
                stroke_color: "#000000".into(),
                stroke_width: 3.0,
                rotation_rad: 0.0,
                child_text_id: None,
                span_cells: 3, // Stretches across 3 grid lines
                span_angle_deg: 90.0,
            },
            1,
        );

        let primitives = tree.evaluate(&[]);
        assert_eq!(primitives.len(), 1);
        if let crate::graph::RenderPrimitiveType::Station { center, end_pos, span_cells, .. } = primitives[0].primitive_type {
            assert_eq!(span_cells, 3);
            assert_eq!(center, DVec2::new(100.0, 100.0));
            assert_eq!(end_pos, DVec2::new(100.0, 160.0));
        } else {
            panic!("Expected station primitive");
        }
    }
}
