//! CLI Command Handlers
//!
//! This module organizes CLI command implementations into focused sub-modules
//! to improve maintainability and reduce the size of the main CLI file.

pub mod discover;
pub mod deploy;
pub mod health;
pub mod monitor;
pub mod utils;
pub mod chimera;
pub mod niche;

// Re-export command handlers
pub use discover::handle_discover;
pub use deploy::{handle_deploy, handle_create};
pub use health::{handle_health, handle_probe, handle_scan, handle_status};
pub use monitor::{handle_monitor, handle_dashboard, handle_logs, handle_exec, handle_scale};
pub use utils::{create_spinner, parse_capabilities, display_results};

// Chimera commands
pub use chimera::{handle_chimera_list, handle_chimera_show, handle_chimera_build};

// Niche commands
pub use niche::{handle_niche_list, handle_niche_show, handle_primal_list}; 