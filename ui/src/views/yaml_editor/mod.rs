//! YAML Editor View
//!
//! Advanced YAML editor for biome.yaml files with syntax highlighting, validation,
//! template loading, and live preview capabilities.

pub mod editor;
pub mod highlighting;
pub mod integration;
pub mod preview;
pub mod structured;
pub mod templates;
pub mod types;
pub mod validation;

// Re-export main types
pub use editor::*;
pub use types::*;
