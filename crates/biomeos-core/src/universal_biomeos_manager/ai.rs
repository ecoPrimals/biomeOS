// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! AI Capability Routing
//!
//! biomeOS does not embed AI logic. All AI capabilities are provided by
//! **Squirrel** (the AI/MCP bridge primal) discovered at runtime via
//! `capability.discover { domain: "ai" }`.
//!
//! This module exposes thin routing methods that:
//! 1. Attempt to discover Squirrel via the capability registry.
//! 2. Forward the request as a JSON-RPC `ai.*` capability call.
//! 3. Return a structured "unavailable" response when no AI primal is present,
//!    allowing biomeOS to be fully deployable with ecoBins alone.
//!
//! A user can "tag in" an AI primal at any point — deploying Squirrel into
//! the ecosystem enables agentic deployment assistance without restarting
//! biomeOS.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::core::UniversalBiomeOSManager;

/// AI primal availability status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiAvailability {
    /// An AI primal (Squirrel) is registered and healthy.
    Available,
    /// No AI primal is currently discovered in the ecosystem.
    Unavailable,
}

impl UniversalBiomeOSManager {
    /// AI-powered biome management assistance.
    ///
    /// Delegates to the Squirrel AI primal when available. Returns a
    /// structured "no AI primal available" response otherwise, so callers
    /// always get a valid JSON-RPC result.
    pub async fn ai_assist(
        &self,
        query: &str,
        context: Option<String>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("Processing AI assistance request");

        let mut result = HashMap::new();
        result.insert("query".to_string(), serde_json::json!(query));
        result.insert("context".to_string(), serde_json::json!(context));
        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );

        let (availability, provider_info) = self.probe_ai_primal().await;

        match availability {
            AiAvailability::Available => {
                tracing::info!(provider = %provider_info.name, "AI primal available — forwarding query");

                let params = serde_json::json!({
                    "query": query,
                    "context": context,
                });

                match self
                    .send_capability_call(&provider_info, "ai.assist", params)
                    .await
                {
                    Ok(response) => {
                        result.insert("status".to_string(), serde_json::json!("success"));
                        result.insert("source".to_string(), serde_json::json!(provider_info.name));
                        result.insert("response".to_string(), response);
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "AI capability call failed — degrading gracefully");
                        Self::fill_unavailable_response(&mut result, query);
                    }
                }
            }
            AiAvailability::Unavailable => {
                Self::fill_unavailable_response(&mut result, query);
            }
        }

        Ok(result)
    }

    /// Check whether an AI primal is present in the ecosystem.
    pub async fn get_ai_status(&self) -> Result<HashMap<String, serde_json::Value>> {
        let (availability, provider_info) = self.probe_ai_primal().await;
        let is_available = availability == AiAvailability::Available;

        let mut result = HashMap::new();
        result.insert("ai_available".to_string(), serde_json::json!(is_available));
        result.insert("availability".to_string(), serde_json::json!(availability));
        result.insert(
            "provider".to_string(),
            serde_json::json!(if is_available {
                provider_info.name.as_str()
            } else {
                "none"
            }),
        );
        result.insert(
            "deploy_hint".to_string(),
            serde_json::json!(
                "Deploy Squirrel into the ecosystem to enable AI-assisted operations. \
                 Use `biomeos deploy --graph tower_ai.toml` or register Squirrel via \
                 `lifecycle.register`."
            ),
        );

        Ok(result)
    }

    /// Probe the capability registry for an AI primal.
    ///
    /// Returns the availability status and, when available, the provider info
    /// (name + endpoint) for immediate IPC.
    async fn probe_ai_primal(&self) -> (AiAvailability, AiProviderInfo) {
        let primals = self.get_registered_primals().await;

        for primal in &primals {
            let has_ai = primal
                .capabilities
                .iter()
                .any(|c| c.category == "ai" || c.name.starts_with("ai"));
            if has_ai {
                return (
                    AiAvailability::Available,
                    AiProviderInfo {
                        name: primal.name.clone(),
                        endpoint: primal.endpoint.clone(),
                    },
                );
            }
        }

        (AiAvailability::Unavailable, AiProviderInfo::none())
    }

    /// Forward a JSON-RPC capability call to a specific provider.
    async fn send_capability_call(
        &self,
        provider: &AiProviderInfo,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        tracing::debug!(method, provider = %provider.name, "Routing capability call");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let socket_path = std::path::Path::new(&provider.endpoint);

        if !socket_path.exists() {
            anyhow::bail!(
                "Endpoint '{}' for primal '{}' is not a reachable socket",
                provider.endpoint,
                provider.name
            );
        }

        // Unix socket IPC — newline-delimited JSON-RPC
        let stream = tokio::net::UnixStream::connect(socket_path).await?;
        let (reader, mut writer) = tokio::io::split(stream);

        let mut payload = serde_json::to_vec(&request)?;
        payload.push(b'\n');

        use tokio::io::AsyncWriteExt;
        writer.write_all(&payload).await?;
        writer.shutdown().await?;

        use tokio::io::AsyncBufReadExt;
        let mut buf_reader = tokio::io::BufReader::new(reader);
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;

        let response: serde_json::Value = serde_json::from_str(line.trim())?;

        if let Some(r) = response.get("result") {
            Ok(r.clone())
        } else if let Some(e) = response.get("error") {
            anyhow::bail!("Provider returned error: {e}")
        } else {
            Ok(response)
        }
    }

    /// Populate the response map when no AI primal is available.
    fn fill_unavailable_response(result: &mut HashMap<String, serde_json::Value>, query: &str) {
        result.insert("status".to_string(), serde_json::json!("no_ai_primal"));
        result.insert("source".to_string(), serde_json::json!("biomeos"));
        result.insert(
            "response".to_string(),
            serde_json::json!(format!(
                "No AI primal is currently deployed. Your query '{query}' requires an AI \
                 capability provider (Squirrel). Deploy one with \
                 `biomeos deploy --graph tower_ai.toml` or use `biomeos --help` for \
                 available non-AI commands."
            )),
        );
    }
}

