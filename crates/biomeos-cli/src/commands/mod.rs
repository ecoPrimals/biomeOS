//! CLI Command Handlers
//!
//! This module organizes CLI command implementations into focused sub-modules
//! to improve maintainability and reduce the size of the main CLI file.

pub mod chimera;
pub mod deploy;
pub mod discover;
pub mod health;
pub mod monitor;
pub mod niche;
pub mod spore;
pub mod verify;
pub mod utils;

// Re-export command handlers
pub use deploy::{handle_create, handle_deploy};
pub use discover::handle_discover;
pub use health::{handle_health, handle_probe, handle_scan, handle_status};
pub use monitor::{handle_dashboard, handle_exec, handle_logs, handle_monitor, handle_scale};
pub use utils::{create_spinner, display_results, parse_capabilities};

// Chimera commands
pub use chimera::{handle_chimera_build, handle_chimera_list, handle_chimera_show};

// Niche commands
pub use niche::{handle_niche_list, handle_niche_show, handle_primal_list};

// Spore commands
pub use spore::{
    handle_spore_clone, handle_spore_create, handle_spore_info,
    handle_spore_list, handle_spore_verify,
};
