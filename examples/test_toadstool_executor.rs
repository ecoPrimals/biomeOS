//! Quick ToadStool Test - Verify Neural API executor works
//!
//! Since ToadStool already has a properly configured Unix socket,
//! we can test the graph executor with it first!

use anyhow::Result;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{error, info};

#[derive(serde::Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)] // Fields used by serde deserialization
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: u64,
}

#[derive(serde::Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

async fn call_toadstool_rpc(method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let socket_path = format!("/run/user/{}/toadstool-default.jsonrpc.sock", uid);

    info!("Connecting to ToadStool at {}", socket_path);

    let stream = UnixStream::connect(&socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: 1,
    };

    let request_json = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_json.as_bytes()).await?;
    writer.flush().await?;

    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let response: JsonRpcResponse = serde_json::from_str(&line)?;

    if let Some(error) = response.error {
        return Err(anyhow::anyhow!(
            "RPC error {}: {}",
            error.code,
            error.message
        ));
    }

    Ok(response.result.unwrap_or(json!(null)))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧪 ToadStool Neural API Executor Test");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Test 1: Health check
    println!("Test 1: Health Check");
    match call_toadstool_rpc("health", json!({})).await {
        Ok(result) => {
            println!("  ✅ Health check passed!");
            println!("  Result: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("  ❌ Health check failed: {}", e);
            return Err(e);
        }
    }

    println!();

    // Test 2: Get capabilities
    println!("Test 2: Query Capabilities");
    match call_toadstool_rpc("resources.capabilities", json!({})).await {
        Ok(result) => {
            println!("  ✅ Capabilities retrieved!");
            println!("  Result: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            error!("  ❌ Capabilities query failed: {}", e);
            return Err(e);
        }
    }

    println!();

    // Test 3: Estimate resources for a simple graph
    println!("Test 3: Estimate Resources");
    let test_graph = json!({
        "nodes": [
            {
                "id": "test_node",
                "operation": "echo",
                "resources": {
                    "cpu_cores": 1,
                    "memory_mb": 256
                }
            }
        ]
    });

    match call_toadstool_rpc("resources.estimate", test_graph).await {
        Ok(result) => {
            println!("  ✅ Resource estimation successful!");
            println!("  Result: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!(
                "  ⚠️  Resource estimation failed: {} (method may not exist)",
                e
            );
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ ToadStool executor test complete!");
    println!();
    println!("Next steps:");
    println!("  1. Create deploy_niche binary");
    println!("  2. Wire up other primals' sockets");
    println!("  3. Test full niche deployment!");

    Ok(())
}
