//! Tests for capability translation socket communication
//!
//! These tests reveal the actual socket communication issues

use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Test: Can we read from BearDog-style socket that doesn't close?
#[tokio::test]
async fn test_beardog_style_socket_communication() {
    let socket_path = "/tmp/test-beardog-style.sock";
    let _ = std::fs::remove_file(socket_path);
    
    // Simulate BearDog: Responds but keeps socket open
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        
        // Read request
        let mut buf = vec![0u8; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        println!("Server received: {}", request);
        
        // Send response WITHOUT closing socket (BearDog behavior)
        let response = r#"{"jsonrpc":"2.0","result":{"test":"value"},"id":1}"#;
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
        
        // KEEP SOCKET OPEN (this is what BearDog does)
        println!("Server sent response, keeping socket open...");
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
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
        stream.read_to_end(&mut response)
    ).await;
    
    match timeout_result {
        Ok(Ok(_)) => {
            println!("✅ Read completed in {:?}", start.elapsed());
            println!("Response: {}", String::from_utf8_lossy(&response));
        }
        Ok(Err(e)) => {
            println!("❌ Read error: {}", e);
        }
        Err(_) => {
            println!("❌ Timeout after {:?}", start.elapsed());
            if !response.is_empty() {
                println!("Partial response: {}", String::from_utf8_lossy(&response));
            }
        }
    }
    
    let _ = std::fs::remove_file(socket_path);
}

/// Test: Read with JSON detection
#[tokio::test]
async fn test_json_aware_reading() {
    let socket_path = "/tmp/test-json-aware.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 1024];
        let _ = socket.read(&mut buf).await;
        
        // Send response without newline
        let response = r#"{"jsonrpc":"2.0","result":{"algorithm":"X25519"},"id":1}"#;
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
        
        // Don't close, keep open
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut stream = UnixStream::connect(socket_path).await.unwrap();
    stream.write_all(b"{\"test\":\"request\"}\n").await.unwrap();
    stream.flush().await.unwrap();
    stream.shutdown().await.unwrap();
    
    // JSON-aware reading
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
                println!("Read error: {}", e);
                break;
            }
            Err(_) => {
                println!("Read timeout, checking buffer...");
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
    
    let _ = std::fs::remove_file(socket_path);
}

/// Test: What nc does that works
#[test]
fn test_nc_behavior() {
    use std::process::Command;
    
    println!("Testing how nc handles this...");
    
    // This works with 5 second timeout
    let output = Command::new("timeout")
        .args(&["5", "bash", "-c", 
            "echo '{\"jsonrpc\":\"2.0\",\"method\":\"crypto.x25519_generate_ephemeral\",\"params\":{},\"id\":1}' | nc -U /tmp/beardog-nat0.sock"])
        .output();
    
    match output {
        Ok(output) => {
            println!("Exit code: {}", output.status);
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => println!("Command error: {}", e),
    }
}

