//! Mouse event mapping: select, pan (drag), zoom (scroll).

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Mouse-derived commands.
#[derive(Debug, Clone, Copy)]
pub enum MouseCommand {
    SelectAt { col: u16, row: u16 },
    DragTo { col: u16, row: u16 },
    DragEnd,
    ZoomAt { col: u16, row: u16, zoom_in: bool },
}

pub fn map_mouse(event: MouseEvent) -> Option<MouseCommand> {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => Some(MouseCommand::SelectAt {
            col: event.column,
            row: event.row,
        }),
        MouseEventKind::Drag(MouseButton::Left) => Some(MouseCommand::DragTo {
            col: event.column,
            row: event.row,
        }),
        MouseEventKind::Up(MouseButton::Left) => Some(MouseCommand::DragEnd),
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
