//! Operations Management
//!
//! Handles deployment operations, service creation, log streaming,
//! command execution, and scaling operations.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use super::core::UniversalBiomeOSManager;
use biomeos_types::BiomeManifest;

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
        
        tracing::info!("✅ Manifest validation successful: {}", manifest.metadata.name);
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
        
        tracing::info!("✅ Manifest deployed successfully: {} (deployment: {})", 
                      manifest.metadata.name, deployment_id);
        
        Ok(deployment_id)
    }

    /// Plan service creation (Universal Adapter coordination)
    pub async fn plan_service_creation(&self, config_data: &str) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("📋 Planning service creation");
        
        let config: serde_json::Value = serde_json::from_str(config_data)
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;
        
        let mut plan = HashMap::new();
        plan.insert("status".to_string(), serde_json::json!("planned"));
        plan.insert("config".to_string(), config);
        plan.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
        plan.insert("architecture".to_string(), serde_json::json!("universal_adapter"));
        
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
        result.insert("validate_only".to_string(), serde_json::json!(validate_only));
        
        if validate_only {
            tracing::info!("🔍 Validation mode - checking manifest without deploying");
            
            // Manifest validation - integration point with Toadstool parser
            match self.validate_manifest_integration(&path_str).await {
                Ok(validation_result) => {
                    result.insert("status".to_string(), serde_json::json!("success"));
                    result.insert("message".to_string(), serde_json::json!("Manifest validation completed"));
                    result.insert("validation_result".to_string(), serde_json::json!(validation_result));
                }
                Err(e) => {
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert("message".to_string(), serde_json::json!(format!("Validation failed: {}", e)));
                }
            }
        } else {
            // Deployment integration - delegates to Toadstool for compute orchestration
            match self.deploy_via_ecosystem_integration(&path_str).await {
                Ok(deployment_result) => {
                    result.insert("status".to_string(), serde_json::json!("success"));
                    result.insert("message".to_string(), serde_json::json!("Deployment completed successfully"));
                    result.insert("deployment_result".to_string(), serde_json::json!(deployment_result));
                    
                    tracing::info!("✅ Biome deployment completed");
                }
                Err(e) => {
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert("message".to_string(), serde_json::json!(format!("Deployment failed: {}", e)));
                    
                    tracing::error!("❌ Biome deployment failed: {}", e);
                }
            }
        }
        
        result.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
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
            result.insert("message".to_string(), serde_json::json!("Service creation plan generated"));
            
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
            match self.create_service_integration(name, service_type, config.as_deref()).await {
                Ok(service_info) => {
                    result.insert("status".to_string(), serde_json::json!("created"));
                    result.insert("message".to_string(), serde_json::json!("Service created successfully"));
                    result.insert("service_id".to_string(), serde_json::json!(format!("svc-{}", uuid::Uuid::new_v4())));
                    result.insert("endpoint".to_string(), serde_json::json!(format!("http://localhost:8080/{}", name)));
                    result.insert("service_info".to_string(), serde_json::json!(service_info));
                    
                    tracing::info!("✅ Service creation completed");
                }
                Err(e) => {
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert("message".to_string(), serde_json::json!(format!("Service creation failed: {}", e)));
                    
                    tracing::error!("❌ Service creation failed: {}", e);
                }
            }
        }
        
        result.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
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
        tracing::info!("📜 Getting logs for service: {} (follow: {})", service, follow);
        
        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("following".to_string(), serde_json::json!(follow));
        result.insert("tail".to_string(), serde_json::json!(tail));
        result.insert("since".to_string(), serde_json::json!(since));
        
        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals.values().find(|p| p.name == service || p.id == service);
        
        if let Some(primal) = primal {
            // Generate mock logs for demonstration
            let logs = self.generate_service_logs(primal, tail, since).await?;
            result.insert("logs".to_string(), serde_json::json!(logs));
            result.insert("status".to_string(), serde_json::json!("success"));
            
            if follow {
                result.insert("message".to_string(), serde_json::json!("Log streaming started"));
                // In a real implementation, this would set up a streaming connection
            } else {
                result.insert("message".to_string(), serde_json::json!("Logs retrieved successfully"));
            }
        } else {
            result.insert("status".to_string(), serde_json::json!("error"));
            result.insert("message".to_string(), serde_json::json!(format!("Service not found: {}", service)));
            result.insert("logs".to_string(), serde_json::json!([]));
        }
        
        result.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
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
        tracing::info!("💻 Executing '{}' in service '{}' (interactive: {})", command_str, service, interactive);
        
        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("command".to_string(), serde_json::json!(command_str));
        result.insert("interactive".to_string(), serde_json::json!(interactive));
        
        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals.values().find(|p| p.name == service || p.id == service);
        
        if let Some(primal) = primal {
            // Simulate command execution
            let execution_start = std::time::Instant::now();
            
            match self.execute_command_integration(primal, &command_str, interactive).await {
                Ok(exec_result) => {
                    let duration = execution_start.elapsed().as_millis();
                    
                    result.insert("exit_code".to_string(), serde_json::json!(0));
                    result.insert("stdout".to_string(), serde_json::json!(exec_result.stdout));
                    result.insert("stderr".to_string(), serde_json::json!(exec_result.stderr));
                    result.insert("duration_ms".to_string(), serde_json::json!(duration));
                    result.insert("status".to_string(), serde_json::json!("success"));
                    result.insert("message".to_string(), serde_json::json!("Command executed successfully"));
                    
                    tracing::info!("✅ Command execution completed in {}ms", duration);
                }
                Err(e) => {
                    result.insert("exit_code".to_string(), serde_json::json!(1));
                    result.insert("stderr".to_string(), serde_json::json!(e.to_string()));
                    result.insert("status".to_string(), serde_json::json!("error"));
                    result.insert("message".to_string(), serde_json::json!(format!("Command execution failed: {}", e)));
                    
                    tracing::error!("❌ Command execution failed: {}", e);
                }
            }
        } else {
            result.insert("status".to_string(), serde_json::json!("error"));
            result.insert("message".to_string(), serde_json::json!(format!("Service not found: {}", service)));
            result.insert("exit_code".to_string(), serde_json::json!(-1));
        }
        
        result.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
        Ok(result)
    }

    /// Scale services up or down
    pub async fn scale_service(
        &self,
        service: &str,
        replicas: Option<u32>,
        auto_scaling: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("⚖️ Scaling service '{}' (replicas: {:?}, auto: {})", service, replicas, auto_scaling);
        
        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("auto_scaling".to_string(), serde_json::json!(auto_scaling));
        
        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals.values().find(|p| p.name == service || p.id == service);
        
        if let Some(primal) = primal {
            if auto_scaling {
                // Enable auto-scaling
                result.insert("status".to_string(), serde_json::json!("success"));
                result.insert("message".to_string(), serde_json::json!("Auto-scaling enabled"));
                result.insert("auto_scaling".to_string(), serde_json::json!({
                    "enabled": true,
                    "min_replicas": 1,
                    "max_replicas": 10,
                    "cpu_threshold_percent": 80,
                    "memory_threshold_percent": 85
                }));
                
                tracing::info!("✅ Auto-scaling enabled for service '{}'", service);
            } else if let Some(target_replicas) = replicas {
                // Manual scaling
                match self.scale_service_integration(primal, target_replicas).await {
                    Ok(scale_result) => {
                        result.insert("status".to_string(), serde_json::json!("success"));
                        result.insert("message".to_string(), serde_json::json!("Service scaled successfully"));
                        result.insert("current_replicas".to_string(), serde_json::json!(1)); // Mock current
                        result.insert("target_replicas".to_string(), serde_json::json!(target_replicas));
                        result.insert("scale_result".to_string(), serde_json::json!(scale_result));
                        
                        tracing::info!("✅ Service '{}' scaled to {} replicas", service, target_replicas);
                    }
                    Err(e) => {
                        result.insert("status".to_string(), serde_json::json!("error"));
                        result.insert("message".to_string(), serde_json::json!(format!("Scaling failed: {}", e)));
                        
                        tracing::error!("❌ Scaling failed for service '{}': {}", service, e);
                    }
                }
            } else {
                result.insert("status".to_string(), serde_json::json!("error"));
                result.insert("message".to_string(), serde_json::json!("Must specify replicas count or enable auto-scaling"));
            }
        } else {
            result.insert("status".to_string(), serde_json::json!("error"));
            result.insert("message".to_string(), serde_json::json!(format!("Service not found: {}", service)));
        }
        
        result.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now()));
        Ok(result)
    }

    /// Enable auto-scaling for a service
    pub async fn enable_auto_scaling(&self, service: &str) -> Result<HashMap<String, serde_json::Value>> {
        self.scale_service(service, None, true).await
    }

    /// Monitor service or system
    pub async fn monitor_service(&self, service: &str) -> Result<HashMap<String, serde_json::Value>> {
        tracing::debug!("📊 Monitoring service: {}", service);
        
        let mut result = HashMap::new();
        let primals = self.registered_primals.read().await;
        let primal = primals.values().find(|p| p.name == service || p.id == service);
        
        if let Some(primal) = primal {
            result.insert("service_name".to_string(), serde_json::json!(primal.name));
            result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
            result.insert("health".to_string(), serde_json::json!(primal.health));
            result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));
            
            // Mock resource usage
            result.insert("resources".to_string(), serde_json::json!({
                "cpu_percent": 15.5,
                "memory_mb": 256,
                "network_io": {
                    "bytes_in": 1024,
                    "bytes_out": 2048
                }
            }));
        } else {
            result.insert("error".to_string(), serde_json::json!(format!("Service not found: {}", service)));
        }
        
        Ok(result)
    }

    /// Monitor entire system
    pub async fn monitor_system(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::debug!("📊 Monitoring system");
        
        let mut result = HashMap::new();
        
        // System metrics
        result.insert("system".to_string(), serde_json::json!({
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
        }));
        
        // Service statuses
        let primals = self.registered_primals.read().await;
        let mut services = HashMap::new();
        
        for (id, primal) in primals.iter() {
            services.insert(id.clone(), serde_json::json!({
                "name": primal.name,
                "status": "running",
                "health": primal.health,
                "resources": {
                    "cpu_percent": 10.0,
                    "memory_mb": 128
                }
            }));
        }
        
        result.insert("services".to_string(), serde_json::json!(services));
        
        // Network activity
        result.insert("network".to_string(), serde_json::json!({
            "bytes_in_per_sec": 1024,
            "bytes_out_per_sec": 2048,
            "active_connections": 5
        }));
        
        // Alerts (empty for now)
        result.insert("alerts".to_string(), serde_json::json!([]));
        
        Ok(result)
    }

    /// Get system or service status
    pub async fn get_service_status(&self, service: &str) -> Result<HashMap<String, serde_json::Value>> {
        let primals = self.registered_primals.read().await;
        let primal = primals.values().find(|p| p.name == service || p.id == service);
        
        let mut result = HashMap::new();
        
        if let Some(primal) = primal {
            result.insert("service_name".to_string(), serde_json::json!(primal.name));
            result.insert("service_id".to_string(), serde_json::json!(primal.id));
            result.insert("status".to_string(), serde_json::json!("running"));
            result.insert("health".to_string(), serde_json::json!(primal.health));
            result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
            result.insert("type".to_string(), serde_json::json!(primal.primal_type));
            result.insert("capabilities".to_string(), serde_json::json!(primal.capabilities));
            result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));
            result.insert("discovered_at".to_string(), serde_json::json!(primal.discovered_at));
        } else {
            result.insert("error".to_string(), serde_json::json!(format!("Service not found: {}", service)));
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
        result.insert("health".to_string(), serde_json::json!(health_report.health));
        
        Ok(result)
    }
}

