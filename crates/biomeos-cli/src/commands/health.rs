// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health Command Handlers
//!
//! Handles health monitoring operations including health checks,
//! probes, system scans, and status reporting.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use serde_json::Value;
use std::collections::HashMap;

use super::utils::{create_spinner, display_results, format_bytes};

/// Maps health status strings to display icons.
pub(crate) fn status_to_icon(status: &str) -> &'static str {
    match status {
        "Healthy" => "✅",
        "Degraded" => "⚠️",
        "Critical" => "🔴",
        "Unhealthy" => "❌",
        "Starting" => "🔄",
        "Stopping" => "⏹️",
        "Maintenance" => "🔧",
        "Unknown" => "❓",
        _ => "🔹",
    }
}

/// Computes memory usage percentage. Returns 0.0 if total is 0.
pub(crate) fn compute_memory_percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 / total as f64) * 100.0
}

/// Builds all display lines for health check results without printing.
pub(crate) fn format_health_summary(
    results: &HashMap<String, Value>,
    detailed: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(overall_status) = results.get("overall_status") {
        let status_str = overall_status.as_str().unwrap_or("Unknown");
        let icon = status_to_icon(status_str);
        lines.push(format!("{icon} Overall Status: {status_str}"));
    }

    if let Some(services) = results.get("services").and_then(|s| s.as_object()) {
        lines.push(format!(
            "\n🎯 Service Health ({} services):",
            services.len()
        ));

        for (service_name, service_health) in services {
            let health_status = service_health
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("Unknown");
            let icon = status_to_icon(health_status);

            lines.push(format!("  {icon} {service_name}: {health_status}"));

            if detailed {
                if let Some(issues) = service_health.get("issues").and_then(|i| i.as_array()) {
                    for issue in issues {
                        if let Some(message) = issue.get("message") {
                            lines.push(format!("    • {message}"));
                        }
                    }
                }

                if let Some(metrics) = service_health.get("metrics") {
                    lines.extend(format_health_metrics(metrics, 2));
                }
            }
        }
    }

    if let Some(system_metrics) = results.get("system_metrics") {
        lines.push("\n📊 System Metrics:".to_string());
        lines.extend(format_health_metrics(system_metrics, 1));
    }

    lines.push(String::new());
    lines
}

/// Builds display lines for probe results.
pub(crate) fn format_probe_results(service: &str, results: &HashMap<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("🔍 Deep probe results for '{service}':"));

    if let Some(connectivity) = results.get("connectivity") {
        lines.push("\n🌐 Connectivity:".to_string());
        lines.extend(format_connectivity_results(connectivity));
    }

    if let Some(performance) = results.get("performance") {
        lines.push("\n⚡ Performance:".to_string());
        lines.extend(format_performance_metrics(performance));
    }

    if let Some(diagnostics) = results.get("diagnostics") {
        lines.push("\n🔧 Diagnostics:".to_string());
        lines.extend(format_diagnostics(diagnostics));
    }

    lines
}

/// Builds scan results as a formatted string based on format type.
pub(crate) fn format_scan_results(
    results: &HashMap<String, Value>,
    format: &str,
) -> Result<String> {
    match format {
        "json" => serde_json::to_string_pretty(results).map_err(Into::into),
        "summary" => Ok(format_scan_summary(results)),
        _ => Ok(format_scan_default(results)),
    }
}

fn format_health_metrics(metrics: &Value, indent_level: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let indent = "  ".repeat(indent_level);

    if let Some(cpu) = metrics.get("cpu_usage") {
        lines.push(format!("{indent}💻 CPU Usage: {cpu}%"));
    }

    if let Some(memory) = metrics.get("memory_usage") {
        if let Some(used) = memory.get("used_bytes") {
            if let Some(total) = memory.get("total_bytes") {
                let used_gb = used.as_u64().unwrap_or(0) as f64 / 1_073_741_824.0;
                let total_gb = total.as_u64().unwrap_or(0) as f64 / 1_073_741_824.0;
                let percent =
                    compute_memory_percent(used.as_u64().unwrap_or(0), total.as_u64().unwrap_or(0));
                lines.push(format!(
                    "{indent}🧠 Memory: {used_gb:.1}GB / {total_gb:.1}GB ({percent:.1}%)"
                ));
            }
        }
    }

    if let Some(disk) = metrics.get("disk_usage") {
        if let Some(used) = disk.get("used_bytes") {
            lines.push(format!(
                "{}💾 Disk Usage: {}",
                indent,
                format_bytes(used.as_u64().unwrap_or(0))
            ));
        }
    }

    if let Some(network) = metrics.get("network") {
        if let Some(bytes_sent) = network.get("bytes_sent") {
            if let Some(bytes_received) = network.get("bytes_received") {
                lines.push(format!(
                    "{}🌐 Network: ↑{} ↓{}",
                    indent,
                    format_bytes(bytes_sent.as_u64().unwrap_or(0)),
                    format_bytes(bytes_received.as_u64().unwrap_or(0))
                ));
            }
        }
    }

    lines
}

