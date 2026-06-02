//! Mouse event mapping (Phase 4 expands this; basic pan/zoom/select in Phase 2).

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Mouse-derived commands.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum MouseCommand {
    SelectAt { col: u16, row: u16 },
    PanBy { dx: f64, dy: f64 },
    ZoomAt { col: u16, row: u16, zoom_in: bool },
}

pub fn map_mouse(event: MouseEvent) -> Option<MouseCommand> {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => Some(MouseCommand::SelectAt {
            col: event.column,
            row: event.row,
        }),
        MouseEventKind::ScrollUp => Some(MouseCommand::ZoomAt {
            col: event.column,
            row: event.row,
            zoom_in: true,
        }),
        MouseEventKind::ScrollDown => Some(MouseCommand::ZoomAt {
            col: event.column,
            row: event.row,
            zoom_in: false,
        }),
        _ => None,
    }
}
