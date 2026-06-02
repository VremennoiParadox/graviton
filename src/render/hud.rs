//! Status bar, selected-body panel, help overlay, and diagnostics.

use glam::DVec3;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::App;
use crate::physics::constants::{DAY, G};
use crate::physics::field::gravitational_field_magnitude;
use crate::physics::units::meters_to_au;
use crate::render::colors::log_field_intensity;

/// Draw HUD chrome around the simulation viewport.
pub fn render_hud(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    if !app.render_settings.hud_visible {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(if app.render_settings.show_energy_diagnostics {
                2
            } else {
                1
            }),
        ])
        .split(area);

    render_status_bar(frame, app, chunks[0]);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(36)])
        .split(chunks[1]);

    render_selected_panel(frame, app, main[1]);

    render_footer(frame, app, chunks[2]);
}

pub fn simulation_area(area: Rect, hud_visible: bool) -> Rect {
    if !hud_visible {
        return area;
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(36)])
        .split(chunks[1]);
    main[0]
}

fn render_status_bar(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let paused = if app.clock.paused {
        "paused"
    } else {
        "running"
    };
    let proj = match app.camera.projection {
        crate::render::camera::Projection::Xy => "XY",
        crate::render::camera::Projection::Xz => "XZ",
        crate::render::camera::Projection::Yz => "YZ",
    };
    let scenario = app
        .scenario_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("scenario");
    let heat = if app.render_settings.heatmap_enabled {
        "heatmap"
    } else {
        "no heatmap"
    };
    let line = Line::from(vec![
        Span::styled("graviton", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::raw(scenario),
        Span::raw(" | "),
        Span::raw(paused),
        Span::raw(" | "),
        Span::raw(format!("warp {:.1}x", app.clock.time_warp)),
        Span::raw(" | "),
        Span::raw(format!("dt {:.0}s", app.system.settings.dt_s)),
        Span::raw(" | "),
        Span::raw(if app.system.settings.use_barnes_hut {
            "BH"
        } else {
            "RK4/direct"
        }),
        Span::raw(" | "),
        Span::raw(proj),
        Span::raw(" | "),
        Span::raw(heat),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_footer(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let drift = app
        .current_diagnostics
        .energy_drift_fraction(app.initial_diagnostics.total_energy_j);
    let mut parts = vec![
        format!("energy drift: {drift:.4e}"),
        format!("bodies: {}", app.system.bodies.len()),
        format!("fps: {:.0}", app.fps),
        "? help".into(),
    ];

    if let Some(stats) = app.barnes_hut_tree_stats {
        parts.push(format!("octree: {} nodes depth {}", stats.nodes, stats.max_depth));
    }

    if app.render_settings.show_momentum_diagnostics {
        let p = app.current_diagnostics.linear_momentum_kg_mps.length();
        parts.insert(1, format!("|p|: {p:.4e} kg·m/s"));
    }

    let mut line = Line::from(parts.join(" | "));

    if app.render_settings.show_energy_diagnostics {
        let spark = energy_sparkline(&app.energy_history.samples);
        line.spans.push(Span::raw(" | "));
        line.spans.push(Span::styled(
            spark,
            Style::default().fg(Color::Yellow),
        ));
    }

    frame.render_widget(Paragraph::new(line), area);
}

fn energy_sparkline(samples: &std::collections::VecDeque<f64>) -> String {
    if samples.is_empty() {
        return String::new();
    }
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let min = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let max = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1e-20);
    samples
        .iter()
        .map(|v| {
            let t = ((*v - min) / span).clamp(0.0, 1.0);
            let idx = (t * 7.0).round() as usize;
            BARS[idx.min(7)]
        })
        .collect()
}

fn render_selected_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let block = Block::default()
        .title("Selected body")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(idx) = app.selected_body else {
        frame.render_widget(
            Paragraph::new("Tab: next body\nShift+Tab: prev\nClick: select"),
            inner,
        );
        return;
    };

    let body = &app.system.bodies[idx];
    let speed = body.velocity_mps.length();
    let r = body.position_m.length();
    let r_au = meters_to_au(r);

    let positions: Vec<DVec3> = app.system.bodies.iter().map(|b| b.position_m).collect();
    let field_g = gravitational_field_magnitude(
        body.position_m,
        &app.system.bodies,
        &positions,
        app.system.settings.softening_m,
    );
    let log_g = log_field_intensity(field_g);

    let mut lines = vec![
        Line::from(format!("name: {}", body.name)),
        Line::from(format!("class: {:?}", body.class)),
        Line::from(format!("mass: {:.6e} kg", body.mass_kg)),
        Line::from(format!("radius: {:.3e} m", body.radius_m)),
        Line::from(format!(
            "position: ({:.3e}, {:.3e}, {:.3e}) m",
            body.position_m.x, body.position_m.y, body.position_m.z
        )),
        Line::from(format!(
            "velocity: ({:.3e}, {:.3e}, {:.3e}) m/s",
            body.velocity_mps.x, body.velocity_mps.y, body.velocity_mps.z
        )),
        Line::from(format!(
            "speed: {:.3} m/s ({:.3} km/s)",
            speed,
            speed / 1000.0
        )),
        Line::from(format!("|r|: {:.6} AU", r_au)),
        Line::from(format!("|g|: {field_g:.4e} m/s² (log₁₀ {log_g:.2})")),
        Line::from(format!("trail points: {}", app.trails[idx].len())),
    ];

    if let Some((peri, apo)) = estimate_periapsis_apoapsis_m(app, idx) {
        lines.push(Line::from(format!(
            "periapsis: {:.6} AU",
            meters_to_au(peri)
        )));
        lines.push(Line::from(format!(
            "apoapsis: {:.6} AU",
            meters_to_au(apo)
        )));
    }

    if let Some(period_s) = estimate_orbital_period_s(app, idx) {
        lines.push(Line::from(format!(
            "period (est.): {:.3} days",
            period_s / DAY
        )));
    } else {
        lines.push(Line::from("orbit: unbound or strongly perturbed"));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn estimate_periapsis_apoapsis_m(app: &App, body_idx: usize) -> Option<(f64, f64)> {
    let body = &app.system.bodies[body_idx];
    let primary = app
        .system
        .bodies
        .iter()
        .max_by(|a, b| a.mass_kg.partial_cmp(&b.mass_kg).unwrap())?;
    if primary.id == body.id {
        return None;
    }
    let r_vec = body.position_m - primary.position_m;
    let v_rel = body.velocity_mps - primary.velocity_mps;
    let r = r_vec.length();
    if r <= 0.0 {
        return None;
    }
    let mu = G * (primary.mass_kg + body.mass_kg);
    let epsilon = 0.5 * v_rel.length_squared() - mu / r;
    if epsilon >= 0.0 {
        return None;
    }
    let a = -mu / (2.0 * epsilon);
    let h = r_vec.cross(v_rel).length();
    let e = (1.0 + 2.0 * epsilon * h * h / (mu * mu)).sqrt();
    if e >= 1.0 {
        return None;
    }
    let peri = a * (1.0 - e);
    let apo = a * (1.0 + e);
    Some((peri.max(0.0), apo))
}

fn estimate_orbital_period_s(app: &App, body_idx: usize) -> Option<f64> {
    let body = &app.system.bodies[body_idx];
    if app.system.bodies.len() < 2 {
        return None;
    }
    let primary = app
        .system
        .bodies
        .iter()
        .max_by(|a, b| a.mass_kg.partial_cmp(&b.mass_kg).unwrap())?;
    if primary.id == body.id {
        return None;
    }
    let r = (body.position_m - primary.position_m).length();
    if r <= 0.0 {
        return None;
    }
    let mu = G * (primary.mass_kg + body.mass_kg);
    let v_rel = (body.velocity_mps - primary.velocity_mps).length();
    let epsilon = 0.5 * v_rel * v_rel - mu / r;
    if epsilon >= 0.0 {
        return None;
    }
    let a = -mu / (2.0 * epsilon);
    if a <= 0.0 {
        return None;
    }
    Some(2.0 * std::f64::consts::PI * (a.powi(3) / mu).sqrt())
}

pub fn render_help(frame: &mut ratatui::Frame<'_>, area: Rect) {
    let text = vec![
        Line::from("graviton controls (Phase 4)"),
        Line::from(""),
        Line::from("q/Esc  quit    Space  pause    r  reset    Shift+R  reload"),
        Line::from("+/-  zoom    0  reset zoom    arrows/hjkl  pan"),
        Line::from("f  follow    F  frame all    1/2/3  XY/XZ/YZ projection"),
        Line::from("Tab/Shift+Tab  select    click  select    drag  pan"),
        Line::from("scroll  zoom at cursor"),
        Line::from(""),
        Line::from("g  heatmap    c  COM marker    e  energy    p  momentum"),
        Line::from("o  cycle overlays    T  trails    H  HUD"),
        Line::from("s  scenario menu    v  validate    B  Barnes–Hut    b  tree debug"),
        Line::from(". ,  time warp    [ ]  dt"),
    ];
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(
        Paragraph::new(text).block(block),
        centered_rect(62, 48, area),
    );
}

pub fn render_scenario_menu(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .scenario_paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            let marker = if i == app.scenario_menu_index {
                "▸ "
            } else {
                "  "
            };
            ListItem::new(format!("{marker}{name}"))
        })
        .collect();

    let block = Block::default()
        .title("Scenario switcher (Enter load, Esc cancel)")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    let list = List::new(items).block(block);
    frame.render_widget(list, centered_rect(50, 60, area));
}

pub fn render_toast(frame: &mut ratatui::Frame<'_>, area: Rect, message: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));
    let rect = Rect::new(
        area.x + 2,
        area.bottom().saturating_sub(4),
        area.width.saturating_sub(4).min(80),
        3,
    );
    frame.render_widget(Paragraph::new(message).block(block), rect);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
