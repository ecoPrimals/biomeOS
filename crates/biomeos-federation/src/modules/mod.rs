//! Federation CLI Modules
//!
//! Modular organization of federation CLI functionality

pub mod config;
pub mod manifest;
pub mod status;

pub use config::{FederationConfig, load_config, validate_config};
pub use manifest::{find_manifest, validate_manifest, deploy_manifest, list_manifests};
pub use status::show_status; 