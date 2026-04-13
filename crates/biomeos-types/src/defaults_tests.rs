// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for defaults.rs

#![expect(clippy::unwrap_used, reason = "test")]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::defaults::*;

#[test]
fn test_socket_path_with_env_var() {
    let custom_path = "/custom/path/test.sock";
    let mut env = HashMap::new();
    env.insert("TEST_SERVICE_SOCKET".to_string(), custom_path.to_string());

    let path = socket_path_with("test-service", &env).unwrap();
    assert_eq!(path.to_str().unwrap(), custom_path);
}

#[test]
fn test_socket_path_fallback() {
    let env: HashMap<String, String> = HashMap::new();

    let path = socket_path_with("unknown-service", &env).unwrap();
    assert!(path.to_str().unwrap().ends_with("unknown-service.sock"));
}

#[test]
fn test_socket_path_with_socket_dir() {
    // Use unique service name to avoid env var collisions
    let unique_svc = "socket-dir-test-83726";
    let mut env = HashMap::new();
    env.insert("BIOMEOS_SOCKET_DIR".to_string(), "/run/biomeos".to_string());

    let path = socket_path_with(unique_svc, &env).unwrap();
    let path_str = path.to_str().unwrap();
    assert_eq!(path_str, format!("/run/biomeos/{unique_svc}.sock"));
}

#[test]
fn test_socket_path_env_var_takes_precedence() {
    // Both env var and socket dir set - env var should win
    let mut env = HashMap::new();
    env.insert(
        "PRECEDENCE_TEST_SOCKET".to_string(),
        "/explicit/socket.sock".to_string(),
    );
    env.insert("BIOMEOS_SOCKET_DIR".to_string(), "/run/biomeos".to_string());

    let path = socket_path_with("precedence-test", &env).unwrap();
    assert_eq!(path.to_str().unwrap(), "/explicit/socket.sock");
}

#[test]
fn test_socket_path_normalizes_hyphens() {
    // Hyphens should be converted to underscores in env var name
    let mut env = HashMap::new();
    env.insert(
        "NEURAL_API_SOCKET".to_string(),
        "/test/neural-api.sock".to_string(),
    );

    let path = socket_path_with("neural-api", &env).unwrap();
    assert_eq!(path.to_str().unwrap(), "/test/neural-api.sock");
}

#[test]
fn test_join_socket_path_basic() {
    let path = join_socket_path("/run", "neural-api");
    assert_eq!(path.to_str().unwrap(), "/run/neural-api.sock");
}

#[test]
fn test_join_socket_path_with_subdir() {
    let path = join_socket_path("/var/run/biomeos", "beardog");
    assert_eq!(path.to_str().unwrap(), "/var/run/biomeos/beardog.sock");
}

#[test]
fn test_runtime_config() {
    let env: HashMap<String, String> = HashMap::new();
    let config = RuntimeConfig::with_socket_dir("/test");

    assert!(config.neural_api_socket_with(&env).starts_with("/test"));
    assert!(
        config
            .service_socket_with("beardog", &env)
            .starts_with("/test")
    );
}

#[test]
fn test_runtime_config_from_env() {
    let config = RuntimeConfig::from_env_with(Some("/tmp/biomeos"), None);
    let socket_path = config.neural_api_socket();
    let path_str = socket_path.to_string_lossy();
    assert!(
        path_str.contains("biomeos") || path_str.starts_with(DEFAULT_SOCKET_DIR),
        "Socket path should be XDG-resolved or fallback: {path_str}"
    );
}

#[test]
fn test_runtime_config_from_env_with_custom_dir() {
    let mut env = HashMap::new();
    env.insert(
        "BIOMEOS_SOCKET_DIR".to_string(),
        "/custom/socket/dir".to_string(),
    );
    let config = RuntimeConfig::from_env_with_map(&env, None, None);

    assert!(
        config
            .neural_api_socket_with(&env)
            .starts_with("/custom/socket/dir")
    );
}

#[test]
fn test_runtime_config_all_socket_methods() {
    let config = RuntimeConfig::with_socket_dir("/run/biomeos");

    assert!(config.neural_api_socket().ends_with("neural-api.sock"));
    // Use service_socket() for all primals (deprecated per-primal methods removed)
    assert!(config.service_socket("beardog").ends_with("beardog.sock"));
    assert!(config.service_socket("songbird").ends_with("songbird.sock"));
    assert!(config.service_socket("squirrel").ends_with("squirrel.sock"));
    assert!(config.service_socket("nestgate").ends_with("nestgate.sock"));
    assert!(
        config
            .service_socket("toadstool")
            .ends_with("toadstool.sock")
    );
    assert!(
        config
            .service_socket("petaltongue")
            .ends_with("petaltongue.sock")
    );
}

