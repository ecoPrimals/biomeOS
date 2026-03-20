// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Spring-as-Provider Pattern
//!
//! Implements the standard pattern from `wateringHole/SPRING_AS_PROVIDER_PATTERN.md`
//! for springs to register as capability providers with biomeOS's Neural API.
//!
//! Absorbed from airSpring's `HttpProvider` / `BiomeosProvider` abstraction and
//! neuralSpring's `discover_by_capability()` pattern.
//!
//! # Architecture
//!
//! ```text
//! Spring                        biomeOS Neural API            Consumer
//!   │                                │                           │
//!   │ 1. Bind UDS socket             │                           │
//!   │ 2. capability.register ──────► │                           │
//!   │                                │ ◄── capability.call ──── │
//!   │ ◄── forwarded JSON-RPC ─────── │                           │
//!   │ ──── response ───────────────► │ ──── response ──────────► │
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A data provider that springs use to abstract their data source.
///
/// Springs implement this trait for each data backend (HTTP, biomeOS capability,
/// file, etc.), enabling graceful fallback when one source is unavailable.
///
/// # Example
///
/// ```rust,no_run
/// use biomeos_primal_sdk::provider::{Provider, ProviderError};
/// use async_trait::async_trait;
///
/// struct MyProvider;
///
/// #[async_trait]
/// impl Provider for MyProvider {
///     fn name(&self) -> &str { "my-provider" }
///     fn is_available(&self) -> bool { true }
///
///     async fn fetch(
///         &self,
///         operation: &str,
///         params: serde_json::Value,
///     ) -> Result<serde_json::Value, ProviderError> {
///         Ok(serde_json::json!({ "result": "ok" }))
///     }
/// }
/// ```
#[async_trait]
pub trait Provider: Send + Sync {
    /// Human-readable provider name (e.g. `"biomeOS"`, `"http"`, `"file"`).
    fn name(&self) -> &str;

    /// Whether the provider is currently reachable.
    fn is_available(&self) -> bool;

    /// Fetch data by invoking an operation with parameters.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] if the operation fails.
    async fn fetch(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError>;
}

/// Errors from a [`Provider`] operation.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Provider is not available (socket down, network error, etc.).
    #[error("provider unavailable: {0}")]
    Unavailable(String),

    /// Operation not supported by this provider.
    #[error("unsupported operation: {0}")]
    Unsupported(String),

    /// Transport or protocol error.
    #[error("transport error: {0}")]
    Transport(#[from] anyhow::Error),

    /// The provider returned an application-level error.
    #[error("provider error: {code}: {message}")]
    Application {
        /// Error code.
        code: i64,
        /// Error message.
        message: String,
    },
}

/// biomeOS capability-based provider.
///
/// Routes requests through the Neural API `capability.call` mechanism,
/// sending JSON-RPC over a Unix domain socket.
pub struct BiomeosProvider {
    /// Capability domain (e.g. `"ecology"`, `"science"`, `"physics"`).
    capability_domain: String,
    /// Socket path to the Neural API.
    neural_api_socket: PathBuf,
}

impl BiomeosProvider {
    /// Create a new biomeOS provider.
    ///
    /// # Arguments
    ///
    /// * `capability_domain` — The capability domain (e.g. `"ecology"`)
    /// * `neural_api_socket` — Path to the Neural API Unix socket
    pub fn new(
        capability_domain: impl Into<String>,
        neural_api_socket: impl Into<PathBuf>,
    ) -> Self {
        Self {
            capability_domain: capability_domain.into(),
            neural_api_socket: neural_api_socket.into(),
        }
    }

    /// Resolve the Neural API socket path using standard XDG resolution.
    pub fn from_env(capability_domain: impl Into<String>) -> Result<Self> {
        let socket_dir = if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
            PathBuf::from(dir)
        } else if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(xdg).join("biomeos")
        } else {
            biomeos_types::SystemPaths::new_lazy()
                .runtime_dir()
                .to_path_buf()
        };

        let socket = socket_dir.join("neural-api.sock");
        Ok(Self::new(capability_domain, socket))
    }

    /// Build a `capability.call` JSON-RPC request.
    fn build_request(&self, operation: &str, params: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.call",
            "params": {
                "capability": self.capability_domain,
                "operation": operation,
                "args": params
            },
            "id": uuid::Uuid::new_v4().to_string()
        })
    }
}

