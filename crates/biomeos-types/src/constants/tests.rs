// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;

#[test]
fn test_version_constants() {
    assert_eq!(version::TYPES_VERSION, version::VERSION);
    assert_eq!(version::API_VERSION, "biomeOS/v1");
    assert_eq!(version::MCP_PROTOCOL_VERSION, "1.0");
}

#[test]
fn test_endpoint_constants() {
    assert_eq!(endpoints::DEFAULT_LOCALHOST, "127.0.0.1");
    assert_eq!(endpoints::PRODUCTION_BIND_ADDRESS, "0.0.0.0");
    assert_eq!(endpoints::HEALTH_ENDPOINT, "/health");
    assert_eq!(endpoints::METRICS_ENDPOINT, "/metrics");
    assert_eq!(endpoints::DISCOVERY_ENDPOINT, "/discovery");
}

#[test]
fn test_tcp_bind_addr_with_host() {
    let default = endpoints::tcp_bind_addr_with_host(None, 9000);
    assert_eq!(default.port(), 9000);
    assert_eq!(default.ip(), std::net::Ipv4Addr::LOCALHOST);

    let localhost = endpoints::tcp_bind_addr_with_host(Some("127.0.0.1"), 9000);
    assert_eq!(localhost.port(), 9000);
    assert_eq!(localhost.ip(), std::net::Ipv4Addr::LOCALHOST);

    let all_interfaces = endpoints::tcp_bind_addr_with_host(Some("0.0.0.0"), 9000);
    assert_eq!(all_interfaces.port(), 9000);
    assert_eq!(all_interfaces.ip(), std::net::Ipv4Addr::UNSPECIFIED);

    let ipv6 = endpoints::tcp_bind_addr_with_host(Some("::1"), 8080);
    assert_eq!(ipv6.port(), 8080);
    assert!(ipv6.ip().is_loopback());

    let full_addr = endpoints::tcp_bind_addr_with_host(Some("10.0.0.1:3000"), 9000);
    assert_eq!(full_addr.port(), 3000);

    let invalid = endpoints::tcp_bind_addr_with_host(Some("not-an-ip"), 9000);
    assert_eq!(invalid.port(), 9000);
    assert_eq!(invalid.ip(), std::net::Ipv4Addr::LOCALHOST);
}

#[test]
fn test_default_tcp_bind_addr_is_localhost() {
    let addr = endpoints::default_tcp_bind_addr(8080);
    assert_eq!(addr.ip(), std::net::Ipv4Addr::LOCALHOST);
    assert_eq!(addr.port(), 8080);
}

#[test]
fn test_timeout_constants() {
    assert_eq!(timeouts::DEFAULT_CONNECTION_TIMEOUT.as_secs(), 30);
    assert_eq!(timeouts::DEFAULT_REQUEST_TIMEOUT.as_secs(), 30);
    assert_eq!(timeouts::DEFAULT_OPERATION_TIMEOUT.as_secs(), 60);
    assert_eq!(timeouts::DEFAULT_SESSION_TIMEOUT.as_secs(), 3600);
    assert_eq!(timeouts::DEFAULT_RETRY_DELAY.as_millis(), 1000);
    assert_eq!(timeouts::DEFAULT_DISCOVERY_TIMEOUT_MS, 5000);
    assert_eq!(timeouts::DEFAULT_CONNECTION_TIMEOUT_MS, 5000);
    assert_eq!(timeouts::SHORT_TIMEOUT_MS, 3000);
}

#[test]
fn test_limit_constants() {
    assert_eq!(limits::DEFAULT_MAX_CONNECTIONS, 1000);
    assert_eq!(limits::DEFAULT_BUFFER_SIZE, 8192);
    assert_eq!(limits::DEFAULT_MAX_MESSAGE_SIZE, 1024 * 1024);
    assert_eq!(limits::DEFAULT_RATE_LIMIT_PER_MINUTE, 100);
}

#[test]
fn test_network_constants() {
    assert_eq!(network::DEFAULT_HTTP_PORT, 8080);
    assert_eq!(network::DEFAULT_HTTPS_PORT, 8443);
    assert_eq!(network::DEFAULT_WS_PORT, 8081);
    assert_eq!(network::DEFAULT_MCP_PORT, 3000);
    assert_eq!(network::DEFAULT_BEARDOG_PORT, 9000);
    assert_eq!(network::DEFAULT_SONGBIRD_PORT, 3000);
    assert_eq!(network::DEFAULT_BROADCAST_DISCOVERY_PORT, 9199);
    assert_eq!(network::DEFAULT_DEV_PORT, 5000);
    assert_eq!(network::DEFAULT_USER_AGENT, "biomeOS/1.0");
    assert_eq!(network::DEFAULT_CONTENT_TYPE, "application/json");
}

