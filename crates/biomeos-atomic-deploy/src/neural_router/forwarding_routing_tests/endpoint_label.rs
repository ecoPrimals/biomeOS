// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::create_router;
use biomeos_core::TransportEndpoint;
use crate::neural_router::NeuralRouter;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_primal_label_for_endpoint_variants() {
    let router = create_router("test");

    let unix = TransportEndpoint::UnixSocket {
        path: PathBuf::from("/tmp/beardog.sock"),
    };
    assert_eq!(
        router.primal_label_for_endpoint(&unix),
        Some("beardog".to_string())
    );

    let tcp = TransportEndpoint::TcpSocket {
        host: Arc::from("192.0.2.100"),
        port: 9001,
    };
    assert_eq!(
        router.primal_label_for_endpoint(&tcp),
        Some("192.0.2.100:9001".to_string())
    );

    let abs = TransportEndpoint::AbstractSocket {
        name: Arc::from("squirrel_abc"),
    };
    assert_eq!(
        router.primal_label_for_endpoint(&abs),
        Some("squirrel_abc".to_string())
    );

    let http = TransportEndpoint::HttpJsonRpc {
        host: Arc::from("songbird.local"),
        port: 8080,
    };
    assert_eq!(
        router.primal_label_for_endpoint(&http),
        Some("songbird.local:8080".to_string())
    );
}

#[tokio::test]
async fn lazy_rescan_excludes_self_socket() {
    let router = NeuralRouter::new("self-excl");
    let self_sock = std::path::PathBuf::from("/run/user/9999/biomeos/neural-api-self-excl.sock");
    router.set_self_socket_path(self_sock.clone()).await;

    let guard = router.self_socket_path.read().await;
    assert_eq!(guard.as_ref(), Some(&self_sock));
}
