//! Health Command Handlers
//!
//! Handles health monitoring operations including health checks,
//! probes, system scans, and status reporting.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use serde_json::Value;
use std::collections::HashMap;

use super::utils::{create_spinner, display_results, format_bytes};

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
    let manager = UniversalBiomeOSManager::new(config).await?;

    if continuous {
        println!(
            "🔄 Starting continuous health monitoring (interval: {}s)",
            interval
        );
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
    use biomeos_core::graph_deployment::GraphDeploymentCoordinator;
    use super::create_spinner;
    
    let niche = niche_path.ok_or_else(|| {
        anyhow::anyhow!("--niche required for graph-based health check")
    })?;
    
    if continuous {
        println!("🔄 Starting continuous health monitoring via Neural API (interval: {}s)", interval);
        println!("Press Ctrl+C to stop");
        
        loop {
            perform_graph_health_check(&niche).await?;
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            println!("\n{}", "─".repeat(80));
        }
    } else {
        perform_graph_health_check(&niche).await?;
    }
    
    Ok(())
}

/// Perform a single graph-based health check
async fn perform_graph_health_check(niche_path: &std::path::Path) -> Result<()> {
    use biomeos_core::graph_deployment::GraphDeploymentCoordinator;
    
    let spinner = create_spinner("🧠 Running health check via Neural API...");
    
    let coordinator = GraphDeploymentCoordinator::new();
    
    // Execute the health_check graph
    let result = coordinator
        .deploy_niche_with_graph(niche_path, "health_check")
        .await?;
    
    spinner.finish_and_clear();
    
    if result.success {
        println!("✅ Health Check: ALL PRIMALS HEALTHY");
        println!("📊 Check results:");
        
        for metric in &result.metrics {
            let status_icon = if metric.success { "✅" } else { "❌" };
            let health_status = if metric.success { "HEALTHY" } else { "UNHEALTHY" };
            
            println!(
                "  {} {} → {} ({}ms)",
                status_icon,
                metric.primal_id,
                health_status,
                metric.duration_ms
            );
            
            if let Some(error) = &metric.error {
                println!("     ⚠️  Error: {}", error);
            }
        }
        
        let total_duration: u64 = result.metrics.iter().map(|m| m.duration_ms).sum();
        println!("\n⏱️  Total check time: {}ms", total_duration);
    } else {
        let failed: Vec<_> = result.metrics.iter().filter(|m| !m.success).collect();
        
        println!("❌ Health Check: {} PRIMAL(S) UNHEALTHY", failed.len());
        println!("📊 Failed checks:");
        
        for metric in &failed {
            println!("  ❌ {} → UNHEALTHY ({}ms)", metric.primal_id, metric.duration_ms);
            
            if let Some(error) = &metric.error {
                println!("     Error: {}", error);
            }
        }
        
        anyhow::bail!("Health check failed: {} unhealthy primal(s)", failed.len());
    }
    
    Ok(())
}

/// Handle probe command for deep health diagnostics
pub async fn handle_probe(service: String, timeout: u64) -> Result<()> {
    let spinner = create_spinner(&format!("🔍 Probing service '{}'...", service));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    let probe_result = manager.probe_service_health(&service, timeout).await?;

    spinner.finish_with_message("✅ Probe completed");

    display_probe_results(&service, &probe_result).await?;

    Ok(())
}

/// Handle system scan command
pub async fn handle_scan(quick: bool, output_format: String) -> Result<()> {
    let scan_type = if quick { "Quick" } else { "Comprehensive" };
    let spinner = create_spinner(&format!("🔬 {} system scan...", scan_type));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    let scan_result = if quick {
        manager.quick_system_scan().await?
    } else {
        manager.comprehensive_system_scan().await?
    };

    spinner.finish_with_message("✅ Scan completed");

    display_scan_results(&scan_result, &output_format).await?;

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
    let manager = UniversalBiomeOSManager::new(config).await?;

    let status_result = match service {
        Some(service_name) => manager.get_service_status(&service_name).await?,
        None => manager.get_system_status().await?,
    };

    spinner.finish_with_message("✅ Status collected");

    display_status_results(&status_result, &format, show_metrics).await?;

    Ok(())
}

/// Perform a health check operation
async fn perform_health_check(
    manager: &UniversalBiomeOSManager,
    service: Option<&str>,
    detailed: bool,
) -> Result<()> {
    let health_result = match service {
        Some(service_name) => {
            println!("🩺 Health check for service: {}", service_name);
            manager.check_service_health(service_name).await?
        }
        None => {
            println!("🩺 System-wide health check");
            manager.check_system_health().await?
        }
    };

    display_health_results(&health_result, detailed).await?;
    Ok(())
}

/// Display health check results
async fn display_health_results(results: &HashMap<String, Value>, detailed: bool) -> Result<()> {
    if let Some(overall_status) = results.get("overall_status") {
        let icon = match overall_status.as_str() {
            Some("Healthy") => "✅",
            Some("Degraded") => "⚠️",
            Some("Critical") => "🔴",
            Some("Unhealthy") => "❌",
            Some("Starting") => "🔄",
            Some("Stopping") => "⏹️",
            Some("Maintenance") => "🔧",
            Some("Unknown") => "❓",
            _ => "🔹",
        };
        println!("{icon} Overall Status: {overall_status}");
    }

    if let Some(services) = results.get("services").and_then(|s| s.as_object()) {
        println!("\n🎯 Service Health ({} services):", services.len());

        for (service_name, service_health) in services {
            let health_status = service_health
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("Unknown");
            let icon = match health_status {
                "Healthy" => "✅",
                "Degraded" => "⚠️",
                "Critical" => "🔴",
                "Unhealthy" => "❌",
                "Starting" => "🔄",
                "Stopping" => "⏹️",
                "Maintenance" => "🔧",
                _ => "❓",
            };

            println!("  {} {}: {}", icon, service_name, health_status);

            if detailed {
                if let Some(issues) = service_health.get("issues").and_then(|i| i.as_array()) {
                    for issue in issues {
                        if let Some(message) = issue.get("message") {
                            println!("    • {}", message);
                        }
                    }
                }

                if let Some(metrics) = service_health.get("metrics") {
                    display_health_metrics(metrics, 2).await?;
                }
            }
        }
    }

    if let Some(system_metrics) = results.get("system_metrics") {
        println!("\n📊 System Metrics:");
        display_health_metrics(system_metrics, 1).await?;
    }

    println!();
    Ok(())
}

