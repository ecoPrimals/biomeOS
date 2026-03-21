// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Console display helpers for monitor command output.

use serde_json::Value;
use std::collections::HashMap;

use super::format::{
    format_alert_rows, format_exec_output, format_log_entry, format_network_activity,
    format_scale_output, format_service_rows, format_system_overview,
};

/// Display monitoring results (thin wrapper)
pub(super) fn display_monitoring_results(results: &HashMap<String, Value>) {
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
pub(super) fn display_logs_result(service: &str, results: &HashMap<String, Value>) {
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
pub(super) fn display_exec_result(service: &str, command: &str, results: &HashMap<String, Value>) {
    println!("⚡ Execution results for '{command}' in service '{service}':");

    let lines = format_exec_output(results);
    for line in lines {
        println!("{line}");
    }
}

/// Display scaling results (thin wrapper)
pub(super) fn display_scale_result(
    service: &str,
    results: &HashMap<String, Value>,
    auto_scaling: bool,
) {
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
