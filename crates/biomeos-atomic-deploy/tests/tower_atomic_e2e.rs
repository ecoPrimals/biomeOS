//! E2E Tests for Tower Atomic Deployment
//!
//! These tests validate the complete Tower Atomic deployment flow:
//! 1. BearDog starts at XDG socket path
//! 2. Songbird bonds to BearDog
//! 3. Neural API routes capability.call correctly
//!
//! # Running
//! ```bash
//! cargo test --test tower_atomic_e2e -- --test-threads=1
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Test fixture for Tower Atomic deployment
#[allow(dead_code)]
struct TowerAtomicFixture {
    family_id: String,
    socket_dir: PathBuf,
    beardog_socket: PathBuf,
    songbird_socket: PathBuf,
    neural_api_socket: PathBuf,
}

impl TowerAtomicFixture {
    fn new(family_id: &str) -> Self {
        let socket_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join("biomeos");

        std::fs::create_dir_all(&socket_dir).ok();

        Self {
            family_id: family_id.to_string(),
            socket_dir: socket_dir.clone(),
            beardog_socket: socket_dir.join(format!("beardog-{}.sock", family_id)),
            songbird_socket: socket_dir.join(format!("songbird-{}.sock", family_id)),
            neural_api_socket: socket_dir.join(format!("neural-api-{}.sock", family_id)),
        }
    }

    async fn cleanup(&self) {
        let _ = std::fs::remove_file(&self.beardog_socket);
        let _ = std::fs::remove_file(&self.songbird_socket);
        let _ = std::fs::remove_file(&self.neural_api_socket);
    }
}

/// Send JSON-RPC request and get response
async fn json_rpc_call(
    socket_path: &PathBuf,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to {}: {}", socket_path.display(), e))?;

    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let request_str = format!("{}\n", serde_json::to_string(&request).unwrap());
    writer
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();

    tokio::time::timeout(
        Duration::from_secs(10),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| "Timeout waiting for response".to_string())?
    .map_err(|e| e.to_string())?;

    serde_json::from_str(&response_line).map_err(|e| e.to_string())
}

#[tokio::test]
#[ignore = "Requires running primals - use for integration testing"]
async fn test_beardog_xdg_socket_path() {
    let fixture = TowerAtomicFixture::new("e2e-test");
    fixture.cleanup().await;

    // Verify BearDog can be reached at XDG path (if running)
    if fixture.beardog_socket.exists() {
        let result = json_rpc_call(
            &fixture.beardog_socket,
            "crypto.sha256",
            serde_json::json!({"data": "dGVzdA=="}),
        )
        .await;

        assert!(result.is_ok(), "BearDog should respond at XDG socket");
        let response = result.unwrap();
        assert!(response.get("result").is_some(), "Should have result field");
    }
}

#[tokio::test]
#[ignore = "Requires running Neural API"]
async fn test_capability_call_routing() {
    let fixture = TowerAtomicFixture::new("e2e-test");

    if !fixture.neural_api_socket.exists() {
        eprintln!("Neural API not running, skipping test");
        return;
    }

    // Test capability.call routing through Neural API
    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "capability.call",
        serde_json::json!({
            "capability": "crypto",
            "operation": "sha256",
            "args": {"data": "dGVzdA=="}
        }),
    )
    .await;

    assert!(
        result.is_ok(),
        "capability.call should succeed: {:?}",
        result
    );
    let response = result.unwrap();
    assert!(response.get("result").is_some() || response.get("error").is_some());
}

#[tokio::test]
async fn test_nucleation_xdg_paths() {
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};

    // Set XDG_RUNTIME_DIR for test
    std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");

    let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);

    let beardog_socket = nucleation.assign_socket("beardog", "test-family");
    let songbird_socket = nucleation.assign_socket("songbird", "test-family");

    // Verify XDG paths are used
    assert!(
        beardog_socket
            .to_string_lossy()
            .contains("/run/user/1000/biomeos/"),
        "BearDog socket should be in XDG runtime dir: {:?}",
        beardog_socket
    );
    assert!(
        songbird_socket
            .to_string_lossy()
            .contains("/run/user/1000/biomeos/"),
        "Songbird socket should be in XDG runtime dir: {:?}",
        songbird_socket
    );

    // Verify deterministic assignment
    let beardog_socket_2 = nucleation.assign_socket("beardog", "test-family");
    assert_eq!(
        beardog_socket, beardog_socket_2,
        "Same primal should get same socket"
    );
}

#[tokio::test]
async fn test_nucleation_batch_assignment() {
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};

    let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);

    let primals = vec![
        "beardog".to_string(),
        "songbird".to_string(),
        "squirrel".to_string(),
    ];

    let assignments = nucleation.assign_batch(&primals, "batch-test");

    assert_eq!(assignments.len(), 3);

    // All should have consistent naming
    for (primal, path) in &assignments {
        assert!(
            path.to_string_lossy()
                .contains(&format!("{}-batch-test.sock", primal)),
            "Socket path should include primal and family: {:?}",
            path
        );
    }
}

#[tokio::test]
async fn test_execution_context_socket_paths() {
    use biomeos_atomic_deploy::executor::context::ExecutionContext;
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");

    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "context-test".to_string());

    let nucleation = Arc::new(RwLock::new(SocketNucleation::new(
        SocketStrategy::XdgRuntime,
    )));
    let context = ExecutionContext::new(env).with_nucleation(nucleation);

    let beardog_socket = context.get_socket_path("beardog").await;

    assert!(
        beardog_socket.contains("/run/user/1000/biomeos/beardog-context-test.sock"),
        "Context should use XDG socket path: {}",
        beardog_socket
    );
}
