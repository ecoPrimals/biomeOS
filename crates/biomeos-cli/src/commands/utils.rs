// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
#[expect(clippy::expect_used, reason = "static template string is always valid")]
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["🌱", "🌿", "🍃", "🌳", "🌲", "🌴", "🎋", "🎍"])
            .template("{spinner:.green} {msg}")
            .expect("valid progress bar template; static format string"),
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
            "No valid capabilities found in: {caps_str}"
        ));
    }

    Ok(capabilities)
}

/// Display results in a formatted manner
#[expect(
    clippy::implicit_hasher,
    reason = "HashMap with default hasher is sufficient for display"
)]
pub async fn display_results(
    title: &str,
    results: &HashMap<String, Value>,
    show_details: bool,
) -> Result<()> {
    if results.is_empty() {
        println!("📋 {title}: No results");
        return Ok(());
    }

    println!("📋 {} ({} items):", title, results.len());
    println!();

    for (key, value) in results {
        println!("🔹 {key}");

        if show_details {
            if let Ok(pretty) = serde_json::to_string_pretty(value) {
                // Indent the JSON output
                for line in pretty.lines() {
                    println!("   {line}");
                }
            } else {
                println!("   {value}");
            }
        } else {
            // Show just a summary
            if let Some(status) = value.get("status") {
                println!("   Status: {status}");
            }
            if let Some(health) = value.get("health") {
                println!("   Health: {health}");
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
        format!("{secs}s")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capabilities_single() {
        let caps = parse_capabilities("storage").expect("single cap should parse");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].category, "general");
        assert_eq!(caps[0].name, "storage");
        assert_eq!(caps[0].version, "1.0");
    }

    #[test]
    fn test_parse_capabilities_with_slash() {
        let caps = parse_capabilities("storage/file").expect("category/name should parse");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].category, "storage");
        assert_eq!(caps[0].name, "file");
        assert_eq!(caps[0].version, "1.0");
    }

    #[test]
    fn test_parse_capabilities_multiple() {
        let caps =
            parse_capabilities("storage,compute,security").expect("multiple caps should parse");
        assert_eq!(caps.len(), 3);
        assert_eq!(caps[0].name, "storage");
        assert_eq!(caps[1].name, "compute");
        assert_eq!(caps[2].name, "security");
    }

    #[test]
    fn test_parse_capabilities_with_spaces() {
        let caps = parse_capabilities("storage , compute").expect("spaces should trim");
        assert_eq!(caps.len(), 2);
        assert_eq!(caps[0].name, "storage");
        assert_eq!(caps[1].name, "compute");
    }

    #[test]
    fn test_parse_capabilities_empty_fails() {
        let result = parse_capabilities("");
        assert!(result.is_err(), "Empty string should fail");
        let result2 = parse_capabilities("  ,  ,  ");
        assert!(result2.is_err(), "Only whitespace should fail");
    }

    #[test]
    fn test_parse_capabilities_mixed_format() {
        let caps = parse_capabilities("general/foo,bar").expect("mixed format should parse");
        assert_eq!(caps.len(), 2);
        assert_eq!(caps[0].category, "general");
        assert_eq!(caps[0].name, "foo");
        assert_eq!(caps[1].category, "general");
        assert_eq!(caps[1].name, "bar");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1m 0s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h 0m");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(512 * 1024), "512.0 KB");
    }
}
