//! Status bar, selected-body panel, and help overlay.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::physics::constants::{DAY, G};
use crate::physics::units::meters_to_au;

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
            Constraint::Length(1),
        ])
        .split(area);

    render_status_bar(frame, app, chunks[0]);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(32)])
        .split(chunks[1]);

    render_selected_panel(frame, app, main[1]);
    // Simulation canvas drawn by caller into main[0]

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
        .constraints([Constraint::Min(10), Constraint::Length(32)])
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
        Span::raw("RK4"),
        Span::raw(" | "),
        Span::raw(proj),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_footer(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let drift = app
        .current_diagnostics
        .energy_drift_fraction(app.initial_diagnostics.total_energy_j);
    let line = Line::from(format!(
        "energy drift: {drift:.4e} | bodies: {} | fps: {:.0} | ? help",
        app.system.bodies.len(),
        app.fps
    ));
    frame.render_widget(Paragraph::new(line), area);
}

fn render_selected_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let block = Block::default()
        .title("Selected body")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(idx) = app.selected_body else {
        frame.render_widget(Paragraph::new("Tab: next body\nShift+Tab: prev"), inner);
        return;
    };

    let body = &app.system.bodies[idx];
    let speed = body.velocity_mps.length();
    let r = body.position_m.length();
    let r_au = meters_to_au(r);

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
        Line::from(format!("trail points: {}", app.trails[idx].len())),
    ];

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
        Line::from("Controls (Phase 2)"),
        Line::from("q/Esc  quit    Space  pause    r  reset"),
        Line::from("+/-  zoom    0  reset zoom    arrows/hjkl  pan"),
        Line::from("f  follow selected    F  frame all    1/2/3  projection"),
        Line::from("Tab/Shift+Tab  select body    T  trails    H  HUD"),
        Line::from(". ,  time warp    [ ]  adjust dt"),
    ];
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(
        Paragraph::new(text).block(block),
        centered_rect(60, 40, area),
    );
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
