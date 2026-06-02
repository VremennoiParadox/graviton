//! Typed errors for graviton modules.
//!
//! Use [`GravitonError`] inside the library and convert to `anyhow::Error` at the CLI boundary.

use std::path::PathBuf;

/// Application-wide error type.
#[derive(Debug, thiserror::Error)]
pub enum GravitonError {
    #[error("scenario error: {0}")]
    Scenario(#[from] ScenarioError),

    #[error("physics error: {0}")]
    Physics(#[from] PhysicsError),

    #[error("HORIZONS error: {0}")]
    Horizons(#[from] HorizonsError),

    #[error("render error: {0}")]
    Render(#[from] RenderError),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error: {0}")]
    StdIo(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Scenario loading and validation failures.
#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("unsupported schema version: {found}, expected {expected}")]
    UnsupportedSchema { found: u32, expected: u32 },

    #[error("body `{id}` has non-positive mass")]
    NonPositiveMass { id: String },

    #[error("body `{id}` has duplicate id")]
    DuplicateId { id: String },

    #[error("physics.dt must be greater than zero")]
    InvalidTimeStep,

    #[error("invalid scenario:\n{}", format_validation_messages(.0))]
    Validation(Vec<String>),

    #[error("missing required unit scale `{field}`")]
    MissingUnitScale { field: String },

    #[error("unknown {kind} unit `{value}`")]
    UnknownUnit { kind: String, value: String },

    #[error("unknown body class `{class}`")]
    UnknownBodyClass { class: String },

    #[error("failed to read scenario: {0}")]
    Read(#[from] std::io::Error),

    #[error("failed to parse TOML: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Physics engine failures.
#[derive(Debug, thiserror::Error)]
pub enum PhysicsError {
    #[error("simulation produced non-finite state")]
    NonFiniteState,

    #[error("integrator `{0}` is not implemented yet")]
    NotImplemented(String),
}

/// NASA JPL HORIZONS client and parser failures.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum HorizonsError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("response did not contain vector table between $$SOE and $$EOE")]
    MissingVectorTable,

    #[error("unknown units in HORIZONS response")]
    UnknownUnits,
}

/// Terminal rendering failures.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum RenderError {
    #[error("terminal too small: need at least {min_width}x{min_height}, got {width}x{height}")]
    TerminalTooSmall {
        min_width: u16,
        min_height: u16,
        width: u16,
        height: u16,
    },
}

/// Result alias used inside graviton modules.
pub type Result<T> = std::result::Result<T, GravitonError>;

fn format_validation_messages(messages: &[String]) -> String {
    messages
        .iter()
        .map(|m| format!("  - {m}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convenience for unimplemented subsystems during early phases.
#[allow(dead_code)]
pub fn not_implemented(feature: &str, phase: &str) -> GravitonError {
    GravitonError::Other(format!("{feature} is not implemented yet ({phase})"))
}

#[cfg(test)]
mod compile_tests {
    use super::*;

    #[test]
    fn error_variants_are_constructible() {
        let _ = ScenarioError::UnsupportedSchema {
            found: 0,
            expected: 1,
        };
        let _ = ScenarioError::NonPositiveMass { id: "x".into() };
        let _ = ScenarioError::DuplicateId { id: "x".into() };
        let _ = ScenarioError::InvalidTimeStep;
        let _ = ScenarioError::Validation(vec!["example".into()]);
        let _ = ScenarioError::MissingUnitScale {
            field: "distance_scale_m".into(),
        };
        let _ = ScenarioError::UnknownUnit {
            kind: "distance".into(),
            value: "foo".into(),
        };
        let _ = ScenarioError::UnknownBodyClass {
            class: "foo".into(),
        };
        let _ = PhysicsError::NonFiniteState;
        let _ = PhysicsError::NotImplemented("rk4".into());
        let _ = HorizonsError::Http("timeout".into());
        let _ = HorizonsError::MissingVectorTable;
        let _ = HorizonsError::UnknownUnits;
        let _ = GravitonError::StdIo(std::io::Error::from(std::io::ErrorKind::NotFound));
        let _ = RenderError::TerminalTooSmall {
            min_width: 80,
            min_height: 24,
            width: 10,
            height: 10,
        };
    }
}
