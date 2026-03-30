// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    pub fn plan_service_creation(
        &self,
        config_data: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("📋 Planning service creation");

        let config: serde_json::Value = serde_json::from_str(config_data)
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {e}"))?;

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
    pub fn deploy_biome(
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
            match self.validate_manifest_integration(&path_str) {
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
            match self.deploy_via_ecosystem_integration(&path_str) {
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

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use crate::universal_biomeos_manager::UniversalBiomeOSManager;
    use biomeos_types::BiomeOSConfig;
    use std::path::Path;
    use tempfile::NamedTempFile;

    fn test_manager() -> UniversalBiomeOSManager {
        UniversalBiomeOSManager::new(BiomeOSConfig::default()).expect("create test manager")
    }

    #[tokio::test]
    async fn test_plan_service_creation() {
        let manager = test_manager();
        let config_data = r#"{"service": "test", "replicas": 2}"#;
        let plan = manager
            .plan_service_creation(config_data)
            .expect("plan_service_creation should succeed");
        assert_eq!(plan.get("status").and_then(|v| v.as_str()), Some("planned"));
        assert!(plan.contains_key("config"));
        assert!(plan.contains_key("timestamp"));
        assert_eq!(
            plan.get("architecture").and_then(|v| v.as_str()),
            Some("universal_adapter")
        );
    }

    #[tokio::test]
    async fn test_plan_service_creation_invalid_json() {
        let manager = test_manager();
        let err = manager
            .plan_service_creation("not valid json {")
            .expect_err("invalid JSON should fail");
        assert!(err.to_string().contains("parse"));
    }

    #[tokio::test]
    async fn test_deploy_biome_validate_only_success() {
        let manager = test_manager();
        let temp = NamedTempFile::new().expect("create temp file");
        let path = temp.path();
        let result = manager
            .deploy_biome(path, true)
            .expect("deploy_biome validate_only should succeed");
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("success")
        );
        assert!(result.contains_key("validation_result"));
    }

    #[tokio::test]
    async fn test_deploy_biome_full_deploy() {
        let manager = test_manager();
        let temp = NamedTempFile::new().expect("create temp file");
        let path = temp.path();
        let result = manager
            .deploy_biome(path, false)
            .expect("deploy_biome should succeed");
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("success")
        );
        assert!(result.contains_key("deployment_result"));
    }

    #[tokio::test]
    async fn test_deploy_biome_manifest_path_in_result() {
        let manager = test_manager();
        let path = Path::new("/tmp/test-manifest.yaml");
        let result = manager
            .deploy_biome(path, true)
            .expect("deploy_biome should succeed");
        assert_eq!(
            result.get("manifest_path").and_then(|v| v.as_str()),
            Some("/tmp/test-manifest.yaml")
        );
        assert_eq!(
            result
                .get("validate_only")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }
}
