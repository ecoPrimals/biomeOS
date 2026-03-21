// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Monitor / dashboard / logs / exec / scale command handlers.

use anyhow::Result;
use std::time::Duration;

use biomeos_core::UniversalBiomeOSManager;

use super::display::{
    display_exec_result, display_logs_result, display_monitoring_results, display_scale_result,
};
use super::format::should_stop_monitoring;
use crate::commands::utils::{create_spinner, format_duration};

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

/// Handle dashboard command — redirects to petalTongue (the universal UI primal).
pub async fn handle_dashboard(_interval: u64, _refresh: bool) -> Result<()> {
    eprintln!("The built-in TUI dashboard has been removed.");
    eprintln!("petalTongue is the universal UI primal for the ecoPrimals ecosystem.");
    eprintln!();
    eprintln!("  To launch petalTongue:  biomeos start petaltongue");
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
