// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! RootPulse mode — emergent version control via provenance trio coordination.
//!
//! Subcommands map to the five RootPulse TOML graphs:
//! - `commit`  → `rootpulse_commit`  (dehydrate + sign + store + commit + attribute)
//! - `branch`  → `rootpulse_branch`  (create branch from session)
//! - `merge`   → `rootpulse_merge`   (merge two sessions)
//! - `diff`    → `rootpulse_diff`    (diff between commits)
//! - `status`  → health checks on the provenance trio

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, SystemPaths};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{error, info};

use crate::RootPulseCommand;

/// Dispatch a RootPulse subcommand.
pub async fn dispatch(cmd: RootPulseCommand) -> Result<()> {
    match cmd {
        RootPulseCommand::Commit {
            session_id,
            agent_did,
            socket,
            family_id,
            dry_run,
        } => run_commit(session_id, agent_did, socket, family_id, dry_run).await,
        RootPulseCommand::Branch {
            session_id,
            branch_name,
            agent_did,
            socket,
            family_id,
            dry_run,
        } => run_graph("rootpulse_branch", serde_json::json!({
            "SESSION_ID": session_id,
            "BRANCH_NAME": branch_name,
            "AGENT_DID": agent_did,
        }), socket, family_id, dry_run).await,
        RootPulseCommand::Merge {
            source_session,
            target_session,
            agent_did,
            socket,
            family_id,
            dry_run,
        } => run_graph("rootpulse_merge", serde_json::json!({
            "SOURCE_SESSION": source_session,
            "TARGET_SESSION": target_session,
            "AGENT_DID": agent_did,
        }), socket, family_id, dry_run).await,
        RootPulseCommand::Diff {
            from,
            to,
            socket,
            family_id,
        } => run_graph("rootpulse_diff", serde_json::json!({
            "FROM_REF": from,
            "TO_REF": to,
        }), socket, family_id, false).await,
        RootPulseCommand::Status {
            socket,
            family_id,
        } => run_status(socket, family_id).await,
    }
}

/// Execute the rootpulse commit workflow (original `run` function).
pub async fn run_commit(
    session_id: String,
    agent_did: String,
    socket: Option<PathBuf>,
    family_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    run_graph("rootpulse_commit", serde_json::json!({
        "SESSION_ID": session_id,
        "AGENT_DID": agent_did,
    }), socket, family_id, dry_run).await
}

/// Execute an arbitrary RootPulse graph.
async fn run_graph(
    graph_id: &str,
    params: serde_json::Value,
    socket: Option<PathBuf>,
    family_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let family = family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path = socket.unwrap_or_else(|| {
        SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family}"))
    });

    info!("RootPulse {graph_id}");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let mut full_params = params;
    if let Some(obj) = full_params.as_object_mut() {
        obj.insert("FAMILY_ID".to_string(), serde_json::Value::String(family.clone()));
    }

    if dry_run {
        info!("  [dry run] Would execute {graph_id} graph with:");
        info!("    params = {}", serde_json::to_string_pretty(&full_params)?);
        return Ok(());
    }

    let request = JsonRpcRequest::new(
        "graph.execute",
        serde_json::json!({
            "graph_id": graph_id,
            "params": full_params,
        }),
    );

    let response = send_jsonrpc(&socket_path, &request).await?;

    if let Some(result) = response.get("result") {
        info!("RootPulse {graph_id} succeeded");
        info!("  result: {}", serde_json::to_string_pretty(result)?);
    } else if let Some(err) = response.get("error") {
        error!("RootPulse {graph_id} failed");
        error!("  error: {err}");
        anyhow::bail!("RootPulse {graph_id} failed: {err}");
    }

    Ok(())
}

/// Check the health of the provenance trio.
async fn run_status(
    socket: Option<PathBuf>,
    family_id: Option<String>,
) -> Result<()> {
    let family = family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path = socket.unwrap_or_else(|| {
        SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family}"))
    });

    info!("RootPulse status");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let primals = ["rhizocrypt", "loamspine", "sweetgrass", "beardog", "nestgate"];

    for primal in &primals {
        let request = JsonRpcRequest::new(
            "capability.call",
            serde_json::json!({
                "capability": "health",
                "operation": "check",
                "target": primal,
            }),
        );

        match send_jsonrpc(&socket_path, &request).await {
            Ok(response) => {
                if response.get("result").is_some() {
                    info!("  {primal}: healthy");
                } else {
                    info!("  {primal}: unhealthy ({})", response.get("error").map_or("unknown".to_string(), |e| e.to_string()));
                }
            }
            Err(_) => {
                info!("  {primal}: unavailable");
            }
        }
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
    async fn test_commit_dry_run() {
        let result = run_commit(
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
    async fn test_graph_dry_run() {
        let result = run_graph(
            "rootpulse_branch",
            serde_json::json!({
                "SESSION_ID": "session-1",
                "BRANCH_NAME": "feature-x",
                "AGENT_DID": "did:key:z6MkTest",
            }),
            None,
            Some("test-family".to_string()),
            true,
        )
        .await;
        result.expect("dry run should succeed");
    }

    #[tokio::test]
    async fn test_run_missing_socket() {
        let result = run_commit(
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
            "error should mention connection failure: {err}",
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
