//! Interactive application state and main loop.

use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::EnableMouseCapture;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use glam::DVec2;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::cli::RunArgs;
use crate::error::{not_implemented, GravitonError, Result};
use crate::input::keymap::{map_key, AppCommand};
use crate::input::mouse::{map_mouse, MouseCommand};
use crate::physics::diagnostics::{compute, Diagnostics};
use crate::physics::integrator::{Integrator, Rk4Integrator};
use crate::physics::system::SystemState;
use crate::render::camera::{Camera, Projection};
use crate::render::trails::Trail;
use crate::scenario::{LoadedScenario, RenderConfig};
use crate::time::clock::SimulationClock;
use crate::time::scheduler::PhysicsScheduler;

/// UI and simulation settings for rendering.
#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub trails_enabled: bool,
    pub trail_sample_every: u64,
    pub hud_visible: bool,
}

/// Top-level interactive application state.
pub struct App {
    pub system: SystemState,
    snapshot: SystemState,
    pub initial_diagnostics: Diagnostics,
    pub current_diagnostics: Diagnostics,
    pub camera: Camera,
    pub clock: SimulationClock,
    pub scheduler: PhysicsScheduler,
    pub selected_body: Option<usize>,
    pub render_settings: RenderSettings,
    pub trails: Vec<Trail>,
    pub scenario_path: PathBuf,
    pub default_meters_per_cell: f64,
    pub follow_selected: bool,
    pub show_help: bool,
    pub should_quit: bool,
    pub fps: f64,
    physics_tick: u64,
    fps_timer: Instant,
    fps_frames: u32,
}

impl App {
    pub fn from_loaded(
        loaded: LoadedScenario,
        scenario_path: PathBuf,
        args: &RunArgs,
        render: RenderConfig,
    ) -> Result<Self> {
        if loaded.system.settings.use_barnes_hut {
            return Err(not_implemented("Barnes-Hut integration", "Phase 5"));
        }

        let mut system = loaded.system;
        if let Some(dt) = args.dt {
            if dt <= 0.0 || !dt.is_finite() {
                return Err(GravitonError::Scenario(
                    crate::error::ScenarioError::InvalidTimeStep,
                ));
            }
            system.settings.dt_s = dt;
        }

        let initial_diagnostics = compute(&system);
        let snapshot = system.clone();
        let trail_capacity = render.trail_capacity;
        let trails = (0..system.bodies.len())
            .map(|_| Trail::new(trail_capacity))
            .collect();

        let mut camera = Camera::new(render.meters_per_cell);
        let projected: Vec<DVec2> = system
            .bodies
            .iter()
            .map(|b| camera.project(b.position_m))
            .collect();
        camera.frame_positions(&projected, 80, 24);

        if render.follow_center_of_mass {
            camera.center_m = camera.project(initial_diagnostics.center_of_mass_m);
        }

        let selected_body = if system.bodies.is_empty() {
            None
        } else {
            Some(0)
        };

        Ok(Self {
            system,
            snapshot,
            initial_diagnostics,
            current_diagnostics: initial_diagnostics,
            camera,
            clock: SimulationClock::new(),
            scheduler: PhysicsScheduler::new(),
            selected_body,
            render_settings: RenderSettings {
                trails_enabled: !args.no_trails,
                trail_sample_every: render.trail_sample_every,
                hud_visible: true,
            },
            trails,
            scenario_path,
            default_meters_per_cell: render.meters_per_cell,
            follow_selected: false,
            show_help: false,
            should_quit: false,
            fps: 0.0,
            physics_tick: 0,
            fps_timer: Instant::now(),
            fps_frames: 0,
        })
    }