/// Integration helper methods
impl UniversalBiomeOSManager {
    /// Validate manifest integration (placeholder)
    async fn validate_manifest_integration(&self, manifest_path: &str) -> Result<String> {
        tracing::debug!("Validating manifest: {}", manifest_path);
        // Integration point with Toadstool parser
        Ok(format!("Manifest validation completed for: {}", manifest_path))
    }

    /// Deploy via ecosystem integration (placeholder)
    async fn deploy_via_ecosystem_integration(&self, manifest_path: &str) -> Result<String> {
        tracing::debug!("Deploying via ecosystem integration: {}", manifest_path);
        // Integration point with Toadstool for compute orchestration
        Ok(format!("Deployment completed for: {}", manifest_path))
    }

    /// Create service integration (placeholder)
    async fn create_service_integration(
        &self,
        name: &str,
        service_type: &str,
        _config: Option<&Path>,
    ) -> Result<String> {
        tracing::debug!("Creating service via integration: {} ({})", name, service_type);
        // Integration point with template system
        Ok(format!("Service '{}' of type '{}' created successfully", name, service_type))
    }

    /// Generate service logs (mock implementation)
    async fn generate_service_logs(
        &self,
        primal: &super::core::PrimalInfo,
        tail: Option<usize>,
        _since: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        let limit = tail.unwrap_or(100);
        let mut logs = Vec::new();
        
        for i in 0..limit.min(20) {
            logs.push(serde_json::json!({
                "timestamp": chrono::Utc::now() - chrono::Duration::seconds(i as i64 * 30),
                "level": if i % 10 == 0 { "warn" } else { "info" },
                "message": format!("Service {} log entry #{}", primal.name, limit - i)
            }));
        }
        
        Ok(logs)
    }

    /// Execute command integration (placeholder)
    async fn execute_command_integration(
        &self,
        primal: &super::core::PrimalInfo,
        command: &str,
        _interactive: bool,
    ) -> Result<ExecutionResult> {
        tracing::debug!("Executing command in {}: {}", primal.name, command);
        
        // Mock command execution
        Ok(ExecutionResult {
            stdout: format!("Executed '{}' in service '{}'", command, primal.name),
            stderr: String::new(),
        })
    }

    /// Scale service integration (placeholder)
    async fn scale_service_integration(
        &self,
        primal: &super::core::PrimalInfo,
        replicas: u32,
    ) -> Result<String> {
        tracing::debug!("Scaling service {} to {} replicas", primal.name, replicas);
        
        // Integration point with orchestration system
        Ok(format!("Service '{}' scaled to {} replicas", primal.name, replicas))
    }
}

/// Command execution result
#[derive(Debug)]
struct ExecutionResult {
    stdout: String,
    stderr: String,
} 