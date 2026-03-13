// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Server lifecycle management: bootstrap, mode detection, and initialization
//!
//! Handles the server startup sequence including:
//! - Operating mode detection (Bootstrap vs Coordinated)
//! - Bootstrap sequence execution
//! - Mode transitions
//! - Socket setup and listening

use anyhow::{Context, Result};
use tokio::net::UnixListener;
use tracing::{debug, error, info, warn};

use super::NeuralApiServer;
use crate::mode::BiomeOsMode;

/// Check if mode string indicates explicit coordinated mode
#[must_use]
pub fn is_explicit_coordinated_mode_str(mode: &str) -> bool {
    let m = mode.to_lowercase();
    m == "coordinated" || m == "coord" || m == "join"
}

/// Check if BIOMEOS_MODE env var indicates explicit coordinated mode
#[must_use]
pub fn is_explicit_coordinated_mode() -> bool {
    std::env::var("BIOMEOS_MODE")
        .as_deref()
        .map(is_explicit_coordinated_mode_str)
        .unwrap_or(false)
}

impl NeuralApiServer {
    /// Start the Neural API server
    ///
    /// Performs mode detection, bootstrap if needed, and starts accepting connections.
    pub async fn serve(&self) -> Result<()> {
        // 1. Detect operating mode
        info!("🔍 Detecting biomeOS operating mode...");
        let detected_mode = BiomeOsMode::detect(&self.family_id).await;
        {
            let mut mode = self.mode.write().await;
            *mode = detected_mode;
        }

        // 2. Bootstrap if needed
        if detected_mode == BiomeOsMode::Bootstrap {
            self.handle_bootstrap_mode().await?;
        } else {
            self.handle_coordinated_mode().await?;
        }

        // Start lifecycle monitoring
        info!("🔍 Starting primal lifecycle monitoring...");
        if let Err(e) = self.lifecycle_handler.start_monitoring().await {
            warn!("⚠️ Failed to start lifecycle monitoring: {}", e);
        }

        // ALWAYS load semantic translations from Tower Atomic graph
        // This is ecosystem-wide configuration, not mode-specific
        self.load_translations_on_startup().await?;

        // 3. Setup socket and start listening
        self.start_listening().await?;

        Ok(())
    }

    /// Handle bootstrap mode: execute bootstrap sequence and transition to coordinated
    async fn handle_bootstrap_mode(&self) -> Result<()> {
        info!("🌱 === BIOMEOS BOOTSTRAP MODE ===");
        info!("🌍 No existing ecosystem detected");
        info!("🏗️  Creating ecosystem foundation...");

        // Register biomeOS in its own capability registry
        crate::bootstrap::register_self_in_registry(
            &self.router,
            &self.family_id,
            &self.socket_path,
            &self.mode,
        )
        .await?;

        // Execute bootstrap sequence (germinate Tower Atomic)
        info!("");
        info!("🏰 Germinating Tower Atomic (ecosystem genesis)...");

        // Load translations before executing bootstrap sequence
        let bootstrap_graph_path = self.graphs_dir.join("tower_atomic_bootstrap.toml");
        if bootstrap_graph_path.exists() {
            if let Ok(graph) = crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path) {
                if let Err(e) = self.load_translations_from_graph(&graph).await {
                    warn!("⚠️  Failed to load translations before bootstrap: {}", e);
                }
            }
        }

        match crate::bootstrap::execute_bootstrap_sequence(
            &self.graphs_dir,
            &self.family_id,
            &self.nucleation,
        )
        .await
        {
            Ok(_) => {
                info!("✅ Tower Atomic genesis complete!");
                info!("🔄 Transitioning to COORDINATED MODE...");

                // Transition to coordinated mode
                if let Err(e) = crate::bootstrap::transition_to_coordinated(&self.family_id).await {
                    error!("⚠️  Mode transition failed: {}", e);
                    warn!("   Continuing in bootstrap mode (Tower Atomic may be unhealthy)");
                } else {
                    // Update mode
                    let mut mode = self.mode.write().await;
                    *mode = BiomeOsMode::Coordinated;
                    info!("✅ biomeOS now operating in COORDINATED MODE (gen 1)");
                }
            }
            Err(e) => {
                error!("❌ Bootstrap sequence failed: {}", e);
                error!("   biomeOS will continue in bootstrap mode");
                error!("   Manual intervention may be required");
            }
        }
        info!("");

