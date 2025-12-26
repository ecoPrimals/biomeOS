// Formatting utilities for CLI output
// Additional formatting functions implemented: table, duration, file_size, percentage, timestamp

use colored::*;

pub fn format_json_pretty<T: serde::Serialize>(data: &T) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|_| "Error serializing to JSON".to_string())
}

pub fn format_yaml<T: serde::Serialize>(data: &T) -> String {
    // Use serde_yaml for proper YAML formatting
    serde_yaml::to_string(data).unwrap_or_else(|_| "Error serializing to YAML".to_string())
}

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
        format!("{}s", seconds)
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
    let formatted = format!("{:.1}%", value);
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
