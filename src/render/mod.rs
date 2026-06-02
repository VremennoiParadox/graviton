//! Terminal rendering: camera, canvas, trails, HUD.

pub mod camera;
pub mod canvas;
pub mod colors;
pub mod hud;
pub mod trails;

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::render::canvas::SimulationCanvas;
use crate::render::colors::body_color;

/// Draw the full frame for the interactive application.
pub fn draw(frame: &mut Frame<'_>, app: &App) {
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

fn draw_simulation(frame: &mut Frame<'_>, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(app.system.scenario_name.as_str());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut canvas = SimulationCanvas::new(inner.width, inner.height);
    let camera = &app.camera;

    if app.render_settings.trails_enabled {
        for (i, body) in app.system.bodies.iter().enumerate() {
            let base = body_color(body);
            let projected: Vec<_> = app.trails[i].points().map(|p| camera.project(*p)).collect();
            canvas.plot_trail(camera, projected.into_iter(), base);
        }
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
}
