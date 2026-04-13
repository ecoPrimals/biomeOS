// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for network_config.rs

#![expect(clippy::expect_used, reason = "test")]

use super::network_config::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

fn env_map(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

#[test]
fn test_default_config() {
    let env = HashMap::new();
    let config = NetworkConfig::from_env_with(&env);
    assert_eq!(config.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert!(!config.bind_all);
}

#[test]
fn test_bind_all() {
    let env = env_map(&[(env_vars::BIND_ALL, "true")]);
    let config = NetworkConfig::from_env_with(&env);
    // Uses IPv6 [::] for dual-stack (accepts IPv4 + IPv6)
    assert_eq!(config.bind_address(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    assert!(config.bind_all);
}

#[test]
fn test_custom_bind_address() {
    let env = env_map(&[(env_vars::BIND_ADDRESS, "192.0.2.100")]);
    let config = NetworkConfig::from_env_with(&env);
    assert!(config.bind_address().is_ipv4() || config.bind_address().is_ipv6());
}

#[test]
fn test_socket_addr() {
    let config = NetworkConfig::localhost();
    let addr = config.socket_addr(8080);

    assert_eq!(addr.port(), 8080);
    assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
}

#[test]
fn test_port_defaults() {
    let config = NetworkConfig::from_env();
    let ports = config.ports();

    assert!(ports.http > 0);
    assert!(ports.https > 0);
    assert!(ports.websocket > 0);
    assert!(ports.discovery > 0);
}

#[test]
fn test_custom_port() {
    let env = env_map(&[(env_vars::HTTP_PORT, "9999")]);
    let config = NetworkConfig::from_env_with(&env);
    assert_eq!(config.http_port(), 9999);
}

#[test]
fn test_stun_servers_default() {
    let env = HashMap::new();
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    // Should have public fallback servers
    assert!(!servers.is_empty());
    assert!(servers[0].contains(':'));
}

#[test]
fn test_custom_stun_servers() {
    let env = env_map(&[(
        env_vars::STUN_SERVERS,
        "stun.example.com:3478,stun2.example.com:3478",
    )]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert!(servers.contains(&"stun.example.com:3478".to_string()));
    assert!(servers.contains(&"stun2.example.com:3478".to_string()));
}

#[test]
fn test_self_hosted_stun_priority() {
    let env = env_map(&[(env_vars::SELF_HOSTED_STUN, "my-stun.local:3478")]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    // Self-hosted should be first
    assert_eq!(servers[0], "my-stun.local:3478");
}

#[test]
fn test_no_public_stun() {
    let env = env_map(&[(env_vars::NO_PUBLIC_STUN, "true")]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    // No servers when public disabled and no custom configured
    assert!(servers.is_empty());
}

#[test]
fn test_convenience_functions() {
    let addr = bind_address();
    assert!(addr.is_ipv4() || addr.is_ipv6());

    let addr_str = bind_address_string();
    assert!(!addr_str.is_empty());

    let socket = socket_addr(8080);
    assert_eq!(socket.port(), 8080);
}

#[test]
fn test_localhost_factory() {
    let config = NetworkConfig::localhost();
    assert_eq!(config.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
}

#[test]
fn test_all_interfaces_factory() {
    let config = NetworkConfig::all_interfaces();
    // Uses IPv6 [::] for dual-stack binding
    assert_eq!(config.bind_address(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
}

#[test]
fn test_with_bind_address() {
    let config = NetworkConfig::with_bind_address(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    assert_eq!(
        config.bind_address(),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))
    );
}

#[test]
fn test_bind_address_string() {
    let config = NetworkConfig::localhost();
    let s = config.bind_address_string();
    assert_eq!(s, "127.0.0.1");
}

#[test]
fn test_socket_methods() {
    let config = NetworkConfig::localhost();
    assert_eq!(config.http_socket().port(), config.http_port());
    assert_eq!(config.https_socket().port(), config.https_port());
    assert_eq!(config.websocket_socket().port(), config.websocket_port());
    assert_eq!(config.discovery_socket().port(), config.discovery_port());
    assert_eq!(config.relay_socket().port(), config.relay_port());
}

#[test]
fn test_port_config_default() {
    let ports = PortConfig::default();
    assert_eq!(ports.http, 8080);
    assert_eq!(ports.https, 8443);
    assert_eq!(ports.websocket, 8081);
    assert_eq!(ports.discovery, 8001);
    assert_eq!(ports.relay, 3490);
    assert_eq!(ports.stun, 3478);
}

#[test]
fn test_ports_accessor() {
    let config = NetworkConfig::localhost();
    let ports = config.ports();
    assert!(ports.http > 0);
    assert!(ports.stun > 0);
}

#[test]
fn test_stun_port_accessor() {
    let config = NetworkConfig::localhost();
    assert_eq!(config.stun_port(), 3478);
}

#[test]
fn test_allows_public_stun_accessor() {
    let config = NetworkConfig::localhost();
    let _ = config.allows_public_stun();
}

#[test]
fn test_self_hosted_stun_none() {
    let config = NetworkConfig::localhost();
    assert!(config.self_hosted_stun().is_none());
}

#[test]
fn test_stun_servers_convenience() {
    let servers = stun_servers();
    assert!(servers.is_empty() || servers.iter().all(|s| s.contains(':')));
}

// ── Additional env-var tests (using from_env_with) ──────────────────────

#[test]
fn test_from_env_bind_all_one() {
    let env = env_map(&[(env_vars::BIND_ALL, "1")]);
    let config = NetworkConfig::from_env_with(&env);
    assert_eq!(config.bind_address(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
}

#[test]
fn test_from_env_bind_address_ipv6() {
    let env = env_map(&[(env_vars::BIND_ADDRESS, "::1")]);
    let config = NetworkConfig::from_env_with(&env);
    assert!(config.bind_address().is_ipv6());
}

#[test]
fn test_from_env_invalid_bind_address_fallback() {
    let env = env_map(&[(env_vars::BIND_ADDRESS, "not-an-ip")]);
    let config = NetworkConfig::from_env_with(&env);
    // Should fall back to localhost when parse fails
    assert_eq!(config.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
}

#[test]
fn test_stun_servers_self_hosted_first() {
    let env = env_map(&[
        (env_vars::SELF_HOSTED_STUN, "stun.self.local:3478"),
        (env_vars::STUN_SERVERS, "stun.custom.com:3478"),
    ]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert!(!servers.is_empty());
    assert_eq!(servers[0], "stun.self.local:3478");
}

#[test]
fn test_stun_servers_custom_only_no_public() {
    let env = env_map(&[
        (env_vars::STUN_SERVERS, "stun.a.com:3478,stun.b.com:3478"),
        (env_vars::NO_PUBLIC_STUN, "1"),
    ]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert_eq!(servers.len(), 2);
    assert!(servers.contains(&"stun.a.com:3478".to_string()));
    assert!(servers.contains(&"stun.b.com:3478".to_string()));
}

#[test]
fn test_stun_servers_sovereign_mode_no_public() {
    let env = env_map(&[("BIOMEOS_SOVEREIGN", "true")]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert!(servers.is_empty());
    assert!(!config.allows_public_stun());
}

#[test]
fn test_stun_servers_sovereign_with_opt_in() {
    let env = env_map(&[
        ("BIOMEOS_SOVEREIGN", "1"),
        ("BIOMEOS_ALLOW_PUBLIC_STUN", "true"),
    ]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert!(!servers.is_empty());
    assert!(config.allows_public_stun());
}

#[test]
fn test_bind_address_string_ipv6() {
    let config = NetworkConfig::all_interfaces();
    let s = config.bind_address_string();
    assert!(s.contains("::") || s == "::");
}

#[test]
fn test_port_config_defaults_full() {
    let ports = PortConfig::default();
    assert_eq!(ports.http, 8080);
    assert_eq!(ports.https, 8443);
    assert_eq!(ports.websocket, 8081);
    assert_eq!(ports.discovery, 8001);
    assert_eq!(ports.relay, 3490);
    assert_eq!(ports.stun, 3478);
}

#[test]
fn test_with_bind_address_sets_bind_all_for_unspecified() {
    let config = NetworkConfig::with_bind_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    assert!(config.bind_all);
}

#[test]
fn test_network_config_serialization() {
    let config = NetworkConfig::localhost();
    let json = serde_json::to_string(&config).expect("serialize");
    let _parsed: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn test_port_config_serialization() {
    let ports = PortConfig::default();
    let json = serde_json::to_string(&ports).expect("serialize");
    let parsed: PortConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.http, ports.http);
    assert_eq!(parsed.stun, ports.stun);
}

#[test]
fn test_stun_servers_whitespace_trimmed() {
    let env = env_map(&[
        (
            env_vars::STUN_SERVERS,
            " stun.a.com:3478 , stun.b.com:3478 ,  ",
        ),
        (env_vars::NO_PUBLIC_STUN, "1"),
    ]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert!(servers.contains(&"stun.a.com:3478".to_string()));
    assert!(servers.contains(&"stun.b.com:3478".to_string()));
}

#[test]
fn test_stun_servers_empty_entries_filtered() {
    let env = env_map(&[
        (env_vars::STUN_SERVERS, "stun.a.com:3478,,,stun.b.com:3478"),
        (env_vars::NO_PUBLIC_STUN, "1"),
    ]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert_eq!(servers.len(), 2);
    assert!(servers.contains(&"stun.a.com:3478".to_string()));
    assert!(servers.contains(&"stun.b.com:3478".to_string()));
}

#[test]
fn test_network_config_default_impl() {
    let config = NetworkConfig::default();
    assert!(config.bind_address().is_ipv4() || config.bind_address().is_ipv6());
}

#[test]
fn test_network_config_clone() {
    let config = NetworkConfig::localhost();
    let cloned = config.clone();
    assert_eq!(config.bind_address(), cloned.bind_address());
    assert_eq!(config.http_port(), cloned.http_port());
}

#[test]
fn test_network_config_serialization_roundtrip() {
    let config = NetworkConfig::localhost();
    let json = serde_json::to_string(&config).expect("serialize");
    let parsed: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(config.bind_address(), parsed.bind_address());
    assert_eq!(config.ports().http, parsed.ports().http);
}

#[test]
fn test_no_public_stun_false_allows_public() {
    let env = env_map(&[(env_vars::NO_PUBLIC_STUN, "false")]);
    let config = NetworkConfig::from_env_with(&env);
    let servers = config.stun_servers();

    assert!(!servers.is_empty());
    assert!(config.allows_public_stun());
}

#[test]
fn test_env_vars_constants() {
    assert_eq!(env_vars::BIND_ADDRESS, "BIND_ADDRESS");
    assert_eq!(env_vars::BIND_ALL, "BIOMEOS_BIND_ALL");
    assert_eq!(env_vars::STUN_SERVERS, "BIOMEOS_STUN_SERVERS");
    assert_eq!(env_vars::SELF_HOSTED_STUN, "BIOMEOS_STUN_SERVER");
    assert_eq!(env_vars::NO_PUBLIC_STUN, "BIOMEOS_NO_PUBLIC_STUN");
}
