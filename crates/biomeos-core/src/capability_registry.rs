// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! # biomeOS Capability Registry
//!
//! Central registry for primal capabilities. Enables O(N) scaling by providing
//! a single lookup point for "who provides what?" queries.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │    biomeOS Capability Registry          │
//! ├─────────────────────────────────────────┤
//! │                                         │
//! │  ┌──────────────────────────────────┐  │
//! │  │   Registry Core                   │  │
//! │  │  • Primal registration            │  │
//! │  │  • Capability lookup              │  │
//! │  │  • Health tracking                │  │
//! │  └──────────────────────────────────┘  │
//! │                                         │
//! │  ┌──────────────────────────────────┐  │
//! │  │   Unix Socket IPC Server          │  │
//! │  │  • XDG runtime dir/registry.sock  │ │
//! │  │  • JSON-RPC protocol              │  │
//! │  │  • Async connection handling      │  │
//! │  └──────────────────────────────────┘  │
//! │                                         │
//! └─────────────────────────────────────────┘
//! ```ignore
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::capability_registry::CapabilityRegistry;
//! use biomeos_core::family_discovery::get_family_id;
//! use biomeos_types::Capability;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create registry with dynamic family discovery
//!     let family_id = get_family_id(); // Discovers from .family.seed or env
//!     let registry = CapabilityRegistry::new(family_id);
//!     
//!     // Start Unix socket server
//!     registry.serve().await?;
//!     
//!     Ok(())
//! }
//! ```ignore

use crate::Capability;
use biomeos_types::paths::SystemPaths;
use biomeos_types::{BiomeError, PrimalId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Information about a registered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal ID
    pub id: PrimalId,

    /// Capabilities this primal provides
    pub provides: Vec<Capability>,

    /// Capabilities this primal requires
    pub requires: Vec<Capability>,

    /// Unix socket path for IPC
    pub socket_path: Option<String>,

    /// HTTP endpoint (if any)
    pub http_endpoint: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,

    /// Last heartbeat timestamp
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Registry request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum RegistryRequest {
    /// Register a primal
    Register {
        /// Primal identifier
        id: String,
        /// Correlation request identifier
        request_id: String,
        /// Registration parameters
        params: RegisterParams,
    },

    /// Query for capability provider
    GetProvider {
        /// Correlation request identifier
        request_id: String,
        /// Capability to look up
        capability: Capability,
    },

    /// List all registered primals
    ListPrimals {
        /// Correlation request identifier
        request_id: String,
    },

    /// Heartbeat
    Heartbeat {
        /// Correlation request identifier
        request_id: String,
        /// Primal sending the heartbeat
        primal_id: String,
    },

    /// Unregister a primal
    Unregister {
        /// Correlation request identifier
        request_id: String,
        /// Primal to unregister
        primal_id: String,
    },
}

/// Registration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    /// Capabilities this primal provides
    pub provides: Vec<Capability>,
    /// Capabilities this primal requires from others
    pub requires: Vec<Capability>,
    /// Unix socket path for JSON-RPC
    pub socket_path: Option<String>,
    /// HTTP endpoint URL (temporary bridge)
    pub http_endpoint: Option<String>,
    /// Arbitrary key-value metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Registry response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryResponse {
    /// Correlation request identifier
    pub request_id: String,
    /// Response status
    pub status: ResponseStatus,
    /// Payload (if any)
    pub data: Option<serde_json::Value>,
    /// Error message (if any)
    pub error: Option<String>,
}

/// Response status codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Operation succeeded
    Success,
    /// Operation failed
    Error,
    /// Requested resource not found
    NotFound,
}

/// biomeOS Capability Registry
///
/// Central registry for primal capabilities. Maintains a mapping of
/// capabilities to providers, enabling O(N) capability resolution.
pub struct CapabilityRegistry {
    /// Family ID
    family_id: String,

    /// Registered primals (`PrimalId` -> `PrimalInfo`)
    primals: Arc<RwLock<HashMap<PrimalId, PrimalInfo>>>,

    /// Capability index (`Capability` -> `Vec<PrimalId>`)
    capability_index: Arc<RwLock<HashMap<Capability, Vec<PrimalId>>>>,

    /// Unix socket path
    socket_path: PathBuf,
}

impl CapabilityRegistry {
    /// Create a new capability registry with XDG-resolved socket path.
    pub fn new(family_id: String) -> Self {
        let paths = SystemPaths::new_lazy();
        let socket_path = paths
            .runtime_dir()
            .join(format!("biomeos-registry-{family_id}.sock"));
        Self::with_socket_path(family_id, socket_path)
    }

    /// Create a registry with an explicit socket path (useful for tests).
    pub fn with_socket_path(family_id: String, socket_path: PathBuf) -> Self {
        info!("🔧 Creating biomeOS capability registry");
        info!("   Family: {}", family_id);
        info!("   Socket: {:?}", socket_path);

        Self {
            family_id,
            primals: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
            socket_path,
        }
    }