#[test]
fn test_security_constants() {
    assert_eq!(security::DEFAULT_AUTH_TIMEOUT.as_secs(), 300);
    assert_eq!(security::DEFAULT_TOKEN_EXPIRY.as_secs(), 3600);
    assert_eq!(security::DEFAULT_LOCKOUT_DURATION.as_secs(), 1800);
}

#[test]
fn test_capability_constants() {
    assert_eq!(capabilities::COMPUTE, "compute");
    assert_eq!(capabilities::STORAGE, "storage");
    assert_eq!(capabilities::SECURITY, "security");
    assert_eq!(capabilities::AI, "ai");
    assert_eq!(capabilities::DISCOVERY, "discovery");
    assert_eq!(capabilities::ORCHESTRATION, "orchestration");
}

#[test]
fn test_files_plugin_dir() {
    let dir = files::default_plugin_dir("squirrel");
    assert_eq!(dir, ".squirrel/plugins");

    let dir2 = files::default_plugin_dir("beardog");
    assert_eq!(dir2, ".beardog/plugins");
}

#[test]
fn test_files_constants() {
    assert_eq!(files::DEFAULT_CONFIG_FILE, "biome.yaml");
    assert_eq!(files::DEFAULT_RULES_DIR, ".rules");
    assert_eq!(files::DEFAULT_HISTORY_FILE, "command_history.json");
}

#[test]
fn test_runtime_ipc_neural_api_basename_prefix() {
    assert_eq!(runtime_ipc::NEURAL_API_BASENAME_PREFIX, "neural-api-");
}

#[test]
fn test_event_constants() {
    assert_eq!(events::PLUGIN_INITIALIZED, "plugin.initialized");
    assert_eq!(events::SYSTEM_READY, "system.ready");
    assert_eq!(events::COMMAND_EXECUTED, "command.executed");
}

#[test]
fn test_env_var_constants() {
    assert_eq!(env_vars::BIND_ADDRESS, "BIND_ADDRESS");
    assert_eq!(env_vars::HTTP_PORT, "HTTP_PORT");
    assert_eq!(env_vars::MAX_CONNECTIONS, "MAX_CONNECTIONS");
}

#[test]
fn test_network_accessors() {
    let _ = network::http_port();
    let _ = network::https_port();
    let _ = network::websocket_port();
    let _ = network::mcp_port();
    let _ = network::discovery_port();
    assert_eq!(network::DEFAULT_HTTP_PORT, 8080);
    assert_eq!(network::DEFAULT_DISCOVERY_PORT, 8001);
    assert_eq!(network::LINK_LOCAL_RANGE, "169.254.0.0/16");
    assert_eq!(network::PRIVATE_CLASS_A, "10.0.0.0/8");
    assert_eq!(network::PRIVATE_CLASS_B, "172.16.0.0/12");
    assert_eq!(network::PRIVATE_CLASS_C, "192.168.0.0/16");
    assert_eq!(network::DEFAULT_MCP_SUBPROTOCOL, "mcp");
}

#[test]
fn test_endpoints_bind_address() {
    let addr = endpoints::bind_address();
    assert!(!addr.is_empty());
    assert!(addr.contains('.') || addr.contains(':'));
}

#[test]
fn test_endpoints_production_bind_address() {
    let addr = endpoints::production_bind_address();
    assert!(!addr.is_empty());
}

#[test]
fn test_files_current_primal_plugin_dir() {
    let dir = files::current_primal_plugin_dir();
    assert!(dir.contains("plugins"));
}

#[test]
fn test_limits_constants() {
    assert_eq!(limits::DEFAULT_SERVICE_MESH_MAX_SERVICES, 100);
    assert_eq!(limits::DEFAULT_MEMORY_LIMIT_MB, 1024);
    assert_eq!(limits::DEFAULT_CPU_LIMIT_MILLICORES, 1000);
    assert_eq!(limits::DEFAULT_DISK_LIMIT_GB, 10);
}

#[test]
fn test_timeouts_more_constants() {
    assert_eq!(timeouts::DEFAULT_HEALTH_CHECK_TIMEOUT.as_secs(), 10);
    assert_eq!(timeouts::DEFAULT_HEALTH_CHECK_INTERVAL.as_secs(), 30);
    assert_eq!(timeouts::DEFAULT_CACHE_TTL.as_secs(), 300);
    assert_eq!(timeouts::DEFAULT_HEARTBEAT_INTERVAL.as_secs(), 30);
}

