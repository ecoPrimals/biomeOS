//! BYOB UI Rendering Modules
//!
//! This module organizes the BYOB rendering functions into focused sub-modules
//! for better maintainability and to keep files under the 2000 line limit.

pub mod team;
pub mod niche;
pub mod manifest;
pub mod deployment;

// Re-export all render functions for compatibility
pub use team::render_team_selection;
pub use niche::{render_niche_selection, render_niche_customization};
pub use manifest::{render_manifest_generation, render_yaml_editing};
pub use deployment::{render_deployment, render_completion}; 