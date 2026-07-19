// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::socket_dir_path;
use super::common::mock_env;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn socket_dir_path_uses_biomeos_socket_dir() {
    let mut env = HashMap::new();
    env.insert(
        "BIOMEOS_SOCKET_DIR".to_string(),
        "/custom/sockets".to_string(),
    );
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/custom/sockets"));
}

#[test]
fn socket_dir_path_uses_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/run/user/1000/membrane/sockets"));
}

#[test]
fn socket_dir_path_falls_back_to_family_tmp() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "gamma".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/membrane-gamma/sockets"));
}

#[test]
fn socket_dir_prefers_biomeos_socket_dir_over_xdg() {
    let mut env = HashMap::new();
    env.insert(
        "BIOMEOS_SOCKET_DIR".to_string(),
        "/sock/override".to_string(),
    );
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1".to_string());
    assert_eq!(
        socket_dir_path(&mock_env(&env)),
        PathBuf::from("/sock/override")
    );
}

#[test]
fn socket_dir_path_defaults_without_any_env() {
    let env: HashMap<String, String> = HashMap::new();
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/membrane-default/sockets"));
}

#[test]
fn socket_dir_path_uses_biomeos_family_id() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "beta".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/membrane-beta/sockets"));
}

#[test]
fn socket_dir_path_xdg_over_family_fallback() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/42".to_string());
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "ignored".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/run/user/42/membrane/sockets"));
}
