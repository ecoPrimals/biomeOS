// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Formatting helpers for monitor output.

use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::commands::utils::format_bytes;

/// Builds system overview display lines (memory, cpu, disk).
pub fn format_system_overview(system: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(cpu) = system.get("cpu_usage_percent") {
        lines.push(format!("  💻 CPU: {cpu}%"));
    }

    if let Some(memory) = system.get("memory") {
        if let Some(used_gb) = memory.get("used_gb") {
            if let Some(total_gb) = memory.get("total_gb") {
                let percent = memory
                    .get("usage_percent")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                lines.push(format!(
                    "  🧠 Memory: {used_gb:.1}GB / {total_gb:.1}GB ({percent:.1}%)"
                ));
            }
        }
    }

    if let Some(disk) = system.get("disk") {
        if let Some(usage_percent) = disk.get("usage_percent") {
            lines.push(format!("  💾 Disk: {usage_percent}%"));
        }
    }

    if let Some(load) = system.get("load_average") {
        if let Some(load_1m) = load.get("1m") {
            lines.push(format!("  📊 Load: {load_1m}"));
        }
    }

    lines
}

/// Builds per-service status lines.
pub fn format_service_rows(services: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(services_obj) = services.as_object() {
        if services_obj.is_empty() {
            return lines;
        }
        lines.push(format!(
            "\n🎯 Service Status ({} services):",
            services_obj.len()
        ));

        for (service_name, service_data) in services_obj {
            let status = service_data
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            let health = service_data
                .get("health")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");

            let status_icon = service_status_icon(status);
            let health_icon = service_health_icon(health);

            lines.push(format!(
                "  {status_icon} {health_icon} {service_name}: {status} | {health}"
            ));

            if let Some(resources) = service_data.get("resources") {
                if let Some(cpu) = resources.get("cpu_percent") {
                    if let Some(memory_mb) = resources.get("memory_mb") {
                        lines.push(format!("    📊 CPU: {cpu}% | Memory: {memory_mb}MB"));
                    }
                }
            }
        }
    }

    lines
}

/// Builds network activity display lines.
pub fn format_network_activity(network: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(bytes_in) = network.get("bytes_in_per_sec") {
        if let Some(bytes_out) = network.get("bytes_out_per_sec") {
            lines.push(format!(
                "  ↓ In: {}/s | ↑ Out: {}/s",
                format_bytes(bytes_in.as_u64().unwrap_or(0)),
                format_bytes(bytes_out.as_u64().unwrap_or(0))
            ));
        }
    }

    if let Some(connections) = network.get("active_connections") {
        lines.push(format!("  🔗 Active connections: {connections}"));
    }

    lines
}

/// Builds alert display rows.
pub fn format_alert_rows(alerts: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(alerts_arr) = alerts.as_array() {
        if !alerts_arr.is_empty() {
            lines.push("\n🚨 Active Alerts:".to_string());
            for alert in alerts_arr {
                if let Some(message) = alert.get("message") {
                    let severity = alert
                        .get("severity")
                        .and_then(|s| s.as_str())
                        .unwrap_or("info");
                    let icon = alert_severity_icon(severity);
                    lines.push(format!("  {icon} {message}"));
                }
            }
        }
    }

    lines
}

/// Formats a single log entry to a display string.
pub fn format_log_entry(entry: &Value) -> String {
    let timestamp = entry
        .get("timestamp")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");
    let level = entry
        .get("level")
        .and_then(|l| l.as_str())
        .unwrap_or("info");
    let message = entry.get("message").and_then(|m| m.as_str()).unwrap_or("");

    let level_icon = log_level_icon(level);

    format!(
        "{} [{}] {}: {}",
        level_icon,
        timestamp,
        level.to_uppercase(),
        message
    )
}

