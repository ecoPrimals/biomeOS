//! Tests for TRUE PRIMAL discovery implementations
//!
//! These tests verify that we query primals for their self-knowledge
//! rather than making hardcoded assumptions.
//!
//! **Concurrency-First Design**: All tests use proper synchronization instead of sleep()
//! - Uses oneshot channels for server readiness signals
//! - Fully concurrent and deterministic
//! - No artificial delays or race conditions

use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

/// Mock primal server for testing
///
/// **Returns**: oneshot receiver that signals when server is ready
async fn start_mock_primal(
    socket_path: PathBuf,
    identity: &str,
    capabilities: Vec<&str>,
) -> oneshot::Receiver<()> {
    let (ready_tx, ready_rx) = oneshot::channel();
    let listener = UnixListener::bind(&socket_path).unwrap();
    let identity = identity.to_string();
    let capabilities: Vec<String> = capabilities.iter().map(|s| s.to_string()).collect();

    tokio::spawn(async move {
        // Signal ready immediately after bind
        let _ = ready_tx.send(());
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let identity = identity.clone();
                let capabilities = capabilities.clone();

                tokio::spawn(async move {
                    let (read, mut write) = stream.into_split();
                    let mut reader = BufReader::new(read);
                    let mut request_line = String::new();

                    if reader.read_line(&mut request_line).await.is_ok() {
                        if let Ok(req) = serde_json::from_str::<serde_json::Value>(&request_line) {
                            let method = req["method"].as_str().unwrap_or("");
                            let id = req["id"].clone();

                            let response = match method {
                                "identity.get" => serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "result": {
                                        "name": identity,
                                        "version": "1.0.0",
                                        "type": "primal"
                                    },
                                    "id": id
                                }),
                                "capabilities.list" => serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "result": {
                                        "capabilities": capabilities
                                    },
                                    "id": id
                                }),
                                "health.check" => serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "result": {
                                        "status": "healthy",
                                        "health": 1.0,
                                        "load": 0.2
                                    },
                                    "id": id
                                }),
                                _ => serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "error": {
                                        "code": -32601,
                                        "message": "Method not found"
                                    },
                                    "id": id
                                }),
                            };

                            let response_str = serde_json::to_string(&response).unwrap() + "\n";
                            let _ = write.write_all(response_str.as_bytes()).await;
                        }
                    }
                });
            }
        }
    });

    ready_rx
}

/// Test querying primal identity (TRUE PRIMAL principle)
///
/// **Concurrency**: Uses oneshot channel for server readiness (no sleep!)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_primal_identity_beardog() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-beardog.sock");

    // Start mock BearDog primal
    let ready = start_mock_primal(socket_path.clone(), "beardog", vec!["security", "crypto"]).await;

    // Wait for server to be ready (deterministic!)
    ready.await.unwrap();

    // Query identity via JSON-RPC
    let identity = query_identity(&socket_path).await;

    assert_eq!(identity, "beardog", "Should get actual primal identity");
}

/// Test querying primal identity for different primal type
///
/// **Concurrency**: Uses oneshot channel for server readiness (no sleep!)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_primal_identity_songbird() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-songbird.sock");

    // Start mock Songbird primal
    let ready = start_mock_primal(socket_path.clone(), "songbird", vec!["discovery", "mesh"]).await;
    ready.await.unwrap();

    let identity = query_identity(&socket_path).await;

    assert_eq!(identity, "songbird", "Should get actual primal identity");
}

/// Test querying primal capabilities (capability-based discovery)
///
/// **Concurrency**: Uses oneshot channel for server readiness (no sleep!)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_primal_capabilities() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-primal-caps.sock");

    // Start mock primal with specific capabilities
    let ready = start_mock_primal(
        socket_path.clone(),
        "test-primal",
        vec!["security", "crypto", "storage"],
    )
    .await;
    ready.await.unwrap();

    let capabilities = query_capabilities(&socket_path).await;

    assert_eq!(capabilities.len(), 3, "Should get all capabilities");
    assert!(capabilities.contains(&"security".to_string()));
    assert!(capabilities.contains(&"crypto".to_string()));
    assert!(capabilities.contains(&"storage".to_string()));
}

/// Test querying primal health (TRUE PRIMAL health probe)
///
/// **Concurrency**: Uses oneshot channel for server readiness (no sleep!)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_primal_health() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-primal-health.sock");

    let ready = start_mock_primal(socket_path.clone(), "healthy-primal", vec!["test"]).await;
    ready.await.unwrap();

    let (health, load, status) = query_health(&socket_path).await;

    assert_eq!(health, 1.0, "Should get health metric");
    assert_eq!(load, 0.2, "Should get load metric");
    assert_eq!(status, "healthy", "Should get status");
}

