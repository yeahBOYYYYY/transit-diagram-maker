use serde::{Deserialize, Serialize};

use crate::coord::{clamp_to_world_bounds, snap_to_grid, Aabb, DVec2};

/// The Viewport camera managing 2D pan/zoom and coordinate transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportCamera {
    /// Screen center world coordinate offset
    pub pan: DVec2,
    /// Zoom scale factor (e.g. 1.0 = 100%, 2.0 = 200%)
    pub zoom: f64,
    /// Viewport width in screen pixels
    pub width: f64,
    /// Viewport height in screen pixels
    pub height: f64,
    /// Base grid spacing in world units (default 20.0)
    pub grid_spacing: f64,
}

impl ViewportCamera {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            pan: DVec2::ZERO,
            zoom: 1.0,
            width,
            height,
            grid_spacing: 20.0,
        }
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width.max(1.0);
        self.height = height.max(1.0);
    }

    /// Converts screen pixel coordinates (e.g. mouse position) to world coordinates
    pub fn screen_to_world(&self, screen_pos: DVec2) -> DVec2 {
        let center = DVec2::new(self.width * 0.5, self.height * 0.5);
        let delta = screen_pos - center;
        let world = self.pan + delta / self.zoom;
        clamp_to_world_bounds(world)
    }

    /// Converts world coordinates to screen pixel coordinates
    pub fn world_to_screen(&self, world_pos: DVec2) -> DVec2 {
        let center = DVec2::new(self.width * 0.5, self.height * 0.5);
        let delta = world_pos - self.pan;
        center + delta * self.zoom
    }

    /// Applies 2D translation delta to camera (middle-mouse drag)
    pub fn translate(&mut self, screen_delta: DVec2) {
        let world_delta = screen_delta / self.zoom;
        self.pan = clamp_to_world_bounds(self.pan - world_delta);
    }

    /// Scales camera zoom anchored at the cursor's screen position
    pub fn zoom_at_screen_pos(&mut self, screen_pos: DVec2, zoom_factor: f64) {
        let world_before = self.screen_to_world(screen_pos);
        let new_zoom = (self.zoom * zoom_factor).clamp(0.05, 50.0);
        self.zoom = new_zoom;

        // Re-adjust pan so world_before remains under screen_pos
        let center = DVec2::new(self.width * 0.5, self.height * 0.5);
        let delta = screen_pos - center;
        self.pan = clamp_to_world_bounds(world_before - delta / self.zoom);
    }

    /// Snaps coordinate to grid, unless shift is held
    pub fn snap_coordinate(&self, world_pos: DVec2, shift_held: bool) -> DVec2 {
        if shift_held {
            // Bypass grid snapping for meticulous manual adjustment
            world_pos
        } else {
            snap_to_grid(world_pos, self.grid_spacing)
        }
    }

    /// Recenters camera to fit an AABB with comfortable padding
    pub fn recenter_to_bounds(&mut self, bounds: Aabb) {
        let center = bounds.center();
        let w = bounds.width().max(100.0);
        let h = bounds.height().max(100.0);

        let padding = 100.0;
        let available_w = (self.width - padding * 2.0).max(100.0);
        let available_h = (self.height - padding * 2.0).max(100.0);

        let zoom_x = available_w / w;
        let zoom_y = available_h / h;
        let fit_zoom = zoom_x.min(zoom_y).clamp(0.1, 4.0);

        self.pan = clamp_to_world_bounds(center);
        self.zoom = fit_zoom;
    }

    /// Computes procedural grid parameters for rendering:
    /// returns (effective_spacing, minor_line_opacity, major_grid_multiplier)
    pub fn grid_render_params(&self) -> (f64, f64, f64) {
        // Pixel distance of one grid unit
        let screen_unit = self.grid_spacing * self.zoom;

        // Dynamic density fading: fade minor lines as screen spacing drops below 16px
        let minor_opacity = if screen_unit < 12.0 {
            0.0
        } else if screen_unit < 24.0 {
            (screen_unit - 12.0) / 12.0
        } else {
            1.0
        };

        (self.grid_spacing, minor_opacity, 5.0)
    }
}
