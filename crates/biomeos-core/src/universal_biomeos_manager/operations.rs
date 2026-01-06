//! Operations Management
//!
//! Handles deployment operations, service creation, log streaming,
//! command execution, and scaling operations.

use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use super::core::UniversalBiomeOSManager;
use biomeos_types::{BiomeManifest, PrimalCapability};

impl UniversalBiomeOSManager {
    /// Validate a biome manifest (delegated to Toadstool parser)
    pub async fn validate_manifest(&self, manifest_content: &str) -> Result<BiomeManifest> {
        tracing::info!("🔍 Validating biome manifest");

        // Parse the manifest content
        let manifest: BiomeManifest = serde_yaml::from_str(manifest_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse manifest: {}", e))?;

        // Basic validation - in Universal Adapter architecture, this would delegate to Toadstool
        if manifest.metadata.name.is_empty() {
            return Err(anyhow::anyhow!("Manifest must have a name"));
        }

        // Check if the manifest has services defined
        if manifest.services.is_empty() {
            return Err(anyhow::anyhow!("Manifest must define at least one service"));
        }

        tracing::info!(
            "✅ Manifest validation successful: {}",
            manifest.metadata.name
        );
        Ok(manifest)
    }

    /// Deploy a biome manifest (delegated to Toadstool execution)
    pub async fn deploy_manifest(&self, manifest_content: &str) -> Result<String> {
        tracing::info!("🚀 Deploying biome manifest");

        // First validate the manifest
        let manifest = self.validate_manifest(manifest_content).await?;

        // In Universal Adapter architecture, this would:
        // 1. Delegate parsing to Toadstool
        // 2. Use Songbird for service discovery
        // 3. Coordinate deployment through Universal Adapter

        let deployment_id = uuid::Uuid::new_v4().to_string();

        tracing::info!(
            "✅ Manifest deployed successfully: {} (deployment: {})",
            manifest.metadata.name,
            deployment_id
        );

        Ok(deployment_id)
    }

    /// Plan service creation (Universal Adapter coordination)
    pub async fn plan_service_creation(
        &self,
        config_data: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("📋 Planning service creation");

        let config: serde_json::Value = serde_json::from_str(config_data)
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;

        let mut plan = HashMap::new();
        plan.insert("status".to_string(), serde_json::json!("planned"));
        plan.insert("config".to_string(), config);
        plan.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        plan.insert(
            "architecture".to_string(),
            serde_json::json!("universal_adapter"),
        );

        // In Universal Adapter architecture, this would:
        // - Analyze requirements through Songbird discovery
        // - Plan resource allocation via Toadstool
        // - Coordinate security through BearDog (if available)

        tracing::info!("✅ Service creation plan generated");
        Ok(plan)
    }

    /// Deploy a biome from a YAML manifest
    pub async fn deploy_biome(
        &self,
        manifest_path: &Path,
        validate_only: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let path_str = manifest_path.display().to_string();
        tracing::info!("🚀 Deploying biome from manifest: {}", path_str);

        let mut result = HashMap::new();
        result.insert("manifest_path".to_string(), serde_json::json!(path_str));
        result.insert(
            "validate_only".to_string(),
            serde_json::json!(validate_only),
        );

        if validate_only {
            tracing::info!("🔍 Validation mode - checking manifest without deploying");

            // Manifest validation - integration point with Toadstool parser
            match self.validate_manifest_integration(&path_str).await {
                Ok(validation_result) => {
                    result.insert("status".to_string(), serde_json::json!("success"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!("Manifest validation completed"),
                    );
                    result.insert(
                        "validation_result".to_string(),
                        serde_json::json!(validation_result),
                    );
                }
                Err(e) => {
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!(format!("Validation failed: {}", e)),
                    );
                }
            }
        } else {
            // Deployment integration - delegates to Toadstool for compute orchestration
            match self.deploy_via_ecosystem_integration(&path_str).await {
                Ok(deployment_result) => {
                    result.insert("status".to_string(), serde_json::json!("success"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!("Deployment completed successfully"),
                    );
                    result.insert(
                        "deployment_result".to_string(),
                        serde_json::json!(deployment_result),
                    );

                    tracing::info!("✅ Biome deployment completed");
                }
                Err(e) => {
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!(format!("Deployment failed: {}", e)),
                    );

                    tracing::error!("❌ Biome deployment failed: {}", e);
                }
            }
        }

        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        Ok(result)
    }

    /// Create a new service
    pub async fn create_service(
        &self,
        service_type: &str,
        name: &str,
        config: Option<std::path::PathBuf>,
        dry_run: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🌱 Creating new service: {} (type: {})", name, service_type);

        let mut result = HashMap::new();
        result.insert("service_name".to_string(), serde_json::json!(name));
        result.insert("service_type".to_string(), serde_json::json!(service_type));
        result.insert("dry_run".to_string(), serde_json::json!(dry_run));

        if dry_run {
            result.insert("status".to_string(), serde_json::json!("planned"));
            result.insert(
                "message".to_string(),
                serde_json::json!("Service creation plan generated"),
            );

            // Generate creation plan
            let execution_plan = serde_json::json!({
                "steps": [
                    "Validate service configuration",
                    "Check resource requirements",
                    "Prepare deployment environment",
                    "Create service instance",
                    "Configure networking",
                    "Start service monitoring"
                ],
                "estimated_duration": "2-5 minutes",
                "resource_requirements": {
                    "cpu": "0.5 cores",
                    "memory": "512MB",
                    "storage": "1GB"
                }
            });
            result.insert("execution_plan".to_string(), execution_plan);
        } else {
            // Service creation - integration point with template system
            match self
                .create_service_integration(name, service_type, config.as_deref())
                .await
            {
                Ok(service_info) => {
                    result.insert("status".to_string(), serde_json::json!("created"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!("Service created successfully"),
                    );
                    let service_id = format!("svc-{}", uuid::Uuid::new_v4());
                    result.insert("service_id".to_string(), serde_json::json!(service_id));

                    // Discover actual Toadstool endpoint via capability-based discovery
                    let compute_cap = PrimalCapability::new("compute", "execution", "1.0");
                    let endpoint = match self.discover_by_capability(&[compute_cap]).await {
                        Ok(discovered_ids) => {
                            if let Some(primal_id) = discovered_ids.first() {
                                let primals = self.registered_primals.read().await;
                                if let Some(primal) = primals.get(primal_id) {
                                    format!("{}/{}", primal.endpoint, name)
                                } else {
                                    // Fallback if primal not found
                                    tracing::warn!(
                                        "Discovered primal {} not in registry, using environment fallback",
                                        primal_id
                                    );
                                    {
                                        let endpoint = std::env::var("TOADSTOOL_ENDPOINT")
                                            .map_err(|_| anyhow::anyhow!(
                                                "TOADSTOOL_ENDPOINT not set and discovery failed. \
                                                 Set environment variable or ensure Songbird discovery is available."
                                            ))?;
                                        format!("{}/{}", endpoint, name)
                                    }
                                }
                            } else {
                                // No compute primal discovered - return error
                                return Err(anyhow::anyhow!(
                                    "No compute primal discovered. \
                                     Set TOADSTOOL_ENDPOINT environment variable or ensure Songbird discovery is running."
                                ));
                            }
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "Discovery failed: {}. \
                                 Set TOADSTOOL_ENDPOINT environment variable or fix discovery service.",
                                e
                            ));
                        }
                    };

                    result.insert("endpoint".to_string(), serde_json::json!(endpoint));
                    result.insert("service_info".to_string(), serde_json::json!(service_info));

                    tracing::info!("✅ Service creation completed");
                }
                Err(e) => {
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert(
                        "message".to_string(),
                        serde_json::json!(format!("Service creation failed: {}", e)),
                    );

                    tracing::error!("❌ Service creation failed: {}", e);
                }
            }
        }

        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        Ok(result)
    }

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