/// Display probe results
async fn display_probe_results(service: &str, results: &HashMap<String, Value>) -> Result<()> {
    println!("🔍 Deep probe results for '{}':", service);

    if let Some(connectivity) = results.get("connectivity") {
        println!("\n🌐 Connectivity:");
        display_connectivity_results(connectivity).await?;
    }

    if let Some(performance) = results.get("performance") {
        println!("\n⚡ Performance:");
        display_performance_metrics(performance).await?;
    }

    if let Some(diagnostics) = results.get("diagnostics") {
        println!("\n🔧 Diagnostics:");
        if let Some(diag_obj) = diagnostics.as_object() {
            for (key, value) in diag_obj {
                match value {
                    Value::String(s) => println!("  {key}: {s}"),
                    Value::Number(n) => println!("  {key}: {n}"),
                    Value::Bool(b) => println!("  {key}: {b}"),
                    Value::Array(arr) => println!("  {key}: {} items", arr.len()),
                    Value::Object(_) => println!("  {key}: [object]"),
                    Value::Null => println!("  {key}: null"),
                }
            }
        } else {
            println!("  {diagnostics}");
        }
    }

    Ok(())
}

/// Display scan results
async fn display_scan_results(results: &HashMap<String, Value>, format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(results)?);
        }
        "summary" => {
            println!("📋 System Scan Summary:");
            if let Some(issues_found) = results.get("issues_count") {
                println!("  Issues found: {}", issues_found);
            }
            if let Some(services_scanned) = results.get("services_scanned") {
                println!("  Services scanned: {}", services_scanned);
            }
        }
        _ => {
            display_results("System Scan Results", results, true).await?;
        }
    }

    Ok(())
}

/// Display status results
async fn display_status_results(
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
                println!("Status: {}", status);
            }
        }
        _ => {
            display_results("Status Report", results, show_metrics).await?;
        }
    }

    Ok(())
}

/// Display health metrics with indentation
async fn display_health_metrics(metrics: &Value, indent_level: usize) -> Result<()> {
    let indent = "  ".repeat(indent_level);

    if let Some(cpu) = metrics.get("cpu_usage") {
        println!("{}💻 CPU Usage: {}%", indent, cpu);
    }

    if let Some(memory) = metrics.get("memory_usage") {
        if let Some(used) = memory.get("used_bytes") {
            if let Some(total) = memory.get("total_bytes") {
                let used_gb = used.as_u64().unwrap_or(0) as f64 / 1_073_741_824.0;
                let total_gb = total.as_u64().unwrap_or(0) as f64 / 1_073_741_824.0;
                let percent = if total_gb > 0.0 {
                    (used_gb / total_gb) * 100.0
                } else {
                    0.0
                };
                println!(
                    "{}🧠 Memory: {:.1}GB / {:.1}GB ({:.1}%)",
                    indent, used_gb, total_gb, percent
                );
            }
        }
    }

    if let Some(disk) = metrics.get("disk_usage") {
        if let Some(used) = disk.get("used_bytes") {
            println!(
                "{}💾 Disk Usage: {}",
                indent,
                format_bytes(used.as_u64().unwrap_or(0))
            );
        }
    }

    if let Some(network) = metrics.get("network") {
        if let Some(bytes_sent) = network.get("bytes_sent") {
            if let Some(bytes_received) = network.get("bytes_received") {
                println!(
                    "{}🌐 Network: ↑{} ↓{}",
                    indent,
                    format_bytes(bytes_sent.as_u64().unwrap_or(0)),
                    format_bytes(bytes_received.as_u64().unwrap_or(0))
                );
            }
        }
    }

    Ok(())
}

/// Display connectivity results
async fn display_connectivity_results(connectivity: &Value) -> Result<()> {
    if let Some(reachable) = connectivity.get("reachable") {
        let icon = if reachable.as_bool().unwrap_or(false) {
            "✅"
        } else {
            "❌"
        };
        println!("  {} Reachable: {}", icon, reachable);
    }

    if let Some(response_time) = connectivity.get("response_time_ms") {
        println!("  ⏱️  Response Time: {}ms", response_time);
    }

    if let Some(endpoints) = connectivity.get("endpoints").and_then(|e| e.as_array()) {
        println!("  🔗 Endpoints:");
        for endpoint in endpoints {
            if let Some(url) = endpoint.get("url") {
                let status = endpoint
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let icon = if status == "ok" { "✅" } else { "❌" };
                println!("    {} {}: {}", icon, url, status);
            }
        }
    }

    Ok(())
}

/// Display performance metrics
async fn display_performance_metrics(performance: &Value) -> Result<()> {
    if let Some(throughput) = performance.get("throughput_rps") {
        println!("  📈 Throughput: {} req/s", throughput);
    }

    if let Some(latency) = performance.get("avg_latency_ms") {
        println!("  ⏱️  Avg Latency: {}ms", latency);
    }

    if let Some(error_rate) = performance.get("error_rate_percent") {
        println!("  ❌ Error Rate: {}%", error_rate);
    }

    Ok(())
}
