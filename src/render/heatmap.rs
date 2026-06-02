//! Gravitational field heatmap sampling and cache.

use glam::{DVec2, DVec3};

use crate::physics::body::Body;
use crate::physics::field::gravitational_field_magnitude;
use crate::render::camera::Camera;
use crate::render::colors::{heatmap_char, heatmap_color, log_field_intensity};

/// Cached heatmap raster (terminal cells).
#[derive(Debug, Default)]
pub struct HeatmapCache {
    cells: Vec<Option<HeatmapCell>>,
    width: u16,
    height: u16,
    frames_since_rebuild: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct HeatmapCell {
    pub ch: char,
    pub rgb: [u8; 3],
}

/// Inputs for one heatmap rebuild pass.
pub struct HeatmapBuildContext<'a> {
    pub camera: &'a Camera,
    pub bodies: &'a [Body],
    pub positions: &'a [DVec3],
    pub softening_m: f64,
    pub width: u16,
    pub height: u16,
    pub sample_divisor: u32,
    pub body_count: usize,
    pub fps: f64,
}

impl HeatmapCache {
    pub const REBUILD_INTERVAL: u32 = 4;
    pub const AUTO_DISABLE_BODIES: usize = 64;

    pub fn invalidate(&mut self) {
        self.frames_since_rebuild = Self::REBUILD_INTERVAL;
    }

    /// Rebuild if stale; returns false if heatmap should be skipped (performance).
    pub fn maybe_rebuild(&mut self, ctx: &HeatmapBuildContext<'_>) -> bool {
        if ctx.body_count > Self::AUTO_DISABLE_BODIES && ctx.fps > 0.0 && ctx.fps < 20.0 {
            return false;
        }

        self.frames_since_rebuild += 1;
        let size_changed = self.width != ctx.width || self.height != ctx.height;
        if !size_changed
            && self.frames_since_rebuild < Self::REBUILD_INTERVAL
            && self.cells.len() == usize::from(ctx.width) * usize::from(ctx.height)
        {
            return true;
        }

        self.frames_since_rebuild = 0;
        self.width = ctx.width;
        self.height = ctx.height;
        let len = usize::from(ctx.width) * usize::from(ctx.height);
        self.cells.resize(len, None);

        let step = ctx.sample_divisor.max(1) as u16;
        let positions_3d: Vec<DVec3> = ctx.positions.to_vec();

        for sy in (0..ctx.height).step_by(step as usize) {
            for sx in (0..ctx.width).step_by(step as usize) {
                let screen = DVec2::new(f64::from(sx) + 0.5, f64::from(sy) + 0.5);
                let world_3d = ctx
                    .camera
                    .screen_to_world_3d(screen, ctx.width, ctx.height);
                let magnitude = gravitational_field_magnitude(
                    world_3d,
                    ctx.bodies,
                    &positions_3d,
                    ctx.softening_m,
                );
                let log_i = log_field_intensity(magnitude);
                let rgb = heatmap_color(log_i);
                let ch = heatmap_char(log_i);

                for dy in 0..step {
                    for dx in 0..step {
                        let x = sx + dx;
                        let y = sy + dy;
                        if x >= ctx.width || y >= ctx.height {
                            continue;
                        }
                        let idx = usize::from(y) * usize::from(ctx.width) + usize::from(x);
                        self.cells[idx] = Some(HeatmapCell { ch, rgb });
                    }
                }
            }
        }

        true
    }

    pub fn plot_onto<F>(&self, mut plot: F)
    where
        F: FnMut(u16, u16, char, [u8; 3]),
    {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = usize::from(y) * usize::from(self.width) + usize::from(x);
                if let Some(cell) = self.cells[idx] {
                    plot(x, y, cell.ch, cell.rgb);
                }
            }
        }
    }
}
