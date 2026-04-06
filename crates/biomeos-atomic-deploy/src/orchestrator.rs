// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Atomic deployment orchestrator
//!
//! Replaces bash "jelly strings" with modern idiomatic Rust

use anyhow::{Context, Result};
use biomeos_core::deployment_mode::DeploymentMode;
use biomeos_spore::seed::FamilySeed;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::health_check::HealthChecker;
use crate::primal_launcher::{PrimalInstance, PrimalLauncher};
use biomeos_types::primal_names;

/// Deployment role identifiers — primal name + role suffix.
///
/// Evolution: these should be replaced by capability-based requirements
/// (e.g., "needs security capability" instead of "needs beardog-server")
/// resolved at runtime via the Neural API capability router.
const BEARDOG_SERVER_ROLE: &str = "beardog-server";
const SONGBIRD_ORCHESTRATOR_ROLE: &str = "songbird-orchestrator";

/// Atomic type for deployment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtomicType {
    /// Tower orchestrator atomic
    Tower,
    /// Node atomic (compute unit)
    Node,
    /// Nest atomic (embedded/niche)
    Nest,
}

impl AtomicType {
    /// Get the node ID for this atomic
    #[must_use]
    pub const fn node_id(&self) -> &'static str {
        match self {
            Self::Tower => "tower",
            Self::Node => "node",
            Self::Nest => "nest",
        }
    }

    /// Get required primals for this atomic.
    ///
    /// Evolution: migrate to capability-based requirements resolved via
    /// Neural API discovery rather than fixed primal role names.
    #[must_use]
    pub fn required_primals(&self) -> Vec<&'static str> {
        match self {
            Self::Tower => vec![BEARDOG_SERVER_ROLE, SONGBIRD_ORCHESTRATOR_ROLE],
            Self::Node => {
                vec![
                    BEARDOG_SERVER_ROLE,
                    SONGBIRD_ORCHESTRATOR_ROLE,
                    primal_names::TOADSTOOL,
                ]
            }
            Self::Nest => {
                vec![
                    BEARDOG_SERVER_ROLE,
                    SONGBIRD_ORCHESTRATOR_ROLE,
                    primal_names::NESTGATE,
                ]
            }
        }
    }
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Path to USB parent seed
    pub usb_seed_path: PathBuf,

    /// Family ID (e.g., "1894e909e454")
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
        use biomeos_types::identifiers::FamilyId;

        Self {
            usb_seed_path,
            family_id: FamilyId::new("1894e909e454").to_string(),
            deployment_batch: chrono::Utc::now().format("%Y%m%d").to_string(),
            binary_dir: std::env::var("BIOMEOS_PLASMID_BIN_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join("plasmidBin")
                }),
            runtime_dir: std::env::var("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let uid = std::env::var("UID")
                        .or_else(|_| std::env::var("EUID"))
                        .or_else(|_| {
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::MetadataExt;
                                std::fs::metadata("/proc/self")
                                    .map(|m| m.uid().to_string())
                                    .map_err(|_| std::env::VarError::NotPresent)
                            }
                            #[cfg(not(unix))]
                            Err(std::env::VarError::NotPresent)
                        });
                    match uid {
                        Ok(uid) => PathBuf::from(format!("/run/user/{uid}")),
                        Err(_) => std::env::temp_dir().join("biomeos"),
                    }
                }),
            deployment_mode: DeploymentMode::detect().unwrap_or_else(|_| {
                DeploymentMode::SiblingSpore {
                    host_os: biomeos_core::deployment_mode::HostOS::Linux {
                        distro: "Unknown".to_string(),
                    },
                    install_dir: std::env::temp_dir().join("biomeos"),
                    isolation: biomeos_core::deployment_mode::IsolationLevel::Shared,
                }
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
        let launcher = PrimalLauncher::new(config.binary_dir.clone(), config.runtime_dir.clone())?;

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
            anyhow::bail!(
                "USB seed not found: {}",
                self.config.usb_seed_path.display()
            );
        }

        info!(
            "   ✅ USB seed verified: {}",
            self.config.usb_seed_path.display()
        );

        // Step 2: Derive child seed for this atomic
        let child_seed_path = self
            .derive_child_seed(atomic_type)
            .context("Failed to derive child seed")?;

        info!("   🧬 Child seed derived: {}", child_seed_path.display());

        // Step 3: Launch primals with genetic lineage
        let mut instances = Vec::new();

        for primal_name in atomic_type.required_primals() {
            match self
                .launch_primal_with_lineage(primal_name, atomic_type, &child_seed_path)
                .await
            {
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
                result.errors.push(format!("Tower deployment failed: {e}"));
            }
        }

        // Deploy Node
        match self.deploy_atomic(AtomicType::Node).await {
            Ok(instances) => {
                result.node = Some(instances);
                result.success_count += 1;
            }
            Err(e) => {
                result.errors.push(format!("Node deployment failed: {e}"));
            }
        }

        // Deploy Nest
        match self.deploy_atomic(AtomicType::Nest).await {
            Ok(instances) => {
                result.nest = Some(instances);
                result.success_count += 1;
            }
            Err(e) => {
                result.errors.push(format!("Nest deployment failed: {e}"));
            }
        }

        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!(
            "✅ Deployment complete: {}/3 atomics operational",
            result.success_count
        );
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        Ok(result)
    }

    /// Derive child seed for an atomic
    fn derive_child_seed(&self, atomic_type: AtomicType) -> Result<PathBuf> {
        let node_id = atomic_type.node_id();
        let child_seed_path = self.config.runtime_dir.join(format!(
            ".family-{}-{}.seed",
            node_id, self.config.family_id
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
            primal_name
                .replace("-server", "")
                .replace("-orchestrator", ""),
            atomic_type.node_id()
        ));

        // Build environment with genetic lineage
        let mut env = std::collections::HashMap::new();
        env.insert(
            "BEARDOG_FAMILY_SEED_FILE".to_string(),
            child_seed_path.display().to_string(),
        );
        env.insert(
            "BEARDOG_FAMILY_ID".to_string(),
            self.config.family_id.clone(),
        );
        env.insert(
            "BEARDOG_NODE_ID".to_string(),
            atomic_type.node_id().to_string(),
        );

        let socket_env = biomeos_types::defaults::env_vars::socket_env_key(primal_name);
        env.insert(socket_env, socket_path.display().to_string());

        // For Songbird, set security provider (BearDog)
        if primal_name == SONGBIRD_ORCHESTRATOR_ROLE {
            let security_socket = self.config.runtime_dir.join(format!(
                "{}-{}.sock",
                primal_names::BEARDOG,
                atomic_type.node_id()
            ));
            env.insert(
                "SONGBIRD_SECURITY_PROVIDER".to_string(),
                security_socket.display().to_string(),
            );
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
            let health = self
                .health_checker
                .check_primal(&instance.socket_path)
                .await?;

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
    /// Deployed tower primal instances
    pub tower: Option<Vec<PrimalInstance>>,
    /// Deployed node primal instances
    pub node: Option<Vec<PrimalInstance>>,
    /// Deployed nest primal instances
    pub nest: Option<Vec<PrimalInstance>>,
    /// Number of atomics successfully deployed
    pub success_count: usize,
    /// Error messages from failed deployments
    pub errors: Vec<String>,
}

impl DeploymentResult {
    const fn new() -> Self {
        Self {
            tower: None,
            node: None,
            nest: None,
            success_count: 0,
            errors: Vec::new(),
        }
    }

    /// Check if deployment was successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success_count == 3 && self.errors.is_empty()
    }

    /// Get all running primal instances
    #[must_use]
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
#[path = "orchestrator_tests.rs"]
mod tests;
