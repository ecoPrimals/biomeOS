//! Monitor Command Handlers
//!
//! Handles monitoring operations including system monitoring,
//! dashboard display, log handling, exec operations, and scaling.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use serde_json::Value;
use std::collections::HashMap;

use crate::tui::BiomeOSDashboard;

use super::utils::{create_spinner, format_duration};

/// Handle monitoring command
pub async fn handle_monitor(
    service: Option<String>,
    interval: u64,
    duration: Option<u64>,
) -> Result<()> {
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;
    
    println!("🔍 Starting system monitoring (interval: {}s)", interval);
    if let Some(duration_s) = duration {
        println!("Duration: {}s", duration_s);
    }
    println!("Press Ctrl+C to stop\n");
    
    let start_time = std::time::Instant::now();
    let mut iteration = 0;
    
    loop {
        iteration += 1;
        println!("📊 Monitoring iteration {} ({})", iteration, 
                 format_duration(start_time.elapsed()));
        
        let monitoring_result = match &service {
            Some(service_name) => {
                manager.monitor_service(service_name).await?
            }
            None => {
                manager.monitor_system().await?
            }
        };
        
        display_monitoring_results(&monitoring_result).await?;
        
        // Check if we should stop based on duration
        if let Some(duration_s) = duration {
            if start_time.elapsed().as_secs() >= duration_s {
                println!("✅ Monitoring completed (duration reached)");
                break;
            }
        }
        
        println!("\n{}", "─".repeat(80));
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
    
    Ok(())
}

/// Handle dashboard command
pub async fn handle_dashboard(interval: u64, refresh: bool) -> Result<()> {
    println!("🎛️  Starting BiomeOS Dashboard...");
    
    // Initialize the manager and TUI dashboard
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;
    let mut dashboard = BiomeOSDashboard::new(manager);
    
    if refresh {
        println!("Auto-refresh enabled (interval: {}s)", interval);
    }
    
    // Run the dashboard - this will take over the terminal
    dashboard.run().await?;
    
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
        println!("📜 Following logs for service '{}' (Press Ctrl+C to stop)", service);
    } else {
        println!("📜 Fetching logs for service '{}'", service);
    }
    
    let logs_result = manager.get_service_logs(&service, follow, tail, since.as_deref()).await?;
    
    display_logs_result(&service, &logs_result).await?;
    
    Ok(())
}

/// Handle exec command
pub async fn handle_exec(
    service: String,
    command: Vec<String>,
    interactive: bool,
) -> Result<()> {
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;
    
    let command_str = command.join(" ");
    println!("⚡ Executing '{}' in service '{}'", command_str, service);
    
    if interactive {
        println!("Interactive mode enabled");
    }
    
    let exec_result = manager.exec_in_service(&service, &command, interactive).await?;
    
    display_exec_result(&service, &command_str, &exec_result).await?;
    
    Ok(())
}

/// Handle scale command
pub async fn handle_scale(
    service: String,
    replicas: Option<u32>,
    auto: bool,
) -> Result<()> {
    let spinner = create_spinner(&format!("⚖️  Scaling service '{}'...", service));
    
    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;
    
    let scale_result = if auto {
        manager.enable_auto_scaling(&service).await?
    } else if let Some(replica_count) = replicas {
        manager.scale_service(&service, Some(replica_count), false).await?
    } else {
        return Err(anyhow::anyhow!("Must specify either --replicas or --auto"));
    };
    
    spinner.finish_with_message("✅ Scaling operation completed");
    
    display_scale_result(&service, &scale_result, auto).await?;
    
    Ok(())
}

/// Display monitoring results
async fn display_monitoring_results(results: &HashMap<String, Value>) -> Result<()> {
    // System overview
    if let Some(system) = results.get("system") {
        println!("🖥️  System Status:");
        
        if let Some(cpu) = system.get("cpu_usage_percent") {
            println!("  💻 CPU: {}%", cpu);
        }
        
        if let Some(memory) = system.get("memory") {
            if let Some(used_gb) = memory.get("used_gb") {
                if let Some(total_gb) = memory.get("total_gb") {
                    let percent = memory.get("usage_percent").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    println!("  🧠 Memory: {:.1}GB / {:.1}GB ({:.1}%)", used_gb, total_gb, percent);
                }
            }
        }
        
        if let Some(disk) = system.get("disk") {
            if let Some(usage_percent) = disk.get("usage_percent") {
                println!("  💾 Disk: {}%", usage_percent);
            }
        }
        
        if let Some(load) = system.get("load_average") {
            if let Some(load_1m) = load.get("1m") {
                println!("  📊 Load: {}", load_1m);
            }
        }
    }
    
    // Service status
    if let Some(services) = results.get("services").and_then(|s| s.as_object()) {
        println!("\n🎯 Service Status ({} services):", services.len());
        
        for (service_name, service_data) in services {
            let status = service_data.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
            let health = service_data.get("health").and_then(|h| h.as_str()).unwrap_or("unknown");
            
            let status_icon = match status {
                "running" => "✅",
                "starting" => "🔄",
                "stopping" => "⏹️",
                "stopped" => "⏸️",
                "error" => "❌",
                _ => "❓",
            };
            
            let health_icon = match health {
                "Healthy" => "💚",
                "Degraded" => "💛",
                "Critical" => "🧡",
                "Unhealthy" => "❤️",
                _ => "🤍",
            };
            
            println!("  {} {} {}: {} | {}", status_icon, health_icon, service_name, status, health);
            
            // Show resource usage if available
            if let Some(resources) = service_data.get("resources") {
                if let Some(cpu) = resources.get("cpu_percent") {
                    if let Some(memory_mb) = resources.get("memory_mb") {
                        println!("    📊 CPU: {}% | Memory: {}MB", cpu, memory_mb);
                    }
                }
            }
        }
    }
    
    // Network activity
    if let Some(network) = results.get("network") {
        println!("\n🌐 Network Activity:");
        if let Some(bytes_in) = network.get("bytes_in_per_sec") {
            if let Some(bytes_out) = network.get("bytes_out_per_sec") {
                println!("  ↓ In: {}/s | ↑ Out: {}/s", 
                         format_bytes(bytes_in.as_u64().unwrap_or(0)),
                         format_bytes(bytes_out.as_u64().unwrap_or(0)));
            }
        }
        
        if let Some(connections) = network.get("active_connections") {
            println!("  🔗 Active connections: {}", connections);
        }
    }
    
    // Alerts or issues
    if let Some(alerts) = results.get("alerts").and_then(|a| a.as_array()) {
        if !alerts.is_empty() {
            println!("\n🚨 Active Alerts:");
            for alert in alerts {
                if let Some(message) = alert.get("message") {
                    let severity = alert.get("severity").and_then(|s| s.as_str()).unwrap_or("info");
                    let icon = match severity {
                        "critical" => "🔴",
                        "warning" => "🟡",
                        "info" => "🔵",
                        _ => "⚪",
                    };
                    println!("  {} {}", icon, message);
                }
            }
        }
    }
    
    Ok(())
}

