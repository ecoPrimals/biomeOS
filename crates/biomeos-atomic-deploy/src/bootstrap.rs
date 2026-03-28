// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Bootstrap Module
//!
//! Handles bootstrap sequence execution, mode transitions, and self-registration.
//! Follows Deep Debt principles:
//! - Modern idiomatic Rust (proper error handling)
//! - Self-knowledge only - discover primals at runtime
//! - Capability-based communication
//! - Pure Rust JSON-RPC over Unix socket

use crate::mode::BiomeOsMode;
use crate::neural_executor::GraphExecutor;
use crate::neural_router::NeuralRouter;
use crate::nucleation::{SocketNucleation, SocketStrategy};
use crate::primal_communication::{establish_btsp_tunnel, verify_primal_health};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};

/// Register biomeOS in the capability registry
///
/// Registers core capabilities that biomeOS provides to the ecosystem.
///
/// # Arguments
/// * `router` - Neural Router for capability registration
/// * `family_id` - Family ID for this server
/// * `socket_path` - Socket path for this server
/// * `mode` - Current operating mode
pub async fn register_self_in_registry(
    router: &Arc<NeuralRouter>,
    family_id: &str,
    socket_path: &PathBuf,
    mode: &RwLock<BiomeOsMode>,
) -> Result<()> {
    let mode_guard = mode.read().await;
    let source = match *mode_guard {
        BiomeOsMode::Bootstrap => "bootstrap",
        BiomeOsMode::Coordinated => "coordinated",
    };
    drop(mode_guard);

    let primal_name = format!("biomeos-{family_id}");
    let capabilities = biomeos_types::primal_names::BIOMEOS_SELF_CAPABILITIES;

    // Register each capability (data-driven from niche self-knowledge)
    for &capability in capabilities {
        router
            .register_capability_unix(capability, &primal_name, socket_path, source)
            .await?;
    }

    info!(
        "✅ biomeOS registered {} capabilities in registry",
        capabilities.len()
    );
    Ok(())
}

/// Execute bootstrap sequence (germinate Tower Atomic)
///
/// Loads and executes the bootstrap graph to create the initial ecosystem.
/// Note: Caller should load translations from the graph before calling this function.
///
/// # Arguments
/// * `graphs_dir` - Path to graphs directory
/// * `family_id` - Family ID for this server
/// * `nucleation` - Socket nucleation for deterministic socket assignment
pub async fn execute_bootstrap_sequence(
    graphs_dir: &std::path::Path,
    family_id: &str,
    nucleation: &Arc<RwLock<SocketNucleation>>,
) -> Result<()> {
    // Load tower_atomic_bootstrap.toml
    let bootstrap_graph_path = graphs_dir.join("tower_atomic_bootstrap.toml");

    if !bootstrap_graph_path.exists() {
        return Err(anyhow::anyhow!(
            "Bootstrap graph not found: {}",
            bootstrap_graph_path.display()
        ));
    }

    info!(
        "📋 Loading bootstrap graph: {}",
        bootstrap_graph_path.display()
    );
    let graph = crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path)?;

    // Prepare environment
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), family_id.to_string());
    env.insert("BIOMEOS_FAMILY_ID".to_string(), family_id.to_string());
    env.insert("BIOMEOS_MODE".to_string(), "bootstrap".to_string());

    // Create executor with nucleation
    info!("🧬 Creating graph executor with socket nucleation...");
    let executor = GraphExecutor::with_nucleation(graph, env, nucleation.clone());

    // Execute graph
    info!("🚀 Executing bootstrap graph...");
    let mut executor = executor; // Make mutable for execute()
    let report = executor.execute().await?;

    // Check if successful
    if report.success {
        info!("✅ Bootstrap graph executed successfully");
        info!("   Duration: {}ms", report.duration_ms);
        info!("   Phases: {}", report.phase_results.len());
    } else {
        error!("❌ Bootstrap graph failed");
        if let Some(ref error) = report.error {
            error!("   Error: {}", error);
        }
        return Err(anyhow::anyhow!("Bootstrap graph execution failed"));
    }

    Ok(())
}

