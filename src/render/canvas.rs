//! Terminal-cell drawing buffer for the simulation viewport.

use glam::DVec2;
use ratatui::style::{Color, Style};
use ratatui::text::Span;

use crate::render::camera::Camera;
use crate::render::colors::{body_color, com_marker_color, selection_accent, star_glow_color, trail_color};

/// One drawable cell in the simulation view.
#[derive(Debug, Clone)]
struct Cell {
    ch: char,
    fg: Color,
    priority: u8,
}

/// Raster buffer for bodies and trails.
#[derive(Debug)]
pub struct SimulationCanvas {
    width: u16,
    height: u16,
    cells: Vec<Option<Cell>>,
}

impl SimulationCanvas {
    pub fn new(width: u16, height: u16) -> Self {
        let len = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![None; len],
        }
    }

    pub fn draw_into<F>(&self, mut put: F)
    where
        F: FnMut(u16, u16, Span<'static>),
    {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = usize::from(y) * usize::from(self.width) + usize::from(x);
                if let Some(cell) = &self.cells[idx] {
                    put(
                        x,
                        y,
                        Span::styled(cell.ch.to_string(), Style::default().fg(cell.fg)),
                    );
                }
            }
        }
    }

    pub fn plot_trail(
        &mut self,
        camera: &Camera,
        points: impl ExactSizeIterator<Item = DVec2>,
        base_rgb: [u8; 3],
    ) {
        let n = points.len();
        if n == 0 {
            return;
        }
        for (i, world) in points.enumerate() {
            let age = if n <= 1 {
                1.0
            } else {
                i as f64 / (n - 1) as f64
            };
            let rgb = trail_color(base_rgb, age);
            let ch = if age > 0.8 { '•' } else { '·' };
            self.plot_world(camera, world, ch, rgb, 1);
        }
    }

    pub fn plot_heatmap_cell(&mut self, x: u16, y: u16, ch: char, rgb: [u8; 3]) {
        if ch == ' ' {
            return;
        }
        self.put_screen(x, y, ch, rgb, 0);
    }

    pub fn plot_star_glow(&mut self, camera: &Camera, world: DVec2, base_rgb: [u8; 3]) {
        let glow = star_glow_color(base_rgb);
        let screen = camera.world_to_screen(world, self.width, self.height);
        let cx = screen.x.round() as i32;
        let cy = screen.y.round() as i32;
        for (dx, dy) in [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ] {
            let x = cx + dx;
            let y = cy + dy;
            if x >= 0 && y >= 0 && x < i32::from(self.width) && y < i32::from(self.height) {
                self.put_screen(x as u16, y as u16, '·', glow, 2);
            }
        }
    }

    pub fn plot_selection_marker(&mut self, camera: &Camera, world: DVec2) {
        let accent = selection_accent();
        let screen = camera.world_to_screen(world, self.width, self.height);
        let cx = screen.x.round() as i32;
        let cy = screen.y.round() as i32;
        for (dx, dy, ch) in [
            (0, -2, '┴'),
            (0, 2, '┬'),
            (-2, 0, '┤'),
            (2, 0, '├'),
        ] {
            let x = cx + dx;
            let y = cy + dy;
            if x >= 0 && y >= 0 && x < i32::from(self.width) && y < i32::from(self.height) {
                self.put_screen(x as u16, y as u16, ch, accent, 8);
            }
        }
    }

    pub fn plot_com_marker(&mut self, camera: &Camera, world: DVec2) {
        self.plot_world(camera, world, '╋', com_marker_color(), 7);
    }

    pub fn plot_body(
        &mut self,
        camera: &Camera,
        world: DVec2,
        body: &crate::physics::body::Body,
        selected: bool,
    ) {
        let rgb = body_color(body);
        if matches!(body.class, crate::physics::body::BodyClass::Star) {
            self.plot_star_glow(camera, world, rgb);
        }
        let ch = if selected {
            '█'
        } else if matches!(body.class, crate::physics::body::BodyClass::Star) {
            '◉'
        } else {
            '●'
        };
        let priority = if selected { 10 } else { 5 };
        self.plot_world(camera, world, ch, rgb, priority);
        if selected {
            self.plot_selection_marker(camera, world);
        }
    }

    fn plot_world(&mut self, camera: &Camera, world: DVec2, ch: char, rgb: [u8; 3], priority: u8) {
        let screen = camera.world_to_screen(world, self.width, self.height);
        let x = screen.x.round() as i32;
        let y = screen.y.round() as i32;
        if x < 0 || y < 0 || x >= i32::from(self.width) || y >= i32::from(self.height) {
            return;
        }
        self.put_screen(x as u16, y as u16, ch, rgb, priority);
    }

    fn put_screen(&mut self, x: u16, y: u16, ch: char, rgb: [u8; 3], priority: u8) {
        let idx = usize::from(y) * usize::from(self.width) + usize::from(x);
        let fg = Color::Rgb(rgb[0], rgb[1], rgb[2]);
        match &self.cells[idx] {
            None => self.cells[idx] = Some(Cell { ch, fg, priority }),
            Some(existing) if priority >= existing.priority => {
                self.cells[idx] = Some(Cell { ch, fg, priority });
            }
            _ => {}
        }
    }
}