/// Minimal info needed to route to an AI provider.
#[derive(Debug, Clone)]
struct AiProviderInfo {
    name: String,
    endpoint: String,
}

impl AiProviderInfo {
    fn none() -> Self {
        Self {
            name: String::new(),
            endpoint: String::new(),
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_biomeos_manager::UniversalBiomeOSManager;
    use biomeos_types::BiomeOSConfig;

    async fn test_manager() -> UniversalBiomeOSManager {
        UniversalBiomeOSManager::new(BiomeOSConfig::default())
            .await
            .expect("create test manager")
    }

    #[tokio::test]
    async fn test_ai_assist_without_ai_primal() {
        let manager = test_manager().await;
        let result = manager
            .ai_assist("what is the system health?", None)
            .await
            .expect("ai_assist should succeed even without AI primal");

        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("no_ai_primal"),
            "Without Squirrel deployed, status should be no_ai_primal"
        );
        assert_eq!(
            result.get("source").and_then(|v| v.as_str()),
            Some("biomeos")
        );
        assert!(result.contains_key("response"));
        assert!(result.contains_key("query"));
        assert!(result.contains_key("timestamp"));
    }

    #[tokio::test]
    async fn test_ai_assist_with_context() {
        let manager = test_manager().await;
        let result = manager
            .ai_assist("deploy my graph", Some("tower context".to_string()))
            .await
            .expect("ai_assist should succeed");

        assert_eq!(
            result.get("query").and_then(|v| v.as_str()),
            Some("deploy my graph")
        );
        assert_eq!(
            result.get("context").and_then(|v| v.as_str()),
            Some("tower context")
        );
    }

    #[tokio::test]
    async fn test_get_ai_status_no_provider() {
        let manager = test_manager().await;
        let status = manager
            .get_ai_status()
            .await
            .expect("get_ai_status should succeed");

        assert_eq!(
            status
                .get("ai_available")
                .and_then(serde_json::Value::as_bool),
            Some(false),
            "Without AI primal, ai_available should be false"
        );
        assert_eq!(
            status.get("provider").and_then(|v| v.as_str()),
            Some("none")
        );
        assert!(status.contains_key("deploy_hint"));
    }

    #[tokio::test]
    async fn test_get_ai_status_with_provider() {
        use biomeos_primal_sdk::PrimalType;
        use biomeos_types::{Health, PrimalCapability};

        let manager = test_manager().await;
        {
            let mut registry = manager.registered_primals().write().await;
            registry.insert(
                "squirrel".to_string(),
                super::super::core::PrimalInfo {
                    id: "squirrel".to_string(),
                    name: "squirrel".to_string(),
                    primal_type: PrimalType::from_discovered("ai", "squirrel".to_string(), "1.0.0"),
                    endpoint: "/tmp/squirrel-test.sock".to_string(),
                    capabilities: vec![
                        PrimalCapability::new("ai", "assist", "1.0"),
                        PrimalCapability::new("ai", "query", "1.0"),
                    ],
                    health: Health::Healthy,
                    last_seen: chrono::Utc::now(),
                    discovered_at: chrono::Utc::now(),
                    metadata: std::collections::HashMap::new(),
                },
            );
        }

        let status = manager
            .get_ai_status()
            .await
            .expect("get_ai_status should succeed");

        assert_eq!(
            status
                .get("ai_available")
                .and_then(serde_json::Value::as_bool),
            Some(true),
            "With Squirrel registered, ai_available should be true"
        );
        assert_eq!(
            status.get("provider").and_then(|v| v.as_str()),
            Some("squirrel")
        );
    }

    #[tokio::test]
    async fn test_probe_ai_primal_empty_ecosystem() {
        let manager = test_manager().await;
        let (availability, _provider) = manager.probe_ai_primal().await;
        assert_eq!(availability, AiAvailability::Unavailable);
    }

    #[tokio::test]
    async fn test_ai_availability_serialization() {
        let available =
            serde_json::to_string(&AiAvailability::Available).expect("serialize Available");
        assert_eq!(available, "\"Available\"");

        let unavailable =
            serde_json::to_string(&AiAvailability::Unavailable).expect("serialize Unavailable");
        assert_eq!(unavailable, "\"Unavailable\"");

        let roundtrip: AiAvailability =
            serde_json::from_str(&available).expect("deserialize Available");
        assert_eq!(roundtrip, AiAvailability::Available);
    }

    #[test]
    fn test_fill_unavailable_response() {
        let mut result = HashMap::new();
        UniversalBiomeOSManager::fill_unavailable_response(&mut result, "test query");

        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("no_ai_primal")
        );
        assert_eq!(
            result.get("source").and_then(|v| v.as_str()),
            Some("biomeos")
        );
        let response = result
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(response.contains("test query"));
        assert!(response.contains("Squirrel"));
    }
}