        Ok(())
    }

    /// Handle coordinated mode: join existing ecosystem
    async fn handle_coordinated_mode(&self) -> Result<()> {
        info!("🔄 === BIOMEOS COORDINATED MODE ===");
        info!("🏰 Tower Atomic detected or explicit coordinated mode");
        info!("🌍 Joining existing ecosystem");

        // Check if this is explicit coordinated mode (primals will auto-register)
        let explicit_mode = is_explicit_coordinated_mode();

        if explicit_mode {
            // Explicit coordinated mode: don't wait for sockets
            // Primals will register themselves via auto-registration
            info!("📋 Explicit coordinated mode - primals will auto-register");
            info!("   Neural API will accept capability registrations dynamically");
        } else {
            // Auto-detected coordinated mode: establish connection
            if let Err(e) = crate::bootstrap::transition_to_coordinated(&self.family_id).await {
                warn!("⚠️  Failed to establish BTSP tunnel: {}", e);
                warn!("   Operating without inherited security");
            }
        }

        // Register in ecosystem
        crate::bootstrap::register_self_in_registry(
            &self.router,
            &self.family_id,
            &self.socket_path,
            &self.mode,
        )
        .await?;

        Ok(())
    }

    /// Load translations from Tower Atomic graph on startup
    async fn load_translations_on_startup(&self) -> Result<()> {
        info!("📝 Loading semantic translations from Tower Atomic graph...");
        let bootstrap_graph_path = self.graphs_dir.join("tower_atomic_bootstrap.toml");
        if bootstrap_graph_path.exists() {
            match crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path) {
                Ok(graph) => match self.load_translations_from_graph(&graph).await {
                    Ok(_) => info!("✅ Semantic translations loaded from graph"),
                    Err(e) => warn!("⚠️  Failed to load translations: {}", e),
                },
                Err(e) => warn!("⚠️  Failed to parse graph: {}", e),
            }
        } else {
            debug!("   No Tower Atomic graph found (will use direct method names)");
        }
        Ok(())
    }

    /// Setup socket and start accepting connections
    async fn start_listening(&self) -> Result<()> {
        // Remove old socket if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).context("Failed to remove old socket")?;
        }

        // Create Unix socket listener
        let listener =
            UnixListener::bind(&self.socket_path).context("Failed to bind Unix socket")?;

        let mode_str = {
            let mode = self.mode.read().await;
            match *mode {
                BiomeOsMode::Bootstrap => "BOOTSTRAP (genesis)",
                BiomeOsMode::Coordinated => "COORDINATED (gen 1)",
            }
        };

        info!(
            "🧠 Neural API server listening on: {}",
            self.socket_path.display()
        );
        info!("   Mode: {}", mode_str);
        info!("   Graphs directory: {}", self.graphs_dir.display());
        info!("   Family ID: {}", self.family_id);

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_is_explicit_coordinated_mode_str_coordinated() {
        assert!(is_explicit_coordinated_mode_str("coordinated"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_coord() {
        assert!(is_explicit_coordinated_mode_str("coord"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_join() {
        assert!(is_explicit_coordinated_mode_str("join"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_case_insensitive() {
        assert!(is_explicit_coordinated_mode_str("COORDINATED"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_bootstrap_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str("bootstrap"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_empty_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str(""));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_join_uppercase() {
        assert!(is_explicit_coordinated_mode_str("JOIN"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_unknown_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str("unknown"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_mixed_case() {
        assert!(is_explicit_coordinated_mode_str("Coordinated"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_whitespace_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str(" coordinated"));
    }

    #[test]
    #[ignore = "Requires BIOMEOS_MODE env var - run with explicit env"]
    fn test_is_explicit_coordinated_mode_env() {
        // When BIOMEOS_MODE=coordinated, should return true
        let _ = is_explicit_coordinated_mode();
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_partial_no_match() {
        assert!(!is_explicit_coordinated_mode_str("coordinatedx"));
        assert!(!is_explicit_coordinated_mode_str("xcoordinated"));
        assert!(!is_explicit_coordinated_mode_str("coordinat")); // partial, no match
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_join_suffix() {
        // "join" is valid; "joiner" would not match (different string)
        assert!(is_explicit_coordinated_mode_str("join"));
        assert!(!is_explicit_coordinated_mode_str("joiner"));
    }
}
