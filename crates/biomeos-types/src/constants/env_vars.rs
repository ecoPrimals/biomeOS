// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Environment variable names used across biomeOS constants and accessors.

/// Bind address environment variable
pub const BIND_ADDRESS: &str = "BIND_ADDRESS";

/// HTTP port environment variable
pub const HTTP_PORT: &str = "HTTP_PORT";

/// WebSocket port environment variable
pub const WEBSOCKET_PORT: &str = "WEBSOCKET_PORT";

/// HTTPS port environment variable
pub const HTTPS_PORT: &str = "HTTPS_PORT";

/// MCP WebSocket port environment variable
pub const MCP_WEBSOCKET_PORT: &str = "MCP_WEBSOCKET_PORT";

/// `BearDog` endpoint URL environment variable
pub const BEARDOG_ENDPOINT: &str = "BEARDOG_ENDPOINT";

/// `BearDog` port environment variable
pub const BEARDOG_PORT: &str = "BEARDOG_PORT";

/// Songbird endpoint URL environment variable
pub const SONGBIRD_ENDPOINT: &str = "SONGBIRD_ENDPOINT";

/// Songbird port environment variable
pub const SONGBIRD_PORT: &str = "SONGBIRD_PORT";

/// Connection timeout environment variable
pub const CONNECTION_TIMEOUT: &str = "CONNECTION_TIMEOUT";

/// Request timeout environment variable
pub const REQUEST_TIMEOUT: &str = "REQUEST_TIMEOUT";

/// Operation timeout environment variable
pub const OPERATION_TIMEOUT: &str = "OPERATION_TIMEOUT";

/// Database timeout environment variable
pub const DATABASE_TIMEOUT: &str = "DATABASE_TIMEOUT";

/// Heartbeat interval environment variable
pub const HEARTBEAT_INTERVAL: &str = "HEARTBEAT_INTERVAL";

/// Maximum connections environment variable
pub const MAX_CONNECTIONS: &str = "MAX_CONNECTIONS";

/// Buffer size environment variable
pub const BUFFER_SIZE: &str = "BUFFER_SIZE";

/// Service mesh maximum services environment variable
pub const SERVICE_MESH_MAX_SERVICES: &str = "SERVICE_MESH_MAX_SERVICES";

/// Maximum message size environment variable
pub const MAX_MESSAGE_SIZE: &str = "MAX_MESSAGE_SIZE";
