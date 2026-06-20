// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::registry::CapabilityRegistry;
use super::types::{RegistryRequest, RegistryResponse, ResponseStatus};
use biomeos_types::{BiomeError, PrimalId};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

impl CapabilityRegistry {
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
