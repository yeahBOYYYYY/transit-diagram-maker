use serde::{Deserialize, Serialize};

use crate::coord::DVec2;
use crate::graph::{NodeId, ParameterSchema, RenderPrimitive};
use crate::viewport::ViewportCamera;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    Select,
    Pen,
    Erase,
    Station,
    Text,
}

/// Brief summary of a node for the frontend Layer/Node hierarchy panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub id: NodeId,
    pub name: String,
    pub type_name: String,
    pub visible: bool,
    pub locked: bool,
    pub z_index: i32,
}

/// Actions received across WASM boundary from the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum EditorAction {
    SelectTool(ToolType),
    PointerDown {
        screen_pos: DVec2,
        shift: bool,
        alt: bool,
        ctrl: bool,
        button: u32,
    },
    PointerMove {
        screen_pos: DVec2,
        shift: bool,
        alt: bool,
        ctrl: bool,
    },
    PointerUp {
        screen_pos: DVec2,
        shift: bool,
        alt: bool,
        ctrl: bool,
        button: u32,
    },
    Wheel {
        screen_pos: DVec2,
        delta_y: f64,
    },
    KeyDown {
        key: String,
        shift: bool,
        ctrl: bool,
    },
    KeyUp {
        key: String,
        shift: bool,
        ctrl: bool,
    },
    ResizeViewport {
        width: f64,
        height: f64,
    },
    RecenterView,
    Undo,
    Redo,
    SetNodeParameter {
        node_id: NodeId,
        param_name: String,
        value: serde_json::Value,
    },
    SelectNode {
        node_id: NodeId,
        multi: bool,
    },
    ClearSelection,
    DeleteSelected,
    ToggleNodeVisibility(NodeId),
    ToggleNodeLock(NodeId),
    ReorderNodeZIndex {
        node_id: NodeId,
        new_z_index: i32,
    },
    ExportSvg,
    ExportTransitDocument,
    LoadTransitDocument(String),
}

/// Complete state snapshot serialized to frontend after action processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorStateResponse {
    pub primitives: Vec<RenderPrimitive>,
    pub selected_node_ids: Vec<NodeId>,
    pub selected_parameters: Option<Vec<ParameterSchema>>,
    pub active_tool: ToolType,
    pub camera: ViewportCamera,
    pub can_undo: bool,
    pub can_redo: bool,
    pub colliding_text_count: usize,
    pub node_summaries: Vec<NodeSummary>,
    pub export_payload: Option<ExportPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    pub format: String,
    pub content: String,
    pub filename: String,
}
