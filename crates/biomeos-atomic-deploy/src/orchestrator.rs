//! Atomic deployment orchestrator
//!
//! Replaces bash "jelly strings" with modern idiomatic Rust

use anyhow::{Context, Result};
use biomeos_core::deployment_mode::DeploymentMode;
use biomeos_spore::seed::FamilySeed;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::primal_launcher::{PrimalLauncher, PrimalInstance};
use crate::health_check::HealthChecker;

/// Atomic type for deployment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtomicType {
    Tower,
    Node,
    Nest,
}

impl AtomicType {
    /// Get the node ID for this atomic
    pub fn node_id(&self) -> &'static str {
        match self {
            AtomicType::Tower => "tower",
            AtomicType::Node => "node",
            AtomicType::Nest => "nest",
        }
    }

    /// Get required primals for this atomic
    pub fn required_primals(&self) -> Vec<&'static str> {
        match self {
            AtomicType::Tower => vec!["beardog-server", "songbird-orchestrator"],
            AtomicType::Node => vec!["beardog-server", "songbird-orchestrator", "toadstool"],
            AtomicType::Nest => vec!["beardog-server", "songbird-orchestrator", "nestgate"],
        }
    }
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Path to USB parent seed
    pub usb_seed_path: PathBuf,

    /// Family ID (e.g., "nat0")
    pub family_id: String,

    /// Deployment batch (e.g., "20260112")
    pub deployment_batch: String,

    /// Path to primal binaries
    pub binary_dir: PathBuf,

    /// Runtime socket directory
    pub runtime_dir: PathBuf,

    /// Deployment mode
    pub deployment_mode: DeploymentMode,

    /// Enable Neural API reporting
    pub neural_api_enabled: bool,

    /// Neural API endpoint (if enabled)
    pub neural_api_endpoint: Option<String>,
}

impl DeploymentConfig {
    /// Create default config for testing
    pub fn test_config(usb_seed_path: PathBuf) -> Self {
        Self {
            usb_seed_path,
            family_id: "nat0".to_string(),
            deployment_batch: chrono::Utc::now().format("%Y%m%d").to_string(),
            binary_dir: PathBuf::from("/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin"),
            runtime_dir: std::env::var("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() }))),
            deployment_mode: DeploymentMode::detect().unwrap_or_else(|_| DeploymentMode::SiblingSpore {
                host_os: biomeos_core::deployment_mode::HostOS::Linux {
                    distro: "Unknown".to_string(),
                },
                install_dir: PathBuf::from("/tmp/biomeos"),
                isolation: biomeos_core::deployment_mode::IsolationLevel::Shared,
            }),
            neural_api_enabled: false,
            neural_api_endpoint: None,
        }
    }
}

/// Deployment orchestrator - replaces bash scripts with Rust
pub struct DeploymentOrchestrator {
    config: DeploymentConfig,
    launcher: PrimalLauncher,
    health_checker: HealthChecker,
}

impl DeploymentOrchestrator {
    /// Create new orchestrator
    pub fn new(config: DeploymentConfig) -> Result<Self> {
        let launcher = PrimalLauncher::new(
            config.binary_dir.clone(),
            config.runtime_dir.clone(),
        )?;

        let health_checker = HealthChecker::new(config.runtime_dir.clone());

        Ok(Self {
            config,
            launcher,
            health_checker,
        })
    }

    /// Deploy a single atomic from USB seed
    pub async fn deploy_atomic(&mut self, atomic_type: AtomicType) -> Result<Vec<PrimalInstance>> {
        info!("🚀 Deploying {:?} atomic from USB seed", atomic_type);

        // Step 1: Verify USB seed exists
        if !self.config.usb_seed_path.exists() {
            anyhow::bail!("USB seed not found: {}", self.config.usb_seed_path.display());
        }

        info!("   ✅ USB seed verified: {}", self.config.usb_seed_path.display());

        // Step 2: Derive child seed for this atomic
        let child_seed_path = self.derive_child_seed(atomic_type)
            .context("Failed to derive child seed")?;

        info!("   🧬 Child seed derived: {}", child_seed_path.display());

        // Step 3: Launch primals with genetic lineage
        let mut instances = Vec::new();

        for primal_name in atomic_type.required_primals() {
            match self.launch_primal_with_lineage(
                primal_name,
                atomic_type,
                &child_seed_path,
            ).await {
                Ok(instance) => {
                    info!("   ✅ {} launched (PID: {})", primal_name, instance.pid);
                    instances.push(instance);
                }
                Err(e) => {
                    warn!("   ⚠️  Failed to launch {}: {}", primal_name, e);
                    // Continue with other primals (degraded mode)
                }
            }
        }

        // Step 4: Health check
        self.verify_atomic_health(atomic_type, &instances).await?;

        info!("   🎊 {:?} atomic deployed successfully!", atomic_type);

        Ok(instances)
    }

