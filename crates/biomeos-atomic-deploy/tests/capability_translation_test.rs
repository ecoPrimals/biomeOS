// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for capability translation socket communication
//!
//! **Concurrency-First Design**: Tests use proper synchronization (oneshot channels)
//! instead of arbitrary sleep() calls. Test issues will be production issues!
//!
//! These tests reveal and validate socket communication patterns

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::oneshot;

/// Cleanup helper for socket paths
struct SocketCleanup(String);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Test: Can we read from BearDog-style socket that doesn't close?
#[tokio::test]
async fn test_beardog_style_socket_communication() {
    let socket_path = "/tmp/test-beardog-style.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());
    let _ = std::fs::remove_file(socket_path);

    let (ready_tx, ready_rx) = oneshot::channel();

    // Simulate BearDog: Responds but keeps socket open
    let listener = UnixListener::bind(socket_path).unwrap();

    tokio::spawn(async move {
        // Signal ready AFTER bind succeeds
        let _ = ready_tx.send(());

        let (mut socket, _) = listener.accept().await.unwrap();

        // Read request
        let mut buf = vec![0u8; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        println!("Server received: {request}");

        // Send response WITHOUT closing socket (BearDog behavior)
        let response = r#"{"jsonrpc":"2.0","result":{"test":"value"},"id":1}"#;
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();

        // KEEP SOCKET OPEN (this is what BearDog does) — hang without wall-clock delay
        println!("Server sent response, keeping socket open for client to read...");
        std::future::pending::<()>().await;
    });

    // Wait for server ready (no arbitrary sleep!)
    ready_rx.await.expect("Server failed to start");

    // Client: Try to read response
    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    // Send request
    stream.write_all(b"{\"test\":\"request\"}\n").await.unwrap();
    stream.flush().await.unwrap();

    // Try shutdown
    println!("Client: Shutting down write half...");
    stream.shutdown().await.unwrap();

    // Try to read
    println!("Client: Reading response...");
    let start = std::time::Instant::now();
    let mut response = Vec::new();
    let timeout_result = tokio::time::timeout(
        tokio::time::Duration::from_millis(500),
        stream.read_to_end(&mut response),
    )
    .await;

    match timeout_result {
        Ok(Ok(_)) => {
            println!("✅ Read completed in {:?}", start.elapsed());
            println!("Response: {}", String::from_utf8_lossy(&response));
        }
        Ok(Err(e)) => {
            println!("❌ Read error: {e}");
        }
        Err(_) => {
            println!("❌ Timeout after {:?}", start.elapsed());
            if !response.is_empty() {
                println!("Partial response: {}", String::from_utf8_lossy(&response));
            }
        }
    }
}

/// Test: Read with JSON detection
#[tokio::test]
async fn test_json_aware_reading() {
    let socket_path = "/tmp/test-json-aware.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());
    let _ = std::fs::remove_file(socket_path);

    let (ready_tx, ready_rx) = oneshot::channel();

    let listener = UnixListener::bind(socket_path).unwrap();

    tokio::spawn(async move {
        // Signal ready AFTER bind succeeds
        let _ = ready_tx.send(());

        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 1024];
        let _ = socket.read(&mut buf).await;

        // Send response without newline
        let response = r#"{"jsonrpc":"2.0","result":{"algorithm":"X25519"},"id":1}"#;
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();

        // Keep connection open for client to read — no arbitrary wall-clock wait
        std::future::pending::<()>().await;
    });

    // Wait for server ready (no arbitrary sleep!)
    ready_rx.await.expect("Server failed to start");

    let mut stream = UnixStream::connect(socket_path).await.unwrap();
    stream.write_all(b"{\"test\":\"request\"}\n").await.unwrap();
    stream.flush().await.unwrap();
    stream.shutdown().await.unwrap();

    // JSON-aware reading with proper timeouts
    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 4096];
    let read_timeout = tokio::time::Duration::from_millis(100);
    let overall_timeout = tokio::time::Duration::from_secs(1);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > overall_timeout {
            println!("Overall timeout");
            break;
        }

        match tokio::time::timeout(read_timeout, stream.read(&mut temp_buf)).await {
            Ok(Ok(0)) => {
                println!("EOF received");
                break;
            }
            Ok(Ok(n)) => {
                buffer.extend_from_slice(&temp_buf[..n]);
                println!("Read {} bytes, total: {}", n, buffer.len());

                // Check for complete JSON
                if let Ok(s) = std::str::from_utf8(&buffer) {
                    if serde_json::from_str::<serde_json::Value>(s).is_ok() {
                        println!("✅ Complete JSON detected!");
                        break;
                    }
                }
            }
            Ok(Err(e)) => {
                println!("Read error: {e}");
                break;
            }
            Err(_) => {
                // Timeout on read, check if we have complete JSON already
                if !buffer.is_empty() {
                    if let Ok(s) = std::str::from_utf8(&buffer) {
                        if serde_json::from_str::<serde_json::Value>(s).is_ok() {
                            println!("✅ Complete JSON found after timeout!");
                            break;
                        }
                    }
                }
                // Continue trying to read more
            }
        }
    }

    println!("Total time: {:?}", start.elapsed());
    println!("Response: {}", String::from_utf8_lossy(&buffer));

    // Validate we got a proper response
    assert!(!buffer.is_empty(), "Should have received response");
    let response: serde_json::Value =
        serde_json::from_slice(&buffer).expect("Should be valid JSON");
    assert_eq!(response["result"]["algorithm"], "X25519");
}

