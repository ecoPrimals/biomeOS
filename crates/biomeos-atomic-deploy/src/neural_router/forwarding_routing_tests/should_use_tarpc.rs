// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{create_router, tcp_ep, unix_ep};
use crate::living_graph::{LivingGraph, PrimalProtocolState, ProtocolMode};
use biomeos_types::tarpc_types::ProtocolPreference;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_should_use_tarpc_jsonrpc_only_returns_false() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);
    let ep = unix_ep(&std::path::PathBuf::from("/tmp/test-primal.sock"));
    assert!(!router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_tarpc_only_returns_true() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::TarpcOnly);
    let ep = unix_ep(&std::path::PathBuf::from("/tmp/test-primal.sock"));
    assert!(router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_prefer_jsonrpc_returns_false() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::PreferJsonRpc);
    let ep = unix_ep(&std::path::PathBuf::from("/tmp/test-primal.sock"));
    assert!(!router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_prefer_tarpc_no_graph_returns_false() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::PreferTarpc);
    let ep = unix_ep(&std::path::PathBuf::from("/tmp/test-primal.sock"));
    assert!(!router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_tarpc_available() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    state.current_mode = ProtocolMode::Tarpc;
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_jsonrpc_mode_returns_false() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(!router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_tarpc_mode() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    state.current_mode = ProtocolMode::Tarpc;
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_hybrid_mode() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    state.current_mode = ProtocolMode::Hybrid;
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_no_tarpc_socket_returns_false() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let graph = Arc::new(LivingGraph::new("test"));
    let state = PrimalProtocolState::new("beardog", json_sock.clone());
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(!router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_no_primal_in_graph_returns_false() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("other.sock");
    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", temp.path().join("beardog.sock"))
        .with_tarpc_socket(temp.path().join("beardog.tarpc.sock"));
    state.current_mode = ProtocolMode::Tarpc;
    let _ = std::fs::File::create(temp.path().join("beardog.tarpc.sock"));
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(!router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_tcp_endpoint_with_auto_no_graph() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::Auto);
    assert!(!router.should_use_tarpc(&tcp_ep()).await);
}
