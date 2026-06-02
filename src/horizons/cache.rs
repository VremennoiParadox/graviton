//! On-disk cache for raw HORIZONS responses.

use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::error::{GravitonError, Result};
use crate::horizons::client::HorizonsResponse;

/// Resolve `~/.cache/graviton/horizons/` (via `directories` crate).
#[must_use]
pub fn cache_root() -> PathBuf {
    ProjectDirs::from("dev", "graviton", "graviton")
        .map(|dirs| dirs.cache_dir().join("horizons"))
        .unwrap_or_else(|| PathBuf::from(".cache/graviton/horizons"))
}

pub fn raw_cache_path(command_id: &str, date: &str, center: &str) -> PathBuf {
    let safe_center = center.replace('@', "_at_");
    cache_root()
        .join("raw")
        .join(format!("{command_id}_{date}_{safe_center}.json"))
}

pub fn read_raw_cache(path: &Path) -> Result<HorizonsResponse> {
    let text = fs::read_to_string(path).map_err(|e| GravitonError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_json::from_str(&text).map_err(|e| {
        GravitonError::Horizons(crate::error::HorizonsError::Http(format!(
            "invalid cached JSON at {}: {e}",
            path.display()
        )))
    })
}

pub fn write_raw_cache(path: &Path, response: &HorizonsResponse) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| GravitonError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    let json = serde_json::to_string_pretty(response).map_err(|e| {
        GravitonError::Horizons(crate::error::HorizonsError::Http(e.to_string()))
    })?;
    fs::write(path, json).map_err(|e| GravitonError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}