/// Display logs results
async fn display_logs_result(
    service: &str,
    results: &HashMap<String, Value>,
) -> Result<()> {
    if let Some(logs) = results.get("logs").and_then(|l| l.as_array()) {
        println!("📜 Logs for service '{}' ({} entries):", service, logs.len());
        println!();
        
        for log_entry in logs {
            let timestamp = log_entry.get("timestamp").and_then(|t| t.as_str()).unwrap_or("unknown");
            let level = log_entry.get("level").and_then(|l| l.as_str()).unwrap_or("info");
            let message = log_entry.get("message").and_then(|m| m.as_str()).unwrap_or("");
            
            let level_icon = match level {
                "error" => "❌",
                "warn" => "⚠️",
                "info" => "ℹ️",
                "debug" => "🐛",
                "trace" => "🔍",
                _ => "📝",
            };
            
            println!("{} [{}] {}: {}", level_icon, timestamp, level.to_uppercase(), message);
        }
    } else {
        println!("📜 No logs available for service '{}'", service);
    }
    
    if let Some(follow_info) = results.get("following") {
        if follow_info.as_bool().unwrap_or(false) {
            println!("\n🔄 Following logs... (Press Ctrl+C to stop)");
        }
    }
    
    Ok(())
}

/// Display exec results
async fn display_exec_result(
    service: &str,
    command: &str,
    results: &HashMap<String, Value>,
) -> Result<()> {
    println!("⚡ Execution results for '{}' in service '{}':", command, service);
    
    if let Some(exit_code) = results.get("exit_code") {
        let icon = if exit_code.as_i64().unwrap_or(-1) == 0 { "✅" } else { "❌" };
        println!("{} Exit code: {}", icon, exit_code);
    }
    
    if let Some(stdout) = results.get("stdout").and_then(|s| s.as_str()) {
        if !stdout.trim().is_empty() {
            println!("\n📤 STDOUT:");
            for line in stdout.lines() {
                println!("  {}", line);
            }
        }
    }
    
    if let Some(stderr) = results.get("stderr").and_then(|s| s.as_str()) {
        if !stderr.trim().is_empty() {
            println!("\n📥 STDERR:");
            for line in stderr.lines() {
                println!("  {}", line);
            }
        }
    }
    
    if let Some(duration_ms) = results.get("duration_ms") {
        println!("\n⏱️  Execution time: {}ms", duration_ms);
    }
    
    Ok(())
}

/// Display scaling results
async fn display_scale_result(
    service: &str,
    results: &HashMap<String, Value>,
    auto_scaling: bool,
) -> Result<()> {
    let operation = if auto_scaling { "Auto-scaling" } else { "Manual scaling" };
    println!("⚖️  {} results for service '{}':", operation, service);
    
    if let Some(status) = results.get("status") {
        let icon = match status.as_str() {
            Some("success") => "✅",
            Some("in_progress") => "🔄",
            Some("failed") => "❌",
            _ => "🔹",
        };
        println!("{} Status: {}", icon, status);
    }
    
    if let Some(current_replicas) = results.get("current_replicas") {
        println!("📊 Current replicas: {}", current_replicas);
    }
    
    if let Some(target_replicas) = results.get("target_replicas") {
        println!("🎯 Target replicas: {}", target_replicas);
    }
    
    if auto_scaling {
        if let Some(auto_scaling_info) = results.get("auto_scaling") {
            println!("\n🤖 Auto-scaling configuration:");
            if let Some(min_replicas) = auto_scaling_info.get("min_replicas") {
                println!("  📉 Min replicas: {}", min_replicas);
            }
            if let Some(max_replicas) = auto_scaling_info.get("max_replicas") {
                println!("  📈 Max replicas: {}", max_replicas);
            }
            if let Some(cpu_threshold) = auto_scaling_info.get("cpu_threshold_percent") {
                println!("  🖥️  CPU threshold: {}%", cpu_threshold);
            }
        }
    }
    
    if let Some(message) = results.get("message") {
        println!("\n💬 {}", message);
    }
    
    Ok(())
}

/// Format bytes helper (duplicated from utils for now)
fn format_bytes(bytes: u64) -> String {
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