/// Test: Concurrent socket connections
#[tokio::test]
async fn test_concurrent_socket_connections() {
    let socket_path = "/tmp/test-concurrent-sockets.sock";
    let _cleanup = SocketCleanup(socket_path.to_string());
    let _ = std::fs::remove_file(socket_path);

    let (ready_tx, ready_rx) = oneshot::channel();

    let listener = UnixListener::bind(socket_path).unwrap();

    // Server handles multiple connections
    tokio::spawn(async move {
        let _ = ready_tx.send(());

        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 1024];
                    if let Ok(n) = socket.read(&mut buf).await {
                        let request = String::from_utf8_lossy(&buf[..n]);
                        // Extract id from request for response
                        let id = serde_json::from_str::<serde_json::Value>(&request)
                            .ok()
                            .and_then(|v| v.get("id").cloned())
                            .unwrap_or_else(|| serde_json::json!(1));

                        let response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "result": {"success": true},
                            "id": id
                        });
                        let _ = socket.write_all(response.to_string().as_bytes()).await;
                        let _ = socket.flush().await;
                    }
                });
            }
        }
    });

    ready_rx.await.expect("Server failed to start");

    // Make 10 concurrent connections
    let mut handles = Vec::new();
    for i in 0..10 {
        let socket_path = socket_path.to_string();
        handles.push(tokio::spawn(async move {
            let mut stream = UnixStream::connect(&socket_path).await?;
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "test",
                "params": {},
                "id": i
            });
            stream.write_all(request.to_string().as_bytes()).await?;
            stream.flush().await?;
            stream.shutdown().await?;

            let mut response = vec![0u8; 1024];
            let n = tokio::time::timeout(
                tokio::time::Duration::from_millis(500),
                stream.read(&mut response),
            )
            .await??;

            let parsed: serde_json::Value = serde_json::from_slice(&response[..n])?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(parsed)
        }));
    }

    // All should succeed
    // Join all handles without futures crate
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await);
    }
    let mut successes = 0;
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(response)) => {
                assert_eq!(response["result"]["success"], true);
                successes += 1;
            }
            Ok(Err(e)) => {
                println!("Connection {i} failed: {e}");
            }
            Err(e) => {
                println!("Task {i} panicked: {e}");
            }
        }
    }

    assert!(
        successes >= 8,
        "At least 8/10 concurrent connections should succeed, got {successes}"
    );
}

/// Test: What nc does that works
/// NOTE: This test requires a running BearDog, skip if not available
#[test]
#[ignore = "Requires running BearDog instance — run with cargo test -- --ignored"]
fn test_nc_behavior() {
    use std::process::Command;

    println!("Testing how nc handles this...");

    // This works with 5 second timeout
    let output = Command::new("timeout")
        .args(["5", "bash", "-c", 
            "echo '{\"jsonrpc\":\"2.0\",\"method\":\"crypto.x25519_generate_ephemeral\",\"params\":{},\"id\":1}' | nc -U /tmp/beardog-family.sock"])
        .output();

    match output {
        Ok(output) => {
            println!("Exit code: {}", output.status);
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => println!("Command error: {e}"),
    }
}
