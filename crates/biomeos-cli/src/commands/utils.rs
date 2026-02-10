//! CLI Utility Functions
//!
//! Shared utility functions used across CLI command handlers.

use anyhow::Result;
use biomeos_primal_sdk::PrimalCapability;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Create a spinner with biomeOS styling
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["🌱", "🌿", "🍃", "🌳", "🌲", "🌴", "🎋", "🎍"])
            .template("{spinner:.green} {msg}")
            .expect("valid progress bar template"),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Parse comma-separated capabilities string into PrimalCapability vector
pub fn parse_capabilities(caps_str: &str) -> Result<Vec<PrimalCapability>> {
    let mut capabilities = Vec::new();

    for cap_str in caps_str.split(',') {
        let trimmed = cap_str.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse category/name format
        if let Some((category, name)) = trimmed.split_once('/') {
            capabilities.push(PrimalCapability {
                category: category.trim().to_string(),
                name: name.trim().to_string(),
                version: "1.0".to_string(),
                parameters: Vec::new(),
                performance: None,
            });
        } else {
            // Default to "general" category if no slash found
            capabilities.push(PrimalCapability {
                category: "general".to_string(),
                name: trimmed.to_string(),
                version: "1.0".to_string(),
                parameters: Vec::new(),
                performance: None,
            });
        }
    }

    if capabilities.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid capabilities found in: {}",
            caps_str
        ));
    }

    Ok(capabilities)
}

/// Display results in a formatted manner
pub async fn display_results(
    title: &str,
    results: &HashMap<String, Value>,
    show_details: bool,
) -> Result<()> {
    if results.is_empty() {
        println!("📋 {}: No results", title);
        return Ok(());
    }

    println!("📋 {} ({} items):", title, results.len());
    println!();

    for (key, value) in results {
        println!("🔹 {}", key);

        if show_details {
            if let Ok(pretty) = serde_json::to_string_pretty(value) {
                // Indent the JSON output
                for line in pretty.lines() {
                    println!("   {}", line);
                }
            } else {
                println!("   {}", value);
            }
        } else {
            // Show just a summary
            if let Some(status) = value.get("status") {
                println!("   Status: {}", status);
            }
            if let Some(health) = value.get("health") {
                println!("   Health: {}", health);
            }
        }
        println!();
    }

    Ok(())
}

/// Format duration for display
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Format bytes for display
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
