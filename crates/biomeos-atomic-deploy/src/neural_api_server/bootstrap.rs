// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Bootstrap vs coordinated mode handling and explicit coordinated-mode detection.

use anyhow::Result;
use tracing::{error, info, warn};

use super::NeuralApiServer;
use crate::mode::BiomeOsMode;

/// Check if mode string indicates explicit coordinated mode
#[must_use]
pub fn is_explicit_coordinated_mode_str(mode: &str) -> bool {
    let m = mode.to_lowercase();
    m == "coordinated" || m == "coord" || m == "join"
}

/// Check if `BIOMEOS_MODE` env var indicates explicit coordinated mode
#[must_use]
pub fn is_explicit_coordinated_mode() -> bool {
    is_explicit_coordinated_mode_with(None)
}

/// Check coordinated mode with optional override (for testing without env mutation).
///
/// - `Some(s)` — use the given mode string
/// - `None` — read from `BIOMEOS_MODE` env var (or false if unset)
#[must_use]
pub fn is_explicit_coordinated_mode_with(env_override: Option<&str>) -> bool {
    let mode: Option<String> = env_override
        .map(String::from)
        .or_else(|| std::env::var("BIOMEOS_MODE").ok());
    mode.as_deref()
        .map(is_explicit_coordinated_mode_str)
        .unwrap_or(false)
}

impl NeuralApiServer {
    /// Handle bootstrap mode: execute bootstrap sequence and transition to coordinated
    pub(crate) async fn handle_bootstrap_mode(&self) -> Result<()> {
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
    pub(crate) async fn handle_coordinated_mode(&self) -> Result<()> {
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
}

#[cfg(test)]
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
    fn test_is_explicit_coordinated_mode_env_coordinated() {
        assert!(is_explicit_coordinated_mode_with(Some("coordinated")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_coord() {
        assert!(is_explicit_coordinated_mode_with(Some("coord")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_join() {
        assert!(is_explicit_coordinated_mode_with(Some("join")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_bootstrap_returns_false() {
        assert!(!is_explicit_coordinated_mode_with(Some("bootstrap")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_unset_returns_false() {
        // Use Some("") to simulate unset - both yield false without env mutation
        assert!(!is_explicit_coordinated_mode_with(Some("")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_unknown_returns_false() {
        assert!(!is_explicit_coordinated_mode_with(Some("unknown")));
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

    #[test]
    fn test_is_explicit_coordinated_mode_str_numeric_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str("123"));
    }
}
