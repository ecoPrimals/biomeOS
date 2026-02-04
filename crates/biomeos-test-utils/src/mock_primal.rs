//! Mock Primal Server for Testing
//!
//! Provides a lightweight HTTP server that simulates a primal's API
//! for integration testing without requiring real primal binaries.

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Mock primal server that responds to standard primal API requests
#[derive(Clone)]
pub struct MockPrimal {
    name: String,
    addr: SocketAddr,
    capabilities: Vec<String>,
    state: Arc<RwLock<MockState>>,
    handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

#[derive(Default)]
struct MockState {
    health_check_count: u64,
    commands_received: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    name: String,
    capabilities: Vec<String>,
    checks_received: u64,
}

#[derive(Deserialize, Serialize)]
struct CommandRequest {
    command: String,
}

#[derive(Serialize)]
struct CommandResponse {
    status: String,
    message: String,
}

impl MockPrimal {
    /// Create a new mock primal builder
    pub fn builder(name: impl Into<String>) -> MockPrimalBuilder {
        MockPrimalBuilder::new(name)
    }

    /// Start the mock server
    pub async fn start(mut self) -> Result<Self> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/api/v1/command", post(command_handler))
            .route("/api/v1/capabilities", get(capabilities_handler))
            .with_state(self.clone());

        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .context("Failed to bind mock server")?;

        let actual_addr = listener.local_addr()?;

        // Update addr to reflect actual bound address (important for port 0)
        self.addr = actual_addr;

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        {
            let mut handle_lock = self.handle.write().await;
            *handle_lock = Some(handle);
        }

        tracing::info!("Mock primal '{}' started on {}", self.name, actual_addr);

        Ok(self)
    }

    /// Get the address the server is listening on
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get the base URL for the server
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Get the number of health checks received
    pub async fn health_check_count(&self) -> u64 {
        self.state.read().await.health_check_count
    }

    /// Get all commands received
    pub async fn commands_received(&self) -> Vec<String> {
        self.state.read().await.commands_received.clone()
    }

    /// Stop the mock server
    pub async fn stop(self) -> Result<()> {
        let mut handle = self.handle.write().await;
        if let Some(h) = handle.take() {
            h.abort();
            tracing::info!("Mock primal '{}' stopped", self.name);
        }
        Ok(())
    }
}

// Handlers

async fn health_handler(State(mock): State<MockPrimal>) -> impl IntoResponse {
    let mut state = mock.state.write().await;
    state.health_check_count += 1;

    Json(HealthResponse {
        status: "healthy".to_string(),
        name: mock.name.clone(),
        capabilities: mock.capabilities.clone(),
        checks_received: state.health_check_count,
    })
}

async fn command_handler(
    State(mock): State<MockPrimal>,
    Json(req): Json<CommandRequest>,
) -> impl IntoResponse {
    let mut state = mock.state.write().await;
    state.commands_received.push(req.command.clone());

    (
        StatusCode::OK,
        Json(CommandResponse {
            status: "success".to_string(),
            message: format!("Command '{}' executed", req.command),
        }),
    )
}

async fn capabilities_handler(State(mock): State<MockPrimal>) -> impl IntoResponse {
    Json(mock.capabilities.clone())
}

/// Builder for MockPrimal
pub struct MockPrimalBuilder {
    name: String,
    port: Option<u16>,
    capabilities: Vec<String>,
}

impl MockPrimalBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            port: None,
            capabilities: vec![],
        }
    }

    /// Set the port (0 for random available port)
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Add a capability
    pub fn capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add multiple capabilities
    pub fn capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }

    /// Build the MockPrimal (not started yet)
    pub fn build(self) -> MockPrimal {
        let port = self.port.unwrap_or(0);
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        MockPrimal {
            name: self.name,
            addr,
            capabilities: self.capabilities,
            state: Arc::new(RwLock::new(MockState::default())),
            handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Build and start the mock server
    pub async fn start(self) -> Result<MockPrimal> {
        self.build().start().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_primal_creation() {
        let mock = MockPrimal::builder("test-primal")
            .port(0)
            .capability("test")
            .build()
            .start()
            .await
            .expect("Should start");

        assert_eq!(mock.health_check_count().await, 0);
        mock.stop().await.ok();
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let mock = MockPrimal::builder("health-test")
            .port(0)
            .capability("health-test-cap")
            .build()
            .start()
            .await
            .expect("Should start");

        // Use hyper-util HTTP client (Pure Rust, no reqwest dependency)
        use http_body_util::{BodyExt, Empty};
        use hyper::body::Bytes;
        use hyper_util::client::legacy::Client;
        use hyper_util::rt::TokioExecutor;

        let client: Client<_, Empty<Bytes>> = Client::builder(TokioExecutor::new()).build_http();
        let uri = format!("{}/health", mock.url())
            .parse::<hyper::Uri>()
            .expect("Should parse URI");

        let response = client.get(uri).await.expect("Should send request");

        assert!(response.status().is_success());

        let body_bytes = response
            .into_body()
            .collect()
            .await
            .expect("Should read body")
            .to_bytes();
        let health: HealthResponse =
            serde_json::from_slice(&body_bytes).expect("Should parse JSON");

        assert_eq!(health.status, "healthy");
        assert_eq!(health.name, "health-test");
        assert_eq!(health.checks_received, 1);

        assert_eq!(mock.health_check_count().await, 1);
        mock.stop().await.ok();
    }

    #[tokio::test]
    async fn test_command_endpoint() {
        let mock = MockPrimal::builder("command-test")
            .port(0)
            .build()
            .start()
            .await
            .expect("Should start");

        // Use hyper-util HTTP client (Pure Rust, no reqwest dependency)
        use http_body_util::Full;
        use hyper::body::Bytes;
        use hyper_util::client::legacy::Client;
        use hyper_util::rt::TokioExecutor;

        let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build_http();
        let uri = format!("{}/api/v1/command", mock.url())
            .parse::<hyper::Uri>()
            .expect("Should parse URI");

        let json_body = serde_json::to_string(&CommandRequest {
            command: "test-command".to_string(),
        })
        .expect("Should serialize");

        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_body)))
            .expect("Should build request");

        let response = client.request(req).await.expect("Should send request");

        assert!(response.status().is_success());

        let commands = mock.commands_received().await;
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "test-command");

        mock.stop().await.ok();
    }
}