#[test]
fn test_runtime_config_socket_env_override() {
    let mut env = HashMap::new();
    env.insert(
        "BEARDOG_SOCKET".to_string(),
        "/override/beardog.sock".to_string(),
    );
    let config = RuntimeConfig::with_socket_dir("/default");

    let beardog_path = config.service_socket_with("beardog", &env);
    assert_eq!(beardog_path.to_str().unwrap(), "/override/beardog.sock");
}

#[test]
fn test_runtime_config_http_port_default() {
    let env: HashMap<String, String> = HashMap::new();
    let port = RuntimeConfig::http_port_with(&env);

    assert_eq!(port, 8080);
}

#[test]
fn test_runtime_config_http_port_env_override() {
    let mut env = HashMap::new();
    env.insert("HTTP_PORT".to_string(), "9999".to_string());
    let port = RuntimeConfig::http_port_with(&env);

    assert_eq!(port, 9999);
}

#[test]
fn test_runtime_config_mcp_port_fallback() {
    let env: HashMap<String, String> = HashMap::new();
    let port = RuntimeConfig::mcp_port_with(&env);

    assert_eq!(port, 3000);
}

#[test]
fn test_runtime_config_mcp_port_websocket_env() {
    let mut env = HashMap::new();
    env.insert("MCP_WEBSOCKET_PORT".to_string(), "8765".to_string());
    let port = RuntimeConfig::mcp_port_with(&env);

    assert_eq!(port, 8765);
}

#[test]
fn test_runtime_config_bind_address_default() {
    let env: HashMap<String, String> = HashMap::new();
    let addr = RuntimeConfig::bind_address_with(&env);

    assert_eq!(addr, "::1");
}

#[test]
fn test_runtime_config_bind_address_env_override() {
    let test_addr = "192.168.255.254";
    let mut env = HashMap::new();
    env.insert("BIND_ADDRESS".to_string(), test_addr.to_string());

    let addr = RuntimeConfig::bind_address_with(&env);
    assert_eq!(addr, test_addr);
}

#[test]
fn test_runtime_config_service_socket() {
    let env: HashMap<String, String> = HashMap::new();
    let config = RuntimeConfig::with_socket_dir("/run/biomeos");

    let socket = config.service_socket_with("custom-primal", &env);
    assert!(socket.ends_with("custom-primal.sock"));
    assert!(socket.starts_with("/run/biomeos"));
}

#[test]
fn test_join_socket_path() {
    let path = join_socket_path("/run", "test");
    assert_eq!(path.to_str().unwrap(), "/run/test.sock");
}

#[test]
fn test_join_socket_path_various_dirs() {
    assert_eq!(
        join_socket_path("/tmp", "neural-api").to_str().unwrap(),
        "/tmp/neural-api.sock"
    );
    assert_eq!(
        join_socket_path("/run/biomeos", "beardog")
            .to_str()
            .unwrap(),
        "/run/biomeos/beardog.sock"
    );
}

#[test]
fn test_default_constants() {
    assert_eq!(DEFAULT_SOCKET_DIR, "/tmp");
    assert_eq!(DEFAULT_NEURAL_API_SOCKET, "neural-api.sock");
}

#[test]
fn test_service_socket_generates_correct_names() {
    // Verify service_socket() generates the same names the old constants had
    let config = RuntimeConfig::with_socket_dir("/run/biomeos");
    for primal in &[
        "beardog",
        "songbird",
        "squirrel",
        "nestgate",
        "toadstool",
        "petaltongue",
    ] {
        let socket = config.service_socket(primal);
        assert!(
            socket.ends_with(format!("{primal}.sock").as_str()),
            "Expected {primal}.sock, got {socket:?}"
        );
    }
}

#[test]
fn test_env_vars_constants() {
    assert_eq!(env_vars::NEURAL_API_SOCKET, "NEURAL_API_SOCKET");
    assert_eq!(env_vars::BEARDOG_SOCKET, "BEARDOG_SOCKET");
    assert_eq!(env_vars::SONGBIRD_SOCKET, "SONGBIRD_SOCKET");
    assert_eq!(env_vars::SOCKET_DIR, "BIOMEOS_SOCKET_DIR");
}

#[test]
fn test_runtime_config_clone() {
    let config = RuntimeConfig::with_socket_dir("/test");
    let cloned = config.clone();

    assert_eq!(
        config.neural_api_socket().to_str(),
        cloned.neural_api_socket().to_str()
    );
}

#[test]
fn test_runtime_config_debug() {
    let config = RuntimeConfig::with_socket_dir("/test");
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("RuntimeConfig"));
    assert!(debug_str.contains("/test"));
}

#[test]
fn test_https_port_default() {
    let env: HashMap<String, String> = HashMap::new();
    assert_eq!(RuntimeConfig::https_port_with(&env), 8443);
}

