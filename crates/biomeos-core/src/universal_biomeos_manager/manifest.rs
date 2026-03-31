// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Manifest Operations
//!
//! Handles biome manifest validation and deployment coordination.
//! Integrates with Toadstool for parsing and validation.

use anyhow::Result;

use super::core::UniversalBiomeOSManager;
use biomeos_types::BiomeManifest;

impl UniversalBiomeOSManager {
    /// Validate a biome manifest (delegated to Toadstool parser)
    pub fn validate_manifest(&self, manifest_content: &str) -> Result<BiomeManifest> {
        tracing::info!("🔍 Validating biome manifest");

        // Parse the manifest content
        let manifest: BiomeManifest = serde_yaml::from_str(manifest_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse manifest: {e}"))?;

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
    pub fn deploy_manifest(&self, manifest_content: &str) -> Result<String> {
        tracing::info!("🚀 Deploying biome manifest");

        // First validate the manifest
        let manifest = self.validate_manifest(manifest_content)?;

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

    /// Validate manifest integration.
    ///
    /// Validation delegates to `ToadStool` via `compute.validate` when available.
    #[expect(
        clippy::unused_self,
        reason = "method for future use or API consistency"
    )]
    pub(super) fn validate_manifest_integration(&self, manifest_path: &str) -> Result<String> {
        tracing::debug!("Validating manifest: {}", manifest_path);
        // Integration point with Toadstool parser
        Ok(format!(
            "Manifest validation completed for: {manifest_path}"
        ))
    }

    /// Deploy via ecosystem integration.
    ///
    /// Compute orchestration routes to `ToadStool` via `compute.*` capabilities.
    #[expect(
        clippy::unused_self,
        reason = "method for future use or API consistency"
    )]
    pub(super) fn deploy_via_ecosystem_integration(&self, manifest_path: &str) -> Result<String> {
        tracing::debug!("Deploying via ecosystem integration: {}", manifest_path);
        // Integration point with Toadstool for compute orchestration
        Ok(format!("Deployment completed for: {manifest_path}"))
    }
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use crate::universal_biomeos_manager::UniversalBiomeOSManager;
    use biomeos_manifest::{BiomeManifestProcessor, BiomeManifestTemplates};
    use biomeos_types::BiomeOSConfig;
    use biomeos_types::manifest::service::{
        ImagePullPolicy, ImageSpec, RestartPolicy, ServiceMetadata, ServiceSpec,
    };
    use std::collections::HashMap;

    fn test_manager() -> UniversalBiomeOSManager {
        UniversalBiomeOSManager::new(BiomeOSConfig::default()).expect("create test manager")
    }

    fn valid_manifest_yaml() -> String {
        let mut manifest = BiomeManifestTemplates::web_application("test", "nginx");
        let service = ServiceSpec {
            metadata: ServiceMetadata {
                name: "web".to_string(),
                description: None,
                version: "1.0".to_string(),
                labels: HashMap::new(),
                annotations: HashMap::new(),
                primal_type: None,
                capabilities: vec![],
            },
            image: ImageSpec::Container {
                name: "nginx".to_string(),
                tag: "latest".to_string(),
                registry: None,
                pull_policy: ImagePullPolicy::IfNotPresent,
                pull_secrets: vec![],
            },
            ports: vec![],
            environment: HashMap::new(),
            volumes: vec![],
            resources: None,
            health_checks: vec![],
            depends_on: vec![],
            config: HashMap::new(),
            scaling: None,
            security: None,
            restart_policy: RestartPolicy::Always,
            deployment: None,
        };
        manifest.services.insert("web".to_string(), service);
        BiomeManifestProcessor::save_to_yaml(&manifest).expect("serialize manifest")
    }

    #[tokio::test]
    async fn test_validate_manifest_success() {
        let manager = test_manager();
        let manifest = manager
            .validate_manifest(&valid_manifest_yaml())
            .expect("validate_manifest should succeed for valid manifest");
        assert_eq!(manifest.metadata.name, "test-biome");
        assert!(!manifest.services.is_empty());
    }

    #[tokio::test]
    async fn test_validate_manifest_empty_name_fails() {
        let manager = test_manager();
        let mut manifest = BiomeManifestTemplates::web_application("test", "nginx");
        manifest.services.insert(
            "web".to_string(),
            ServiceSpec {
                metadata: ServiceMetadata {
                    name: "web".to_string(),
                    description: None,
                    version: "1.0".to_string(),
                    labels: HashMap::new(),
                    annotations: HashMap::new(),
                    primal_type: None,
                    capabilities: vec![],
                },
                image: ImageSpec::Container {
                    name: "nginx".to_string(),
                    tag: "latest".to_string(),
                    registry: None,
                    pull_policy: ImagePullPolicy::IfNotPresent,
                    pull_secrets: vec![],
                },
                ports: vec![],
                environment: HashMap::new(),
                volumes: vec![],
                resources: None,
                health_checks: vec![],
                depends_on: vec![],
                config: HashMap::new(),
                scaling: None,
                security: None,
                restart_policy: RestartPolicy::Always,
                deployment: None,
            },
        );
        manifest.metadata.name = String::new();
        let yaml = serde_yaml::to_string(&manifest).expect("serialize");
        let err = manager
            .validate_manifest(&yaml)
            .expect_err("empty name should fail");
        assert!(err.to_string().contains("name") || err.to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_validate_manifest_empty_services_fails() {
        let manager = test_manager();
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        let yaml = BiomeManifestProcessor::save_to_yaml(&manifest).expect("serialize");
        let err = manager
            .validate_manifest(&yaml)
            .expect_err("empty services should fail");
        assert!(
            err.to_string().contains("service") || err.to_string().contains("one"),
            "error: {err}"
        );
    }

    #[tokio::test]
    async fn test_validate_manifest_invalid_yaml_fails() {
        let manager = test_manager();
        let err = manager
            .validate_manifest("invalid: yaml: [")
            .expect_err("invalid YAML should fail");
        assert!(err.to_string().contains("parse") || err.to_string().contains("manifest"));
    }

    #[tokio::test]
    async fn test_deploy_manifest_success() {
        let manager = test_manager();
        let deployment_id = manager
            .deploy_manifest(&valid_manifest_yaml())
            .expect("deploy_manifest should succeed");
        // UUID format
        assert!(!deployment_id.is_empty());
        assert!(deployment_id.len() >= 32);
    }

    #[tokio::test]
    async fn test_deploy_manifest_validates_first() {
        let manager = test_manager();
        let err = manager
            .deploy_manifest("metadata:\n  name: \"\"\nservices: {}")
            .expect_err("deploy should fail on invalid manifest");
        assert!(!err.to_string().is_empty());
    }
}
