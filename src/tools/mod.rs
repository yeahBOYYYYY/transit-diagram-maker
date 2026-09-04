use serde::{Deserialize, Serialize};

use crate::coord::DVec2;
use crate::graph::{NodeId, StationStyle};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolState {
    Select {
        drag_start: Option<DVec2>,
        last_world_pos: Option<DVec2>,
        is_dragging_nodes: bool,
    },
    Pen {
        points: Vec<DVec2>,
        current_preview: Option<DVec2>,
        active_color: String,
        active_width: f64,
        corner_radius: f64,
    },
    Erase,
    Station {
        hover_pos: Option<DVec2>,
        snapped_line_id: Option<NodeId>,
        active_style: StationStyle,
        active_radius: f64,
    },
    Text,
}

impl ToolState {
    pub fn new_select() -> Self {
        Self::Select {
            drag_start: None,
            last_world_pos: None,
            is_dragging_nodes: false,
        }
    }

    pub fn new_pen() -> Self {
        Self::Pen {
            points: Vec::new(),
            current_preview: None,
            active_color: "#007aff".into(),
            active_width: 8.0,
            corner_radius: 16.0,
        }
    }

    pub fn new_erase() -> Self {
        Self::Erase
    }

    pub fn new_station() -> Self {
        Self::Station {
            hover_pos: None,
            snapped_line_id: None,
            active_style: StationStyle::Dot,
            active_radius: 7.5,
        }
    }

    pub fn new_text() -> Self {
        Self::Text
    }
}
