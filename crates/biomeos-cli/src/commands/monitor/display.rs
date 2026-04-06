// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use super::{
        display_exec_result, display_logs_result, display_monitoring_results, display_scale_result,
    };

    #[test]
    fn display_monitoring_results_runs_for_full_map() {
        let mut m = HashMap::new();
        m.insert(
            "system".to_string(),
            json!({"cpu_usage_percent": 1, "memory": {"used_gb": 1.0, "total_gb": 2.0, "usage_percent": 50.0}}),
        );
        m.insert(
            "services".to_string(),
            json!({"svc": {"status": "running", "health": "Healthy"}}),
        );
        m.insert(
            "network".to_string(),
            json!({"bytes_in_per_sec": 10, "bytes_out_per_sec": 20, "active_connections": 1}),
        );
        m.insert(
            "alerts".to_string(),
            json!([{"message": "x", "severity": "info"}]),
        );
        display_monitoring_results(&m);
    }

    #[test]
    fn display_logs_result_with_entries_and_follow() {
        let mut m = HashMap::new();
        m.insert(
            "logs".to_string(),
            json!([{"timestamp": "t", "level": "info", "message": "ok"}]),
        );
        m.insert("following".to_string(), json!(true));
        display_logs_result("svc", &m);
    }

    #[test]
    fn display_logs_result_no_logs_branch() {
        let m = HashMap::new();
        display_logs_result("missing", &m);
    }

    #[test]
    fn display_exec_result_formats_output() {
        let mut m = HashMap::new();
        m.insert("exit_code".to_string(), json!(0));
        m.insert("stdout".to_string(), json!("out"));
        display_exec_result("svc", "cmd", &m);
    }

    #[test]
    fn display_scale_manual_and_auto() {
        let mut manual = HashMap::new();
        manual.insert("status".to_string(), json!("success"));
        display_scale_result("svc", &manual, false);

        let mut auto: HashMap<String, serde_json::Value> = HashMap::new();
        auto.insert("status".to_string(), json!("success"));
        auto.insert(
            "auto_scaling".to_string(),
            json!({"min_replicas": 1, "max_replicas": 3, "cpu_threshold_percent": 70}),
        );
        display_scale_result("svc", &auto, true);
    }
}
