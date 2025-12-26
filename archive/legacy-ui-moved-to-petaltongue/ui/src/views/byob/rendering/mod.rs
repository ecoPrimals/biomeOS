//! BYOB UI Rendering Modules
//!
//! This module organizes the BYOB rendering functions into focused sub-modules
//! for better maintainability and to keep files under the 2000 line limit.

pub mod deployment;
pub mod manifest;
pub mod niche;
pub mod team;

// Re-export all render functions for compatibility
pub use deployment::{render_completion, render_deployment};
pub use manifest::{render_manifest_generation, render_yaml_editing};
pub use niche::{render_niche_customization, render_niche_selection};
pub use team::render_team_selection;
