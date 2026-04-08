// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Translation registry and domain registration during server startup.

use anyhow::Result;
use tracing::{info, warn};

use super::NeuralApiServer;

/// Bundled bootstrap graph TOML, compiled into the binary so biomeOS can
/// load capability translations even when the filesystem copy is absent.
pub(crate) const BUNDLED_BOOTSTRAP_GRAPH: &str =
    include_str!("../../../../graphs/tower_atomic_bootstrap.toml");

impl NeuralApiServer {
    /// Load translations from Tower Atomic graph on startup
    pub(crate) async fn load_translations_on_startup(&self) -> Result<()> {
        // 1. Load hardcoded default translations (always available)
        {
            let mut registry = self.translation_registry.write().await;
            let default_count = registry.load_defaults();
            info!(
                "📚 Loaded {} default capability translations",
                default_count
            );
        }

        // 2. Overlay with config/capability_registry.toml if present
        {
            let config_path = self.graphs_dir.join("../config/capability_registry.toml");
            if config_path.exists() {
                let mut registry = self.translation_registry.write().await;
                match registry.load_from_config(&config_path, |provider, family_id| {
                    crate::capability_translation::resolve_primal_socket(provider, family_id)
                }) {
                    Ok(count) => info!(
                        "📚 Loaded {} translations from capability_registry.toml",
                        count
                    ),
                    Err(e) => warn!("⚠️  Failed to load capability_registry.toml: {}", e),
                }

                // Bridge domain providers into the NeuralRouter so capability.call
                // can discover which primal handles each capability domain.
                if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = config_content.parse::<toml::Value>() {
                        if let Some(domains) = config.get("domains").and_then(|d| d.as_table()) {
                            let family_id = biomeos_core::family_discovery::get_family_id();
                            for (domain_name, domain_cfg) in domains {
                                let provider = domain_cfg
                                    .get("provider")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                if provider.is_empty() || provider == "*" {
                                    continue;
                                }
                                let socket = crate::capability_translation::resolve_primal_socket(
                                    provider, &family_id,
                                );
                                let caps = domain_cfg
                                    .get("capabilities")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default();

                                for cap in caps {
                                    if let Err(e) = self
                                        .router
                                        .register_capability_unix(
                                            cap,
                                            provider,
                                            &socket,
                                            "config_registry",
                                        )
                                        .await
                                    {
                                        warn!(
                                            "⚠️  Failed to register domain capability {} → {}: {}",
                                            cap, provider, e
                                        );
                                    }
                                }
                                info!(
                                    "📝 Registered domain '{}' → {} ({})",
                                    domain_name, provider, socket
                                );
                            }
                        }
                    }
                }
            }
        }

        // 3. Load translations from Tower Atomic graph (filesystem or bundled)
        info!("📝 Loading semantic translations from Tower Atomic graph...");
        let bootstrap_graph_path = self.graphs_dir.join("tower_atomic_bootstrap.toml");
        let graph_result = if bootstrap_graph_path.exists() {
            crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path)
        } else {
            info!("   Filesystem graph not found — using bundled bootstrap graph");
            crate::neural_graph::Graph::from_toml_str(BUNDLED_BOOTSTRAP_GRAPH)
        };
        match graph_result {
            Ok(graph) => match self.load_translations_from_graph(&graph).await {
                Ok(_) => info!("✅ Semantic translations loaded from graph"),
                Err(e) => warn!("⚠️  Failed to load translations: {}", e),
            },
            Err(e) => warn!("⚠️  Failed to parse bootstrap graph: {}", e),
        }
        Ok(())
    }
}

#[cfg(test)]
impl NeuralApiServer {
    /// Exercise [`NeuralApiServer::load_translations_on_startup`] in unit tests (private otherwise).
    pub(crate) async fn test_load_translations_on_startup(&self) -> Result<()> {
        self.load_translations_on_startup().await
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test")]
mod tests {
    use super::super::NeuralApiServer;

    #[tokio::test]
    async fn test_load_translations_on_startup_defaults_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("neural-api.sock");
        let server = NeuralApiServer::new(temp.path(), "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("load translations");
    }

    #[tokio::test]
    async fn test_load_translations_on_startup_with_tower_atomic_graph() {
        let temp = tempfile::tempdir().expect("tempdir");
        let graph_toml = r#"
[graph]
id = "tower_atomic_bootstrap"
version = "1.0.0"
description = "Test graph for translations"

[[nodes]]
id = "log1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "test"
"#;
        std::fs::write(temp.path().join("tower_atomic_bootstrap.toml"), graph_toml)
            .expect("write graph");
        let sock = temp.path().join("neural-api.sock");
        let server = NeuralApiServer::new(temp.path(), "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("load translations with graph");
    }

    #[tokio::test]
    async fn test_load_translations_graph_parse_warn_branch_still_ok() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            temp.path().join("tower_atomic_bootstrap.toml"),
            "[[[ not valid graph",
        )
        .expect("write");
        let sock = temp.path().join("neural-api.sock");
        let server = NeuralApiServer::new(temp.path(), "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("startup load tolerates bad graph file");
    }

    /// `load_translations_on_startup` overlays `graphs_dir/../config/capability_registry.toml` when present.
    #[tokio::test]
    async fn test_load_translations_on_startup_with_capability_registry_overlay() {
        let base = tempfile::tempdir().expect("tempdir");
        let graphs_dir = base.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).expect("graphs dir");

        let config_path = graphs_dir.join("../config/capability_registry.toml");
        std::fs::create_dir_all(config_path.parent().expect("parent")).expect("config dir");
        std::fs::write(
            &config_path,
            r#"
[translations.crypto]
"crypto.unit.test_ping" = { provider = "beardog", method = "ping" }
"#,
        )
        .expect("write capability_registry.toml");

        let sock = graphs_dir.join("neural-api.sock");
        let server = NeuralApiServer::new(&graphs_dir, "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("load with overlay");
    }

    /// Invalid TOML at `graphs_dir/../config/capability_registry.toml` triggers the warn branch.
    #[tokio::test]
    async fn test_load_translations_on_startup_capability_registry_toml_parse_error() {
        let base = tempfile::tempdir().expect("tempdir");
        let graphs_dir = base.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).expect("graphs dir");
        let config_path = graphs_dir.join("../config/capability_registry.toml");
        std::fs::create_dir_all(config_path.parent().expect("parent")).expect("config dir");
        std::fs::write(&config_path, "[[[ not valid toml").expect("write broken toml");
        let sock = graphs_dir.join("neural-api.sock");
        let server = NeuralApiServer::new(&graphs_dir, "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("startup tolerates bad capability_registry.toml");
    }

    #[tokio::test]
    async fn test_load_translations_skips_domain_providers_star_and_empty() {
        let base = tempfile::tempdir().expect("tempdir");
        let graphs_dir = base.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).expect("graphs dir");
        let config_path = graphs_dir.join("../config/capability_registry.toml");
        std::fs::create_dir_all(config_path.parent().expect("parent")).expect("config dir");
        std::fs::write(
            &config_path,
            r#"
[translations.crypto]
"crypto.unit.test_ping" = { provider = "beardog", method = "ping" }

[domains.star]
provider = "*"
capabilities = ["cap.a"]

[domains.empty]
provider = ""
capabilities = ["cap.b"]

[domains.valid]
provider = "beardog"
capabilities = ["cap.c"]
"#,
        )
        .expect("write");
        let sock = graphs_dir.join("neural-api.sock");
        let server = NeuralApiServer::new(&graphs_dir, "fam-dom", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("load with domain table");
    }
}
