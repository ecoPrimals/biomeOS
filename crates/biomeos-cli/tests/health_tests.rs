//! Integration tests for biomeos-cli health commands
//!
//! EVOLVED (Jan 28, 2026): Concurrency-First Design
//!
//! Tests the health command handlers using mock primal servers over Unix sockets.
//! This replaces the deprecated HTTP/reqwest approach with Pure Rust JSON-RPC.
//!
//! ## Deep Debt Principles
//! - No C dependencies: Pure Rust Unix socket communication
//! - Mocks isolated to testing: These mock servers only exist in tests
//! - Capability-based: Mock servers respond to capability queries
//! - **Concurrency-First**: Uses oneshot channels for readiness, no arbitrary sleeps

use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::oneshot;

/// Mock primal server for testing JSON-RPC over Unix sockets
///
/// **Concurrency**: Uses oneshot channel to signal readiness instead of sleep
struct MockPrimalServer {
    socket_path: PathBuf,
    handle: tokio::task::JoinHandle<()>,
}

impl MockPrimalServer {
    /// Start a mock primal server
    /// **Concurrency**: Uses oneshot channel for deterministic readiness
    async fn start(socket_path: PathBuf, primal_name: &str, capabilities: Vec<&str>) -> Self {
        let (ready_tx, ready_rx) = oneshot::channel();
        
        let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");
        let name = primal_name.to_string();
        let caps: Vec<String> = capabilities.into_iter().map(|s| s.to_string()).collect();

        let socket_path_clone = socket_path.clone();
        let handle = tokio::spawn(async move {
            // Signal ready AFTER bind succeeds
            let _ = ready_tx.send(());
            
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let name = name.clone();
                        let caps = caps.clone();
                        tokio::spawn(async move {
                            let _ = Self::handle_connection(stream, &name, &caps).await;
                        });
                    }
                    Err(_) => break,
                }
            }
        });

        // Wait for server to be ready (deterministic, no sleep!)
        ready_rx.await.expect("Server failed to start");

        Self {
            socket_path: socket_path_clone,
            handle,
        }
    }

    async fn handle_connection(
        stream: UnixStream,
        primal_name: &str,
        capabilities: &[String],
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        while reader.read_line(&mut line).await? > 0 {
            let request: serde_json::Value = serde_json::from_str(line.trim())?;
            let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let id = request.get("id").and_then(|i| i.as_u64()).unwrap_or(0);

            let response = match method {
                "health.ping" => json!({
                    "jsonrpc": "2.0",
                    "result": { "status": "healthy", "primal": primal_name },
                    "id": id
                }),
                "health.check" | "health.detailed" => json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "healthy": true,
                        "primal": primal_name,
                        "capabilities": capabilities,
                        "uptime_seconds": 1234,
                        "version": "1.0.0"
                    },
                    "id": id
                }),
                "primal.capabilities" => json!({
                    "jsonrpc": "2.0",
                    "result": { "capabilities": capabilities },
                    "id": id
                }),
                _ => json!({
                    "jsonrpc": "2.0",
                    "error": { "code": -32601, "message": "Method not found" },
                    "id": id
                }),
            };

            let response_str = serde_json::to_string(&response)? + "\n";
            writer.write_all(response_str.as_bytes()).await?;
            writer.flush().await?;
            line.clear();
        }

        Ok(())
    }
}

impl Drop for MockPrimalServer {
    fn drop(&mut self) {
        self.handle.abort();
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Helper to query a primal via JSON-RPC
async fn query_primal(socket_path: &str, method: &str) -> Result<serde_json::Value> {
    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": {},
        "id": 1
    });
    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    Ok(serde_json::from_str(response_line.trim())?)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_command_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let socket_path = temp_dir.path().join("test-primal.sock");

    // Start mock primal server
    let _server = MockPrimalServer::start(socket_path.clone(), "test-primal", vec!["test"]).await;

    // Query health
    let response = query_primal(socket_path.to_str().unwrap(), "health.ping").await?;

    assert!(response.get("result").is_some());
    let result = response.get("result").unwrap();
    assert_eq!(
        result.get("status").and_then(|s| s.as_str()),
        Some("healthy")
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_command_detailed() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let socket_path = temp_dir.path().join("beardog-test.sock");

    // Start mock BearDog with security capabilities
    let _server =
        MockPrimalServer::start(socket_path.clone(), "beardog", vec!["security", "crypto"]).await;

    // Query detailed health
    let response = query_primal(socket_path.to_str().unwrap(), "health.detailed").await?;

    assert!(response.get("result").is_some());
    let result = response.get("result").unwrap();

    assert_eq!(result.get("healthy").and_then(|h| h.as_bool()), Some(true));
    assert_eq!(
        result.get("primal").and_then(|p| p.as_str()),
        Some("beardog")
    );

    let capabilities = result.get("capabilities").and_then(|c| c.as_array());
    assert!(capabilities.is_some());
    assert!(capabilities.unwrap().len() >= 2);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_command_multiple_services() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Start multiple mock primals
    let beardog_path = temp_dir.path().join("beardog.sock");
    let songbird_path = temp_dir.path().join("songbird.sock");
    let nestgate_path = temp_dir.path().join("nestgate.sock");

    let _beardog = MockPrimalServer::start(beardog_path.clone(), "beardog", vec!["security"]).await;
    let _songbird =
        MockPrimalServer::start(songbird_path.clone(), "songbird", vec!["discovery"]).await;
    let _nestgate =
        MockPrimalServer::start(nestgate_path.clone(), "nestgate", vec!["storage"]).await;

    // Query all three
    let results = tokio::join!(
        query_primal(beardog_path.to_str().unwrap(), "health.ping"),
        query_primal(songbird_path.to_str().unwrap(), "health.ping"),
        query_primal(nestgate_path.to_str().unwrap(), "health.ping"),
    );

    assert!(results.0.is_ok());
    assert!(results.1.is_ok());
    assert!(results.2.is_ok());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_probe_timeout() -> Result<()> {
    // Test connection to non-existent socket (should fail/timeout)
    let result = tokio::time::timeout(
        tokio::time::Duration::from_millis(100),
        UnixStream::connect("/tmp/nonexistent-primal-socket.sock"),
    )
    .await;

    // Should either timeout or fail to connect
    assert!(result.is_err() || result.unwrap().is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_scan_discovery() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a recognizable pattern of sockets
    let socket1 = temp_dir.path().join("primal1-nat0.sock");
    let socket2 = temp_dir.path().join("primal2-nat0.sock");

    let _server1 = MockPrimalServer::start(socket1.clone(), "primal1", vec!["cap1"]).await;
    let _server2 = MockPrimalServer::start(socket2.clone(), "primal2", vec!["cap2"]).await;

    // Scan directory for sockets
    let mut discovered = Vec::new();
    for entry in std::fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sock") {
            // Try to query capabilities
            if let Ok(response) = query_primal(path.to_str().unwrap(), "primal.capabilities").await
            {
                if let Some(result) = response.get("result") {
                    discovered.push(result.clone());
                }
            }
        }
    }

    assert_eq!(discovered.len(), 2);

    Ok(())
}
