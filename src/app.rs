//! Interactive application state and main loop.
//!
//! Owns scenario state, camera, UI toggles, selected body, and timing.
//! Coordinates input, physics stepping, and rendering (implemented in Phase 2).

/// Top-level interactive application state.
#[derive(Debug, Default)]
pub struct App {
    /// Set by the input loop when the user quits (Phase 2+).
    #[allow(dead_code)]
    pub should_quit: bool,
}

impl App {
    /// Create a new application instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
