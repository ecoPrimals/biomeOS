//! CLI Command Handlers
//!
//! This module organizes CLI command implementations into focused sub-modules
//! to improve maintainability and reduce the size of the main CLI file.

pub mod discover;
pub mod deploy;
pub mod health;
pub mod monitor;
pub mod utils;

// Re-export command handlers
pub use discover::handle_discover;
pub use deploy::{handle_deploy, handle_create};
pub use health::{handle_health, handle_probe, handle_scan, handle_status};
pub use monitor::{handle_monitor, handle_dashboard, handle_logs, handle_exec, handle_scale};
pub use utils::{create_spinner, parse_capabilities, display_results}; 