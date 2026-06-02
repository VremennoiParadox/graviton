//! Optional Kitty graphics protocol (stub — Unicode renderer remains default).

/// Kitty-related settings from scenario TOML.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KittySettings {
    pub enabled: bool,
    pub mode: String,
}

impl Default for KittySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "density-panel".into(),
        }
    }
}

/// Whether the environment looks like Kitty (for future high-res panels).
#[must_use]
pub fn is_kitty_terminal() -> bool {
    std::env::var("TERM")
        .map(|t| t.contains("kitty"))
        .unwrap_or(false)
        || std::env::var("KITTY_WINDOW_ID").is_ok()
}

/// Placeholder for future density-panel rendering; no-op in Phase 4.
pub fn maybe_render_density_panel(enabled: bool, mode: &str) {
    if enabled && is_kitty_terminal() {
        let _ = mode;
        // Phase 4: stub only. Core simulation uses ratatui Unicode cells.
    }
}