    /// Register a primal
    pub async fn register(&self, id: PrimalId, params: RegisterParams) -> Result<(), BiomeError> {
        info!("📝 Registering primal: {:?}", id);
        debug!("   Provides: {:?}", params.provides);
        debug!("   Requires: {:?}", params.requires);

        let now = chrono::Utc::now();

        let info = PrimalInfo {
            id: id.clone(),
            provides: params.provides.clone(),
            requires: params.requires.clone(),
            socket_path: params.socket_path,
            http_endpoint: params.http_endpoint,
            metadata: params.metadata.unwrap_or_default(),
            registered_at: now,
            last_heartbeat: now,
        };

        // Add to primals map
        {
            let mut primals = self.primals.write().await;
            primals.insert(id.clone(), info);
        }

        // Update capability index
        {
            let mut index = self.capability_index.write().await;
            for capability in params.provides {
                index.entry(capability).or_default().push(id.clone());
            }
        }

        info!("✅ Primal registered: {:?}", id);

        Ok(())
    }

    /// Get provider for a capability
    pub async fn get_provider(
        &self,
        capability: &Capability,
    ) -> Result<Option<PrimalInfo>, BiomeError> {
        debug!("🔍 Looking for provider of: {:?}", capability);

        let index = self.capability_index.read().await;

        if let Some(providers) = index.get(capability)
            && let Some(primal_id) = providers.first()
        {
            let primals = self.primals.read().await;
            if let Some(info) = primals.get(primal_id) {
                info!("✅ Found provider: {:?} for {:?}", primal_id, capability);
                return Ok(Some(info.clone()));
            }
        }

        warn!("❌ No provider found for: {:?}", capability);
        Ok(None)
    }

    /// List all registered primals
    pub async fn list_primals(&self) -> Vec<PrimalInfo> {
        let primals = self.primals.read().await;
        primals.values().cloned().collect()
    }

    /// Update heartbeat for a primal
    pub async fn heartbeat(&self, primal_id: &PrimalId) -> Result<(), BiomeError> {
        let mut primals = self.primals.write().await;

        if let Some(info) = primals.get_mut(primal_id) {
            info.last_heartbeat = chrono::Utc::now();
            debug!("💓 Heartbeat received from: {:?}", primal_id);
            Ok(())
        } else {
            Err(BiomeError::resource_error(
                format!("Primal not found: {primal_id:?}"),
                "registry",
                None::<String>,
                None::<String>,
            ))
        }
    }

    /// Unregister a primal
    pub async fn unregister(&self, primal_id: &PrimalId) -> Result<(), BiomeError> {
        info!("🗑️  Unregistering primal: {:?}", primal_id);

        // Remove from primals map
        let info = {
            let mut primals = self.primals.write().await;
            primals.remove(primal_id)
        };

        if let Some(info) = info {
            // Remove from capability index
            let mut index = self.capability_index.write().await;
            for capability in &info.provides {
                if let Some(providers) = index.get_mut(capability) {
                    providers.retain(|id| id != primal_id);
                    if providers.is_empty() {
                        index.remove(capability);
                    }
                }
            }

            info!("✅ Primal unregistered: {:?}", primal_id);
            Ok(())
        } else {
            Err(BiomeError::resource_error(
                format!("Primal not found: {primal_id:?}"),
                "registry",
                None::<String>,
                None::<String>,
            ))
        }
    }

    /// Start Unix socket IPC server
    pub async fn serve(&self) -> Result<(), BiomeError> {
        self.serve_inner(None).await
    }

    /// Start Unix socket IPC server with readiness signal (for tests).
    #[cfg(test)]
    pub async fn serve_with_ready(
        &self,
        mut ready_tx: biomeos_test_utils::ReadySender,
    ) -> Result<(), BiomeError> {
        self.serve_inner(Some(Box::new(move || ready_tx.signal())))
            .await
    }

