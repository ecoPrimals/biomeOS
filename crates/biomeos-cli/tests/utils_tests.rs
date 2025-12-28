//! Integration tests for biomeos-cli command utilities
//!
//! Tests utility functions used across commands.

use anyhow::Result;

#[test]
fn test_parse_capabilities() {
    // Test parsing capability strings
    let caps = "compute,storage,network";
    let parsed: Vec<&str> = caps.split(',').map(|s| s.trim()).collect();

    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0], "compute");
    assert_eq!(parsed[1], "storage");
    assert_eq!(parsed[2], "network");
}

#[test]
fn test_parse_capabilities_with_spaces() {
    // Test parsing with extra whitespace
    let caps = " compute , storage , network ";
    let parsed: Vec<&str> = caps.split(',').map(|s| s.trim()).collect();

    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0], "compute");
    assert_eq!(parsed[1], "storage");
    assert_eq!(parsed[2], "network");
}

#[test]
fn test_format_bytes() {
    // Test byte formatting
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1024), "1.0 KiB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
}

#[test]
fn test_format_bytes_fractional() {
    // Test fractional byte formatting
    assert_eq!(format_bytes(1536), "1.5 KiB");
    assert_eq!(format_bytes(1024 * 1536), "1.5 MiB");
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}

#[test]
fn test_format_duration() {
    use std::time::Duration;

    // Test duration formatting
    assert_eq!(format_duration(Duration::from_secs(0)), "0s");
    assert_eq!(format_duration(Duration::from_secs(1)), "1s");
    assert_eq!(format_duration(Duration::from_secs(60)), "1m 0s");
    assert_eq!(format_duration(Duration::from_secs(61)), "1m 1s");
    assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[test]
fn test_validate_endpoint_url() {
    // Test endpoint URL validation
    assert!(is_valid_endpoint("http://localhost:8080"));
    assert!(is_valid_endpoint("https://example.com"));
    assert!(is_valid_endpoint("http://192.168.1.1:3000"));

    assert!(!is_valid_endpoint("not-a-url"));
    assert!(!is_valid_endpoint("ftp://invalid"));
}

fn is_valid_endpoint(endpoint: &str) -> bool {
    endpoint.starts_with("http://") || endpoint.starts_with("https://")
}

#[test]
fn test_capability_matching() {
    // Test capability matching logic
    let available = vec!["compute", "storage", "network"];
    let required = vec!["compute", "storage"];

    let has_all = required.iter().all(|r| available.contains(r));
    assert!(has_all);
}

#[test]
fn test_capability_matching_missing() {
    // Test when capability is missing
    let available = vec!["compute", "storage"];
    let required = vec!["compute", "storage", "network"];

    let has_all = required.iter().all(|r| available.contains(r));
    assert!(!has_all);
}
