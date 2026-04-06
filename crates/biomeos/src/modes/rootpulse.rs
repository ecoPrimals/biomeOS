// SPDX-License-Identifier: AGPL-3.0-or-later
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
use biomeos_types::CapabilityTaxonomy;
use biomeos_types::primal_names::{BEARDOG, NESTGATE, PROVENANCE_PRIMALS};
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
        } => {
            run_graph(
                "rootpulse_branch",
                serde_json::json!({
                    "SESSION_ID": session_id,
                    "BRANCH_NAME": branch_name,
                    "AGENT_DID": agent_did,
                }),
                socket,
                family_id,
                dry_run,
            )
            .await
        }
        RootPulseCommand::Merge {
            source_session,
            target_session,
            agent_did,
            socket,
            family_id,
            dry_run,
        } => {
            run_graph(
                "rootpulse_merge",
                serde_json::json!({
                    "SOURCE_SESSION": source_session,
                    "TARGET_SESSION": target_session,
                    "AGENT_DID": agent_did,
                }),
                socket,
                family_id,
                dry_run,
            )
            .await
        }
        RootPulseCommand::Diff {
            from,
            to,
            socket,
            family_id,
        } => {
            run_graph(
                "rootpulse_diff",
                serde_json::json!({
                    "FROM_REF": from,
                    "TO_REF": to,
                }),
                socket,
                family_id,
                false,
            )
            .await
        }
        RootPulseCommand::Status { socket, family_id } => run_status(socket, family_id).await,
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
    run_graph(
        "rootpulse_commit",
        serde_json::json!({
            "SESSION_ID": session_id,
            "AGENT_DID": agent_did,
        }),
        socket,
        family_id,
        dry_run,
    )
    .await
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
    let socket_path = socket
        .unwrap_or_else(|| SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family}")));

    info!("RootPulse {graph_id}");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let mut full_params = params;
    if let Some(obj) = full_params.as_object_mut() {
        obj.insert(
            "FAMILY_ID".to_string(),
            serde_json::Value::String(family.clone()),
        );
    }

    if dry_run {
        info!("  [dry run] Would execute {graph_id} graph with:");
        info!(
            "    params = {}",
            serde_json::to_string_pretty(&full_params)?
        );
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
async fn run_status(socket: Option<PathBuf>, family_id: Option<String>) -> Result<()> {
    let family = family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path = socket
        .unwrap_or_else(|| SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family}")));

    info!("RootPulse status");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let primals: Vec<&str> = PROVENANCE_PRIMALS
        .iter()
        .copied()
        .chain([
            CapabilityTaxonomy::resolve_to_primal("crypto").unwrap_or(BEARDOG),
            CapabilityTaxonomy::resolve_to_primal("storage").unwrap_or(NESTGATE),
        ])
        .collect();

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
                    info!(
                        "  {primal}: unhealthy ({})",
                        response.get("error").map_or_else(
                            || "unknown".to_string(),
                            std::string::ToString::to_string
                        )
                    );
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

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::oneshot;

    /// Spawn a mock Neural API server that responds with the given JSON-RPC response.
    async fn spawn_mock_neural_api(
        response: serde_json::Value,
    ) -> (PathBuf, tokio::task::JoinHandle<()>) {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("neural-api.sock");
        let path_buf = socket_path.clone();
        let path_for_listener = path_buf.clone();
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle = tokio::spawn(async move {
            let _temp = temp;
            let listener = UnixListener::bind(&path_for_listener).expect("bind");
            let _ = ready_tx.send(());
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response_str = serde_json::to_string(&response).expect("serialize") + "\n";
                    let _ = writer.write_all(response_str.as_bytes()).await;
                    let _ = writer.flush().await;
                }
            }
        });

        ready_rx.await.expect("server ready");
        (path_buf, handle)
    }

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

    #[tokio::test]
    async fn test_dispatch_branch_dry_run() {
        let result = dispatch(RootPulseCommand::Branch {
            session_id: "sess-1".to_string(),
            branch_name: "feature-x".to_string(),
            agent_did: "did:key:z6MkTest".to_string(),
            socket: None,
            family_id: Some("test-family".to_string()),
            dry_run: true,
        })
        .await;
        result.expect("branch dry run should succeed");
    }

    #[tokio::test]
    async fn test_dispatch_merge_dry_run() {
        let result = dispatch(RootPulseCommand::Merge {
            source_session: "sess-a".to_string(),
            target_session: "sess-b".to_string(),
            agent_did: "did:key:z6MkTest".to_string(),
            socket: None,
            family_id: Some("test-family".to_string()),
            dry_run: true,
        })
        .await;
        result.expect("merge dry run should succeed");
    }

    #[tokio::test]
    async fn test_dispatch_diff_fails_without_socket() {
        let result = dispatch(RootPulseCommand::Diff {
            from: "ref-a".to_string(),
            to: "ref-b".to_string(),
            socket: Some(PathBuf::from("/tmp/nonexistent-neural-diff.sock")),
            family_id: Some("test-family".to_string()),
        })
        .await;
        let err = result.expect_err("diff without socket should fail");
        assert!(
            err.to_string().contains("connect") || err.to_string().contains("Failed"),
            "error should mention connection: {err}"
        );
    }

    #[tokio::test]
    async fn test_run_graph_non_object_params() {
        // When params is not an object, we can't insert FAMILY_ID - should still work for dry run
        let result = run_graph(
            "rootpulse_test",
            serde_json::json!([]), // array, not object
            Some(PathBuf::from("/tmp/nonexistent.sock")),
            Some("test-family".to_string()),
            true, // dry run - won't connect
        )
        .await;
        result.expect("dry run with non-object params should succeed");
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

    #[tokio::test]
    async fn test_dispatch_commit_dry_run() {
        let result = dispatch(RootPulseCommand::Commit {
            session_id: "sess-1".to_string(),
            agent_did: "did:key:z6MkTest".to_string(),
            socket: None,
            family_id: Some("test-family".to_string()),
            dry_run: true,
        })
        .await;
        result.expect("commit dry run should succeed");
    }

    #[tokio::test]
    async fn test_dispatch_status_with_missing_socket() {
        // run_status catches send_jsonrpc errors and logs "unavailable"; returns Ok
        let result = dispatch(RootPulseCommand::Status {
            socket: Some(PathBuf::from("/tmp/nonexistent-neural-status.sock")),
            family_id: Some("test-family".to_string()),
        })
        .await;
        result.expect("status with missing socket should return Ok (logs unavailable)");
    }

    #[tokio::test]
    async fn test_run_commit_success_with_mock_server() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": { "commit_id": "abc123", "status": "ok" },
            "id": 1
        });
        let (socket_path, _handle) = spawn_mock_neural_api(response).await;

        let result = run_commit(
            "test-session".to_string(),
            "did:key:z6MkTest".to_string(),
            Some(socket_path),
            Some("test-family".to_string()),
            false,
        )
        .await;
        result.expect("commit with mock success should succeed");
    }

    #[tokio::test]
    async fn test_run_commit_error_response_from_server() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32000, "message": "Graph execution failed" },
            "id": 1
        });
        let (socket_path, _handle) = spawn_mock_neural_api(response).await;

        let result = run_commit(
            "test-session".to_string(),
            "did:key:z6MkTest".to_string(),
            Some(socket_path),
            Some("test-family".to_string()),
            false,
        )
        .await;
        let err = result.expect_err("commit with error response should fail");
        assert!(
            err.to_string().contains("failed") || err.to_string().contains("error"),
            "error should mention failure: {err}"
        );
    }

    #[tokio::test]
    async fn test_run_commit_response_neither_result_nor_error() {
        // When response has neither result nor error, run_graph returns Ok (no bail)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1
        });
        let (socket_path, _handle) = spawn_mock_neural_api(response).await;

        let result = run_commit(
            "test-session".to_string(),
            "did:key:z6MkTest".to_string(),
            Some(socket_path),
            Some("test-family".to_string()),
            false,
        )
        .await;
        result.expect("response with no result/error still returns Ok");
    }
}