fn format_connectivity_results(connectivity: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(reachable) = connectivity.get("reachable") {
        let icon = if reachable.as_bool().unwrap_or(false) {
            "✅"
        } else {
            "❌"
        };
        lines.push(format!("  {icon} Reachable: {reachable}"));
    }

    if let Some(response_time) = connectivity.get("response_time_ms") {
        lines.push(format!("  ⏱️  Response Time: {response_time}ms"));
    }

    if let Some(endpoints) = connectivity.get("endpoints").and_then(|e| e.as_array()) {
        lines.push("  🔗 Endpoints:".to_string());
        for endpoint in endpoints {
            if let Some(url) = endpoint.get("url") {
                let status = endpoint
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let icon = if status == "ok" { "✅" } else { "❌" };
                lines.push(format!("    {icon} {url}: {status}"));
            }
        }
    }

    lines
}

fn format_performance_metrics(performance: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(throughput) = performance.get("throughput_rps") {
        lines.push(format!("  📈 Throughput: {throughput} req/s"));
    }

    if let Some(latency) = performance.get("avg_latency_ms") {
        lines.push(format!("  ⏱️  Avg Latency: {latency}ms"));
    }

    if let Some(error_rate) = performance.get("error_rate_percent") {
        lines.push(format!("  ❌ Error Rate: {error_rate}%"));
    }

    lines
}

fn format_diagnostics(diagnostics: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(diag_obj) = diagnostics.as_object() {
        for (key, value) in diag_obj {
            let line = match value {
                Value::String(s) => format!("  {key}: {s}"),
                Value::Number(n) => format!("  {key}: {n}"),
                Value::Bool(b) => format!("  {key}: {b}"),
                Value::Array(arr) => format!("  {key}: {} items", arr.len()),
                Value::Object(_) => format!("  {key}: [object]"),
                Value::Null => format!("  {key}: null"),
            };
            lines.push(line);
        }
    } else {
        lines.push(format!("  {diagnostics}"));
    }

    lines
}

fn format_scan_summary(results: &HashMap<String, Value>) -> String {
    let mut lines = Vec::new();
    lines.push("📋 System Scan Summary:".to_string());
    if let Some(issues_found) = results.get("issues_count") {
        lines.push(format!("  Issues found: {issues_found}"));
    }
    if let Some(services_scanned) = results.get("services_scanned") {
        lines.push(format!("  Services scanned: {services_scanned}"));
    }
    lines.join("\n")
}

