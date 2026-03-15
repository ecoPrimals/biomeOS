// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! E2E Tests for Spring Niche Deployment
//!
//! Validates deploying springs as biomeOS niches:
//! - wetSpring with Tower + Nest atomic (NestGate data flow + rootpulse provenance)
//! - hotSpring as Node atomic on GPU hardware (compute.execute routing)
//!
//! These tests are `#[ignore]` and require a running biomeOS environment.
//!
//! # Running
//!
//! ```bash
//! # wetSpring + Nest validation
//! cargo test -p biomeos-atomic-deploy --test spring_niche_deploy_e2e \
//!   -- --ignored test_wetspring_nest --test-threads=1
//!
//! # hotSpring + Node validation
//! cargo test -p biomeos-atomic-deploy --test spring_niche_deploy_e2e \
//!   -- --ignored test_hotspring_node --test-threads=1
//! ```

use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

struct NicheFixture {
    family_id: String,
    socket_dir: PathBuf,
    neural_api_socket: PathBuf,
}

impl NicheFixture {
    fn new(family_id: &str) -> Self {
        let socket_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join("biomeos");

        Self {
            family_id: family_id.to_string(),
            neural_api_socket: socket_dir.join(format!("neural-api-{family_id}.sock")),
            socket_dir,
        }
    }

    fn spring_socket(&self, name: &str) -> PathBuf {
        self.socket_dir
            .join(format!("{}-{}.sock", name, self.family_id))
    }
}

async fn json_rpc_call(
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
        Duration::from_secs(30),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| "Timeout".to_string())?
    .map_err(|e| e.to_string())?;

    serde_json::from_str(&response_line).map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// wetSpring + Nest Atomic: Deploy and validate NestGate data flow
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running biomeOS environment with Tower + Nest atomic"]
async fn test_wetspring_nest_deploy() {
    let fixture = NicheFixture::new("e2e-wet-nest");

    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "niche.deploy",
        json!({ "template_id": "wetspring" }),
    )
    .await
    .expect("niche.deploy(wetspring) should succeed");

    assert!(
        result["result"].is_object(),
        "Deployment should return execution result"
    );
    eprintln!("  wetSpring deploy: {:?}", result["result"]);
}

#[tokio::test]
#[ignore = "Requires running biomeOS environment with wetSpring deployed"]
async fn test_wetspring_nest_health() {
    let fixture = NicheFixture::new("e2e-wet-nest");

    let health = json_rpc_call(
        &fixture.spring_socket("wetspring"),
        "health.check",
        json!({}),
    )
    .await
    .expect("wetSpring health.check should succeed");

    assert_eq!(
        health["result"]["status"].as_str().unwrap_or(""),
        "healthy",
        "wetSpring should be healthy"
    );
    eprintln!("  wetSpring health: {:?}", health["result"]);
}

#[tokio::test]
#[ignore = "Requires running biomeOS environment with wetSpring deployed"]
async fn test_wetspring_nest_capability_list() {
    let fixture = NicheFixture::new("e2e-wet-nest");

    let caps = json_rpc_call(
        &fixture.spring_socket("wetspring"),
        "capability.list",
        json!({}),
    )
    .await
    .expect("wetSpring capability.list should succeed");

    let capabilities = caps["result"]["capabilities"]
        .as_array()
        .expect("capabilities array");
    assert!(
        !capabilities.is_empty(),
        "wetSpring should report capabilities"
    );
    eprintln!("  wetSpring capabilities: {:?}", capabilities);
}

#[tokio::test]
#[ignore = "Requires running biomeOS environment with wetSpring + NestGate"]
async fn test_wetspring_nestgate_data_flow() {
    let fixture = NicheFixture::new("e2e-wet-nest");

    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "capability.call",
        json!({
            "capability": "science",
            "operation": "diversity",
            "args": {
                "analysis_type": "alpha"
            }
        }),
    )
    .await
    .expect("science.diversity via Neural API should succeed");

    assert!(
        result["result"].is_object(),
        "Diversity result should be object"
    );
    eprintln!("  NestGate data flow validated: {:?}", result["result"]);
}