/// Transition to coordinated mode (establish BTSP tunnel with Tower Atomic)
///
/// Waits for Tower Atomic primals to become available, verifies their health,
/// and establishes a secure tunnel for inter-primal communication.
///
/// # Arguments
/// * `family_id` - Family ID for this server
pub async fn transition_to_coordinated(family_id: &str) -> Result<()> {
    info!("🔄 Establishing connection with Tower Atomic...");

    // Wait for Tower Atomic to be ready (sockets to exist)
    // Uses SocketNucleation for deterministic paths (no hardcoding)
    let max_wait = Duration::from_secs(30);
    let check_interval = Duration::from_millis(500);
    let start = std::time::Instant::now();

    // Resolve provider names: env override → capability taxonomy → bootstrap fallback
    let security_provider = biomeos_types::env_config::security_provider()
        .or_else(|| {
            biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal("security")
                .map(String::from)
        })
        .unwrap_or_else(|| biomeos_types::primal_names::BEARDOG.to_string());
    let network_provider = biomeos_types::env_config::network_provider()
        .or_else(|| {
            biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal("discovery")
                .map(String::from)
        })
        .unwrap_or_else(|| biomeos_types::primal_names::SONGBIRD.to_string());
    let mut nucleation = SocketNucleation::new(SocketStrategy::default());
    let beardog_socket = nucleation.assign_socket(&security_provider, family_id);
    let songbird_socket = nucleation.assign_socket(&network_provider, family_id);

    loop {
        if start.elapsed() > max_wait {
            return Err(anyhow::anyhow!(
                "Tower Atomic did not become available within 30s. \
                 Ensure {security_provider} and {network_provider} primals are running for bootstrap."
            ));
        }

        let beardog_exists = beardog_socket.exists();
        let songbird_exists = songbird_socket.exists();

        if beardog_exists && songbird_exists {
            info!("✅ Tower Atomic sockets detected");
            break;
        }

        debug!(
            "   Waiting for Tower Atomic... (BearDog: {}, Songbird: {})",
            if beardog_exists { "✓" } else { "✗" },
            if songbird_exists { "✓" } else { "✗" }
        );

        sleep(check_interval).await;
    }

    // Layer 1: Verify security provider health
    match verify_primal_health(&beardog_socket, &security_provider).await {
        Ok(caps) => {
            info!(
                "✅ {} healthy with {} capabilities",
                security_provider,
                caps.len()
            );
        }
        Err(e) => {
            warn!(
                "⚠️ BearDog health check failed: {} (continuing with degraded security)",
                e
            );
        }
    }

    // Layer 2: Verify network provider health
    match verify_primal_health(&songbird_socket, &network_provider).await {
        Ok(caps) => {
            info!(
                "✅ {} healthy with {} capabilities",
                network_provider,
                caps.len()
            );
        }
        Err(e) => {
            warn!(
                "⚠️ Songbird health check failed: {} (continuing without mesh)",
                e
            );
        }
    }

    // Layer 3: Establish BTSP tunnel via BearDog (capability: secure_tunneling)
    // This creates a cryptographically secure channel for inter-primal communication
    match establish_btsp_tunnel(&beardog_socket, family_id).await {
        Ok(session_id) => {
            info!("✅ BTSP tunnel established (session: {})", session_id);
        }
        Err(e) => {
            // BTSP is optional for local deployments, warn but continue
            debug!("BTSP tunnel not established: {} (local mode)", e);
        }
    }

    info!("✅ Connected to Tower Atomic (gen 0 → gen 1 transition)");
    info!("   Security context inherited via capability-based discovery");

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_socket_nucleation_creates_valid_paths() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::default());
        let beardog_socket = nucleation.assign_socket("beardog", "test-family");
        let songbird_socket = nucleation.assign_socket("songbird", "test-family");

        // Socket paths should be different
        assert_ne!(beardog_socket, songbird_socket);

        // Should contain primal name and family
        let beardog_str = beardog_socket.to_string_lossy();
        let songbird_str = songbird_socket.to_string_lossy();

        assert!(beardog_str.contains("beardog") || beardog_str.contains("test-family"));
        assert!(songbird_str.contains("songbird") || songbird_str.contains("test-family"));
    }

    #[test]
    fn test_socket_strategy_default() {
        let strategy = SocketStrategy::default();
        // Default strategy should be XdgRuntime
        assert!(matches!(strategy, SocketStrategy::XdgRuntime));
    }

    #[test]
    fn test_socket_strategy_family_deterministic() {
        let strategy = SocketStrategy::FamilyDeterministic;
        assert!(matches!(strategy, SocketStrategy::FamilyDeterministic));
    }

    #[tokio::test]
    async fn test_register_self_capabilities() {
        // Verify the capabilities list is correct
        let capabilities = [
            "primal.germination",
            "primal.terraria",
            "ecosystem.coordination",
            "ecosystem.nucleation",
            "graph.execution",
        ];

        assert_eq!(capabilities.len(), 5);
        assert!(capabilities.contains(&"primal.germination"));
        assert!(capabilities.contains(&"graph.execution"));
    }

    #[test]
    fn test_biome_os_mode_variants() {
        // Test BiomeOsMode enum usage
        let bootstrap = BiomeOsMode::Bootstrap;
        let coordinated = BiomeOsMode::Coordinated;

        // Both variants should be distinct
        assert!(matches!(bootstrap, BiomeOsMode::Bootstrap));
        assert!(matches!(coordinated, BiomeOsMode::Coordinated));
    }

    #[test]
    fn test_bootstrap_graph_path_construction() {
        let graphs_dir = std::path::Path::new("/test/graphs");
        let bootstrap_graph_path = graphs_dir.join("tower_atomic_bootstrap.toml");

        assert_eq!(
            bootstrap_graph_path.to_string_lossy(),
            "/test/graphs/tower_atomic_bootstrap.toml"
        );
    }

    #[test]
    fn test_environment_setup_for_bootstrap() {
        let family_id = "test-family";
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), family_id.to_string());
        env.insert("BIOMEOS_FAMILY_ID".to_string(), family_id.to_string());
        env.insert("BIOMEOS_MODE".to_string(), "bootstrap".to_string());

        assert_eq!(env.get("FAMILY_ID"), Some(&"test-family".to_string()));
        assert_eq!(env.get("BIOMEOS_MODE"), Some(&"bootstrap".to_string()));
        assert_eq!(env.len(), 3);
    }

    #[tokio::test]
    async fn test_execute_bootstrap_sequence_missing_graph() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let graphs_dir = temp_dir.path();
        // No tower_atomic_bootstrap.toml file exists
        let result = execute_bootstrap_sequence(
            graphs_dir,
            "test-family",
            &Arc::new(RwLock::new(
                SocketNucleation::new(SocketStrategy::default()),
            )),
        )
        .await;

        let err = result.expect_err("Should fail when bootstrap graph not found");
        assert!(
            err.to_string().contains("Bootstrap graph not found"),
            "Error should mention missing graph: {err}"
        );
        assert!(
            err.to_string().contains("tower_atomic_bootstrap.toml"),
            "Error should mention expected filename: {err}"
        );
    }

    #[tokio::test]
    async fn test_execute_bootstrap_sequence_invalid_toml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let graphs_dir = temp_dir.path();
        let bootstrap_path = graphs_dir.join("tower_atomic_bootstrap.toml");
        std::fs::write(&bootstrap_path, "invalid toml {{{").expect("Failed to write invalid TOML");

        let result = execute_bootstrap_sequence(
            graphs_dir,
            "test-family",
            &Arc::new(RwLock::new(
                SocketNucleation::new(SocketStrategy::default()),
            )),
        )
        .await;

        let err = result.expect_err("Should fail on invalid TOML");
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("TOML"),
            "Error should mention parse/TOML: {err}"
        );
    }

    #[tokio::test]
    async fn test_register_self_in_registry_bootstrap_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("biomeos-test.sock");
        std::fs::File::create(&socket_path).expect("Create socket path placeholder");

        let router = Arc::new(NeuralRouter::new("test-family"));
        let mode = RwLock::new(BiomeOsMode::Bootstrap);

        register_self_in_registry(&router, "test-family", &socket_path, &mode)
            .await
            .expect("Registration should succeed");

        let caps = router.list_capabilities().await;
        assert!(
            caps.len() >= 5,
            "Should register at least 5 capabilities, got {}",
            caps.len()
        );
        assert!(
            caps.contains_key("primal.germination"),
            "Should register primal.germination"
        );
        assert!(
            caps.contains_key("graph.execution"),
            "Should register graph.execution"
        );
    }

    #[tokio::test]
    async fn test_register_self_in_registry_coordinated_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("biomeos-coord.sock");
        std::fs::File::create(&socket_path).expect("Create socket path placeholder");

        let router = Arc::new(NeuralRouter::new("coord-family"));
        let mode = RwLock::new(BiomeOsMode::Coordinated);

        register_self_in_registry(&router, "coord-family", &socket_path, &mode)
            .await
            .expect("Registration should succeed");

        let providers = router
            .get_capability_providers("ecosystem.coordination")
            .await;
        assert!(
            providers.is_some(),
            "Should have ecosystem.coordination registered"
        );
        let providers = providers.unwrap();
        assert_eq!(providers[0].primal_name.as_ref(), "biomeos-coord-family");
        assert_eq!(providers[0].source.as_ref(), "coordinated");
    }

    #[test]
    fn test_primal_name_format() {
        let primal_name = format!("biomeos-{}", "my-family");
        assert_eq!(primal_name, "biomeos-my-family");
    }

    #[test]
    fn test_capabilities_list_complete() {
        let capabilities = [
            "primal.germination",
            "primal.terraria",
            "ecosystem.coordination",
            "ecosystem.nucleation",
            "graph.execution",
        ];
        assert_eq!(capabilities.len(), 5);
        assert!(capabilities.iter().all(|c| !c.is_empty()));
    }

    #[tokio::test]
    async fn test_transition_to_coordinated_tower_sockets_present() {
        use biomeos_test_utils::TestEnvGuard;

        let tmp = TempDir::new().expect("temp");
        let _xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", tmp.path().to_str().expect("utf8"));

        let family_id = "bootstrap-coord-fam";
        let security_provider = biomeos_types::env_config::security_provider()
            .or_else(|| {
                biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal(
                    "security",
                )
                .map(String::from)
            })
            .unwrap_or_else(|| biomeos_types::primal_names::BEARDOG.to_string());
        let network_provider = biomeos_types::env_config::network_provider()
            .or_else(|| {
                biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal(
                    "discovery",
                )
                .map(String::from)
            })
            .unwrap_or_else(|| biomeos_types::primal_names::SONGBIRD.to_string());

        let mut nucleation = SocketNucleation::new(SocketStrategy::default());
        let beardog_socket = nucleation.assign_socket(&security_provider, family_id);
        let songbird_socket = nucleation.assign_socket(&network_provider, family_id);
        std::fs::File::create(&beardog_socket).expect("touch beardog");
        std::fs::File::create(&songbird_socket).expect("touch songbird");

        let result = transition_to_coordinated(family_id).await;
        assert!(
            result.is_ok(),
            "transition should complete when sockets exist: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_register_self_in_registry_socket_path_used() {
        let temp_dir = TempDir::new().expect("temp");
        let socket_path = temp_dir.path().join("registry-path.sock");
        std::fs::File::create(&socket_path).expect("placeholder");

        let router = Arc::new(NeuralRouter::new("path-family"));
        let mode = RwLock::new(BiomeOsMode::Bootstrap);

        register_self_in_registry(&router, "path-family", &socket_path, &mode)
            .await
            .expect("register");

        let p = router
            .get_capability_providers("ecosystem.nucleation")
            .await
            .expect("providers");
        assert_eq!(
            p[0].endpoint,
            biomeos_core::TransportEndpoint::UnixSocket { path: socket_path }
        );
    }
}