/// Test identity query with nonexistent socket
#[tokio::test]
async fn test_query_identity_no_socket() {
    let socket_path = PathBuf::from("/tmp/nonexistent-primal-12345.sock");

    let identity = query_identity(&socket_path).await;

    assert_eq!(
        identity, "unknown",
        "Should return 'unknown' for unreachable primal"
    );
}

/// Test capabilities query with nonexistent socket
#[tokio::test]
async fn test_query_capabilities_no_socket() {
    let socket_path = PathBuf::from("/tmp/nonexistent-primal-caps-12345.sock");

    let capabilities = query_capabilities(&socket_path).await;

    assert!(
        capabilities.is_empty(),
        "Should return empty capabilities for unreachable primal"
    );
}

/// Test that different primals return different capabilities
/// (demonstrating agnostic architecture - no hardcoding)
///
/// **Concurrency**: Waits for all 3 servers to be ready concurrently (no sleep!)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_agnostic_capability_discovery() {
    let temp_dir = TempDir::new().unwrap();

    // Start multiple primals with different capabilities
    let beardog_path = temp_dir.path().join("beardog.sock");
    let songbird_path = temp_dir.path().join("songbird.sock");
    let custom_path = temp_dir.path().join("custom-primal.sock");

    let ready1 =
        start_mock_primal(beardog_path.clone(), "beardog", vec!["security", "crypto"]).await;
    let ready2 =
        start_mock_primal(songbird_path.clone(), "songbird", vec!["discovery", "mesh"]).await;
    let ready3 = start_mock_primal(
        custom_path.clone(),
        "my-custom-primal",
        vec!["ai", "ml", "inference"],
    )
    .await;

    // Wait for all servers to be ready (concurrent!)
    let _ = tokio::join!(ready1, ready2, ready3);

    // Query each primal - should get different capabilities
    let beardog_caps = query_capabilities(&beardog_path).await;
    let songbird_caps = query_capabilities(&songbird_path).await;
    let custom_caps = query_capabilities(&custom_path).await;

    // Verify each has its own capabilities (no hardcoding!)
    assert!(beardog_caps.contains(&"security".to_string()));
    assert!(songbird_caps.contains(&"discovery".to_string()));
    assert!(custom_caps.contains(&"ai".to_string()));

    // Verify they're different (agnostic architecture)
    assert_ne!(beardog_caps, songbird_caps);
    assert_ne!(beardog_caps, custom_caps);
    assert_ne!(songbird_caps, custom_caps);
}

// Helper functions matching our implementation
async fn query_identity(socket_path: &PathBuf) -> String {
    use tokio::net::UnixStream;

    let stream = match UnixStream::connect(socket_path).await {
        Ok(s) => s,
        Err(_) => return "unknown".to_string(),
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "identity.get",
        "params": {},
        "id": 1
    });

    let request_str = serde_json::to_string(&request).unwrap() + "\n";
    let (read, mut write) = stream.into_split();

    if write.write_all(request_str.as_bytes()).await.is_err() {
        return "unknown".to_string();
    }

    let mut reader = BufReader::new(read);
    let mut response_line = String::new();

    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                if let Some(name) = response["result"]["name"].as_str() {
                    return name.to_string();
                }
            }
            "unknown".to_string()
        }
        _ => "unknown".to_string(),
    }
}

async fn query_capabilities(socket_path: &PathBuf) -> Vec<String> {
    use tokio::net::UnixStream;

    let stream = match UnixStream::connect(socket_path).await {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capabilities.list",
        "params": {},
        "id": 1
    });

    let request_str = serde_json::to_string(&request).unwrap() + "\n";
    let (read, mut write) = stream.into_split();

    if write.write_all(request_str.as_bytes()).await.is_err() {
        return vec![];
    }

    let mut reader = BufReader::new(read);
    let mut response_line = String::new();

    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                if let Some(caps) = response["result"]["capabilities"].as_array() {
                    return caps
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
            vec![]
        }
        _ => vec![],
    }
}

async fn query_health(socket_path: &PathBuf) -> (f64, f64, String) {
    use tokio::net::UnixStream;

    let stream = match UnixStream::connect(socket_path).await {
        Ok(s) => s,
        Err(_) => return (0.0, 1.0, "offline".to_string()),
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });

    let request_str = serde_json::to_string(&request).unwrap() + "\n";
    let (read, mut write) = stream.into_split();

    if write.write_all(request_str.as_bytes()).await.is_err() {
        return (0.0, 1.0, "degraded".to_string());
    }

    let mut reader = BufReader::new(read);
    let mut response_line = String::new();

    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                let health = response["result"]["health"].as_f64().unwrap_or(1.0);
                let load = response["result"]["load"].as_f64().unwrap_or(0.0);
                let status = response["result"]["status"]
                    .as_str()
                    .unwrap_or("healthy")
                    .to_string();
                return (health, load, status);
            }
            (0.8, 0.2, "degraded".to_string())
        }
        _ => (0.0, 1.0, "degraded".to_string()),
    }
}