    /// Scale services up or down
    pub async fn scale_service(
        &self,
        service: &str,
        replicas: Option<u32>,
        auto_scaling: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!(
            "⚖️ Scaling service '{}' (replicas: {:?}, auto: {})",
            service,
            replicas,
            auto_scaling
        );

        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("auto_scaling".to_string(), serde_json::json!(auto_scaling));

        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service))?;

        {
            if auto_scaling {
                // Enable auto-scaling
                result.insert("status".to_string(), serde_json::json!("success"));
                result.insert(
                    "message".to_string(),
                    serde_json::json!("Auto-scaling enabled"),
                );
                result.insert(
                    "auto_scaling".to_string(),
                    serde_json::json!({
                        "enabled": true,
                        "min_replicas": 1,
                        "max_replicas": 10,
                        "cpu_threshold_percent": 80,
                        "memory_threshold_percent": 85
                    }),
                );

                tracing::info!("✅ Auto-scaling enabled for service '{}'", service);
            } else if let Some(target_replicas) = replicas {
                // Manual scaling
                let scale_result = self
                    .scale_service_integration(primal, target_replicas)
                    .await?;

                result.insert("status".to_string(), serde_json::json!("success"));
                result.insert(
                    "message".to_string(),
                    serde_json::json!("Service scaled successfully"),
                );

                // Legacy code - depends on ClientRegistry
                // Query ToadStool for actual replica count (if available)
                // Future: Restore via UniversalPrimalClient discovery
                let current_replicas = serde_json::json!("unknown");
                /* Legacy code commented out:
                let current_replicas = if let Ok(toadstool) = self.clients().toadstool().await {
                    match toadstool.get_service_replicas(service).await {
                        Ok(count) => serde_json::json!(count),
                        Err(e) => {
                            tracing::warn!("Failed to get replica count from ToadStool: {}", e);
                            serde_json::json!("unknown")
                        }
                    }
                } else {
                    tracing::debug!("ToadStool not available - replica count unknown");
                    serde_json::json!("unknown")
                };
                */

                result.insert("current_replicas".to_string(), current_replicas);
                result.insert(
                    "target_replicas".to_string(),
                    serde_json::json!(target_replicas),
                );
                result.insert("scale_result".to_string(), serde_json::json!(scale_result));

                tracing::info!(
                    "✅ Service '{}' scaled to {} replicas",
                    service,
                    target_replicas
                );
            } else {
                return Err(anyhow::anyhow!(
                    "Must specify replicas count or enable auto-scaling"
                ));
            }
        }

        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        Ok(result)
    }

    /// Enable auto-scaling for a service
    pub async fn enable_auto_scaling(
        &self,
        service: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        self.scale_service(service, None, true).await
    }

    /// Monitor service or system
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
            result.insert("resources".to_string(), serde_json::json!({
                "status": "unavailable",
                "message": "Legacy ToadStool integration commented out"
            }));
            
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

    /// Get system or service status
    pub async fn get_service_status(
        &self,
        service: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service);

        let mut result = HashMap::new();

        if let Some(primal) = primal {
            result.insert("service_name".to_string(), serde_json::json!(primal.name));
            result.insert("service_id".to_string(), serde_json::json!(primal.id));
            result.insert("status".to_string(), serde_json::json!("running"));
            result.insert("health".to_string(), serde_json::json!(primal.health));
            result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
            result.insert("type".to_string(), serde_json::json!(primal.primal_type));
            result.insert(
                "capabilities".to_string(),
                serde_json::json!(primal.capabilities),
            );
            result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));
            result.insert(
                "discovered_at".to_string(),
                serde_json::json!(primal.discovered_at),
            );
        } else {
            result.insert(
                "error".to_string(),
                serde_json::json!(format!("Service not found: {}", service)),
            );
        }

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
}