    pub fn run_interactive(
        loaded: LoadedScenario,
        scenario_path: PathBuf,
        args: RunArgs,
    ) -> Result<()> {
        let render = loaded.render.clone();
        let mut app = Self::from_loaded(loaded, scenario_path, &args, render)?;

        enable_raw_mode()?;
        stdout()
            .execute(EnterAlternateScreen)?
            .execute(EnableMouseCapture)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let result = app.loop_inner(&mut terminal);

        disable_raw_mode()?;
        stdout()
            .execute(LeaveAlternateScreen)?
            .execute(crossterm::event::DisableMouseCapture)?;
        terminal.show_cursor()?;

        result
    }

    fn loop_inner(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        let integrator = Rk4Integrator;
        let tick_rate = Duration::from_millis(16);

        while !self.should_quit {
            while event::poll(tick_rate)? {
                self.handle_event(event::read()?, terminal)?;
            }

            let real_dt = self.clock.elapsed_real_s();
            self.scheduler.accumulate(real_dt, self.clock.time_warp);
            let dt = self.system.settings.dt_s;

            while self.scheduler.should_step(dt) {
                integrator.step(&mut self.system)?;
                self.physics_tick += 1;
                self.record_trails();
                self.scheduler.consume_step(dt);
            }

            self.current_diagnostics = compute(&self.system);
            self.update_fps();

            terminal.draw(|frame| crate::render::draw(frame, self))?;
        }

        Ok(())
    }

