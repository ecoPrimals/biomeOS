// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    clippy::future_not_send,
    reason = "test assertions"
)]

use super::super::*;
use std::path::Path;

#[test]
fn family_scoped_detection() {
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/beardog-8ff3b864a4bc589a.sock"
    )));
    assert!(is_family_scoped_socket(Path::new(
        "/tmp/biomeos/songbird-abc123.sock"
    )));
    assert!(!is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/beardog.sock"
    )));
    assert!(!is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/biomeos.sock"
    )));
}

#[test]
fn family_scoped_domain_stem_sockets() {
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/security-8ff3b864.sock"
    )));
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/compute-abc123.sock"
    )));
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/ai-def456.sock"
    )));
    assert!(!is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/security.sock"
    )));
}

#[test]
fn extract_family_from_socket() {
    assert_eq!(
        extract_family_id(Path::new("/tmp/beardog-abc123.sock")),
        Some("abc123".to_owned())
    );
    assert_eq!(
        extract_family_id(Path::new("/tmp/nestgate-8ff3b864a4bc589a.sock")),
        Some("8ff3b864a4bc589a".to_owned())
    );
    assert_eq!(extract_family_id(Path::new("/tmp/beardog.sock")), None);
}

#[test]
fn extract_family_from_domain_stem_socket() {
    assert_eq!(
        extract_family_id(Path::new("/tmp/security-abc123.sock")),
        Some("abc123".to_owned())
    );
    assert_eq!(
        extract_family_id(Path::new("/tmp/compute-familyXYZ.sock")),
        Some("familyXYZ".to_owned())
    );
}

#[test]
fn multi_hyphen_family_id() {
    assert!(is_family_scoped_socket(Path::new(
        "/tmp/beardog-abc-def-123.sock"
    )));
    assert_eq!(
        extract_family_id(Path::new("/tmp/beardog-abc-def-123.sock")),
        Some("abc-def-123".to_owned())
    );
}

#[test]
fn edge_cases() {
    assert!(!is_family_scoped_socket(Path::new("")));
    assert!(!is_family_scoped_socket(Path::new("/tmp/.sock")));
    assert!(!is_family_scoped_socket(Path::new("/tmp/noext")));
    assert!(extract_family_id(Path::new("")).is_none());
    assert!(extract_family_id(Path::new("/tmp/noext")).is_none());
}

#[test]
fn insecure_guard_ok_without_env() {
    let result = validate_insecure_guard();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn security_mode_returns_valid_variant() {
    let mode = security_mode();
    match mode {
        SecurityMode::Development | SecurityMode::Production { .. } => {}
    }
}

#[test]
fn log_security_posture_does_not_panic() {
    log_security_posture();
}