/// Builds exec output display lines.
pub fn format_exec_output(results: &HashMap<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(exit_code) = results.get("exit_code") {
        let icon = if exit_code.as_i64().unwrap_or(-1) == 0 {
            "✅"
        } else {
            "❌"
        };
        lines.push(format!("{icon} Exit code: {exit_code}"));
    }

    if let Some(stdout) = results.get("stdout").and_then(|s| s.as_str()) {
        if !stdout.trim().is_empty() {
            lines.push("\n📤 STDOUT:".to_string());
            for line in stdout.lines() {
                lines.push(format!("  {line}"));
            }
        }
    }

    if let Some(stderr) = results.get("stderr").and_then(|s| s.as_str()) {
        if !stderr.trim().is_empty() {
            lines.push("\n📥 STDERR:".to_string());
            for line in stderr.lines() {
                lines.push(format!("  {line}"));
            }
        }
    }

    if let Some(duration_ms) = results.get("duration_ms") {
        lines.push(format!("\n⏱️  Execution time: {duration_ms}ms"));
    }

    lines
}

/// Builds scale operation output display lines.
pub fn format_scale_output(results: &HashMap<String, Value>, auto: bool) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(status) = results.get("status") {
        let icon = scale_status_icon(status.as_str().unwrap_or(""));
        lines.push(format!("{icon} Status: {status}"));
    }

    if let Some(current_replicas) = results.get("current_replicas") {
        lines.push(format!("📊 Current replicas: {current_replicas}"));
    }

    if let Some(target_replicas) = results.get("target_replicas") {
        lines.push(format!("🎯 Target replicas: {target_replicas}"));
    }

    if auto {
        if let Some(auto_scaling_info) = results.get("auto_scaling") {
            lines.push("\n🤖 Auto-scaling configuration:".to_string());
            if let Some(min_replicas) = auto_scaling_info.get("min_replicas") {
                lines.push(format!("  📉 Min replicas: {min_replicas}"));
            }
            if let Some(max_replicas) = auto_scaling_info.get("max_replicas") {
                lines.push(format!("  📈 Max replicas: {max_replicas}"));
            }
            if let Some(cpu_threshold) = auto_scaling_info.get("cpu_threshold_percent") {
                lines.push(format!("  🖥️  CPU threshold: {cpu_threshold}%"));
            }
        }
    }

    if let Some(message) = results.get("message") {
        lines.push(format!("\n💬 {message}"));
    }

    lines
}

/// Returns true if monitoring should stop based on elapsed time and optional duration limit.
pub fn should_stop_monitoring(elapsed: Duration, duration: Option<Duration>) -> bool {
    match duration {
        Some(d) => elapsed >= d,
        None => false,
    }
}
/// Get service status icon for display
pub fn service_status_icon(status: &str) -> &'static str {
    match status {
        "running" => "✅",
        "starting" => "🔄",
        "stopping" => "⏹️",
        "stopped" => "⏸️",
        "error" => "❌",
        _ => "❓",
    }
}

/// Get service health icon for display
pub fn service_health_icon(health: &str) -> &'static str {
    match health {
        "Healthy" => "💚",
        "Degraded" => "💛",
        "Critical" => "🧡",
        "Unhealthy" => "❤️",
        _ => "🤍",
    }
}

/// Get alert severity icon for display
pub fn alert_severity_icon(severity: &str) -> &'static str {
    match severity {
        "critical" => "🔴",
        "warning" => "🟡",
        "info" => "🔵",
        _ => "⚪",
    }
}

/// Get log level icon for display
pub fn log_level_icon(level: &str) -> &'static str {
    match level {
        "error" => "❌",
        "warn" => "⚠️",
        "info" => "ℹ️",
        "debug" => "🐛",
        "trace" => "🔍",
        _ => "📝",
    }
}

/// Get scale operation status icon for display
pub fn scale_status_icon(status: &str) -> &'static str {
    match status {
        "success" => "✅",
        "in_progress" => "🔄",
        "failed" => "❌",
        _ => "🔹",
    }
}
