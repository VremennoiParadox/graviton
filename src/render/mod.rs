//! Terminal rendering: camera, canvas, trails, heatmap, HUD.

pub mod camera;
pub mod canvas;
pub mod colors;
pub mod heatmap;
pub mod hud;
pub mod kitty;
pub mod trails;

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::render::canvas::SimulationCanvas;
use crate::render::heatmap::HeatmapBuildContext;
use crate::render::colors::body_color;

/// Draw the full frame for the interactive application.
pub fn draw(frame: &mut Frame<'_>, app: &mut App) {
    let area = frame.area();

    if area.width < 20 || area.height < 8 {
        frame.render_widget(
            Paragraph::new("Terminal too small — resize to at least 20×8."),
            area,
        );
        return;
    }

    if app.render_settings.hud_visible {
        crate::render::hud::render_hud(frame, app, area);
    }

    let sim_area = crate::render::hud::simulation_area(area, app.render_settings.hud_visible);
    draw_simulation(frame, app, sim_area);

    if let Some(msg) = &app.toast_message {
        crate::render::hud::render_toast(frame, area, msg);
    }

    if app.show_scenario_menu {
        crate::render::hud::render_scenario_menu(frame, app, area);
    }

    if app.show_help {
        crate::render::hud::render_help(frame, area);
    }

    if app.scheduler.overloaded {
        let warn = Paragraph::new("simulation overloaded; reducing time warp recommended");
        frame.render_widget(
            warn,
            Rect::new(area.x, area.bottom().saturating_sub(2), area.width, 1),
        );
    }
}

fn draw_simulation(frame: &mut Frame<'_>, app: &mut App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(app.simulation_title());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut canvas = SimulationCanvas::new(inner.width, inner.height);
    let camera = &app.camera;

    if app.render_settings.heatmap_enabled {
        let positions: Vec<_> = app.system.bodies.iter().map(|b| b.position_m).collect();
        let heat_ctx = HeatmapBuildContext {
            camera,
            bodies: &app.system.bodies,
            positions: &positions,
            softening_m: app.system.settings.softening_m,
            width: inner.width,
            height: inner.height,
            sample_divisor: app.render_settings.heatmap_sample_divisor,
            body_count: app.system.bodies.len(),
            fps: app.fps,
        };
        if app.heatmap.maybe_rebuild(&heat_ctx) {
            app.heatmap.plot_onto(|x, y, ch, rgb| {
                canvas.plot_heatmap_cell(x, y, ch, rgb);
            });
        }
    }

    if app.render_settings.trails_enabled {
        for (i, body) in app.system.bodies.iter().enumerate() {
            let base = body_color(body);
            let projected: Vec<_> = app.trails[i].points().map(|p| camera.project(*p)).collect();
            canvas.plot_trail(camera, projected.into_iter(), base);
        }
    }

    if app.render_settings.show_com_marker {
        let com = app.current_diagnostics.center_of_mass_m;
        canvas.plot_com_marker(camera, camera.project(com));
    }

    for (i, body) in app.system.bodies.iter().enumerate() {
        let world = camera.project(body.position_m);
        let selected = app.selected_body == Some(i);
        canvas.plot_body(camera, world, body, selected);
    }

    canvas.draw_into(|x, y, span| {
        let rect = Rect::new(inner.x + x, inner.y + y, 1, 1);
        frame.render_widget(Paragraph::new(span), rect);
    });

    crate::render::kitty::maybe_render_density_panel(
        app.render_settings.kitty_enabled,
        &app.render_settings.kitty_mode,
    );
}
