//! TOML scenario loading, validation, and conversion to SI simulation state.

pub mod loader;
pub mod schema;
pub mod validate;

pub use loader::{load, LoadedScenario, RenderConfig};
