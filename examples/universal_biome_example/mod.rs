//! Universal Biome Example Modules
//!
//! This module contains the refactored universal biome example split into logical components.
//! Each module demonstrates different types of biome manifests and their characteristics.

pub mod web_app;
pub mod ai_ml;
pub mod database;
pub mod agnostic;
pub mod yaml_output;

pub use web_app::create_web_app_manifest;
pub use ai_ml::create_ai_ml_manifest;
pub use database::create_database_manifest;
pub use agnostic::demonstrate_agnostic_approach;
pub use yaml_output::example_yaml_output; 