#[test]
fn test_events_all_constants() {
    assert_eq!(events::PLUGIN_STOPPED, "plugin.stopped");
    assert_eq!(events::PLUGIN_ERROR, "plugin.error");
    assert_eq!(events::COMMAND_FAILED, "command.failed");
    assert_eq!(events::SYSTEM_SHUTDOWN, "system.shutdown");
    assert_eq!(events::CUSTOM_EVENT, "custom.event");
}

#[test]
fn test_files_base64_alphabet() {
    assert_eq!(files::BASE64_ALPHABET.len(), 64);
    assert!(files::SIZE_UNITS.contains(&"MB"));
    assert!(files::SIZE_UNITS.contains(&"GB"));
}

#[test]
fn test_capabilities_all() {
    assert_eq!(capabilities::VISUALIZATION, "visualization");
    assert_eq!(capabilities::NETWORKING, "networking");
    assert_eq!(capabilities::MONITORING, "monitoring");
    assert_eq!(capabilities::DATA_PROCESSING, "data-processing");
}

#[test]
fn test_capability_domain_constants() {
    assert_eq!(capability::CRYPTO, "crypto");
    assert_eq!(capability::MESH_NETWORKING, "mesh_networking");
    assert_eq!(capability::STORAGE, "storage");
    assert_eq!(capability::GATEWAY, "gateway");
    assert_eq!(capability::CACHING, "caching");
    assert_eq!(capability::GRAPH_DATABASE, "graph_database");
    assert_eq!(capability::PERSISTENCE, "persistence");
    assert_eq!(capability::GPU_COMPUTE, "gpu_compute");
    assert_eq!(capability::SIGNING, "crypto.sign");
    assert_eq!(capability::ENCRYPTION, "crypto.encrypt");
}

#[test]
fn test_network_http_port_from_env() {
    assert_eq!(network::http_port_from(Some("9999")), 9999);
}

#[test]
fn test_network_http_port_invalid_env_falls_back() {
    assert_eq!(
        network::http_port_from(Some("not_a_number")),
        network::DEFAULT_HTTP_PORT
    );
}

#[test]
fn test_network_https_port_from_env() {
    assert_eq!(network::https_port_from(Some("4443")), 4443);
}

#[test]
fn test_network_websocket_port_from_env() {
    assert_eq!(network::websocket_port_from(Some("7777")), 7777);
}

#[test]
fn test_network_mcp_port_from_env() {
    assert_eq!(network::mcp_port_from(Some("5555")), 5555);
}

#[test]
fn test_network_discovery_port_from_env() {
    assert_eq!(network::discovery_port_from(Some("6666")), 6666);
}

#[test]
fn test_endpoints_bind_address_from_env() {
    assert_eq!(endpoints::bind_address_from(Some("10.0.0.1")), "10.0.0.1");
    assert_eq!(
        endpoints::production_bind_address_from(Some("10.0.0.1")),
        "10.0.0.1"
    );
}

#[test]
fn test_ports_test_default() {
    assert_eq!(ports::TEST_DEFAULT, 8083);
}

#[test]
fn test_observability_and_registry_ports_match_endpoint_strings() {
    assert_eq!(ports::STATSD, 8125);
    assert_eq!(ports::ZIPKIN_HTTP, 9411);
    assert_eq!(ports::REGISTRY_HTTP, 9999);
    assert!(endpoints::DEFAULT_STATSD_UDP_ENDPOINT.contains("8125"));
    assert!(endpoints::DEFAULT_ZIPKIN_HTTP_ENDPOINT.contains("9411"));
    assert!(endpoints::DEFAULT_REGISTRY_HTTP_URL.contains("9999"));
    let addr = endpoints::production_tcp_bind_addr(9000);
    assert_eq!(addr.port(), 9000);
}

#[test]
fn test_runtime_paths_fallback_dir() {
    let default = runtime_paths::fallback_runtime_dir("");
    assert_eq!(default, std::path::PathBuf::from("/tmp/biomeos"));

    let family = runtime_paths::fallback_runtime_dir("abc123");
    assert_eq!(family, std::path::PathBuf::from("/tmp/biomeos-abc123"));
}

#[test]
fn test_network_remaining_constants() {
    assert_eq!(network::MULTICAST_RANGE, "224.0.0.0/4");
    assert_eq!(network::DEFAULT_USER_AGENT, "biomeOS/1.0");
    assert_eq!(network::DEFAULT_CONTENT_TYPE, "application/json");
    assert_eq!(network::DEFAULT_DEV_PORT, 5000);
    assert_eq!(network::DEFAULT_BROADCAST_DISCOVERY_PORT, 9199);
}
