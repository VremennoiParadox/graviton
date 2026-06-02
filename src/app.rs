//! Interactive application state and main loop.

use std::collections::VecDeque;
use std::io::{stdout, Stdout};
use std::path::{Path, PathBuf};
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
use crate::render::heatmap::HeatmapCache;
use crate::render::trails::Trail;
use crate::scenario::{discover_scenarios, load, LoadedScenario, RenderConfig};
use crate::time::clock::SimulationClock;
use crate::time::scheduler::PhysicsScheduler;

/// CLI flags that affect the interactive session.
#[derive(Debug, Clone)]
pub struct RunFlags {
    pub no_heatmap: bool,
    pub no_trails: bool,
}

/// Overlay preset cycled with `o`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPreset {
    Standard,
    Science,
    Minimal,
    Full,
}

/// UI and simulation settings for rendering.
#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub trails_enabled: bool,
    pub trail_sample_every: u64,
    pub hud_visible: bool,
    pub heatmap_enabled: bool,
    pub heatmap_sample_divisor: u32,
    pub show_com_marker: bool,
    pub show_energy_diagnostics: bool,
    pub show_momentum_diagnostics: bool,
    pub overlay_preset: OverlayPreset,
    pub kitty_enabled: bool,
    pub kitty_mode: String,
}

/// Rolling energy drift samples for HUD sparkline.
#[derive(Debug, Default)]
pub struct EnergyHistory {
    pub samples: VecDeque<f64>,
}

impl EnergyHistory {
    const CAPACITY: usize = 40;

    pub fn push(&mut self, drift_fraction: f64) {
        if self.samples.len() >= Self::CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back(drift_fraction);
    }
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
    pub heatmap: HeatmapCache,
    pub energy_history: EnergyHistory,
    pub scenario_path: PathBuf,
    pub scenario_paths: Vec<PathBuf>,
    pub scenario_menu_index: usize,
    pub show_scenario_menu: bool,
    pub default_meters_per_cell: f64,
    pub follow_selected: bool,
    pub show_help: bool,
    pub should_quit: bool,
    pub fps: f64,
    pub toast_message: Option<String>,
    pub toast_until: Option<Instant>,
    physics_tick: u64,
    fps_timer: Instant,
    fps_frames: u32,
    mouse_drag_anchor: Option<DVec2>,
    run_flags: RunFlags,
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

        let scenario_paths =
            discover_scenarios(Path::new("scenarios")).unwrap_or_default();
        let scenario_menu_index = scenario_paths
            .iter()
            .position(|p| p == &scenario_path)
            .unwrap_or(0);

        let overlay_preset = OverlayPreset::Standard;
        let render_settings = build_render_settings(&render, args, overlay_preset);

