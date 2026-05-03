// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Network configuration constants
//!
//! **DESIGN PRINCIPLE**: These are FALLBACK defaults only.
//! Production systems MUST use environment variables or capability discovery.

use super::env_vars;
use super::ports;
use std::env;

/// Default HTTP port (fallback only)
pub const DEFAULT_HTTP_PORT: u16 = ports::HTTP_BRIDGE;

/// Default HTTPS port (fallback only)
pub const DEFAULT_HTTPS_PORT: u16 = 8443;

/// Default WebSocket port (fallback only)
pub const DEFAULT_WS_PORT: u16 = 8081;

/// Default MCP port (fallback only)
pub const DEFAULT_MCP_PORT: u16 = ports::API_DEFAULT;

/// Default discovery port (fallback only)
pub const DEFAULT_DISCOVERY_PORT: u16 = ports::WEBSOCKET;

/// Default `BearDog` (security) port (fallback only)
pub const DEFAULT_BEARDOG_PORT: u16 = ports::NEURAL_API;

/// Default Songbird (universal adapter) port (fallback only)
pub const DEFAULT_SONGBIRD_PORT: u16 = ports::API_DEFAULT;

/// Default broadcast discovery port (fallback only)
pub const DEFAULT_BROADCAST_DISCOVERY_PORT: u16 = 9199;

/// RFC 5737 TEST-NET address used as a route probe target.
///
/// Connecting a UDP socket to this unroutable address reveals the local
/// interface IP without sending any traffic. Sovereignty-compliant: avoids
/// phoning home to any third-party DNS or STUN provider.
pub const RFC5737_ROUTE_PROBE: &str = "192.0.2.1:80";

/// Default dev server port (common Flask/alternative HTTP fallback)
pub const DEFAULT_DEV_PORT: u16 = 5000;

/// HTTP port from an optional value (same parse rules as the `HTTP_PORT` env var).
#[must_use]
pub fn http_port_from(val: Option<&str>) -> u16 {
    val.and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_HTTP_PORT)
}

/// Get HTTP port from environment or fallback to default
///
/// Checks `HTTP_PORT` environment variable first.
#[must_use]
pub fn http_port() -> u16 {
    http_port_from(env::var(env_vars::HTTP_PORT).ok().as_deref())
}

/// HTTPS port from an optional value (same parse rules as the `HTTPS_PORT` env var).
#[must_use]
pub fn https_port_from(val: Option<&str>) -> u16 {
    val.and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_HTTPS_PORT)
}

/// Get HTTPS port from environment or fallback to default
///
/// Checks `HTTPS_PORT` environment variable first.
#[must_use]
pub fn https_port() -> u16 {
    https_port_from(env::var(env_vars::HTTPS_PORT).ok().as_deref())
}

/// WebSocket port from an optional value (same parse rules as the `WEBSOCKET_PORT` env var).
#[must_use]
pub fn websocket_port_from(val: Option<&str>) -> u16 {
    val.and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_WS_PORT)
}

/// Get WebSocket port from environment or fallback to default
///
/// Checks `WEBSOCKET_PORT` environment variable first.
#[must_use]
pub fn websocket_port() -> u16 {
    websocket_port_from(env::var(env_vars::WEBSOCKET_PORT).ok().as_deref())
}

/// MCP port from an optional value (same parse rules as `MCP_WEBSOCKET_PORT` / `MCP_PORT`).
#[must_use]
pub fn mcp_port_from(val: Option<&str>) -> u16 {
    val.and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_MCP_PORT)
}

/// Get MCP port from environment or fallback to default
///
/// Checks `MCP_PORT` or `MCP_WEBSOCKET_PORT` environment variable first.
#[must_use]
pub fn mcp_port() -> u16 {
    mcp_port_from(
        env::var(env_vars::MCP_WEBSOCKET_PORT)
            .or_else(|_| env::var("MCP_PORT"))
            .ok()
            .as_deref(),
    )
}

/// Discovery port from an optional value (same parse rules as `DISCOVERY_PORT`).
#[must_use]
pub fn discovery_port_from(val: Option<&str>) -> u16 {
    val.and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_DISCOVERY_PORT)
}

/// Get discovery port from environment or fallback to default
///
/// Checks `DISCOVERY_PORT` environment variable first.
#[must_use]
pub fn discovery_port() -> u16 {
    discovery_port_from(env::var("DISCOVERY_PORT").ok().as_deref())
}

/// BearDog port from an optional value (same parse rules as `BEARDOG_PORT`).
#[must_use]
pub fn beardog_port_from(val: Option<&str>) -> u16 {
    val.and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_BEARDOG_PORT)
}

/// Get security provider port from environment or fallback to default.
///
/// Checks `SECURITY_PORT` environment variable first.
#[must_use]
pub fn beardog_port() -> u16 {
    beardog_port_from(env::var(env_vars::SECURITY_PORT).ok().as_deref())
}

/// Songbird port from an optional value (same parse rules as `SONGBIRD_PORT` / MCP env vars).
#[must_use]
pub fn songbird_port_from(val: Option<&str>) -> u16 {
    val.and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_SONGBIRD_PORT)
}

/// Get discovery provider port from environment or fallback to default.
///
/// Checks `DISCOVERY_PORT` or `MCP_PORT` environment variable first.
#[must_use]
pub fn songbird_port() -> u16 {
    songbird_port_from(
        env::var(env_vars::DISCOVERY_PORT)
            .or_else(|_| env::var(env_vars::MCP_WEBSOCKET_PORT))
            .or_else(|_| env::var("MCP_PORT"))
            .ok()
            .as_deref(),
    )
}

/// Link local address range
pub const LINK_LOCAL_RANGE: &str = "169.254.0.0/16";

/// Multicast address range
pub const MULTICAST_RANGE: &str = "224.0.0.0/4";

/// Private Class A network
pub const PRIVATE_CLASS_A: &str = "10.0.0.0/8";

/// Private Class B network
pub const PRIVATE_CLASS_B: &str = "172.16.0.0/12";

/// Private Class C network
pub const PRIVATE_CLASS_C: &str = "192.168.0.0/16";

/// Default MCP subprotocol
pub const DEFAULT_MCP_SUBPROTOCOL: &str = "mcp";

/// Default user agent
pub const DEFAULT_USER_AGENT: &str = "biomeOS/1.0";

/// Default content type
pub const DEFAULT_CONTENT_TYPE: &str = "application/json";
