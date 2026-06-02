//! World-to-terminal coordinate mapping with zoom, pan, and projection.

use glam::{DVec2, DVec3};

/// Terminal cells are taller than wide; scale Y accordingly.
pub const CELL_ASPECT: f64 = 2.0;

/// 2D projection plane for rendering 3D positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Projection {
    #[default]
    Xy,
    Xz,
    Yz,
}

/// Camera state in simulation space (meters).
#[derive(Debug, Clone)]
pub struct Camera {
    pub center_m: DVec2,
    pub meters_per_cell: f64,
    pub projection: Projection,
}

impl Camera {
    #[must_use]
    pub fn new(meters_per_cell: f64) -> Self {
        Self {
            center_m: DVec2::ZERO,
            meters_per_cell,
            projection: Projection::Xy,
        }
    }

    /// Project a 3D position into the camera's 2D plane (meters).
    #[must_use]
    pub fn project(&self, position: DVec3) -> DVec2 {
        match self.projection {
            Projection::Xy => DVec2::new(position.x, position.y),
            Projection::Xz => DVec2::new(position.x, position.z),
            Projection::Yz => DVec2::new(position.y, position.z),
        }
    }

    /// Map world coordinates (m) to terminal cell coordinates (fractional).
    #[must_use]
    pub fn world_to_screen(&self, world: DVec2, width: u16, height: u16) -> DVec2 {
        let cx = f64::from(width) / 2.0;
        let cy = f64::from(height) / 2.0;
        let sx = cx + (world.x - self.center_m.x) / self.meters_per_cell;
        let sy = cy - (world.y - self.center_m.y) / self.meters_per_cell / CELL_ASPECT;
        DVec2::new(sx, sy)
    }

    /// Map terminal cell to world position in 3D for the active projection (m).
    #[must_use]
    pub fn screen_to_world_3d(&self, screen: DVec2, width: u16, height: u16) -> DVec3 {
        let plane = self.screen_to_world(screen, width, height);
        match self.projection {
            Projection::Xy => DVec3::new(plane.x, plane.y, 0.0),
            Projection::Xz => DVec3::new(plane.x, 0.0, plane.y),
            Projection::Yz => DVec3::new(0.0, plane.x, plane.y),
        }
    }

    /// Map terminal cell coordinates to world (m) in the projected plane.
    #[must_use]
    pub fn screen_to_world(&self, screen: DVec2, width: u16, height: u16) -> DVec2 {
        let cx = f64::from(width) / 2.0;
        let cy = f64::from(height) / 2.0;
        let wx = self.center_m.x + (screen.x - cx) * self.meters_per_cell;
        let wy = self.center_m.y - (screen.y - cy) * self.meters_per_cell * CELL_ASPECT;
        DVec2::new(wx, wy)
    }

    pub fn zoom_in(&mut self) {
        self.meters_per_cell *= 0.8;
        self.clamp_zoom();
    }

    pub fn zoom_out(&mut self) {
        self.meters_per_cell *= 1.25;
        self.clamp_zoom();
    }

    pub fn pan_cells(&mut self, dx_cells: f64, dy_cells: f64) {
        self.center_m.x += dx_cells * self.meters_per_cell;
        self.center_m.y += dy_cells * self.meters_per_cell * CELL_ASPECT;
    }

    /// Zoom while keeping `anchor_screen` fixed in world space.
    pub fn zoom_at_screen(&mut self, anchor_screen: DVec2, width: u16, height: u16, zoom_in: bool) {
        let before = self.screen_to_world(anchor_screen, width, height);
        if zoom_in {
            self.zoom_in();
        } else {
            self.zoom_out();
        }
        let after = self.screen_to_world(anchor_screen, width, height);
        self.center_m += before - after;
    }

    pub fn frame_positions(&mut self, positions: &[DVec2], width: u16, height: u16) {
        if positions.is_empty() {
            return;
        }
        let mut min = positions[0];
        let mut max = positions[0];
        for p in positions.iter().skip(1) {
            min = min.min(*p);
            max = max.max(*p);
        }
        self.center_m = (min + max) * 0.5;
        let span_x = (max.x - min.x).abs().max(1.0);
        let span_y = (max.y - min.y).abs().max(1.0);
        let w = f64::from(width).max(1.0);
        let h = f64::from(height).max(1.0);
        let mpc_x = span_x / w;
        let mpc_y = span_y / (h * CELL_ASPECT);
        self.meters_per_cell = mpc_x.max(mpc_y) * 1.2;
        self.clamp_zoom();
    }

    fn clamp_zoom(&mut self) {
        const MIN_MPC: f64 = 1.0;
        const MAX_MPC: f64 = 1.0e18;
        self.meters_per_cell = self.meters_per_cell.clamp(MIN_MPC, MAX_MPC);
    }
}
