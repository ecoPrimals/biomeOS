//! Biome Deployment Operations
//!
//! Handles full biome deployment orchestration including manifest validation,
//! deployment planning, and coordination with ecosystem services.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use super::core::UniversalBiomeOSManager;

impl UniversalBiomeOSManager {
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
}

