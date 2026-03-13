// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! biomeOS Development Tools
//! 
//! Pure Rust tooling for biomeOS development, testing, and ecosystem management.
//! Eliminates shell scripts in favor of "Rust until the very edge" philosophy.

#![warn(missing_docs)]

use anyhow::Result;
use std::process::{Command, Stdio};
use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tracing::{info, warn, error};

pub mod integration;
pub mod testing;
pub mod demos;
pub mod health;

/// Execute a command and return its output
pub async fn execute_command(cmd: &str, args: &[&str], working_dir: Option<&Path>) -> Result<String> {
    let mut command = AsyncCommand::new(cmd);
    command.args(args);
    
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    
    info!("Executing: {} {}", cmd, args.join(" "));
    
    let output = command.output().await?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Command failed: {}", stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }
}

/// Check if a binary exists in PATH
pub fn binary_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Print a formatted status message
pub fn print_status(icon: &str, title: &str, message: &str) {
    println!("{} {}: {}", icon, title, message);
}

/// Print a section header
pub fn print_section(title: &str) {
    println!("\n🎯 {}", title);
    println!("{}", "=".repeat(title.len() + 3));
}

/// Print a success message
pub fn print_success(message: &str) {
    print_status("✅", "SUCCESS", message);
}

/// Print an error message
pub fn print_error(message: &str) {
    print_status("❌", "ERROR", message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    print_status("⚠️", "WARNING", message);
}

/// Print an info message
pub fn print_info(message: &str) {
    print_status("ℹ️", "INFO", message);
} 