    /// Deploy all 3 atomics (Tower, Node, Nest)
    pub async fn deploy_all(&mut self) -> Result<DeploymentResult> {
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🧬 Genetic Lineage Deployment - All Atomics");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let mut result = DeploymentResult::new();

        // Deploy Tower
        match self.deploy_atomic(AtomicType::Tower).await {
            Ok(instances) => {
                result.tower = Some(instances);
                result.success_count += 1;
            }
            Err(e) => {
                result.errors.push(format!("Tower deployment failed: {}", e));
            }
        }

        // Deploy Node
        match self.deploy_atomic(AtomicType::Node).await {
            Ok(instances) => {
                result.node = Some(instances);
                result.success_count += 1;
            }
            Err(e) => {
                result.errors.push(format!("Node deployment failed: {}", e));
            }
        }

        // Deploy Nest
        match self.deploy_atomic(AtomicType::Nest).await {
            Ok(instances) => {
                result.nest = Some(instances);
                result.success_count += 1;
            }
            Err(e) => {
                result.errors.push(format!("Nest deployment failed: {}", e));
            }
        }

        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("✅ Deployment complete: {}/3 atomics operational", result.success_count);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        Ok(result)
    }

    /// Derive child seed for an atomic
    fn derive_child_seed(&self, atomic_type: AtomicType) -> Result<PathBuf> {
        let node_id = atomic_type.node_id();
        let child_seed_path = self.config.runtime_dir.join(format!(
            ".family-{}-{}.seed",
            node_id,
            self.config.family_id
        ));

        FamilySeed::derive_sibling(
            &self.config.usb_seed_path,
            &child_seed_path,
            node_id,
            Some(&self.config.deployment_batch),
        )?;

        Ok(child_seed_path)
    }

    /// Launch a primal with genetic lineage configuration
    async fn launch_primal_with_lineage(
        &mut self,
        primal_name: &str,
        atomic_type: AtomicType,
        child_seed_path: &Path,
    ) -> Result<PrimalInstance> {
        let socket_path = self.config.runtime_dir.join(format!(
            "{}-{}.sock",
            primal_name.replace("-server", "").replace("-orchestrator", ""),
            atomic_type.node_id()
        ));

        // Build environment with genetic lineage
        let mut env = std::collections::HashMap::new();
        env.insert("BEARDOG_FAMILY_SEED_FILE".to_string(), child_seed_path.display().to_string());
        env.insert("BEARDOG_FAMILY_ID".to_string(), self.config.family_id.clone());
        env.insert("BEARDOG_NODE_ID".to_string(), atomic_type.node_id().to_string());

        // Primal-specific socket env vars
        let socket_env = match primal_name {
            "beardog-server" => "BEARDOG_SOCKET",
            "songbird-orchestrator" => "SONGBIRD_SOCKET",
            "toadstool" => "TOADSTOOL_SOCKET",
            "nestgate" => "NESTGATE_SOCKET",
            _ => return Err(anyhow::anyhow!("Unknown primal: {}", primal_name)),
        };

        env.insert(socket_env.to_string(), socket_path.display().to_string());

        // For Songbird, set security provider (BearDog)
        if primal_name == "songbird-orchestrator" {
            let beardog_socket = self.config.runtime_dir.join(format!(
                "beardog-{}.sock",
                atomic_type.node_id()
            ));
            env.insert("SONGBIRD_SECURITY_PROVIDER".to_string(), beardog_socket.display().to_string());
        }

        self.launcher.launch(primal_name, env).await
    }

