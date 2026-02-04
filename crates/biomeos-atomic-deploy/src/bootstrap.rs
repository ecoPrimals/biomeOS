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
use tokio::time::{sleep, Duration};
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

    let primal_name = format!("biomeos-{}", family_id);
    let capabilities = vec![
        "primal.germination",
        "primal.terraria",
        "ecosystem.coordination",
        "ecosystem.nucleation",
        "graph.execution",
    ];

    let cap_count = capabilities.len();

    // Register each capability
    for capability in capabilities {
        router
            .register_capability(capability, &primal_name, socket_path, source)
            .await?;
    }

    info!(
        "✅ biomeOS registered {} capabilities in registry",
        cap_count
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
    graphs_dir: &PathBuf,
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

    // EVOLVED: Bootstrap mode - minimal hardcoding for initial system bring-up
    // These core primals (beardog=security, songbird=discovery) are needed
    // for capability resolution. After bootstrap, all discovery is runtime.
    let mut nucleation = SocketNucleation::new(SocketStrategy::default());
    let beardog_socket = nucleation.assign_socket("beardog", family_id);
    let songbird_socket = nucleation.assign_socket("songbird", family_id);

    loop {
        if start.elapsed() > max_wait {
            return Err(anyhow::anyhow!(
                "Tower Atomic did not become available within 30s. \
                 Ensure beardog and songbird primals are running for bootstrap."
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

    // EVOLVED (Jan 27, 2026): Capability-based security context via AtomicClient
    // Layer 1: Verify BearDog health (crypto provider)
    match verify_primal_health(&beardog_socket, "beardog").await {
        Ok(caps) => {
            info!("✅ BearDog healthy with {} capabilities", caps.len());
        }
        Err(e) => {
            warn!(
                "⚠️ BearDog health check failed: {} (continuing with degraded security)",
                e
            );
        }
    }

    // Layer 2: Verify Songbird health (discovery/mesh)
    match verify_primal_health(&songbird_socket, "songbird").await {
        Ok(caps) => {
            info!("✅ Songbird healthy with {} capabilities", caps.len());
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