#[async_trait]
impl Provider for BiomeosProvider {
    fn name(&self) -> &'static str {
        "biomeOS"
    }

    fn is_available(&self) -> bool {
        self.neural_api_socket.exists()
    }

    async fn fetch(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        if !self.is_available() {
            return Err(ProviderError::Unavailable(format!(
                "Neural API socket not found: {}",
                self.neural_api_socket.display()
            )));
        }

        let request = self.build_request(operation, &params);

        let response = send_jsonrpc_uds(&self.neural_api_socket, &request)
            .await
            .map_err(ProviderError::Transport)?;

        if let Some(error) = response.get("error") {
            return Err(ProviderError::Application {
                code: error["code"].as_i64().unwrap_or(-1),
                message: error["message"].as_str().unwrap_or("unknown").to_string(),
            });
        }

        Ok(response["result"].clone())
    }
}

/// Capability registration request sent to the Neural API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistration {
    /// Capability domain (e.g. `"ecology"`, `"science"`).
    pub capability: String,
    /// Primal identifier.
    pub primal: String,
    /// Absolute path to the primal's Unix socket.
    pub socket: String,
    /// Registration source (e.g. `"startup"`, `"graph"`).
    pub source: String,
    /// Semantic operation mappings (short name -> full method name).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub semantic_mappings: HashMap<String, String>,
}

/// Register capabilities with the Neural API.
///
/// Sends a `capability.register` JSON-RPC call to the Neural API socket.
///
/// # Errors
///
/// Returns an error if the socket is unreachable or the registration fails.
pub async fn register_capabilities(
    neural_api_socket: &Path,
    registration: &CapabilityRegistration,
) -> Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capability.register",
        "params": registration,
        "id": uuid::Uuid::new_v4().to_string()
    });

    send_jsonrpc_uds(neural_api_socket, &request)
        .await
        .context("capability.register failed")
}

/// Provenance trio helpers — zero compile-time coupling to trio crates.
///
/// Absorbed from airSpring's provenance integration pattern.
pub mod provenance {
    use super::{Provider, ProviderError, Result};

    /// Begin a provenance experiment session via `capability.call`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Neural API is unreachable or the call fails.
    pub async fn begin_experiment_session(
        provider: &dyn Provider,
        experiment_id: &str,
        agent_did: Option<&str>,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut params = serde_json::json!({
            "experiment_id": experiment_id,
        });
        if let Some(did) = agent_did {
            params["agent_did"] = serde_json::json!(did);
        }
        provider.fetch("provenance.begin_session", params).await
    }