#[tokio::test]
#[ignore = "Requires running biomeOS environment with wetSpring + rootpulse trio"]
async fn test_wetspring_provenance_session() {
    let fixture = NicheFixture::new("e2e-wet-nest");

    let session = json_rpc_call(
        &fixture.neural_api_socket,
        "capability.call",
        json!({
            "capability": "provenance",
            "operation": "begin_session",
            "args": {
                "experiment_id": "e2e-wetspring-diversity-001",
                "agent_did": "did:key:z6MkE2eWetNest"
            }
        }),
    )
    .await
    .expect("provenance.begin_session should succeed");

    assert!(session["result"].is_object(), "Session should be created");
    eprintln!("  Provenance session: {:?}", session["result"]);

    let session_id = session["result"]["session_id"]
        .as_str()
        .expect("session_id");

    let record = json_rpc_call(
        &fixture.neural_api_socket,
        "capability.call",
        json!({
            "capability": "provenance",
            "operation": "record_step",
            "args": {
                "session_id": session_id,
                "step": "diversity_analysis",
                "data": { "type": "alpha", "shannon": 2.1 }
            }
        }),
    )
    .await
    .expect("provenance.record_step should succeed");

    assert!(record["result"].is_object());

    let commit = json_rpc_call(
        &fixture.neural_api_socket,
        "capability.call",
        json!({
            "capability": "provenance",
            "operation": "complete_experiment",
            "args": { "session_id": session_id }
        }),
    )
    .await
    .expect("provenance.complete_experiment should succeed");

    assert!(commit["result"].is_object());
    eprintln!("  rootpulse provenance validated");
}

// ═══════════════════════════════════════════════════════════════════════════════
// hotSpring + Node Atomic: Deploy on dual Titan V and validate compute routing
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running biomeOS environment with Node atomic + GPU hardware"]
async fn test_hotspring_node_deploy() {
    let fixture = NicheFixture::new("e2e-hot-node");

    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "niche.deploy",
        json!({ "template_id": "hotspring" }),
    )
    .await
    .expect("niche.deploy(hotspring) should succeed");

    assert!(
        result["result"].is_object(),
        "Deployment should return execution result"
    );
    eprintln!("  hotSpring deploy: {:?}", result["result"]);
}

#[tokio::test]
#[ignore = "Requires running biomeOS environment with hotSpring deployed"]
async fn test_hotspring_node_health() {
    let fixture = NicheFixture::new("e2e-hot-node");

    let health = json_rpc_call(
        &fixture.spring_socket("hotspring"),
        "health.check",
        json!({}),
    )
    .await
    .expect("hotSpring health.check should succeed");

    assert_eq!(health["result"]["status"].as_str().unwrap_or(""), "healthy");
    eprintln!("  hotSpring health: {:?}", health["result"]);
}

#[tokio::test]
#[ignore = "Requires running biomeOS environment with hotSpring + ToadStool"]
async fn test_hotspring_compute_execute_routing() {
    let fixture = NicheFixture::new("e2e-hot-node");

    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "capability.call",
        json!({
            "capability": "physics",
            "operation": "compute",
            "args": {
                "workload": "md_simulation",
                "substrate": "gpu"
            }
        }),
    )
    .await
    .expect("physics.compute via Neural API should succeed");

    assert!(
        result["result"].is_object(),
        "Compute result should be object"
    );
    eprintln!(
        "  compute.execute routing validated: {:?}",
        result["result"]
    );
}

#[tokio::test]
#[ignore = "Requires running biomeOS environment with hotSpring + ToadStool"]
async fn test_hotspring_capability_list() {
    let fixture = NicheFixture::new("e2e-hot-node");

    let caps = json_rpc_call(
        &fixture.spring_socket("hotspring"),
        "capability.list",
        json!({}),
    )
    .await
    .expect("hotSpring capability.list should succeed");

    let capabilities = caps["result"]["capabilities"]
        .as_array()
        .expect("capabilities array");
    assert!(
        !capabilities.is_empty(),
        "hotSpring should report capabilities"
    );

    let expected_domains = ["physics", "md", "spectral", "nuclear_eos"];
    for domain in expected_domains {
        assert!(
            capabilities
                .iter()
                .any(|c| c.as_str().unwrap_or("").contains(domain)),
            "hotSpring should have {domain} capability"
        );
    }
    eprintln!("  hotSpring capabilities: {:?}", capabilities);
}

#[tokio::test]
#[ignore = "Requires running biomeOS with hotSpring on dual Titan V"]
async fn test_hotspring_dual_gpu_detection() {
    let fixture = NicheFixture::new("e2e-hot-node");

    let toadstool_health = json_rpc_call(
        &fixture.spring_socket("toadstool"),
        "health.check",
        json!({}),
    )
    .await
    .expect("ToadStool health should succeed");

    let gpu_count = toadstool_health["result"]["gpu_count"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        gpu_count >= 2,
        "Expected 2+ GPUs (dual Titan V), found {gpu_count}"
    );
    eprintln!("  Dual Titan V detected: {gpu_count} GPUs");
}
