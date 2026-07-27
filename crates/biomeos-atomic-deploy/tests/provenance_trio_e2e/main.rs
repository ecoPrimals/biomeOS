// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

//! E2E Tests for the Provenance Trio (rhizoCrypt + LoamSpine + sweetGrass)
//!
//! Validates the complete RootPulse commit workflow:
//! 1. Deploy the Provenance Trio via `provenance_trio_deploy.toml`
//! 2. Execute `rootpulse_commit.toml` workflow
//! 3. Verify dehydration, signing, storage, commit, and attribution
//!
//! # Prerequisites
//!
//! - Tower Atomic running (BearDog + Songbird)
//! - NestGate running (content storage)
//! - rhizoCrypt, LoamSpine, sweetGrass binaries built
//!
//! # Running
//!
//! ```bash
//! cargo test --test provenance_trio_e2e -- --test-threads=1
//! ```

mod graph_validation;
mod live_workflow;

use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Fixture fields socket_dir kept for debugging; family_id used in test params.
#[expect(dead_code, reason = "serde deserialization requires all fields")]
pub(crate) struct ProvenanceTrioFixture {
    pub family_id: String,
    pub socket_dir: PathBuf,
    pub neural_api_socket: PathBuf,
    pub rhizocrypt_socket: PathBuf,
    pub loamspine_socket: PathBuf,
    pub sweetgrass_socket: PathBuf,
}

impl ProvenanceTrioFixture {
    pub fn new(family_id: &str) -> Self {
        let socket_dir = std::env::var("XDG_RUNTIME_DIR")
            .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
            .join("biomeos");

        std::fs::create_dir_all(&socket_dir).ok();

        Self {
            family_id: family_id.to_string(),
            socket_dir: socket_dir.clone(),
            neural_api_socket: socket_dir.join(format!("neural-api-{family_id}.sock")),
            rhizocrypt_socket: socket_dir.join(format!("rhizocrypt-{family_id}.sock")),
            loamspine_socket: socket_dir.join(format!("loamspine-{family_id}.sock")),
            sweetgrass_socket: socket_dir.join(format!("sweetgrass-{family_id}.sock")),
        }
    }

    pub fn trio_sockets(&self) -> Vec<(&str, &PathBuf)> {
        vec![
            ("rhizocrypt", &self.rhizocrypt_socket),
            ("loamspine", &self.loamspine_socket),
            ("sweetgrass", &self.sweetgrass_socket),
        ]
    }
}

pub(crate) async fn json_rpc_call(
    socket_path: &PathBuf,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Connect to {}: {}", socket_path.display(), e))?;

    let (reader, mut writer) = stream.into_split();

    let request = json!({
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
        Duration::from_secs(15),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| "Timeout waiting for response".to_string())?
    .map_err(|e| e.to_string())?;

    serde_json::from_str(&response_line).map_err(|e| e.to_string())
}

pub(crate) async fn capability_call(
    neural_api: &PathBuf,
    capability: &str,
    operation: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    json_rpc_call(
        neural_api,
        "capability.call",
        json!({
            "capability": capability,
            "operation": operation,
            "args": args
        }),
    )
    .await
}

pub(crate) fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("graphs")
}