fn format_scan_default(results: &HashMap<String, Value>) -> String {
    // display_results is sync; this helper mirrors its layout for scan summaries
    // The "default" format uses display_results which prints. We'll build a similar
    // structure that can be printed line by line.
    let mut lines = Vec::new();
    if results.is_empty() {
        lines.push("📋 System Scan Results: No results".to_string());
        return lines.join("\n");
    }
    lines.push(format!("📋 System Scan Results ({} items):", results.len()));
    lines.push(String::new());
    for (key, value) in results {
        lines.push(format!("🔹 {key}"));
        if let Ok(pretty) = serde_json::to_string_pretty(value) {
            for line in pretty.lines() {
                lines.push(format!("   {line}"));
            }
        } else {
            lines.push(format!("   {value}"));
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

/// Handle health check command
pub async fn handle_health(
    service: Option<String>,
    detailed: bool,
    continuous: bool,
    interval: u64,
    use_graph: bool,
    niche_path: Option<std::path::PathBuf>,
) -> Result<()> {
    // Graph-based health check (Neural API)
    if use_graph {
        return handle_graph_health_check(niche_path, continuous, interval).await;
    }

    // Legacy health check
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config)?;

    if continuous {
        println!("🔄 Starting continuous health monitoring (interval: {interval}s)");
        println!("Press Ctrl+C to stop");

        loop {
            perform_health_check(&manager, service.as_deref(), detailed).await?;
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            println!("\n{}", "─".repeat(80));
        }
    } else {
        perform_health_check(&manager, service.as_deref(), detailed).await?;
    }

    Ok(())
}

/// Handle graph-based health check (Neural API)
async fn handle_graph_health_check(
    niche_path: Option<std::path::PathBuf>,
    continuous: bool,
    interval: u64,
) -> Result<()> {
    let niche = niche_path
        .ok_or_else(|| anyhow::anyhow!("--niche required for graph-based health check"))?;

    if continuous {
        println!("🔄 Starting continuous health monitoring via Neural API (interval: {interval}s)");
        println!("Press Ctrl+C to stop");

        loop {
            perform_graph_health_check(&niche)?;
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            println!("\n{}", "─".repeat(80));
        }
    } else {
        perform_graph_health_check(&niche)?;
    }

    Ok(())
}

/// Perform a single graph-based health check
///
/// ⚠️ DEPRECATED: This uses the old `graph_deployment` module.
fn perform_graph_health_check(_niche_path: &std::path::Path) -> Result<()> {
    println!("⚠️  DEPRECATED: Graph-based health checks via CLI are deprecated.");
    println!("📖 Please use:");
    println!("  • biomeos-api health endpoints");
    println!("  • Individual primal health checks via their APIs");

    anyhow::bail!("Graph-based health check is deprecated. Use biomeos-api instead.");
}

/// Handle probe command for deep health diagnostics
pub async fn handle_probe(service: String, timeout: u64) -> Result<()> {
    let spinner = create_spinner(&format!("🔍 Probing service '{service}'..."));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config)?;

    let probe_result = manager.probe_service_health(&service, timeout).await?;

    spinner.finish_with_message("✅ Probe completed");

    display_probe_results(&service, &probe_result);

    Ok(())
}

/// Handle system scan command
pub async fn handle_scan(quick: bool, output_format: String) -> Result<()> {
    let scan_type = if quick { "Quick" } else { "Comprehensive" };
    let spinner = create_spinner(&format!("🔬 {scan_type} system scan..."));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config)?;

    let scan_result = if quick {
        manager.quick_system_scan().await?
    } else {
        manager.comprehensive_system_scan().await?
    };

    spinner.finish_with_message("✅ Scan completed");

    display_scan_results(&scan_result, &output_format)?;

    Ok(())
}

/// Handle status reporting command
pub async fn handle_status(
    service: Option<String>,
    format: String,
    show_metrics: bool,
) -> Result<()> {
    let spinner = create_spinner("📊 Gathering status information...");

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config)?;

    let status_result = match service {
        Some(service_name) => manager.get_service_status(&service_name).await?,
        None => manager.get_system_status().await?,
    };

    spinner.finish_with_message("✅ Status collected");

    display_status_results(&status_result, &format, show_metrics)?;

    Ok(())
}

/// Perform a health check operation
async fn perform_health_check(
    manager: &UniversalBiomeOSManager,
    service: Option<&str>,
    detailed: bool,
) -> Result<()> {
    let health_result = if let Some(service_name) = service {
        println!("🩺 Health check for service: {service_name}");
        manager.check_service_health(service_name).await?
    } else {
        println!("🩺 System-wide health check");
        manager.check_system_health().await?
    };

    display_health_results(&health_result, detailed);
    Ok(())
}

/// Display health check results (thin wrapper)
fn display_health_results(results: &HashMap<String, Value>, detailed: bool) {
    let lines = format_health_summary(results, detailed);
    for line in lines {
        println!("{line}");
    }
}

/// Display probe results (thin wrapper)
fn display_probe_results(service: &str, results: &HashMap<String, Value>) {
    let lines = format_probe_results(service, results);
    for line in lines {
        println!("{line}");
    }
}

/// Display scan results (thin wrapper)
fn display_scan_results(results: &HashMap<String, Value>, format: &str) -> Result<()> {
    let output = format_scan_results(results, format)?;
    println!("{output}");
    Ok(())
}

/// Display status results
fn display_status_results(
    results: &HashMap<String, Value>,
    format: &str,
    show_metrics: bool,
) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(results)?);
        }
        "brief" => {
            if let Some(status) = results.get("status") {
                println!("Status: {status}");
            }
        }
        _ => {
            display_results("Status Report", results, show_metrics)?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;
