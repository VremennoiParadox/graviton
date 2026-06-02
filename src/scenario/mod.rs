//! TOML scenario loading, validation, and conversion to SI simulation state.

pub mod loader;
pub mod schema;
pub mod validate;

use std::path::{Path, PathBuf};

use crate::error::{GravitonError, Result};

/// Discover `.toml` scenario files in a directory (sorted by path).
pub fn discover_scenarios(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| GravitonError::Io {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| GravitonError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

pub use loader::{load, LoadedScenario, RenderConfig};
