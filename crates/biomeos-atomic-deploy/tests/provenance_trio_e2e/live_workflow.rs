// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Live integration tests for the Provenance Trio workflow.
//! Requires running primals (Tower Atomic + Trio deployed).

use serde_json::json;

use crate::{ProvenanceTrioFixture, capability_call, json_rpc_call};

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

#[expect(
    clippy::too_many_lines,
    reason = "integration test sequences many Neural API steps"
)]
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