    async fn serve_inner(
        &self,
        on_ready: Option<Box<dyn FnOnce() + Send>>,
    ) -> Result<(), BiomeError> {
        // Ensure socket parent directory exists (defense-in-depth for
        // concurrent test setups where the dir may not yet be created).
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                BiomeError::resource_error(
                    format!("Failed to create socket directory: {e}"),
                    "registry_socket",
                    None::<String>,
                    None::<String>,
                )
            })?;
        }

        // Remove existing socket if present
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).map_err(|e| {
                BiomeError::resource_error(
                    format!("Failed to remove existing socket: {e}"),
                    "registry_socket",
                    None::<String>,
                    None::<String>,
                )
            })?;
        }

        // Create Unix listener
        let listener = UnixListener::bind(&self.socket_path).map_err(|e| {
            BiomeError::resource_error(
                format!(
                    "Failed to bind Unix socket at {}: {e}",
                    self.socket_path.display()
                ),
                "registry_socket",
                None::<String>,
                None::<String>,
            )
        })?;

        if let Some(f) = on_ready {
            f();
        }

        info!(
            "🔌 biomeOS capability registry listening on {:?}",
            self.socket_path
        );

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let registry = self.clone();

                    tokio::spawn(async move {
                        if let Err(e) = registry.handle_connection(stream).await {
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

    /// Handle a single connection
    async fn handle_connection(&self, stream: UnixStream) -> Result<(), BiomeError> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // Connection closed
                    break;
                }
                Ok(_) => {
                    // Parse request
                    let request: RegistryRequest = match serde_json::from_str(&line) {
                        Ok(req) => req,
                        Err(e) => {
                            error!("Failed to parse request: {}", e);
                            continue;
                        }
                    };

                    // Handle request
                    let response = self.handle_request(request).await;

                    // Send response
                    let response_json = serde_json::to_string(&response).map_err(|e| {
                        BiomeError::resource_error(
                            e.to_string(),
                            "registry",
                            None::<String>,
                            None::<String>,
                        )
                    })?;

                    writer
                        .write_all(response_json.as_bytes())
                        .await
                        .map_err(|e| {
                            BiomeError::resource_error(
                                e.to_string(),
                                "registry_socket",
                                None::<String>,
                                None::<String>,
                            )
                        })?;

                    writer.write_all(b"\n").await.map_err(|e| {
                        BiomeError::resource_error(
                            e.to_string(),
                            "registry_socket",
                            None::<String>,
                            None::<String>,
                        )
                    })?;
                }
                Err(e) => {
                    error!("Failed to read from stream: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a registry request
    #[expect(
        clippy::too_many_lines,
        reason = "single match arms enumerate all registry RPC variants"
    )]
    async fn handle_request(&self, request: RegistryRequest) -> RegistryResponse {
        match request {
            RegistryRequest::Register {
                id,
                request_id,
                params,
            } => match PrimalId::new(&id) {
                Ok(primal_id) => match self.register(primal_id, params).await {
                    Ok(()) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Success,
                        data: Some(serde_json::json!({
                            "message": "Primal registered successfully"
                        })),
                        error: None,
                    },
                    Err(e) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Error,
                        data: None,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => RegistryResponse {
                    request_id,
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(format!("Invalid primal ID: {e}")),
                },
            },

            RegistryRequest::GetProvider {
                request_id,
                capability,
            } => match self.get_provider(&capability).await {
                Ok(Some(info)) => match serde_json::to_value(info) {
                    Ok(data) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Success,
                        data: Some(data),
                        error: None,
                    },
                    Err(e) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Error,
                        data: None,
                        error: Some(format!("Failed to serialize provider info: {e}")),
                    },
                },
                Ok(None) => RegistryResponse {
                    request_id,
                    status: ResponseStatus::NotFound,
                    data: None,
                    error: Some(format!("No provider found for: {capability:?}")),
                },
                Err(e) => RegistryResponse {
                    request_id,
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(e.to_string()),
                },
            },

            RegistryRequest::ListPrimals { request_id } => {
                let primals = self.list_primals().await;
                match serde_json::to_value(primals) {
                    Ok(data) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Success,
                        data: Some(data),
                        error: None,
                    },
                    Err(e) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Error,
                        data: None,
                        error: Some(format!("Failed to serialize primals list: {e}")),
                    },
                }
            }

            RegistryRequest::Heartbeat {
                request_id,
                primal_id,
            } => match PrimalId::new(&primal_id) {
                Ok(id) => match self.heartbeat(&id).await {
                    Ok(()) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Success,
                        data: Some(serde_json::json!({
                            "message": "Heartbeat received"
                        })),
                        error: None,
                    },
                    Err(e) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Error,
                        data: None,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => RegistryResponse {
                    request_id,
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(format!("Invalid primal ID: {e}")),
                },
            },

            RegistryRequest::Unregister {
                request_id,
                primal_id,
            } => match PrimalId::new(&primal_id) {
                Ok(id) => match self.unregister(&id).await {
                    Ok(()) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Success,
                        data: Some(serde_json::json!({
                            "message": "Primal unregistered successfully"
                        })),
                        error: None,
                    },
                    Err(e) => RegistryResponse {
                        request_id,
                        status: ResponseStatus::Error,
                        data: None,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => RegistryResponse {
                    request_id,
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(format!("Invalid primal ID: {e}")),
                },
            },
        }
    }
}

impl Clone for CapabilityRegistry {
    fn clone(&self) -> Self {
        Self {
            family_id: self.family_id.clone(),
            primals: Arc::clone(&self.primals),
            capability_index: Arc::clone(&self.capability_index),
            socket_path: self.socket_path.clone(),
        }
    }
}

// Tests are in capability_registry_tests.rs to keep this file under 1000 lines
