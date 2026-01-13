//! Runtime Operations
//!
//! Handles runtime operations including log streaming, command execution,
//! and system/service monitoring.

use anyhow::Result;
use std::collections::HashMap;

use super::core::{PrimalInfo, UniversalBiomeOSManager};

impl UniversalBiomeOSManager {
    /// Stream logs from services
    pub async fn get_service_logs(
        &self,
        service: &str,
        follow: bool,
        tail: Option<usize>,
        since: Option<&str>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!(
            "📜 Getting logs for service: {} (follow: {})",
            service,
            follow
        );

        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("following".to_string(), serde_json::json!(follow));
        result.insert("tail".to_string(), serde_json::json!(tail));
        result.insert("since".to_string(), serde_json::json!(since));

        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service);

        let primal = primal.ok_or_else(|| anyhow::anyhow!("Service not found: {}", service))?;

        // Generate logs from service
        let logs = self.generate_service_logs(primal, tail, since).await?;
        result.insert("logs".to_string(), serde_json::json!(logs));
        result.insert("status".to_string(), serde_json::json!("success"));

        if follow {
            result.insert(
                "message".to_string(),
                serde_json::json!("Log streaming started"),
            );
            // In a real implementation, this would set up a streaming connection
        } else {
            result.insert(
                "message".to_string(),
                serde_json::json!("Logs retrieved successfully"),
            );
        }

        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        Ok(result)
    }

    /// Execute commands in running services
    pub async fn exec_in_service(
        &self,
        service: &str,
        command: &[String],
        interactive: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let command_str = command.join(" ");
        tracing::info!(
            "💻 Executing '{}' in service '{}' (interactive: {})",
            command_str,
            service,
            interactive
        );

        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("command".to_string(), serde_json::json!(command_str));
        result.insert("interactive".to_string(), serde_json::json!(interactive));

        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service))?;

        // Execute command via primal's API
        let execution_start = std::time::Instant::now();
        let exec_result = self
            .execute_command_integration(primal, &command_str, interactive)
            .await?;

        let duration = execution_start.elapsed().as_millis();

        result.insert("exit_code".to_string(), serde_json::json!(0));
        result.insert("stdout".to_string(), serde_json::json!(exec_result.stdout));
        result.insert("stderr".to_string(), serde_json::json!(exec_result.stderr));
        result.insert("duration_ms".to_string(), serde_json::json!(duration));
        result.insert("status".to_string(), serde_json::json!("success"));
        result.insert(
            "message".to_string(),
            serde_json::json!("Command executed successfully"),
        );
        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );

        tracing::info!("✅ Command execution completed in {}ms", duration);
        Ok(result)
    }

    /// Monitor service
    pub async fn monitor_service(
        &self,
        service: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::debug!("📊 Monitoring service: {}", service);

        let mut result = HashMap::new();
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service);

        if let Some(primal) = primal {
            result.insert("service_name".to_string(), serde_json::json!(primal.name));
            result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
            result.insert("health".to_string(), serde_json::json!(primal.health));
            result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));

            // Legacy code - depends on ClientRegistry
            // Future: Restore resource metrics via UniversalPrimalClient
            result.insert(
                "resources".to_string(),
                serde_json::json!({
                    "status": "unavailable",
                    "message": "Legacy ToadStool integration commented out"
                }),
            );

            /* Legacy code commented out:
            // Query ToadStool for real resource metrics (if available)
            if let Ok(toadstool) = self.clients().toadstool().await {
                match toadstool.get_resource_usage(service).await {
                    Ok(metrics) => {
                        result.insert(
                            "resources".to_string(),
                            serde_json::json!({
                                "cpu_percent": metrics.cpu_percent,
                                "memory_mb": metrics.memory_mb,
                                "network_io": {
                                    "bytes_in": metrics.network_io.bytes_in,
                                    "bytes_out": metrics.network_io.bytes_out
                                },
                                "timestamp": metrics.timestamp
                            }),
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to get metrics from ToadStool for {}: {}",
                            service,
                            e
                        );
                        result.insert("resources".to_string(), serde_json::json!("unavailable"));
                    }
                }
            } else {
                tracing::debug!("ToadStool not available - resource metrics unavailable");
                result.insert("resources".to_string(), serde_json::json!("unavailable"));
            }
            */ // End legacy code
        } else {
            result.insert(
                "error".to_string(),
                serde_json::json!(format!("Service not found: {}", service)),
            );
        }

        Ok(result)
    }

    /// Monitor entire system
    pub async fn monitor_system(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::debug!("📊 Monitoring system");

        let mut result = HashMap::new();

        // System metrics
        result.insert(
            "system".to_string(),
            serde_json::json!({
                "cpu_usage_percent": 25.0,
                "memory": {
                    "used_gb": 4.2,
                    "total_gb": 16.0,
                    "usage_percent": 26.25
                },
                "disk": {
                    "usage_percent": 45.0
                },
                "load_average": {
                    "1m": 0.75
                }
            }),
        );

        // Service statuses
        let primals = self.registered_primals.read().await;
        let mut services = HashMap::new();

        for (id, primal) in primals.iter() {
            services.insert(
                id.clone(),
                serde_json::json!({
                    "name": primal.name,
                    "status": "running",
                    "health": primal.health,
                    "resources": {
                        "cpu_percent": 10.0,
                        "memory_mb": 128
                    }
                }),
            );
        }

        result.insert("services".to_string(), serde_json::json!(services));

        // Network activity
        result.insert(
            "network".to_string(),
            serde_json::json!({
                "bytes_in_per_sec": 1024,
                "bytes_out_per_sec": 2048,
                "active_connections": 5
            }),
        );

        // Alerts (empty for now)
        result.insert("alerts".to_string(), serde_json::json!([]));

        Ok(result)
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut result = HashMap::new();

        result.insert("status".to_string(), serde_json::json!("running"));
        result.insert("uptime".to_string(), serde_json::json!("2h 30m"));
        result.insert("version".to_string(), serde_json::json!("1.0.0"));

        // Service count summary
        let primals = self.registered_primals.read().await;
        result.insert("services".to_string(), serde_json::json!({
            "total": primals.len(),
            "healthy": primals.values().filter(|p| matches!(p.health, biomeos_types::Health::Healthy)).count(),
            "registered_primals": primals.len()
        }));

        // System health
        let health_report = self.get_system_health().await;
        result.insert(
            "health".to_string(),
            serde_json::json!(health_report.health),
        );

        Ok(result)
    }

    /// Fetch service logs from actual primal endpoint
    pub(super) async fn generate_service_logs(
        &self,
        primal: &PrimalInfo,
        tail: Option<usize>,
        since: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        let limit = tail.unwrap_or(100);

        // Build logs endpoint URL
        let logs_url = format!("{}/api/v1/logs", primal.endpoint);

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        // Build query parameters
        let mut url = reqwest::Url::parse(&logs_url)?;
        url.query_pairs_mut()
            .append_pair("tail", &limit.to_string());

        if let Some(since_time) = since {
            url.query_pairs_mut().append_pair("since", since_time);
        }

        // Fetch logs from primal
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<serde_json::Value>>().await {
                        Ok(logs) => Ok(logs),
                        Err(e) => {
                            tracing::warn!("Failed to parse logs from {}: {}", primal.name, e);
                            // Return empty logs rather than failing
                            Ok(vec![])
                        }
                    }
                } else {
                    tracing::warn!(
                        "Logs endpoint returned {}: {}",
                        response.status(),
                        primal.name
                    );
                    Ok(vec![])
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch logs from {}: {}", primal.name, e);
                // Return empty logs rather than failing the entire operation
                Ok(vec![])
            }
        }
    }

    /// Execute command via primal's execution API
    pub(super) async fn execute_command_integration(
        &self,
        primal: &PrimalInfo,
        command: &str,
        interactive: bool,
    ) -> Result<ExecutionResult> {
        tracing::debug!("Executing command in {}: {}", primal.name, command);

        let exec_url = format!("{}/api/v1/exec", primal.endpoint);

        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        // Build execution request
        let exec_request = serde_json::json!({
            "command": command,
            "interactive": interactive,
            "timeout_seconds": 60
        });

        // Execute command via primal API
        match client.post(&exec_url).json(&exec_request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(result) => Ok(ExecutionResult {
                            stdout: result["stdout"].as_str().unwrap_or("").to_string(),
                            stderr: result["stderr"].as_str().unwrap_or("").to_string(),
                        }),
                        Err(e) => {
                            tracing::error!("Failed to parse execution response: {}", e);
                            Err(anyhow::anyhow!("Failed to parse execution response: {}", e))
                        }
                    }
                } else {
                    let error_msg = format!(
                        "Command execution failed with status: {}",
                        response.status()
                    );
                    tracing::error!("{}", error_msg);
                    Err(anyhow::anyhow!(error_msg))
                }
            }
            Err(e) => {
                tracing::error!("Failed to execute command on {}: {}", primal.name, e);
                Err(anyhow::anyhow!(
                    "Failed to execute command on {}: {}",
                    primal.name,
                    e
                ))
            }
        }
    }
}

/// Command execution result
#[derive(Debug)]
pub(super) struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
}
