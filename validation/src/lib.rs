//! Validation utilities for biomeOS
//!
//! This module provides shared utilities for VM provisioning and federation validation.

pub mod vm_types;
pub mod deployment;
pub mod capabilities;
pub mod primal_startup;
pub mod mdns_validation;

use anyhow::{Context, Result};
use std::path::PathBuf;

pub use vm_types::{Topology, VmConfig, VmType};
pub use deployment::{BiomeOsDeployment, DeployedVm};
pub use capabilities::{Capability, CapabilityProfile, PrimalBinary};
pub use primal_startup::{DiscoveredPrimal, PrimalMatch, PrimalStartup, StartedPrimal};
pub use mdns_validation::{DiscoveredService, MdnsValidator, ValidationResult};

/// Get the path to the agentReagents template
#[must_use]
pub fn get_template_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("validation/ should have a parent")
        .parent()
        .expect("biomeOS should have a parent")
        .parent()
        .expect("phase2 should have a parent")
        .join("primalTools")
        .join("agentReagents")
        .join("images")
        .join("templates")
        .join("rustdesk-ubuntu-22.04-template.qcow2")
}

/// Load SSH public key from user's home directory
pub fn load_ssh_public_key() -> Result<String> {
    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    let ssh_key_path = home_dir.join(".ssh").join("id_rsa.pub");
    
    std::fs::read_to_string(&ssh_key_path).with_context(|| {
        format!("Failed to read SSH public key from {}", ssh_key_path.display())
    })
}

/// Print a formatted header
pub fn print_header(title: &str) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  {:<57} ║", title);
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
}

/// Print a formatted section
pub fn print_section(title: &str) {
    println!("════════════════════════════════════════════════════════════");
    println!("{}", title);
    println!("════════════════════════════════════════════════════════════");
    println!();
}

