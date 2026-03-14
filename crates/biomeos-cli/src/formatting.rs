// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Formatting utilities for CLI output
//!
//! Additional formatting functions: table, duration, file_size, percentage, timestamp.

use colored::*;

/// Format data as pretty-printed JSON
pub fn format_json_pretty<T: serde::Serialize>(data: &T) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|_| "Error serializing to JSON".to_string())
}

/// Format data as YAML
pub fn format_yaml<T: serde::Serialize>(data: &T) -> String {
    // Use serde_yaml for proper YAML formatting
    serde_yaml::to_string(data).unwrap_or_else(|_| "Error serializing to YAML".to_string())
}

/// Colorize a status string based on its value
pub fn colorize_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "healthy" | "ok" | "success" => status.green().to_string(),
        "degraded" | "warning" | "warn" => status.yellow().to_string(),
        "unhealthy" | "error" | "critical" => status.red().to_string(),
        "unknown" => status.bright_black().to_string(),
        _ => status.to_string(),
    }
}

/// Format data as a simple table
pub fn format_table<T: serde::Serialize>(data: &[T]) -> String {
    if data.is_empty() {
        return "No data to display".to_string();
    }

    // Convert to JSON first to get consistent structure
    let json_data = serde_json::to_value(data).unwrap_or_default();

    match json_data.as_array() {
        Some(items) if !items.is_empty() => {
            let mut result = String::new();

            // Get headers from first item
            if let Some(first) = items.first() {
                if let Some(obj) = first.as_object() {
                    let headers: Vec<String> = obj.keys().cloned().collect();
                    result.push_str(&headers.join("\t"));
                    result.push('\n');
                    result.push_str(&"-".repeat(headers.join("\t").len()));
                    result.push('\n');

                    // Add rows
                    for item in items {
                        if let Some(obj) = item.as_object() {
                            let row: Vec<String> = headers
                                .iter()
                                .map(|h| {
                                    obj.get(h)
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("N/A")
                                        .to_string()
                                })
                                .collect();
                            result.push_str(&row.join("\t"));
                            result.push('\n');
                        }
                    }
                }
            }
            result
        }
        _ => "Unable to format as table".to_string(),
    }
}

/// Format duration in human readable format
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}

/// Format file size in human readable format
pub fn format_file_size(bytes: u64) -> String {
    use biomeos_types::files::SIZE_UNITS;
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < SIZE_UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, SIZE_UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, SIZE_UNITS[unit_index])
    }
}

/// Format percentage with color coding
pub fn format_percentage(value: f64) -> String {
    let formatted = format!("{value:.1}%");
    if value >= 90.0 {
        formatted.red().to_string()
    } else if value >= 75.0 {
        formatted.yellow().to_string()
    } else {
        formatted.green().to_string()
    }
}

/// Format timestamp for CLI display
pub fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_json_pretty() {
        let data = serde_json::json!({"name": "test", "count": 42});
        let result = format_json_pretty(&data);
        assert!(
            result.contains("\"name\": \"test\""),
            "Expected JSON to contain name"
        );
        assert!(
            result.contains("\"count\": 42"),
            "Expected JSON to contain count"
        );
    }

    #[test]
    fn test_format_json_pretty_empty_object() {
        let data = serde_json::json!({});
        let result = format_json_pretty(&data);
        assert_eq!(result.trim(), "{}");
    }

    #[test]
    fn test_format_yaml() {
        let data = serde_json::json!({"key": "value", "nested": {"a": 1}});
        let result = format_yaml(&data);
        assert!(
            result.contains("key: value") || result.contains("key:"),
            "Expected YAML output"
        );
        assert!(result.contains("value"));
    }

    #[test]
    fn test_colorize_status_healthy() {
        let result = colorize_status("healthy");
        assert!(
            result.contains("healthy"),
            "Result should contain status text"
        );
    }

    #[test]
    fn test_colorize_status_degraded() {
        let result = colorize_status("degraded");
        assert!(result.contains("degraded"));
    }

    #[test]
    fn test_colorize_status_unhealthy() {
        let result = colorize_status("unhealthy");
        assert!(result.contains("unhealthy"));
    }

    #[test]
    fn test_colorize_status_unknown() {
        let result = colorize_status("unknown");
        assert!(result.contains("unknown"));
    }

    #[test]
    fn test_colorize_status_case_insensitive() {
        let result = colorize_status("HEALTHY");
        assert!(
            result.contains("HEALTHY"),
            "Case-insensitive match should preserve original case"
        );
    }

    #[test]
    fn test_colorize_status_unknown_value() {
        let result = colorize_status("custom_status");
        assert_eq!(
            result, "custom_status",
            "Unknown status should pass through unchanged"
        );
    }

    #[test]
    fn test_colorize_status_ok_success() {
        let result = colorize_status("ok");
        assert!(result.contains("ok"));
        let result2 = colorize_status("success");
        assert!(result2.contains("success"));
    }

    #[test]
    fn test_format_table_empty() {
        let data: Vec<serde_json::Value> = vec![];
        let result = format_table(&data);
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_table_with_data() {
        #[derive(serde::Serialize)]
        struct Row {
            name: String,
            value: String,
        }
        let data = vec![
            Row {
                name: "a".to_string(),
                value: "x".to_string(),
            },
            Row {
                name: "b".to_string(),
                value: "y".to_string(),
            },
        ];
        let result = format_table(&data);
        assert!(result.contains("name"), "Table should have name header");
        assert!(result.contains("value"), "Table should have value header");
        assert!(result.contains("a"), "Table should contain first row name");
        assert!(result.contains("b"), "Table should contain second row name");
        assert!(result.contains("x"), "Table should contain first row value");
        assert!(
            result.contains("y"),
            "Table should contain second row value"
        );
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(3660), "1h 1m");
        assert_eq!(format_duration(86399), "23h 59m");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86400), "1d 0h");
        assert_eq!(format_duration(90000), "1d 1h");
    }

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1023), "1023 B");
    }

    #[test]
    fn test_format_file_size_kb() {
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
    }

    #[test]
    fn test_format_file_size_mb() {
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 5), "5.0 MB");
    }

    #[test]
    fn test_format_file_size_gb() {
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_percentage() {
        let result = format_percentage(50.0);
        assert!(result.contains("50.0%"), "Should format percentage");
    }

    #[test]
    fn test_format_percentage_high() {
        let result = format_percentage(95.0);
        assert!(result.contains("95.0%"));
    }

    #[test]
    fn test_format_percentage_medium() {
        let result = format_percentage(80.0);
        assert!(result.contains("80.0%"));
    }

    #[test]
    fn test_format_percentage_boundaries() {
        assert!(format_percentage(90.0).contains("90.0%"));
        assert!(format_percentage(75.0).contains("75.0%"));
    }

    #[test]
    fn test_format_timestamp() {
        let ts = chrono::DateTime::parse_from_rfc3339("2024-01-15T12:30:45Z")
            .expect("valid timestamp")
            .with_timezone(&chrono::Utc);
        let result = format_timestamp(&ts);
        assert_eq!(result, "2024-01-15 12:30:45 UTC");
    }
}