    /// Verify atomic health after deployment
    async fn verify_atomic_health(
        &self,
        atomic_type: AtomicType,
        instances: &[PrimalInstance],
    ) -> Result<()> {
        debug!("   🔍 Health check: {:?} atomic", atomic_type);

        for instance in instances {
            let health = self.health_checker.check_primal(&instance.socket_path).await?;

            if !health.is_healthy {
                anyhow::bail!(
                    "Health check failed for {}: {}",
                    instance.primal_name,
                    health.message.unwrap_or_default()
                );
            }

            debug!("      ✅ {} healthy", instance.primal_name);
        }

        Ok(())
    }
}

/// Result of a full deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub tower: Option<Vec<PrimalInstance>>,
    pub node: Option<Vec<PrimalInstance>>,
    pub nest: Option<Vec<PrimalInstance>>,
    pub success_count: usize,
    pub errors: Vec<String>,
}

impl DeploymentResult {
    fn new() -> Self {
        Self {
            tower: None,
            node: None,
            nest: None,
            success_count: 0,
            errors: Vec::new(),
        }
    }

    /// Check if deployment was successful
    pub fn is_success(&self) -> bool {
        self.success_count == 3 && self.errors.is_empty()
    }

    /// Get all running primal instances
    pub fn all_instances(&self) -> Vec<&PrimalInstance> {
        let mut instances = Vec::new();

        if let Some(ref tower) = self.tower {
            instances.extend(tower.iter());
        }
        if let Some(ref node) = self.node {
            instances.extend(node.iter());
        }
        if let Some(ref nest) = self.nest {
            instances.extend(nest.iter());
        }

        instances
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_type_node_id() {
        assert_eq!(AtomicType::Tower.node_id(), "tower");
        assert_eq!(AtomicType::Node.node_id(), "node");
        assert_eq!(AtomicType::Nest.node_id(), "nest");
    }

    #[test]
    fn test_atomic_type_required_primals() {
        assert_eq!(
            AtomicType::Tower.required_primals(),
            vec!["beardog-server", "songbird-orchestrator"]
        );
        assert_eq!(
            AtomicType::Node.required_primals(),
            vec!["beardog-server", "songbird-orchestrator", "toadstool"]
        );
        assert_eq!(
            AtomicType::Nest.required_primals(),
            vec!["beardog-server", "songbird-orchestrator", "nestgate"]
        );
    }

    #[test]
    fn test_atomic_type_serialization() {
        // Test JSON serialization
        let tower = AtomicType::Tower;
        let json = serde_json::to_string(&tower).unwrap();
        assert_eq!(json, "\"Tower\"");
        
        let deserialized: AtomicType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AtomicType::Tower);
    }

    #[test]
    fn test_deployment_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let seed_path = temp_dir.path().join("test.seed");
        
        let config = DeploymentConfig::test_config(seed_path.clone());
        
        assert_eq!(config.usb_seed_path, seed_path);
        assert_eq!(config.family_id, "nat0");
        assert!(!config.neural_api_enabled);
        assert!(config.neural_api_endpoint.is_none());
    }

    #[test]
    fn test_deployment_result_is_success() {
        let mut result = DeploymentResult::new();
        assert_eq!(result.success_count, 0);
        assert!(!result.is_success());
        
        result.success_count = 3;
        assert!(result.is_success());
        
        result.errors.push("test error".to_string());
        assert!(!result.is_success()); // Errors mean failure
    }

    #[test]
    fn test_deployment_result_all_instances() {
        let mut result = DeploymentResult::new();
        
        // Empty at start
        assert_eq!(result.all_instances().len(), 0);
        
        // Add tower instance
        result.tower = Some(vec![PrimalInstance {
            primal_name: "beardog-server".to_string(),
            pid: 1234,
            socket_path: PathBuf::from("/tmp/test.sock"),
        }]);
        
        assert_eq!(result.all_instances().len(), 1);
        
        // Add node instances
        result.node = Some(vec![
            PrimalInstance {
                primal_name: "beardog-server".to_string(),
                pid: 2345,
                socket_path: PathBuf::from("/tmp/test2.sock"),
            },
            PrimalInstance {
                primal_name: "songbird-orchestrator".to_string(),
                pid: 3456,
                socket_path: PathBuf::from("/tmp/test3.sock"),
            },
        ]);
        
        assert_eq!(result.all_instances().len(), 3);
    }

    #[test]
    fn test_deployment_config_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let config = DeploymentConfig::test_config(temp_dir.path().join("test.seed"));
        
        // Test JSON round-trip
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DeploymentConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.family_id, deserialized.family_id);
        assert_eq!(config.deployment_batch, deserialized.deployment_batch);
        assert_eq!(config.neural_api_enabled, deserialized.neural_api_enabled);
    }
}