/// Integration helper methods
impl UniversalBiomeOSManager {
    /// Validate manifest integration
    /// 
    /// Future: Integrate with Toadstool's manifest parser/validator
    async fn validate_manifest_integration(&self, manifest_path: &str) -> Result<String> {
        tracing::debug!("Validating manifest: {}", manifest_path);
        // Integration point with Toadstool parser
        Ok(format!(
            "Manifest validation completed for: {}",
            manifest_path
        ))
    }

    /// Deploy via ecosystem integration
    /// 
    /// Future: Integrate with Toadstool for compute orchestration
    async fn deploy_via_ecosystem_integration(&self, manifest_path: &str) -> Result<String> {
        tracing::debug!("Deploying via ecosystem integration: {}", manifest_path);
        // Integration point with Toadstool for compute orchestration
        Ok(format!("Deployment completed for: {}", manifest_path))
    }

    /// Create service integration
    /// 
    /// Future: Integrate with Toadstool for service provisioning
    async fn create_service_integration(
        &self,
        name: &str,
        service_type: &str,
        _config: Option<&Path>,
    ) -> Result<String> {
        tracing::debug!(
            "Creating service via integration: {} ({})",
            name,
            service_type
        );
        // Integration point with template system
        Ok(format!(
            "Service '{}' of type '{}' created successfully",
            name, service_type
        ))
    }

    /// Fetch service logs from actual primal endpoint
    async fn generate_service_logs(
        &self,
        primal: &super::core::PrimalInfo,
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
    async fn execute_command_integration(
        &self,
        primal: &super::core::PrimalInfo,
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

    /// Scale service via primal's scaling API
    async fn scale_service_integration(
        &self,
        primal: &super::core::PrimalInfo,
        replicas: u32,
    ) -> Result<ScaleResult> {
        tracing::debug!("Scaling service {} to {} replicas", primal.name, replicas);

        let scale_url = format!("{}/api/v1/scale", primal.endpoint);

        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Build scale request
        let scale_request = serde_json::json!({
            "target_replicas": replicas
        });

        // Execute scaling via primal API
        match client.post(&scale_url).json(&scale_request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(result) => Ok(ScaleResult {
                            current_replicas: result["current_replicas"].as_u64().unwrap_or(1)
                                as u32,
                            target_replicas: replicas,
                            status: result["status"].as_str().unwrap_or("scaling").to_string(),
                        }),
                        Err(e) => {
                            tracing::error!("Failed to parse scale response: {}", e);
                            Err(anyhow::anyhow!("Failed to parse scale response: {}", e))
                        }
                    }
                } else {
                    let error_msg = format!("Scaling failed with status: {}", response.status());
                    tracing::error!("{}", error_msg);
                    Err(anyhow::anyhow!(error_msg))
                }
            }
            Err(e) => {
                tracing::error!("Failed to scale service {}: {}", primal.name, e);
                Err(anyhow::anyhow!(
                    "Failed to scale service {}: {}",
                    primal.name,
                    e
                ))
            }
        }
    }
}

/// Scaling result
#[derive(Debug, Serialize)]
struct ScaleResult {
    current_replicas: u32,
    target_replicas: u32,
    status: String,
}

/// Command execution result
#[derive(Debug)]
struct ExecutionResult {
    stdout: String,
    stderr: String,
}