        Ok(Self {
            system,
            snapshot,
            initial_diagnostics,
            current_diagnostics: initial_diagnostics,
            camera,
            clock: SimulationClock::new(),
            scheduler: PhysicsScheduler::new(),
            selected_body,
            render_settings,
            trails,
            heatmap: HeatmapCache::default(),
            energy_history: EnergyHistory::default(),
            scenario_path,
            scenario_paths,
            scenario_menu_index,
            show_scenario_menu: false,
            default_meters_per_cell: render.meters_per_cell,
            follow_selected: false,
            show_help: false,
            should_quit: false,
            fps: 0.0,
            toast_message: None,
            toast_until: None,
            physics_tick: 0,
            fps_timer: Instant::now(),
            fps_frames: 0,
            mouse_drag_anchor: None,
            run_flags: RunFlags {
                no_heatmap: args.no_heatmap,
                no_trails: args.no_trails,
            },
        })
    }

    pub fn simulation_title(&self) -> String {
        let mut title = self.system.scenario_name.clone();
        if self.render_settings.heatmap_enabled {
            title.push_str(" | g: field");
        }
        if self.clock.paused {
            title.push_str(" | paused");
        }
        title
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
            self.expire_toast();

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
            let drift = self
                .current_diagnostics
                .energy_drift_fraction(self.initial_diagnostics.total_energy_j);
            self.energy_history.push(drift);
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
        if self.show_scenario_menu {
            if let Event::Key(key) = &event {
                if key.kind == KeyEventKind::Press {
                    let cmd = map_key(*key);
                    match cmd {
                        AppCommand::Pan { dy: 1.0, .. } | AppCommand::SelectPrevious => {
                            self.scenario_menu_up();
                            return Ok(());
                        }
                        AppCommand::Pan { dy: -1.0, .. } | AppCommand::SelectNext => {
                            self.scenario_menu_down();
                            return Ok(());
                        }
                        AppCommand::ScenarioMenuConfirm => {
                            self.confirm_scenario_menu()?;
                            return Ok(());
                        }
                        AppCommand::Quit | AppCommand::ToggleHelp => {
                            self.show_scenario_menu = false;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }

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
        if self.show_scenario_menu {
            return Ok(());
        }

        let sim_area = simulation_rect(terminal, self.render_settings.hud_visible)?;
        match cmd {
            MouseCommand::SelectAt { col, row } => {
                self.mouse_drag_anchor = None;
                self.select_at_terminal_cell(col, row, sim_area);
            }
            MouseCommand::DragTo { col, row } => {
                let local_x = col.saturating_sub(sim_area.x);
                let local_y = row.saturating_sub(sim_area.y);
                let screen = DVec2::new(f64::from(local_x), f64::from(local_y));
                if let Some(anchor) = self.mouse_drag_anchor {
                    let delta = screen - anchor;
                    if delta.length_squared() > 0.25 {
                        self.camera.pan_cells(-delta.x, -delta.y);
                        self.heatmap.invalidate();
                        self.mouse_drag_anchor = Some(screen);
                    }
                } else {
                    self.mouse_drag_anchor = Some(screen);
                }
            }
            MouseCommand::DragEnd => {
                self.mouse_drag_anchor = None;
            }
            MouseCommand::ZoomAt { col, row, zoom_in } => {
                let local_x = col.saturating_sub(sim_area.x);
                let local_y = row.saturating_sub(sim_area.y);
                let anchor = DVec2::new(f64::from(local_x), f64::from(local_y));
                self.camera
                    .zoom_at_screen(anchor, sim_area.width, sim_area.height, zoom_in);
                self.heatmap.invalidate();
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
            AppCommand::ReloadScenario => self.reload_scenario_from_disk()?,
            AppCommand::ZoomIn => {
                let sim = simulation_rect(terminal, self.render_settings.hud_visible)?;
                let center = DVec2::new(f64::from(sim.width) / 2.0, f64::from(sim.height) / 2.0);
                self.camera
                    .zoom_at_screen(center, sim.width, sim.height, true);
                self.heatmap.invalidate();
            }
            AppCommand::ZoomOut => {
                let sim = simulation_rect(terminal, self.render_settings.hud_visible)?;
                let center = DVec2::new(f64::from(sim.width) / 2.0, f64::from(sim.height) / 2.0);
                self.camera
                    .zoom_at_screen(center, sim.width, sim.height, false);
                self.heatmap.invalidate();
            }
            AppCommand::ResetZoom => {
                self.camera.meters_per_cell = self.default_meters_per_cell;
                self.heatmap.invalidate();
            }
            AppCommand::Pan { dx, dy } => {
                self.camera.pan_cells(dx, dy);
                self.heatmap.invalidate();
            }
            AppCommand::FollowSelected => self.follow_selected = !self.follow_selected,
            AppCommand::FrameAll => self.frame_all_bodies(),
            AppCommand::ProjectionXy => {
                self.camera.projection = Projection::Xy;
                self.heatmap.invalidate();
            }
            AppCommand::ProjectionXz => {
                self.camera.projection = Projection::Xz;
                self.heatmap.invalidate();
            }
            AppCommand::ProjectionYz => {
                self.camera.projection = Projection::Yz;
                self.heatmap.invalidate();
            }
            AppCommand::SelectNext => self.cycle_selection(1),
            AppCommand::SelectPrevious => self.cycle_selection(-1),
            AppCommand::ToggleTrails => {
                self.render_settings.trails_enabled = !self.render_settings.trails_enabled;
            }
            AppCommand::ToggleHud => {
                self.render_settings.hud_visible = !self.render_settings.hud_visible;
            }
            AppCommand::ToggleHeatmap => {
                self.render_settings.heatmap_enabled = !self.render_settings.heatmap_enabled;
                self.heatmap.invalidate();
            }
            AppCommand::ToggleEnergyDiagnostics => {
                self.render_settings.show_energy_diagnostics =
                    !self.render_settings.show_energy_diagnostics;
            }
            AppCommand::ToggleMomentumDiagnostics => {
                self.render_settings.show_momentum_diagnostics =
                    !self.render_settings.show_momentum_diagnostics;
            }
            AppCommand::ToggleComMarker => {
                self.render_settings.show_com_marker = !self.render_settings.show_com_marker;
            }
            AppCommand::CycleOverlayPreset => self.cycle_overlay_preset(),
            AppCommand::OpenScenarioMenu => {
                self.show_scenario_menu = true;
                if self.scenario_paths.is_empty() {
                    self.scenario_paths =
                        discover_scenarios(Path::new("scenarios")).unwrap_or_default();
                }
            }
            AppCommand::ValidateScenario => self.validate_current_scenario(),
            AppCommand::IncreaseTimeWarp => self.clock.increase_warp(),
            AppCommand::DecreaseTimeWarp => self.clock.decrease_warp(),
            AppCommand::IncreaseDt => {
                self.system.settings.dt_s *= 1.25;
            }
            AppCommand::DecreaseDt => {
                self.system.settings.dt_s = (self.system.settings.dt_s / 1.25).max(1.0);
            }
            AppCommand::ScenarioMenuConfirm => {
                if self.show_scenario_menu {
                    self.confirm_scenario_menu()?;
                }
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

    fn cycle_overlay_preset(&mut self) {
        self.render_settings.overlay_preset = match self.render_settings.overlay_preset {
            OverlayPreset::Standard => OverlayPreset::Science,
            OverlayPreset::Science => OverlayPreset::Minimal,
            OverlayPreset::Minimal => OverlayPreset::Full,
            OverlayPreset::Full => OverlayPreset::Standard,
        };
        apply_overlay_preset(&mut self.render_settings);
        self.heatmap.invalidate();
    }

    fn validate_current_scenario(&mut self) {
        match load(&self.scenario_path) {
            Ok(loaded) => {
                let n = loaded.system.bodies.len();
                self.show_toast(format!(
                    "ok: {} ({n} bodies)",
                    self.scenario_path.display()
                ));
            }
            Err(e) => self.show_toast(format!("validate failed: {e}")),
        }
    }

    fn reload_scenario_from_disk(&mut self) -> Result<()> {
        let path = self.scenario_path.clone();
        let loaded = load(&path)?;
        let render = loaded.render.clone();
        let overlay = self.render_settings.overlay_preset;
        let trails_on = self.render_settings.trails_enabled;
        let hud_on = self.render_settings.hud_visible;
        let heatmap_on = self.render_settings.heatmap_enabled && !self.run_flags.no_heatmap;

        let mut fresh = Self::from_loaded(
            loaded,
            path,
            &RunArgs {
                scenario: self.scenario_path.clone(),
                headless: false,
                steps: 0,
                dt: Some(self.system.settings.dt_s),
                integrator: crate::cli::IntegratorArg::Rk4,
                barnes_hut: false,
                theta: 0.7,
                no_heatmap: self.run_flags.no_heatmap,
                no_trails: self.run_flags.no_trails,
            },
            render,
        )?;

        fresh.render_settings.overlay_preset = overlay;
        fresh.render_settings.trails_enabled = trails_on;
        fresh.render_settings.hud_visible = hud_on;
        fresh.render_settings.heatmap_enabled = heatmap_on;
        apply_overlay_preset(&mut fresh.render_settings);
        fresh.clock.time_warp = self.clock.time_warp;
        fresh.show_help = self.show_help;

        *self = fresh;
        self.show_toast("scenario reloaded".into());
        Ok(())
    }

    fn confirm_scenario_menu(&mut self) -> Result<()> {
        let Some(path) = self.scenario_paths.get(self.scenario_menu_index).cloned() else {
            self.show_scenario_menu = false;
            return Ok(());
        };
        self.show_scenario_menu = false;
        let loaded = load(&path)?;
        let render = loaded.render.clone();
        let warp = self.clock.time_warp;
        let overlay = self.render_settings.overlay_preset;

        let mut fresh = Self::from_loaded(
            loaded,
            path,
            &RunArgs {
                scenario: self.scenario_path.clone(),
                headless: false,
                steps: 0,
                dt: None,
                integrator: crate::cli::IntegratorArg::Rk4,
                barnes_hut: false,
                theta: 0.7,
                no_heatmap: self.run_flags.no_heatmap,
                no_trails: self.run_flags.no_trails,
            },
            render,
        )?;
        fresh.render_settings.overlay_preset = overlay;
        apply_overlay_preset(&mut fresh.render_settings);
        fresh.clock.time_warp = warp;
        *self = fresh;
        Ok(())
    }

    fn scenario_menu_up(&mut self) {
        if self.scenario_paths.is_empty() {
            return;
        }
        if self.scenario_menu_index == 0 {
            self.scenario_menu_index = self.scenario_paths.len() - 1;
        } else {
            self.scenario_menu_index -= 1;
        }
    }

    fn scenario_menu_down(&mut self) {
        if self.scenario_paths.is_empty() {
            return;
        }
        self.scenario_menu_index = (self.scenario_menu_index + 1) % self.scenario_paths.len();
    }

    fn show_toast(&mut self, message: String) {
        self.toast_message = Some(message);
        self.toast_until = Some(Instant::now() + Duration::from_secs(3));
    }

    fn expire_toast(&mut self) {
        if let Some(until) = self.toast_until {
            if Instant::now() >= until {
                self.toast_message = None;
                self.toast_until = None;
            }
        }
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
        self.heatmap.invalidate();
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
        self.heatmap.invalidate();
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

fn build_render_settings(
    render: &RenderConfig,
    args: &RunArgs,
    overlay_preset: OverlayPreset,
) -> RenderSettings {
    let mut settings = RenderSettings {
        trails_enabled: !args.no_trails,
        trail_sample_every: render.trail_sample_every,
        hud_visible: true,
        heatmap_enabled: render.heatmap_enabled && !args.no_heatmap,
        heatmap_sample_divisor: render.heatmap_sample_divisor,
        show_com_marker: render.show_com_marker,
        show_energy_diagnostics: true,
        show_momentum_diagnostics: false,
        overlay_preset,
        kitty_enabled: render.kitty_enabled,
        kitty_mode: render.kitty_mode.clone(),
    };
    apply_overlay_preset(&mut settings);
    settings
}

fn apply_overlay_preset(settings: &mut RenderSettings) {
    match settings.overlay_preset {
        OverlayPreset::Standard => {
            settings.show_com_marker = false;
            settings.show_energy_diagnostics = true;
            settings.show_momentum_diagnostics = false;
        }
        OverlayPreset::Science => {
            settings.heatmap_enabled = true;
            settings.show_com_marker = true;
            settings.show_energy_diagnostics = true;
            settings.show_momentum_diagnostics = true;
        }
        OverlayPreset::Minimal => {
            settings.heatmap_enabled = false;
            settings.show_com_marker = false;
            settings.show_energy_diagnostics = false;
            settings.show_momentum_diagnostics = false;
        }
        OverlayPreset::Full => {
            settings.heatmap_enabled = true;
            settings.show_com_marker = true;
            settings.show_energy_diagnostics = true;
            settings.show_momentum_diagnostics = true;
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