    fn handle_event(
        &mut self,
        event: Event,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.apply_command(map_key(key), terminal)?;
            }
            Event::Mouse(mouse) => {
                if let Some(cmd) = map_mouse(mouse) {
                    self.apply_mouse(cmd, terminal)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_mouse(
        &mut self,
        cmd: MouseCommand,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        let sim_area = simulation_rect(terminal, self.render_settings.hud_visible)?;
        match cmd {
            MouseCommand::SelectAt { col, row } => {
                self.select_at_terminal_cell(col, row, sim_area);
            }
            MouseCommand::ZoomAt { col, row, zoom_in } => {
                let local_x = col.saturating_sub(sim_area.x);
                let local_y = row.saturating_sub(sim_area.y);
                let anchor = DVec2::new(f64::from(local_x), f64::from(local_y));
                self.camera
                    .zoom_at_screen(anchor, sim_area.width, sim_area.height, zoom_in);
            }
            MouseCommand::PanBy { dx, dy } => {
                self.camera.pan_cells(dx, dy);
            }
        }
        Ok(())
    }

    fn apply_command(
        &mut self,
        cmd: AppCommand,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        match cmd {
            AppCommand::Quit => self.should_quit = true,
            AppCommand::TogglePause => self.clock.toggle_pause(),
            AppCommand::ToggleHelp => self.show_help = !self.show_help,
            AppCommand::ResetSimulation => self.reset_simulation(),
            AppCommand::ZoomIn => {
                let sim = simulation_rect(terminal, self.render_settings.hud_visible)?;
                let center = DVec2::new(f64::from(sim.width) / 2.0, f64::from(sim.height) / 2.0);
                self.camera
                    .zoom_at_screen(center, sim.width, sim.height, true);
            }
            AppCommand::ZoomOut => {
                let sim = simulation_rect(terminal, self.render_settings.hud_visible)?;
                let center = DVec2::new(f64::from(sim.width) / 2.0, f64::from(sim.height) / 2.0);
                self.camera
                    .zoom_at_screen(center, sim.width, sim.height, false);
            }
            AppCommand::ResetZoom => {
                self.camera.meters_per_cell = self.default_meters_per_cell;
            }
            AppCommand::Pan { dx, dy } => self.camera.pan_cells(dx, dy),
            AppCommand::FollowSelected => self.follow_selected = !self.follow_selected,
            AppCommand::FrameAll => self.frame_all_bodies(),
            AppCommand::ProjectionXy => self.camera.projection = Projection::Xy,
            AppCommand::ProjectionXz => self.camera.projection = Projection::Xz,
            AppCommand::ProjectionYz => self.camera.projection = Projection::Yz,
            AppCommand::SelectNext => self.cycle_selection(1),
            AppCommand::SelectPrevious => self.cycle_selection(-1),
            AppCommand::ToggleTrails => {
                self.render_settings.trails_enabled = !self.render_settings.trails_enabled;
            }
            AppCommand::ToggleHud => {
                self.render_settings.hud_visible = !self.render_settings.hud_visible;
            }
            AppCommand::IncreaseTimeWarp => self.clock.increase_warp(),
            AppCommand::DecreaseTimeWarp => self.clock.decrease_warp(),
            AppCommand::IncreaseDt => {
                self.system.settings.dt_s *= 1.25;
            }
            AppCommand::DecreaseDt => {
                self.system.settings.dt_s = (self.system.settings.dt_s / 1.25).max(1.0);
            }
            AppCommand::None => {}
        }

        if self.follow_selected {
            if let Some(idx) = self.selected_body {
                let pos = self.camera.project(self.system.bodies[idx].position_m);
                self.camera.center_m = pos;
            }
        }

        Ok(())
    }

    fn record_trails(&mut self) {
        if !self.render_settings.trails_enabled {
            return;
        }
        let every = self.render_settings.trail_sample_every;
        for (i, body) in self.system.bodies.iter().enumerate() {
            self.trails[i].maybe_push(body.position_m, self.physics_tick, every);
        }
    }

    fn reset_simulation(&mut self) {
        self.system = self.snapshot.clone();
        self.physics_tick = 0;
        for trail in &mut self.trails {
            trail.clear();
        }
        self.current_diagnostics = compute(&self.system);
        self.scheduler = PhysicsScheduler::new();
    }

    fn cycle_selection(&mut self, delta: isize) {
        let n = self.system.bodies.len();
        if n == 0 {
            self.selected_body = None;
            return;
        }
        let current = self.selected_body.unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(n as isize) as usize;
        self.selected_body = Some(next);
    }

    fn frame_all_bodies(&mut self) {
        let projected: Vec<DVec2> = self
            .system
            .bodies
            .iter()
            .map(|b| self.camera.project(b.position_m))
            .collect();
        self.camera.frame_positions(&projected, 80, 24);
    }

    fn select_at_terminal_cell(&mut self, col: u16, row: u16, area: ratatui::layout::Rect) {
        if col < area.x || row < area.y {
            return;
        }
        let local_x = col - area.x;
        let local_y = row - area.y;
        if local_x >= area.width || local_y >= area.height {
            return;
        }
        let screen = DVec2::new(f64::from(local_x), f64::from(local_y));
        let world = self.camera.screen_to_world(screen, area.width, area.height);

        let threshold = 2.0 * self.camera.meters_per_cell;
        let best = self.system.bodies.iter().enumerate().fold(None, |best, (i, body)| {
            let projected = self.camera.project(body.position_m);
            let dist = (projected - world).length();
            if dist > threshold {
                return best;
            }
            match best {
                None => Some((i, dist)),
                Some((_, d0)) if dist < d0 => Some((i, dist)),
                other => other,
            }
        });
        if let Some((idx, _)) = best {
            self.selected_body = Some(idx);
        }
    }

    fn update_fps(&mut self) {
        self.fps_frames += 1;
        let elapsed = self.fps_timer.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.fps_frames as f64 / elapsed.as_secs_f64();
            self.fps_frames = 0;
            self.fps_timer = Instant::now();
        }
    }
}

fn terminal_rect(terminal: &Terminal<CrosstermBackend<Stdout>>) -> Result<ratatui::layout::Rect> {
    let size = terminal.size()?;
    Ok(ratatui::layout::Rect::new(0, 0, size.width, size.height))
}

fn simulation_rect(
    terminal: &Terminal<CrosstermBackend<Stdout>>,
    hud_visible: bool,
) -> Result<ratatui::layout::Rect> {
    let area = terminal_rect(terminal)?;
    Ok(crate::render::hud::simulation_area(area, hud_visible))
}
