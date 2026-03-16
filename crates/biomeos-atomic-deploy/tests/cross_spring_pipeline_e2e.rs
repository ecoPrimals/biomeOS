// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! E2E Tests for Cross-Spring Pipeline Execution
//!
//! Validates cross-spring coordination:
//! - airSpring soil moisture → wetSpring diversity analysis
//! - Cross-spring ecology pipeline (ET₀ → diversity → spectral)
//! - Provenance attribution across spring boundaries
//!
//! # Prerequisites
//!
//! - Tower Atomic running (BearDog + Songbird)
//! - airSpring and wetSpring registered as capability providers
//! - Neural API routing operational
//!
//! # Running
//!
//! ```bash
//! cargo test -p biomeos-atomic-deploy --test cross_spring_pipeline_e2e -- --test-threads=1 --ignored
//! ```

use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Fixture fields socket_dir kept for debugging; family_id used in test params.
#[allow(dead_code)]
struct CrossSpringFixture {
    family_id: String,
    socket_dir: PathBuf,
    neural_api_socket: PathBuf,
}

impl CrossSpringFixture {
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
// Test: Verify both springs are registered as capability providers
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals"]
async fn test_springs_registered() {
    let fixture = CrossSpringFixture::new("e2e-xspring");

    for domain in &["ecology", "science"] {
        let result = json_rpc_call(
            &fixture.neural_api_socket,
            "capability.discover",
            json!({ "capability": domain }),
        )
        .await
        .unwrap_or_else(|e| panic!("capability.discover({domain}) failed: {e}"));

        let primals = result["result"]["primals"]
            .as_array()
            .expect("primals array");
        assert!(!primals.is_empty(), "No providers for {domain} domain");
        eprintln!("  {} domain: {} provider(s)", domain, primals.len());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: airSpring ET₀ → wetSpring diversity (sequential capability calls)
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals"]
async fn test_airspring_to_wetspring_capability_chain() {
    let fixture = CrossSpringFixture::new("e2e-xspring");

    // Step 1: airSpring computes ET₀
    let et0 = json_rpc_call(
        &fixture.neural_api_socket,
        "capability.call",
        json!({
            "capability": "ecology",
            "operation": "et0_fao56",
            "args": {
                "tmax": 30.0,
                "tmin": 18.0,
                "solar_radiation": 22.0,
                "wind_speed_2m": 1.5,
                "actual_vapour_pressure": 1.1,
                "day_of_year": 180,
                "latitude_deg": 42.727,
                "elevation_m": 256.0
            }
        }),
    )
    .await
    .expect("ecology.et0_fao56 should succeed");

    assert!(et0["result"].is_object(), "ET₀ result should be object");
    eprintln!("  airSpring ET₀: {:?}", et0["result"]);

    // Step 2: wetSpring diversity analysis
    let diversity = json_rpc_call(
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
    .expect("science.diversity should succeed");

    assert!(
        diversity["result"].is_object(),
        "Diversity result should be object"
    );
    eprintln!("  wetSpring diversity: {:?}", diversity["result"]);

    eprintln!("\n  Cross-spring chain: PASSED");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: Execute cross_spring_soil_microbiome graph via graph.execute
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals"]
async fn test_soil_microbiome_graph_execution() {
    let fixture = CrossSpringFixture::new("e2e-xspring");

    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "graph.execute",
        json!({
            "graph_id": "cross_spring_soil_microbiome",
            "params": {
                "EXPERIMENT_ID": "e2e-soil-microbiome-001",
                "AGENT_DID": "did:key:z6MkCrossSpringTest",
                "FAMILY_ID": fixture.family_id
            }
        }),
    )
    .await
    .expect("graph.execute(cross_spring_soil_microbiome) should succeed");

    assert!(
        result["result"].is_object(),
        "Graph execution should return result"
    );
    eprintln!("  Soil microbiome graph: {:?}", result["result"]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: Execute cross_spring_ecology graph via graph.execute
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals"]
async fn test_ecology_pipeline_graph_execution() {
    let fixture = CrossSpringFixture::new("e2e-xspring");

    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "graph.execute",
        json!({
            "graph_id": "cross_spring_ecology",
            "params": {
                "FAMILY_ID": fixture.family_id
            }
        }),
    )
    .await
    .expect("graph.execute(cross_spring_ecology) should succeed");

    assert!(
        result["result"].is_object(),
        "Ecology pipeline graph should return result"
    );
    eprintln!("  Ecology pipeline graph: {:?}", result["result"]);
}
