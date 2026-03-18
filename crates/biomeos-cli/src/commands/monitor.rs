// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Monitor Command Handlers
//!
//! Handles monitoring operations including system monitoring,
//! dashboard display, log handling, exec operations, and scaling.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use biomeos_core::UniversalBiomeOSManager;

#[cfg(feature = "deprecated-tui")]
use crate::tui::BiomeOSDashboard;

use super::utils::{create_spinner, format_bytes, format_duration};

/// Builds system overview display lines (memory, cpu, disk).
pub(crate) fn format_system_overview(system: &Value) -> Vec<String> {
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
pub(crate) fn format_service_rows(services: &Value) -> Vec<String> {
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
pub(crate) fn format_network_activity(network: &Value) -> Vec<String> {
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
pub(crate) fn format_alert_rows(alerts: &Value) -> Vec<String> {
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
pub(crate) fn format_log_entry(entry: &Value) -> String {
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
pub(crate) fn format_exec_output(results: &HashMap<String, Value>) -> Vec<String> {
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
pub(crate) fn format_scale_output(results: &HashMap<String, Value>, auto: bool) -> Vec<String> {
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
pub(crate) fn should_stop_monitoring(elapsed: Duration, duration: Option<Duration>) -> bool {
    match duration {
        Some(d) => elapsed >= d,
        None => false,
    }
}

/// Handle monitoring command
pub async fn handle_monitor(
    service: Option<String>,
    interval: u64,
    duration: Option<u64>,
) -> Result<()> {
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    println!("🔍 Starting system monitoring (interval: {interval}s)");
    if let Some(duration_s) = duration {
        println!("Duration: {duration_s}s");
    }
    println!("Press Ctrl+C to stop\n");

    let start_time = std::time::Instant::now();
    let mut iteration = 0;

    loop {
        iteration += 1;
        println!(
            "📊 Monitoring iteration {} ({})",
            iteration,
            format_duration(start_time.elapsed())
        );

        let monitoring_result = match &service {
            Some(service_name) => manager.monitor_service(service_name).await?,
            None => manager.monitor_system().await?,
        };

        display_monitoring_results(&monitoring_result);

        let duration_dur = duration.map(Duration::from_secs);
        if should_stop_monitoring(start_time.elapsed(), duration_dur) {
            println!("✅ Monitoring completed (duration reached)");
            break;
        }

        println!("\n{}", "─".repeat(80));
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }

    Ok(())
}

/// Handle dashboard command.
///
/// The built-in TUI dashboard is deprecated. Use petalTongue for ecosystem visualization.
/// Enable the `deprecated-tui` feature to use the legacy dashboard.
#[cfg(feature = "deprecated-tui")]
pub async fn handle_dashboard(interval: u64, refresh: bool) -> Result<()> {
    eprintln!(
        "WARNING: The built-in dashboard is deprecated. Use petalTongue for ecosystem visualization."
    );
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;
    let mut dashboard = BiomeOSDashboard::new(manager);
    if refresh {
        println!("Auto-refresh enabled (interval: {interval}s)");
    }
    dashboard.run().await?;
    Ok(())
}

/// Handle dashboard command — petalTongue redirect when legacy TUI is not compiled.
#[cfg(not(feature = "deprecated-tui"))]
pub async fn handle_dashboard(_interval: u64, _refresh: bool) -> Result<()> {
    eprintln!("The built-in TUI dashboard has been deprecated.");
    eprintln!("petalTongue is the universal UI primal for the ecoPrimals ecosystem.");
    eprintln!();
    eprintln!("  To launch petalTongue:  biomeos start petaltongue");
    eprintln!("  Legacy TUI (unsupported):  cargo build -p biomeos-cli --features deprecated-tui");
    Ok(())
}

/// Handle logs command
pub async fn handle_logs(
    service: String,
    follow: bool,
    tail: Option<usize>,
    since: Option<String>,
) -> Result<()> {
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    if follow {
        println!("📜 Following logs for service '{service}' (Press Ctrl+C to stop)");
    } else {
        println!("📜 Fetching logs for service '{service}'");
    }

    let logs_result = manager
        .get_service_logs(&service, follow, tail, since.as_deref())
        .await?;

    display_logs_result(&service, &logs_result);

    Ok(())
}

/// Handle exec command
pub async fn handle_exec(service: String, command: Vec<String>, interactive: bool) -> Result<()> {
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    let command_str = command.join(" ");
    println!("⚡ Executing '{command_str}' in service '{service}'");

    if interactive {
        println!("Interactive mode enabled");
    }

    let exec_result = manager
        .exec_in_service(&service, &command, interactive)
        .await?;

    display_exec_result(&service, &command_str, &exec_result);

    Ok(())
}

/// Handle scale command
pub async fn handle_scale(service: String, replicas: Option<u32>, auto: bool) -> Result<()> {
    let spinner = create_spinner(&format!("⚖️  Scaling service '{service}'..."));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    let scale_result = if auto {
        manager.enable_auto_scaling(&service).await?
    } else if let Some(replica_count) = replicas {
        manager
            .scale_service(&service, Some(replica_count), false)
            .await?
    } else {
        return Err(anyhow::anyhow!("Must specify either --replicas or --auto"));
    };

    spinner.finish_with_message("✅ Scaling operation completed");

    display_scale_result(&service, &scale_result, auto);

    Ok(())
}

/// Display monitoring results (thin wrapper)
fn display_monitoring_results(results: &HashMap<String, Value>) {
    if let Some(system) = results.get("system") {
        let overview = format_system_overview(system);
        if !overview.is_empty() {
            println!("🖥️  System Status:");
            for line in overview {
                println!("{line}");
            }
        }
    }

    if let Some(services) = results.get("services") {
        let service_lines = format_service_rows(services);
        for line in service_lines {
            println!("{line}");
        }
    }

    if let Some(network) = results.get("network") {
        let network_lines = format_network_activity(network);
        if !network_lines.is_empty() {
            println!("\n🌐 Network Activity:");
            for line in network_lines {
                println!("{line}");
            }
        }
    }

    if let Some(alerts) = results.get("alerts") {
        let alert_lines = format_alert_rows(alerts);
        for line in alert_lines {
            println!("{line}");
        }
    }
}

/// Display logs results (thin wrapper)
fn display_logs_result(service: &str, results: &HashMap<String, Value>) {
    if let Some(logs) = results.get("logs").and_then(|l| l.as_array()) {
        println!(
            "📜 Logs for service '{}' ({} entries):",
            service,
            logs.len()
        );
        println!();

        for log_entry in logs {
            println!("{}", format_log_entry(log_entry));
        }
    } else {
        println!("📜 No logs available for service '{service}'");
    }

    if let Some(follow_info) = results.get("following") {
        if follow_info.as_bool().unwrap_or(false) {
            println!("\n🔄 Following logs... (Press Ctrl+C to stop)");
        }
    }
}

/// Display exec results (thin wrapper)
fn display_exec_result(service: &str, command: &str, results: &HashMap<String, Value>) {
    println!("⚡ Execution results for '{command}' in service '{service}':");

    let lines = format_exec_output(results);
    for line in lines {
        println!("{line}");
    }
}

/// Display scaling results (thin wrapper)
fn display_scale_result(service: &str, results: &HashMap<String, Value>, auto_scaling: bool) {
    let operation = if auto_scaling {
        "Auto-scaling"
    } else {
        "Manual scaling"
    };
    println!("⚖️  {operation} results for service '{service}':");

    let lines = format_scale_output(results, auto_scaling);
    for line in lines {
        println!("{line}");
    }
}

/// Get service status icon for display
pub(crate) fn service_status_icon(status: &str) -> &'static str {
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
pub(crate) fn service_health_icon(health: &str) -> &'static str {
    match health {
        "Healthy" => "💚",
        "Degraded" => "💛",
        "Critical" => "🧡",
        "Unhealthy" => "❤️",
        _ => "🤍",
    }
}

/// Get alert severity icon for display
pub(crate) fn alert_severity_icon(severity: &str) -> &'static str {
    match severity {
        "critical" => "🔴",
        "warning" => "🟡",
        "info" => "🔵",
        _ => "⚪",
    }
}

/// Get log level icon for display
pub(crate) fn log_level_icon(level: &str) -> &'static str {
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
pub(crate) fn scale_status_icon(status: &str) -> &'static str {
    match status {
        "success" => "✅",
        "in_progress" => "🔄",
        "failed" => "❌",
        _ => "🔹",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::commands::utils::{format_bytes, format_duration};

    #[test]
    fn test_format_system_overview_empty() {
        let system = serde_json::json!({});
        let lines = format_system_overview(&system);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_format_system_overview_with_cpu() {
        let system = serde_json::json!({"cpu_usage_percent": 42});
        let lines = format_system_overview(&system);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("42"));
    }

    #[test]
    fn test_format_system_overview_with_memory() {
        let system = serde_json::json!({
            "memory": {
                "used_gb": 2.5,
                "total_gb": 8.0,
                "usage_percent": 31.25
            }
        });
        let lines = format_system_overview(&system);
        assert!(lines.iter().any(|l| l.contains("Memory")));
        assert!(lines.iter().any(|l| l.contains("2.5") && l.contains("8.0")));
    }

    #[test]
    fn test_format_service_rows_empty() {
        let services = serde_json::json!({});
        let lines = format_service_rows(&services);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_format_service_rows_with_services() {
        let services = serde_json::json!({
            "svc1": {"status": "running", "health": "Healthy"},
            "svc2": {"status": "stopped", "health": "unknown"}
        });
        let lines = format_service_rows(&services);
        assert!(lines.iter().any(|l| l.contains("Service Status")));
        assert!(
            lines
                .iter()
                .any(|l| l.contains("svc1") && l.contains("running"))
        );
    }

    #[test]
    fn test_format_network_activity() {
        let network = serde_json::json!({
            "bytes_in_per_sec": 1024,
            "bytes_out_per_sec": 2048,
            "active_connections": 5
        });
        let lines = format_network_activity(&network);
        assert!(lines.iter().any(|l| l.contains("In")));
        assert!(lines.iter().any(|l| l.contains("Out")));
        assert!(lines.iter().any(|l| l.contains("5")));
    }

    #[test]
    fn test_format_alert_rows_empty() {
        let alerts = serde_json::json!([]);
        let lines = format_alert_rows(&alerts);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_format_alert_rows_with_alerts() {
        let alerts = serde_json::json!([
            {"message": "High CPU", "severity": "warning"},
            {"message": "Disk full", "severity": "critical"}
        ]);
        let lines = format_alert_rows(&alerts);
        assert!(lines.iter().any(|l| l.contains("Active Alerts")));
        assert!(lines.iter().any(|l| l.contains("High CPU")));
    }

    #[test]
    fn test_format_log_entry() {
        let entry = serde_json::json!({
            "timestamp": "2025-01-01T12:00:00Z",
            "level": "info",
            "message": "Hello world"
        });
        let s = format_log_entry(&entry);
        assert!(s.contains("INFO"));
        assert!(s.contains("Hello world"));
        assert!(s.contains("2025-01-01"));
    }

    #[test]
    fn test_format_log_entry_missing_fields() {
        let entry = serde_json::json!({});
        let s = format_log_entry(&entry);
        assert!(s.contains("unknown"));
        assert!(s.contains("INFO"));
    }

    #[test]
    fn test_format_exec_output() {
        let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
            "exit_code": 0,
            "stdout": "hello\nworld",
            "duration_ms": 10
        }))
        .unwrap();
        let lines = format_exec_output(&results);
        assert!(lines.iter().any(|l| l.contains("✅")));
        assert!(lines.iter().any(|l| l.contains("STDOUT")));
        assert!(lines.iter().any(|l| l.contains("10ms")));
    }

    #[test]
    fn test_format_scale_output() {
        let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
            "status": "success",
            "current_replicas": 3,
            "target_replicas": 3
        }))
        .unwrap();
        let lines = format_scale_output(&results, false);
        assert!(lines.iter().any(|l| l.contains("success")));
        assert!(lines.iter().any(|l| l.contains("Current replicas")));
    }

    #[test]
    fn test_should_stop_monitoring_no_duration() {
        assert!(!should_stop_monitoring(Duration::from_secs(100), None));
    }

    #[test]
    fn test_should_stop_monitoring_not_reached() {
        assert!(!should_stop_monitoring(
            Duration::from_secs(50),
            Some(Duration::from_secs(100))
        ));
    }

    #[test]
    fn test_should_stop_monitoring_reached() {
        assert!(should_stop_monitoring(
            Duration::from_secs(100),
            Some(Duration::from_secs(100))
        ));
    }

    #[test]
    fn test_should_stop_monitoring_exceeded() {
        assert!(should_stop_monitoring(
            Duration::from_secs(150),
            Some(Duration::from_secs(100))
        ));
    }

    #[test]
    fn test_service_status_icon_all_variants() {
        assert_eq!(service_status_icon("running"), "✅");
        assert_eq!(service_status_icon("starting"), "🔄");
        assert_eq!(service_status_icon("stopping"), "⏹️");
        assert_eq!(service_status_icon("stopped"), "⏸️");
        assert_eq!(service_status_icon("error"), "❌");
        assert_eq!(service_status_icon("unknown"), "❓");
        assert_eq!(service_status_icon(""), "❓");
    }

    #[test]
    fn test_service_health_icon_all_variants() {
        assert_eq!(service_health_icon("Healthy"), "💚");
        assert_eq!(service_health_icon("Degraded"), "💛");
        assert_eq!(service_health_icon("Critical"), "🧡");
        assert_eq!(service_health_icon("Unhealthy"), "❤️");
        assert_eq!(service_health_icon("unknown"), "🤍");
        assert_eq!(service_health_icon(""), "🤍");
    }

    #[test]
    fn test_alert_severity_icon_all_variants() {
        assert_eq!(alert_severity_icon("critical"), "🔴");
        assert_eq!(alert_severity_icon("warning"), "🟡");
        assert_eq!(alert_severity_icon("info"), "🔵");
        assert_eq!(alert_severity_icon("unknown"), "⚪");
        assert_eq!(alert_severity_icon(""), "⚪");
    }

    #[test]
    fn test_log_level_icon_all_variants() {
        assert_eq!(log_level_icon("error"), "❌");
        assert_eq!(log_level_icon("warn"), "⚠️");
        assert_eq!(log_level_icon("info"), "ℹ️");
        assert_eq!(log_level_icon("debug"), "🐛");
        assert_eq!(log_level_icon("trace"), "🔍");
        assert_eq!(log_level_icon("unknown"), "📝");
        assert_eq!(log_level_icon(""), "📝");
    }

    #[test]
    fn test_scale_status_icon_all_variants() {
        assert_eq!(scale_status_icon("success"), "✅");
        assert_eq!(scale_status_icon("in_progress"), "🔄");
        assert_eq!(scale_status_icon("failed"), "❌");
        assert_eq!(scale_status_icon("unknown"), "🔹");
        assert_eq!(scale_status_icon(""), "🔹");
    }

    #[test]
    fn test_format_bytes_via_utils() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_format_duration_via_utils() {
        assert_eq!(format_duration(std::time::Duration::from_secs(45)), "45s");
        assert_eq!(
            format_duration(std::time::Duration::from_secs(125)),
            "2m 5s"
        );
    }
}
