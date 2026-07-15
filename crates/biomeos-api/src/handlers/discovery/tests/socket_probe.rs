// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_probe_live_sockets_returns_vec() {
    let primals = probe_live_sockets().await;
    // Should return an empty vec if no sockets exist (which is fine in test env)
    // The important thing is it doesn't panic or return fabricated data
    for primal in &primals {
        assert!(
            primal.endpoint.starts_with("unix://"),
            "Probed endpoint should be Unix socket: {}",
            primal.endpoint
        );
        // Trust level should be 1 (discovered, not yet verified)
        assert_eq!(primal.trust_level, Some(1));
        // Probed type should be "probed" (not fabricated)
        assert_eq!(primal.primal_type, "probed");
    }
}

#[tokio::test]
async fn test_probe_live_sockets_correct_structure() {
    let primals = probe_live_sockets().await;
    for primal in &primals {
        assert!(!primal.id.is_empty(), "Probed primal should have an ID");
        assert!(!primal.name.is_empty(), "Probed primal should have a name");
        assert!(
            !primal.endpoint.is_empty(),
            "Probed primal should have an endpoint"
        );
        assert!(
            primal.last_seen > 0,
            "Probed primal should have a timestamp"
        );
    }
}

#[tokio::test]
async fn test_probe_live_sockets_with_sock_files_no_runtime() {
    let temp = tempfile::tempdir().expect("tempdir");
    let sock_dir = temp.path();

    std::fs::write(sock_dir.join("beardog-family1.sock"), "").expect("write");
    std::fs::write(sock_dir.join("songbird-family1.sock"), "").expect("write");
    std::fs::write(sock_dir.join("not-a-socket.txt"), "").expect("write");
    std::fs::write(sock_dir.join("another.log"), "").expect("write");

    let primals = probe_live_sockets_in(sock_dir).await;

    assert_eq!(primals.len(), 2, "should find exactly 2 .sock files");

    let mut names: Vec<&str> = primals.iter().map(|p| p.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["beardog", "songbird"]);

    for primal in &primals {
        assert_eq!(primal.health, "unreachable");
        assert_eq!(primal.version, "unknown");
        assert!(primal.capabilities.is_empty());
        assert!(
            primal.error.is_some(),
            "unreachable probe should include error detail"
        );
        assert_eq!(primal.trust_level, Some(1));
        assert_eq!(primal.primal_type, "probed");
        assert!(primal.endpoint.starts_with("unix://"));
        assert!(primal.id.ends_with("-probed"));
        assert!(primal.family_id.is_none());
    }
}

#[tokio::test]
async fn test_probe_live_sockets_controlled_empty_dir() {
    let temp = tempfile::tempdir().expect("tempdir");

    let primals = probe_live_sockets_in(temp.path()).await;
    assert!(primals.is_empty());
}

#[tokio::test]
async fn test_probe_live_sockets_nonexistent_override_dir() {
    use std::path::Path;

    let primals = probe_live_sockets_in(Path::new("/nonexistent/probe/dir/xyz123")).await;
    assert!(primals.is_empty());
}

#[tokio::test]
async fn test_probe_live_sockets_extracts_primal_name_from_hyphenated_filename() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("beardog-family-a.sock"), "").expect("write");
    std::fs::write(temp.path().join("simple.sock"), "").expect("write");

    let primals = probe_live_sockets_in(temp.path()).await;
    assert_eq!(primals.len(), 2);

    let by_id: std::collections::HashMap<&str, &DiscoveredPrimal> =
        primals.iter().map(|p| (p.name.as_str(), p)).collect();

    assert!(
        by_id.contains_key("beardog"),
        "hyphenated name → first segment"
    );
    assert!(
        by_id.contains_key("simple"),
        "unhyphenated name → full stem"
    );
}