    /// Record a step in a provenance experiment via `capability.call`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Neural API is unreachable or the call fails.
    pub async fn record_step(
        provider: &dyn Provider,
        session_id: &str,
        step_name: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, ProviderError> {
        let params = serde_json::json!({
            "session_id": session_id,
            "step": step_name,
            "data": data,
        });
        provider.fetch("provenance.record_step", params).await
    }

    /// Complete a provenance experiment session via `capability.call`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Neural API is unreachable or the call fails.
    pub async fn complete_experiment(
        provider: &dyn Provider,
        session_id: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        let params = serde_json::json!({
            "session_id": session_id,
        });
        provider
            .fetch("provenance.complete_experiment", params)
            .await
    }
}

/// Send a JSON-RPC request over a Unix domain socket and return the response.
async fn send_jsonrpc_uds(
    socket_path: &Path,
    request: &serde_json::Value,
) -> Result<serde_json::Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let mut stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("connecting to {}", socket_path.display()))?;

    let payload = serde_json::to_vec(request)?;
    stream.write_all(&payload).await?;
    stream.shutdown().await?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;

    serde_json::from_slice(&buf).context("parsing JSON-RPC response")
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockProvider {
        available: AtomicBool,
        response: serde_json::Value,
    }

    impl MockProvider {
        fn new(available: bool, response: serde_json::Value) -> Self {
            Self {
                available: AtomicBool::new(available),
                response,
            }
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn name(&self) -> &'static str {
            "mock"
        }

        fn is_available(&self) -> bool {
            self.available.load(Ordering::Relaxed)
        }

        async fn fetch(
            &self,
            _operation: &str,
            _params: serde_json::Value,
        ) -> Result<serde_json::Value, ProviderError> {
            if !self.is_available() {
                return Err(ProviderError::Unavailable("mock down".into()));
            }
            Ok(self.response.clone())
        }
    }

    #[test]
    fn biomeos_provider_build_request() {
        let provider = BiomeosProvider::new("ecology", "/tmp/test.sock");
        let params = serde_json::json!({"temperature": 25.0});
        let req = provider.build_request("et0_fao56", &params);
        assert_eq!(req["method"], "capability.call");
        assert_eq!(req["params"]["capability"], "ecology");
        assert_eq!(req["params"]["operation"], "et0_fao56");
        assert_eq!(req["params"]["args"]["temperature"], 25.0);
    }

    #[test]
    fn biomeos_provider_not_available_without_socket() {
        let provider = BiomeosProvider::new("ecology", "/nonexistent/path.sock");
        assert!(!provider.is_available());
    }

    #[tokio::test]
    async fn biomeos_provider_fetch_unavailable() {
        let provider = BiomeosProvider::new("ecology", "/nonexistent/path.sock");
        let result = provider.fetch("et0", serde_json::json!({})).await;
        assert!(matches!(result, Err(ProviderError::Unavailable(_))));
    }

    #[tokio::test]
    async fn mock_provider_available() {
        let provider = MockProvider::new(true, serde_json::json!({"et0": 5.2}));
        assert!(provider.is_available());
        let result = provider
            .fetch("compute", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(result["et0"], 5.2);
    }

    #[tokio::test]
    async fn mock_provider_unavailable() {
        let provider = MockProvider::new(false, serde_json::json!({}));
        assert!(!provider.is_available());
        let result = provider.fetch("compute", serde_json::json!({})).await;
        assert!(matches!(result, Err(ProviderError::Unavailable(_))));
    }

    #[test]
    fn capability_registration_serde() {
        let reg = CapabilityRegistration {
            capability: "ecology".into(),
            primal: "airspring".into(),
            socket: "/run/user/1000/biomeos/airspring-abc.sock".into(),
            source: "startup".into(),
            semantic_mappings: std::iter::once(("et0".into(), "science.et0_fao56".into()))
                .collect(),
        };
        let json = serde_json::to_string(&reg).unwrap();
        let parsed: CapabilityRegistration = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.capability, "ecology");
        assert_eq!(parsed.semantic_mappings["et0"], "science.et0_fao56");
    }

    #[test]
    fn provider_error_display() {
        let e = ProviderError::Unavailable("test".into());
        assert!(e.to_string().contains("unavailable"));
        let e = ProviderError::Unsupported("op".into());
        assert!(e.to_string().contains("unsupported"));
        let e = ProviderError::Application {
            code: -32600,
            message: "invalid".into(),
        };
        assert!(e.to_string().contains("-32600"));
    }

    #[tokio::test]
    async fn provenance_begin_session() {
        let mock = MockProvider::new(true, serde_json::json!({"session_id": "sess-001"}));
        let result = provenance::begin_experiment_session(&mock, "exp-001", Some("did:key:abc"))
            .await
            .unwrap();
        assert_eq!(result["session_id"], "sess-001");
    }

    #[tokio::test]
    async fn provenance_record_step() {
        let mock = MockProvider::new(true, serde_json::json!({"recorded": true}));
        let result = provenance::record_step(
            &mock,
            "sess-001",
            "compute_et0",
            serde_json::json!({"et0": 4.2}),
        )
        .await
        .unwrap();
        assert_eq!(result["recorded"], true);
    }

    #[tokio::test]
    async fn provenance_complete_experiment() {
        let mock = MockProvider::new(true, serde_json::json!({"committed": true}));
        let result = provenance::complete_experiment(&mock, "sess-001")
            .await
            .unwrap();
        assert_eq!(result["committed"], true);
    }

    #[test]
    fn biomeos_provider_name() {
        let p = BiomeosProvider::new("ecology", "/tmp/test.sock");
        assert_eq!(p.name(), "biomeOS");
    }

    #[test]
    fn fallback_provider_chain() {
        let providers: Vec<Arc<dyn Provider>> = vec![
            Arc::new(MockProvider::new(false, serde_json::json!({}))),
            Arc::new(MockProvider::new(
                true,
                serde_json::json!({"source": "fallback"}),
            )),
        ];
        let available = providers.iter().find(|p| p.is_available());
        assert!(available.is_some());
        assert_eq!(available.unwrap().name(), "mock");
    }
}
