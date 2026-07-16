// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::helpers::*;
use super::super::*;
use std::sync::Arc;

#[test]
fn test_discovery_mode() {
    assert_eq!(DiscoveryMode::Plaintext, DiscoveryMode::Plaintext);
    assert_ne!(DiscoveryMode::Plaintext, DiscoveryMode::Encrypted);
}

#[test]
fn test_birdsong_coordinator_new() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: false,
        success: true,
    });
    let _coordinator = BirdSongCoordinator::new(security, discovery);
}

#[tokio::test]
async fn test_enable_encrypted_discovery() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: true,
        success: true,
    });
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let mode = coordinator
        .enable_encrypted_discovery("family-1")
        .await
        .expect("enable_encrypted_discovery should succeed");
    assert_eq!(mode, DiscoveryMode::Encrypted);
}

#[tokio::test]
async fn test_enable_encrypted_discovery_fails_when_not_encrypted() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: false,
        success: true,
    });
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let err = coordinator
        .enable_encrypted_discovery("family-1")
        .await
        .expect_err("should fail when broadcast not encrypted");
    assert!(err.to_string().contains("encrypted"));
}

#[tokio::test]
async fn test_enable_encrypted_discovery_fails_when_test_unsuccessful() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: true,
        success: false,
    });
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let err = coordinator
        .enable_encrypted_discovery("family-1")
        .await
        .expect_err("should fail when test unsuccessful");
    assert!(err.to_string().contains("unsuccessful") || err.to_string().contains("verification"));
}

#[tokio::test]
async fn test_disable_encrypted_discovery() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: false,
        success: true,
    });
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let mode = coordinator
        .disable_encrypted_discovery()
        .await
        .expect("disable_encrypted_discovery should succeed");
    assert_eq!(mode, DiscoveryMode::Plaintext);
}

#[tokio::test]
async fn test_get_discovery_mode_encrypted() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: true,
        success: true,
    });
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let mode = coordinator
        .get_discovery_mode()
        .await
        .expect("get_discovery_mode should succeed");
    assert_eq!(mode, DiscoveryMode::Encrypted);
}

#[tokio::test]
async fn test_get_discovery_mode_plaintext() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: false,
        success: true,
    });
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let mode = coordinator
        .get_discovery_mode()
        .await
        .expect("get_discovery_mode should succeed");
    assert_eq!(mode, DiscoveryMode::Plaintext);
}

#[tokio::test]
async fn test_disable_encrypted_discovery_fails_when_provider_errors() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(FailEnableDiscovery),
    );
    let err = coordinator
        .disable_encrypted_discovery()
        .await
        .expect_err("should fail when discovery provider errors");
    let chain = format!("{err:#}");
    assert!(chain.contains("enable-mode-fail"), "got: {chain}");
}

#[tokio::test]
async fn test_get_discovery_mode_error_propagates() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(FailBroadcastDiscovery),
    );
    let err = coordinator
        .get_discovery_mode()
        .await
        .expect_err("should propagate broadcast test error");
    let chain = format!("{err:#}");
    assert!(chain.contains("broadcast-test-fail"), "got: {chain}");
}

#[tokio::test]
async fn test_enable_encrypted_discovery_broadcast_test_error_propagates() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(FailBroadcastDiscovery),
    );
    let err = coordinator
        .enable_encrypted_discovery("fam")
        .await
        .expect_err("broadcast test error should propagate");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("broadcast-test-fail") || chain.contains("test encrypted"),
        "got: {chain}"
    );
}

#[tokio::test]
async fn test_enable_encrypted_discovery_generate_keys_error() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(FailGenerateKeysSecurity),
        Arc::new(MockDiscoveryProvider {
            encrypted: true,
            success: true,
        }),
    );
    let err = coordinator
        .enable_encrypted_discovery("fam")
        .await
        .expect_err("key generation error should propagate");
    let chain = format!("{err:#}");
    assert!(chain.contains("generate-keys-fail"), "got: {chain}");
}
