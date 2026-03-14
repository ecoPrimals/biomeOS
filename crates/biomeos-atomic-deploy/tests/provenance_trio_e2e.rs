// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

use biomeos_atomic_deploy::neural_graph::Graph;
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[allow(dead_code)]
struct ProvenanceTrioFixture {
    family_id: String,
    socket_dir: PathBuf,
    neural_api_socket: PathBuf,
    rhizocrypt_socket: PathBuf,
    loamspine_socket: PathBuf,
    sweetgrass_socket: PathBuf,
}

impl ProvenanceTrioFixture {
    fn new(family_id: &str) -> Self {
        let socket_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
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

    fn trio_sockets(&self) -> Vec<(&str, &PathBuf)> {
        vec![
            ("rhizocrypt", &self.rhizocrypt_socket),
            ("loamspine", &self.loamspine_socket),
            ("sweetgrass", &self.sweetgrass_socket),
        ]
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
        Duration::from_secs(15),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| "Timeout waiting for response".to_string())?
    .map_err(|e| e.to_string())?;

    serde_json::from_str(&response_line).map_err(|e| e.to_string())
}

async fn capability_call(
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

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 0: Graph Validation — verify deployment graphs parse and have correct
//          topology (runs without live primals, safe for CI)
// ═══════════════════════════════════════════════════════════════════════════════

fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("graphs")
}

#[test]
fn test_loamspine_deploy_graph_parses() {
    let path = graphs_dir().join("loamspine_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("loamspine_deploy.toml should parse");

    assert_eq!(graph.id, "loamspine_deploy");
    assert_eq!(graph.nodes.len(), 5);

    let ops: Vec<&str> = graph
        .nodes
        .iter()
        .filter_map(|n| n.operation.as_ref().map(|o| o.name.as_str()))
        .collect();
    assert_eq!(
        ops,
        vec![
            "health_check",
            "health_check",
            "primal.launch",
            "register_capabilities",
            "health_check"
        ]
    );
}

#[test]
fn test_rhizocrypt_deploy_graph_parses() {
    let path = graphs_dir().join("rhizocrypt_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("rhizocrypt_deploy.toml should parse");

    assert_eq!(graph.id, "rhizocrypt_deploy");
    assert_eq!(graph.nodes.len(), 5);

    let start_node = graph
        .nodes
        .iter()
        .find(|n| n.id == "start-rhizocrypt")
        .expect("start-rhizocrypt node");
    let op = start_node.operation.as_ref().expect("operation");
    assert_eq!(op.name, "primal.launch");
    assert!(
        op.environment.is_some(),
        "Environment vars should be present"
    );
    let env = op.environment.as_ref().unwrap();
    assert_eq!(env.get("RHIZOCRYPT_RPC_PORT").unwrap(), "9400");
}

#[test]
fn test_sweetgrass_deploy_graph_parses() {
    let path = graphs_dir().join("sweetgrass_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("sweetgrass_deploy.toml should parse");

    assert_eq!(graph.id, "sweetgrass_deploy");
    assert_eq!(graph.nodes.len(), 5);

    let reg_node = graph
        .nodes
        .iter()
        .find(|n| n.id == "register-capabilities")
        .expect("register-capabilities node");
    assert_eq!(
        reg_node.capabilities,
        vec![
            "attribution",
            "braid",
            "provenance",
            "contribution",
            "privacy"
        ]
    );
}

#[test]
fn test_provenance_trio_deploy_graph_parses() {
    let path = graphs_dir().join("provenance_trio_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("provenance_trio_deploy.toml should parse");

    assert_eq!(graph.id, "provenance_trio_deploy");
    assert_eq!(graph.nodes.len(), 11);

    let launch_nodes: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.operation
                .as_ref()
                .map(|o| o.name == "primal.launch")
                .unwrap_or(false)
        })
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(
        launch_nodes,
        vec!["start-loamspine", "start-rhizocrypt", "start-sweetgrass"]
    );
}

#[test]
fn test_provenance_trio_dependency_order() {
    let path = graphs_dir().join("provenance_trio_deploy.toml");
    let graph = Graph::from_toml_file(&path).unwrap();

    let find_deps = |id: &str| -> Vec<String> {
        graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.depends_on.clone())
            .unwrap_or_default()
    };

    // loamSpine starts after songbird health
    assert_eq!(find_deps("start-loamspine"), vec!["verify-songbird"]);

    // rhizoCrypt starts after loamSpine is healthy
    assert_eq!(find_deps("start-rhizocrypt"), vec!["health-loamspine"]);

    // sweetGrass starts after loamSpine is healthy (parallel with rhizoCrypt)
    assert_eq!(find_deps("start-sweetgrass"), vec!["health-loamspine"]);
}

#[test]
fn test_rootpulse_commit_graph_parses() {
    let path = graphs_dir().join("rootpulse_commit.toml");
    let graph = Graph::from_toml_file(&path).expect("rootpulse_commit.toml should parse");

    assert_eq!(graph.id, "rootpulse_commit");

    let op_names: Vec<&str> = graph
        .nodes
        .iter()
        .filter_map(|n| n.operation.as_ref().map(|o| o.name.as_str()))
        .collect();

    assert!(
        op_names.contains(&"rpc_call"),
        "Should contain rpc_call nodes"
    );
    assert!(
        op_names.contains(&"capability_call"),
        "Should contain capability_call nodes"
    );
}

#[test]
fn test_provenance_pipeline_graph_parses() {
    let path = graphs_dir().join("provenance_pipeline.toml");
    let graph = Graph::from_toml_file(&path).expect("provenance_pipeline.toml should parse");

    assert_eq!(graph.id, "provenance_pipeline");
    assert!(
        graph.nodes.len() >= 4,
        "Pipeline should have at least 4 nodes"
    );
}

#[test]
fn test_all_deployment_graphs_have_environment_on_launch_nodes() {
    let deploy_graphs = [
        "loamspine_deploy.toml",
        "rhizocrypt_deploy.toml",
        "sweetgrass_deploy.toml",
        "provenance_trio_deploy.toml",
    ];

    for graph_name in &deploy_graphs {
        let path = graphs_dir().join(graph_name);
        let graph = Graph::from_toml_file(&path)
            .unwrap_or_else(|e| panic!("{graph_name} should parse: {e}"));

        for node in &graph.nodes {
            if let Some(ref op) = node.operation {
                if op.name == "primal.launch" {
                    assert!(
                        op.environment.is_some(),
                        "{}: launch node '{}' missing environment",
                        graph_name,
                        node.id
                    );
                }
            }
        }
    }
}

#[test]
fn test_provenance_trio_topological_order() {
    let path = graphs_dir().join("provenance_trio_deploy.toml");
    let graph = Graph::from_toml_file(&path).unwrap();

    // Build a dependency map and verify that transitive ordering is correct:
    // loamSpine must start before rhizoCrypt and sweetGrass
    let node_index = |id: &str| -> usize {
        graph
            .nodes
            .iter()
            .position(|n| n.id == id)
            .unwrap_or_else(|| panic!("Node '{id}' not found"))
    };

    let deps_of = |id: &str| -> &Vec<String> { &graph.nodes[node_index(id)].depends_on };

    // rhizoCrypt depends on loamSpine being healthy
    assert!(
        deps_of("start-rhizocrypt").contains(&"health-loamspine".to_string()),
        "rhizoCrypt should depend on health-loamspine"
    );

    // sweetGrass depends on loamSpine being healthy
    assert!(
        deps_of("start-sweetgrass").contains(&"health-loamspine".to_string()),
        "sweetGrass should depend on health-loamspine"
    );

    // health-loamspine depends on start-loamspine
    assert!(
        deps_of("health-loamspine").contains(&"start-loamspine".to_string()),
        "health-loamspine should depend on start-loamspine"
    );

    // start-loamspine depends on verify-songbird
    assert!(
        deps_of("start-loamspine").contains(&"verify-songbird".to_string()),
        "start-loamspine should depend on verify-songbird"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1: Health Checks — verify all trio primals are running
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals - use for integration testing"]
async fn test_trio_health_checks() {
    let fixture = ProvenanceTrioFixture::new("e2e-trio");

    for (name, socket) in fixture.trio_sockets() {
        let result = json_rpc_call(socket, "health", json!({})).await;
        assert!(
            result.is_ok(),
            "{} health check failed: {:?}",
            name,
            result.err()
        );
        let response = result.unwrap();
        assert!(
            response.get("result").is_some(),
            "{name} returned error: {response}"
        );
        eprintln!("  {name} healthy");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2: Neural API capability routing — verify trio capabilities are
//          registered and routable through the Neural API
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals - use for integration testing"]
async fn test_trio_capabilities_registered() {
    let fixture = ProvenanceTrioFixture::new("e2e-trio");

    let required_domains = ["dag", "permanent_storage", "attribution"];

    for domain in &required_domains {
        let result = json_rpc_call(
            &fixture.neural_api_socket,
            "capability.discover",
            json!({ "capability": domain }),
        )
        .await;

        assert!(
            result.is_ok(),
            "capability.discover({}) failed: {:?}",
            domain,
            result.err()
        );

        let response = result.unwrap();
        let primals = response["result"]["primals"]
            .as_array()
            .expect("primals should be an array");
        assert!(
            !primals.is_empty(),
            "No providers for domain '{domain}' — trio not fully registered"
        );
        eprintln!("  {} domain: {} provider(s)", domain, primals.len());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 3: RootPulse Commit — execute the full rootpulse_commit workflow
//          via Neural API graph execution
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals - use for integration testing"]
async fn test_rootpulse_commit_e2e() {
    let fixture = ProvenanceTrioFixture::new("e2e-trio");

    // Step 1: Create an ephemeral session in rhizoCrypt
    let session = capability_call(
        &fixture.neural_api_socket,
        "dag",
        "create_session",
        json!({ "metadata": { "type": "e2e-test" } }),
    )
    .await
    .expect("dag.create_session should succeed");

    let session_id = session["result"]["session_id"]
        .as_str()
        .expect("session_id in response");
    eprintln!("  Created session: {session_id}");

    // Step 2: Append some events to the session
    let _ = capability_call(
        &fixture.neural_api_socket,
        "dag",
        "append_event",
        json!({
            "session_id": session_id,
            "event": {
                "type": "file_edit",
                "path": "src/main.rs",
                "content_hash": "blake3:e2etest000"
            }
        }),
    )
    .await
    .expect("dag.append_event should succeed");

    // Step 3: Dehydrate the session
    let dehydration = capability_call(
        &fixture.neural_api_socket,
        "dag",
        "dehydrate",
        json!({ "session_id": session_id }),
    )
    .await
    .expect("dag.dehydrate should succeed");

    let merkle_root = dehydration["result"]["merkle_root"]
        .as_str()
        .expect("merkle_root in dehydration");
    eprintln!("  Dehydrated — merkle root: {merkle_root}");

    // Step 4: Sign the dehydration summary
    let signed = capability_call(
        &fixture.neural_api_socket,
        "crypto",
        "sign",
        json!({
            "data": merkle_root,
            "did": "did:key:z6MkE2ETest"
        }),
    )
    .await
    .expect("crypto.sign should succeed");

    assert!(
        signed["result"]["signature"].is_string(),
        "Signature should be returned"
    );
    eprintln!("  Signed");

    // Step 5: Commit to permanent history
    let commit = capability_call(
        &fixture.neural_api_socket,
        "commit",
        "session",
        json!({
            "summary": dehydration["result"],
            "content_hash": merkle_root
        }),
    )
    .await
    .expect("commit.session should succeed");

    let commit_id = commit["result"]["commit_id"]
        .as_str()
        .or_else(|| commit["result"]["entry_id"].as_str())
        .expect("commit_id or entry_id in response");
    eprintln!("  Committed: {commit_id}");

    // Step 6: Create attribution braid
    let braid = capability_call(
        &fixture.neural_api_socket,
        "provenance",
        "create_braid",
        json!({
            "commit_ref": commit_id,
            "agents": [{
                "did": "did:key:z6MkE2ETest",
                "role": "author",
                "contribution": 1.0
            }]
        }),
    )
    .await
    .expect("provenance.create_braid should succeed");

    let braid_id = braid["result"]["braid_id"]
        .as_str()
        .or_else(|| braid["result"]["id"].as_str())
        .expect("braid_id in response");
    eprintln!("  Attribution braid: {braid_id}");

    // Step 7: Verify the braid exists
    let verify = capability_call(
        &fixture.neural_api_socket,
        "provenance",
        "get_braid",
        json!({ "braid_id": braid_id }),
    )
    .await
    .expect("provenance.get_braid should succeed");

    assert!(verify["result"].is_object(), "Braid should be retrievable");
    eprintln!("  Verified braid retrieval");

    eprintln!("\n  RootPulse commit E2E: PASSED");
    eprintln!("    session:    {session_id}");
    eprintln!("    merkle:     {merkle_root}");
    eprintln!("    commit:     {commit_id}");
    eprintln!("    braid:      {braid_id}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 4: Graph execution — trigger rootpulse_commit.toml via graph.execute
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals - use for integration testing"]
async fn test_rootpulse_graph_execution() {
    let fixture = ProvenanceTrioFixture::new("e2e-trio");

    // First create a session to commit
    let session = capability_call(
        &fixture.neural_api_socket,
        "dag",
        "create_session",
        json!({ "metadata": { "type": "graph-e2e" } }),
    )
    .await
    .expect("Create session for graph test");

    let session_id = session["result"]["session_id"]
        .as_str()
        .expect("session_id");

    // Execute the rootpulse_commit graph
    let graph_result = json_rpc_call(
        &fixture.neural_api_socket,
        "graph.execute",
        json!({
            "graph_id": "rootpulse_commit",
            "params": {
                "SESSION_ID": session_id,
                "AGENT_DID": "did:key:z6MkGraphTest",
                "FAMILY_ID": fixture.family_id
            }
        }),
    )
    .await
    .expect("graph.execute(rootpulse_commit) should succeed");

    assert!(
        graph_result["result"].is_object(),
        "Graph execution should return result"
    );
    eprintln!(
        "  rootpulse_commit graph executed: {:?}",
        graph_result["result"]
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5: Niche deployment — deploy via niche.deploy("rootpulse")
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running primals - use for integration testing"]
async fn test_rootpulse_niche_deploy() {
    let fixture = ProvenanceTrioFixture::new("e2e-trio");

    let result = json_rpc_call(
        &fixture.neural_api_socket,
        "niche.deploy",
        json!({
            "template_id": "rootpulse",
            "params": {
                "SESSION_ID": "niche-e2e-test",
                "AGENT_DID": "did:key:z6MkNicheTest",
                "FAMILY_ID": fixture.family_id
            }
        }),
    )
    .await
    .expect("niche.deploy(rootpulse) should succeed");

    assert!(
        result["result"].is_object(),
        "Niche deployment should return result"
    );
    eprintln!("  rootpulse niche deployed: {:?}", result["result"]);
}
