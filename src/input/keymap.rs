//! Keyboard event mapping to application commands.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Semantic commands produced by user input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppCommand {
    Quit,
    TogglePause,
    ToggleHelp,
    ResetSimulation,
    ReloadScenario,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    Pan { dx: f64, dy: f64 },
    FollowSelected,
    FrameAll,
    ProjectionXy,
    ProjectionXz,
    ProjectionYz,
    SelectNext,
    SelectPrevious,
    ToggleTrails,
    ToggleHud,
    ToggleHeatmap,
    ToggleEnergyDiagnostics,
    ToggleMomentumDiagnostics,
    ToggleComMarker,
    CycleOverlayPreset,
    OpenScenarioMenu,
    ValidateScenario,
    ToggleBarnesHut,
    ToggleBarnesHutDebug,
    ScenarioMenuConfirm,
    IncreaseTimeWarp,
    DecreaseTimeWarp,
    IncreaseDt,
    DecreaseDt,
    None,
}

pub fn map_key(event: KeyEvent) -> AppCommand {
    match event.code {
        KeyCode::Char('r') | KeyCode::Char('R')
            if event.modifiers.contains(KeyModifiers::SHIFT) =>
        {
            AppCommand::ReloadScenario
        }
        KeyCode::Char('q') | KeyCode::Esc => AppCommand::Quit,
        KeyCode::Char('?') => AppCommand::ToggleHelp,
        KeyCode::Char(' ') => AppCommand::TogglePause,
        KeyCode::Char('r') => AppCommand::ResetSimulation,
        KeyCode::Char('+') | KeyCode::Char('=') => AppCommand::ZoomIn,
        KeyCode::Char('-') => AppCommand::ZoomOut,
        KeyCode::Char('0') => AppCommand::ResetZoom,
        KeyCode::Char('f') => AppCommand::FollowSelected,
        KeyCode::Char('F') => AppCommand::FrameAll,
        KeyCode::Char('1') => AppCommand::ProjectionXy,
        KeyCode::Char('2') => AppCommand::ProjectionXz,
        KeyCode::Char('3') => AppCommand::ProjectionYz,
        KeyCode::Char('t') | KeyCode::Char('T') => AppCommand::ToggleTrails,
        KeyCode::Char('H') => AppCommand::ToggleHud,
        KeyCode::Char('g') => AppCommand::ToggleHeatmap,
        KeyCode::Char('e') => AppCommand::ToggleEnergyDiagnostics,
        KeyCode::Char('p') => AppCommand::ToggleMomentumDiagnostics,
        KeyCode::Char('c') => AppCommand::ToggleComMarker,
        KeyCode::Char('o') => AppCommand::CycleOverlayPreset,
        KeyCode::Char('s') => AppCommand::OpenScenarioMenu,
        KeyCode::Char('v') => AppCommand::ValidateScenario,
        KeyCode::Char('b') => AppCommand::ToggleBarnesHutDebug,
        KeyCode::Char('B') => AppCommand::ToggleBarnesHut,
        KeyCode::Char('.') => AppCommand::IncreaseTimeWarp,
        KeyCode::Char(',') => AppCommand::DecreaseTimeWarp,
        KeyCode::Char(']') => AppCommand::IncreaseDt,
        KeyCode::Char('[') => AppCommand::DecreaseDt,
        KeyCode::Up | KeyCode::Char('k') => AppCommand::Pan {
            dx: 0.0,
            dy: 1.0,
        },
        KeyCode::Down | KeyCode::Char('j') => AppCommand::Pan {
            dx: 0.0,
            dy: -1.0,
        },
        KeyCode::Left | KeyCode::Char('h') => AppCommand::Pan {
            dx: -1.0,
            dy: 0.0,
        },
        KeyCode::Right | KeyCode::Char('l') => AppCommand::Pan {
            dx: 1.0,
            dy: 0.0,
        },
        KeyCode::Tab if event.modifiers.contains(KeyModifiers::SHIFT) => {
            AppCommand::SelectPrevious
        }
        KeyCode::Enter => AppCommand::ScenarioMenuConfirm,
        KeyCode::Tab | KeyCode::BackTab => AppCommand::SelectNext,
        _ => AppCommand::None,
    }
}
