// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! RootPulse mode — commit workflow orchestration
//!
//! Executes the `rootpulse_commit` graph via the Neural API server to
//! orchestrate the provenance trio: rhizoCrypt dehydrate -> BearDog sign
//! -> NestGate store -> LoamSpine commit -> sweetGrass attribute.

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, SystemPaths};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{error, info};

/// Execute the rootpulse commit workflow.
///
/// Connects to the Neural API server and sends a `graph.execute` request
/// for the `rootpulse_commit` graph with the provided session and agent.
pub async fn run(
    session_id: String,
    agent_did: String,
    socket: Option<PathBuf>,
    family_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    info!("RootPulse commit");
    info!("  session:  {}", session_id);
    info!("  agent:    {}", agent_did);

    let family = family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path = socket.unwrap_or_else(|| {
        SystemPaths::new_lazy().primal_socket(&format!("neural-api-{}", family))
    });

    info!("  family:   {}", family);
    info!("  socket:   {}", socket_path.display());

    if dry_run {
        info!("  [dry run] Would execute rootpulse_commit graph with:");
        info!("    SESSION_ID = {}", session_id);
        info!("    AGENT_DID  = {}", agent_did);
        info!("    FAMILY_ID  = {}", family);
        return Ok(());
    }

    let request = JsonRpcRequest::new(
        "graph.execute",
        serde_json::json!({
            "graph_id": "rootpulse_commit",
            "params": {
                "SESSION_ID": session_id,
                "AGENT_DID": agent_did,
                "FAMILY_ID": family,
            }
        }),
    );

    let response = send_jsonrpc(&socket_path, &request).await?;

    if let Some(result) = response.get("result") {
        info!("RootPulse commit succeeded");
        info!("  result: {}", serde_json::to_string_pretty(result)?);
    } else if let Some(err) = response.get("error") {
        error!("RootPulse commit failed");
        error!("  error: {}", err);
        anyhow::bail!("RootPulse commit failed: {}", err);
    }

    Ok(())
}

/// Send a JSON-RPC request over Unix socket and return the parsed response.
async fn send_jsonrpc(
    socket_path: &PathBuf,
    request: &JsonRpcRequest,
) -> Result<serde_json::Value> {
    let stream = UnixStream::connect(socket_path).await.with_context(|| {
        format!(
            "Failed to connect to Neural API at {}",
            socket_path.display()
        )
    })?;

    let (reader, mut writer) = stream.into_split();

    let payload = format!("{}\n", serde_json::to_string(&request)?);
    writer.write_all(payload.as_bytes()).await?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();

    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Timeout waiting for Neural API response"))?
    .context("Failed to read Neural API response")?;

    serde_json::from_str(&response_line).context("Failed to parse Neural API response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_dry_run() {
        let result = run(
            "test-session-123".to_string(),
            "did:key:z6MkTest".to_string(),
            None,
            Some("test-family".to_string()),
            true,
        )
        .await;
        result.expect("dry run should succeed");
    }

    #[tokio::test]
    async fn test_run_missing_socket() {
        let result = run(
            "test-session-123".to_string(),
            "did:key:z6MkTest".to_string(),
            Some(PathBuf::from("/tmp/nonexistent-neural-api.sock")),
            Some("test-family".to_string()),
            false,
        )
        .await;
        let err = result.expect_err("missing socket should fail");
        assert!(
            err.to_string().contains("connect") || err.to_string().contains("Failed"),
            "error should mention connection failure: {}",
            err
        );
    }

    #[test]
    fn test_request_format() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "graph.execute",
            "params": {
                "graph_id": "rootpulse_commit",
                "params": {
                    "SESSION_ID": "session-1",
                    "AGENT_DID": "did:key:z6MkTest",
                    "FAMILY_ID": "family-1",
                }
            },
            "id": 1
        });

        assert_eq!(request["method"], "graph.execute");
        assert_eq!(request["params"]["graph_id"], "rootpulse_commit");
        assert_eq!(request["params"]["params"]["SESSION_ID"], "session-1");
    }
}