#[test]
fn test_https_port_env_override() {
    let mut env = HashMap::new();
    env.insert("HTTPS_PORT".to_string(), "9443".to_string());
    assert_eq!(RuntimeConfig::https_port_with(&env), 9443);
}

#[test]
fn test_websocket_port_default() {
    let env: HashMap<String, String> = HashMap::new();
    assert_eq!(RuntimeConfig::websocket_port_with(&env), 8081);
}

#[test]
fn test_websocket_port_env_override() {
    let mut env = HashMap::new();
    env.insert("WEBSOCKET_PORT".to_string(), "9081".to_string());
    assert_eq!(RuntimeConfig::websocket_port_with(&env), 9081);
}

#[test]
fn test_discovery_port_default() {
    let env: HashMap<String, String> = HashMap::new();
    assert_eq!(RuntimeConfig::discovery_port_with(&env), 8001);
}

#[test]
fn test_discovery_port_env_override() {
    let mut env = HashMap::new();
    env.insert("DISCOVERY_PORT".to_string(), "9001".to_string());
    assert_eq!(RuntimeConfig::discovery_port_with(&env), 9001);
}

#[test]
fn test_mcp_port_mcp_env_fallback() {
    let mut env = HashMap::new();
    env.insert("MCP_PORT".to_string(), "4000".to_string());
    assert_eq!(RuntimeConfig::mcp_port_with(&env), 4000);
}

#[test]
fn test_bind_address_biomeos_precedence() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_BIND_ADDRESS".to_string(), "127.0.0.1".to_string());
    env.insert("BIND_ADDRESS".to_string(), "0.0.0.0".to_string());
    assert_eq!(RuntimeConfig::bind_address_with(&env), "127.0.0.1");
}

#[test]
fn test_runtime_config_default_impl() {
    let config = RuntimeConfig::default();
    assert!(!config.socket_dir().as_os_str().is_empty());
}

#[test]
fn test_neural_api_socket_env_override() {
    let mut env = HashMap::new();
    env.insert(
        "NEURAL_API_SOCKET".to_string(),
        "/custom/neural.sock".to_string(),
    );
    let config = RuntimeConfig::with_socket_dir("/default");
    let path = config.neural_api_socket_with(&env);
    assert_eq!(path.to_str().unwrap(), "/custom/neural.sock");
}

#[test]
fn test_socket_path_empty_service_name() {
    let path = socket_path("");
    assert!(path.is_ok());
    assert!(path.unwrap().to_string_lossy().ends_with(".sock"));
}

#[test]
fn test_http_port_invalid_parse_fallback() {
    let mut env = HashMap::new();
    env.insert("HTTP_PORT".to_string(), "not_a_number".to_string());
    assert_eq!(RuntimeConfig::http_port_with(&env), 8080);
}

#[test]
fn test_runtime_config_from_env_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert(
        "XDG_RUNTIME_DIR".to_string(),
        "/tmp/xdg-test-12345".to_string(),
    );
    let config = RuntimeConfig::from_env_with_map(&env, None, None);
    let socket_dir = config.socket_dir();
    assert!(socket_dir.to_string_lossy().contains("biomeos"));
    assert!(socket_dir.to_string_lossy().contains("xdg-test"));
}

#[test]
fn test_join_socket_path_with_pathbuf() {
    let dir = PathBuf::from("/var/run");
    let path = join_socket_path(dir, "myservice");
    assert_eq!(path.to_str().unwrap(), "/var/run/myservice.sock");
}

#[test]
fn test_runtime_config_socket_dir_accessor() {
    let config = RuntimeConfig::with_socket_dir("/run/biomeos");
    assert_eq!(config.socket_dir(), Path::new("/run/biomeos"));
}

#[test]
fn test_service_socket_env_override_takes_precedence() {
    let mut env = HashMap::new();
    env.insert(
        "OVERRIDE_SVC_SOCKET".to_string(),
        "/absolute/override.sock".to_string(),
    );
    let config = RuntimeConfig::with_socket_dir("/default/dir");
    let path = config.service_socket_with("override-svc", &env);
    assert_eq!(path.to_str().unwrap(), "/absolute/override.sock");
}

#[test]
fn test_env_vars_all_constants() {
    assert_eq!(env_vars::SQUIRREL_SOCKET, "SQUIRREL_SOCKET");
    assert_eq!(env_vars::NESTGATE_SOCKET, "NESTGATE_SOCKET");
    assert_eq!(env_vars::TOADSTOOL_SOCKET, "TOADSTOOL_SOCKET");
    assert_eq!(env_vars::PETALTONGUE_SOCKET, "PETALTONGUE_SOCKET");
    assert_eq!(
        env_vars::DISCOVERY_REGISTRY_SOCKET,
        "DISCOVERY_REGISTRY_SOCKET"
    );
}
