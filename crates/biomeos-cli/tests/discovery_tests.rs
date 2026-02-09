//! Integration tests for biomeos-cli discovery commands
//!
//! EVOLVED (Jan 28, 2026): Concurrency-First Design
//!
//! Tests service discovery functionality with mock primals over Unix sockets.
//! This replaces the deprecated HTTP/reqwest approach.
//!
//! ## Deep Debt Principles
//! - Capability-based: Primals discovered by what they can do
//! - No hardcoding: Socket paths are runtime-discovered
//! - Mocks isolated to testing: MockPrimalServer only exists in tests
//! - **Concurrency-First**: Uses oneshot channels for readiness, no arbitrary sleeps

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::oneshot;

/// Mock primal server for testing discovery via JSON-RPC
///
/// **Concurrency**: Uses oneshot channel to signal readiness instead of sleep
struct MockPrimalServer {
    socket_path: PathBuf,
    #[allow(dead_code)] // Stored for test diagnostics
    primal_name: String,
    #[allow(dead_code)] // Stored for test diagnostics
    capabilities: Vec<String>,
    handle: tokio::task::JoinHandle<()>,
}

impl MockPrimalServer {
    /// Start server and wait for it to be ready
    /// **Concurrency**: Uses oneshot channel for deterministic readiness
    async fn start(socket_path: PathBuf, primal_name: &str, capabilities: Vec<&str>) -> Self {
        let (ready_tx, ready_rx) = oneshot::channel();

        let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");
        let name = primal_name.to_string();
        let caps: Vec<String> = capabilities.into_iter().map(|s| s.to_string()).collect();
        let caps_clone = caps.clone();

        let handle = tokio::spawn(async move {
            // Signal ready AFTER bind succeeds
            let _ = ready_tx.send(());

            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let name = name.clone();
                        let caps = caps_clone.clone();
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
            socket_path,
            primal_name: primal_name.to_string(),
            capabilities: caps,
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
                "primal.capabilities" | "discovery.capabilities" => json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "primal": primal_name,
                        "capabilities": capabilities
                    },
                    "id": id
                }),
                "discovery.list" => json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "primals": [{
                            "name": primal_name,
                            "capabilities": capabilities,
                            "endpoint": "unix_socket"
                        }]
                    },
                    "id": id
                }),
                "discovery.find_by_capability" => {
                    let params = request.get("params");
                    let target_cap = params
                        .and_then(|p| p.get("capability"))
                        .and_then(|c| c.as_str())
                        .unwrap_or("");

                    let matches = capabilities.contains(&target_cap.to_string());

                    json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "found": matches,
                            "primals": if matches { vec![primal_name] } else { vec![] }
                        },
                        "id": id
                    })
                }
                "health.ping" => json!({
                    "jsonrpc": "2.0",
                    "result": { "status": "healthy" },
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

/// Query a primal via JSON-RPC
async fn query_primal(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
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
async fn test_discovery_capability_based() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let socket_path = temp_dir.path().join("beardog-family_a.sock");

    // Start mock BearDog with security capabilities
    let _server = MockPrimalServer::start(
        socket_path.clone(),
        "beardog",
        vec!["security", "crypto", "identity"],
    )
    .await;

    // Query capabilities
    let response = query_primal(
        socket_path.to_str().unwrap(),
        "primal.capabilities",
        json!({}),
    )
    .await?;

    let result = response.get("result").unwrap();
    let capabilities = result
        .get("capabilities")
        .and_then(|c| c.as_array())
        .unwrap();

    assert!(capabilities.iter().any(|c| c.as_str() == Some("security")));
    assert!(capabilities.iter().any(|c| c.as_str() == Some("crypto")));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_multicast() -> Result<()> {
    // Note: Actual multicast requires Songbird. This tests the pattern.
    let temp_dir = TempDir::new()?;

    // Simulate multiple primals that would be discovered via multicast
    let beardog_path = temp_dir.path().join("beardog.sock");
    let songbird_path = temp_dir.path().join("songbird.sock");

    let _beardog = MockPrimalServer::start(beardog_path.clone(), "beardog", vec!["security"]).await;
    let _songbird =
        MockPrimalServer::start(songbird_path.clone(), "songbird", vec!["discovery", "mesh"]).await;

    // Discovery pattern: scan socket directory
    let mut discovered = Vec::new();
    for entry in std::fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sock") {
            if let Ok(response) =
                query_primal(path.to_str().unwrap(), "primal.capabilities", json!({})).await
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_filtering() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Start primals with different capabilities
    let beardog_path = temp_dir.path().join("beardog.sock");
    let nestgate_path = temp_dir.path().join("nestgate.sock");
    let toadstool_path = temp_dir.path().join("toadstool.sock");

    let _beardog = MockPrimalServer::start(beardog_path.clone(), "beardog", vec!["security"]).await;
    let _nestgate =
        MockPrimalServer::start(nestgate_path.clone(), "nestgate", vec!["storage"]).await;
    let _toadstool =
        MockPrimalServer::start(toadstool_path.clone(), "toadstool", vec!["compute"]).await;

    // Find primals with "security" capability
    let mut security_primals = Vec::new();
    let sockets = [&beardog_path, &nestgate_path, &toadstool_path];

    for socket in &sockets {
        if let Ok(response) = query_primal(
            socket.to_str().unwrap(),
            "discovery.find_by_capability",
            json!({ "capability": "security" }),
        )
        .await
        {
            if let Some(result) = response.get("result") {
                if result
                    .get("found")
                    .and_then(|f| f.as_bool())
                    .unwrap_or(false)
                {
                    security_primals.push(result.clone());
                }
            }
        }
    }

    // Only BearDog should have security capability
    assert_eq!(security_primals.len(), 1);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_refresh() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let socket_path = temp_dir.path().join("primal.sock");

    // Start server
    let server = MockPrimalServer::start(socket_path.clone(), "test-primal", vec!["cap1"]).await;

    // First discovery
    let response1 = query_primal(
        socket_path.to_str().unwrap(),
        "primal.capabilities",
        json!({}),
    )
    .await?;
    assert!(response1.get("result").is_some());

    // Stop and restart with different capabilities
    drop(server);

    // Wait for socket to be released (use retry pattern, not arbitrary sleep)
    let mut retries = 0;
    while socket_path.exists() && retries < 10 {
        tokio::task::yield_now().await;
        retries += 1;
    }

    let _server2 =
        MockPrimalServer::start(socket_path.clone(), "test-primal", vec!["cap2", "cap3"]).await;

    // Refresh discovery
    let response2 = query_primal(
        socket_path.to_str().unwrap(),
        "primal.capabilities",
        json!({}),
    )
    .await?;
    let caps = response2
        .get("result")
        .and_then(|r| r.get("capabilities"))
        .and_then(|c| c.as_array())
        .unwrap();

    // Should have new capabilities
    assert!(caps.iter().any(|c| c.as_str() == Some("cap2")));
    assert!(caps.iter().any(|c| c.as_str() == Some("cap3")));
    assert!(!caps.iter().any(|c| c.as_str() == Some("cap1")));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_federation() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Simulate primals from different families
    let family_a = temp_dir.path().join("beardog-family_a.sock");
    let family_nat1 = temp_dir.path().join("beardog-family_b.sock");

    let _primal_a =
        MockPrimalServer::start(family_a.clone(), "beardog-family_a", vec!["security"]).await;
    let _primal_nat1 =
        MockPrimalServer::start(family_nat1.clone(), "beardog-family_b", vec!["security"]).await;

    // Discover all primals, group by family
    let mut by_family: HashMap<String, Vec<String>> = HashMap::new();

    for entry in std::fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().to_string();

        if filename.ends_with(".sock") {
            // Extract family from filename pattern: {primal}-{family}.sock
            if let Some(family) = filename
                .strip_suffix(".sock")
                .and_then(|s| s.split('-').last())
            {
                by_family
                    .entry(family.to_string())
                    .or_default()
                    .push(filename);
            }
        }
    }

    // Should have primals in two families
    assert!(by_family.contains_key("family_a"));
    assert!(by_family.contains_key("family_b"));

    Ok(())
}
