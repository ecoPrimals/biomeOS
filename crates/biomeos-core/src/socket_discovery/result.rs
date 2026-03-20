// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Discovery Result Types
//!
//! Types representing the results of socket discovery operations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use super::transport::TransportEndpoint;

/// Socket discovery result
#[derive(Debug, Clone)]
pub struct DiscoveredSocket {
    /// Path to the socket (deprecated - use `endpoint` for multi-transport)
    pub path: PathBuf,

    /// Transport endpoint (Universal IPC v3.0)
    pub endpoint: TransportEndpoint,

    /// How it was discovered
    pub discovered_via: DiscoveryMethod,

    /// Primal name (if known)
    pub primal_name: Option<Arc<str>>,

    /// Capabilities provided (if known)
    pub capabilities: Vec<String>,
}

impl DiscoveredSocket {
    /// Create from a Unix socket path (convenience constructor)
    pub fn from_unix_path(path: PathBuf, via: DiscoveryMethod) -> Self {
        Self {
            endpoint: TransportEndpoint::UnixSocket { path: path.clone() },
            path,
            discovered_via: via,
            primal_name: None,
            capabilities: Vec::new(),
        }
    }

    /// Create from a transport endpoint
    pub fn from_endpoint(endpoint: TransportEndpoint, via: DiscoveryMethod) -> Self {
        let path = match &endpoint {
            TransportEndpoint::UnixSocket { path } => path.clone(),
            _ => PathBuf::new(), // Non-path transports
        };
        Self {
            endpoint,
            path,
            discovered_via: via,
            primal_name: None,
            capabilities: Vec::new(),
        }
    }

    /// Set the primal name
    pub fn with_primal_name(mut self, name: impl AsRef<str>) -> Self {
        self.primal_name = Some(Arc::from(name.as_ref()));
        self
    }

    /// Set capabilities
    pub fn with_capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }
}

/// How a socket was discovered
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Via environment variable hint
    EnvironmentHint(Arc<str>),

    /// Via XDG runtime directory
    XdgRuntime,

    /// Via abstract socket (Linux/Android)
    AbstractSocket,

    /// Via family-scoped /tmp
    FamilyTmp,

    /// Via filesystem manifest (`$XDG_RUNTIME_DIR/ecoPrimals/manifests/{primal}.json`)
    Manifest,

    /// Via centralized socket registry (`$XDG_RUNTIME_DIR/biomeos/socket-registry.json`)
    SocketRegistry,

    /// Via capability registry query
    CapabilityRegistry,

    /// Via TCP fallback
    TcpFallback,

    /// Via socket scanning
    SocketScan,

    /// Cached from previous discovery
    Cached,
}

/// Serialize helper for Arc<str> (serialize as &str for JSON compatibility).
fn serialize_arc_str<S>(s: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.as_ref().serialize(serializer)
}

/// Deserialize helper for Arc<str> (str doesn't implement Deserialize; deserialize as String first).
fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

/// Lightweight filesystem manifest written by primals at startup.
///
/// Primals write this to `$XDG_RUNTIME_DIR/ecoPrimals/manifests/{primal}.json`
/// so other primals can discover them without the Neural API running.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalManifest {
    /// Primal name
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub primal: Arc<str>,
    /// Socket path
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub socket: Arc<str>,
    /// Capabilities this primal provides
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Process ID (for liveness verification)
    #[serde(default)]
    pub pid: Option<u32>,
}

/// Entry in the centralized socket registry.
///
/// Absorbed from Squirrel's `SocketRegistryDiscovery` pattern. Squirrel writes
/// this file; biomeOS and other primals read it as a discovery source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketRegistryEntry {
    /// Primal name
    pub primal: String,
    /// Socket path
    pub socket: String,
    /// Capabilities this primal provides
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Process ID for liveness verification
    #[serde(default)]
    pub pid: Option<u32>,
    /// Unix timestamp when this entry was last updated
    #[serde(default)]
    pub updated_at: Option<u64>,
}

/// Centralized socket registry file format.
///
/// Written by Squirrel to `$XDG_RUNTIME_DIR/biomeos/socket-registry.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketRegistry {
    /// Registry version
    #[serde(default = "default_registry_version")]
    pub version: String,
    /// Registered primal sockets
    pub entries: Vec<SocketRegistryEntry>,
}

fn default_registry_version() -> String {
    "1.0".to_owned()
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_unix_path() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        );

        assert_eq!(socket.path, PathBuf::from("/tmp/test.sock"));
        assert!(matches!(
            socket.endpoint,
            TransportEndpoint::UnixSocket { .. }
        ));
        assert_eq!(socket.discovered_via, DiscoveryMethod::FamilyTmp);
    }

    #[test]
    fn test_from_endpoint() {
        let endpoint = TransportEndpoint::TcpSocket {
            host: Arc::from("127.0.0.1"),
            port: 9100,
        };
        let socket = DiscoveredSocket::from_endpoint(endpoint, DiscoveryMethod::TcpFallback);

        assert!(matches!(
            socket.endpoint,
            TransportEndpoint::TcpSocket { .. }
        ));
        assert_eq!(socket.discovered_via, DiscoveryMethod::TcpFallback);
        // TCP sockets have empty path
        assert!(socket.path.as_os_str().is_empty());
    }

    #[test]
    fn test_builders() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        )
        .with_primal_name("beardog")
        .with_capabilities(vec!["crypto".to_string(), "identity".to_string()]);

        assert_eq!(socket.primal_name.as_deref(), Some("beardog"));
        assert_eq!(socket.capabilities.len(), 2);
        assert!(socket.capabilities.contains(&"crypto".to_string()));
    }

    #[test]
    fn test_from_endpoint_tcp() {
        let endpoint = TransportEndpoint::TcpSocket {
            host: Arc::from("127.0.0.1"),
            port: 9100,
        };
        let socket = DiscoveredSocket::from_endpoint(endpoint, DiscoveryMethod::TcpFallback);

        assert!(matches!(
            socket.endpoint,
            TransportEndpoint::TcpSocket { .. }
        ));
        assert!(socket.path.as_os_str().is_empty());
        assert_eq!(socket.discovered_via, DiscoveryMethod::TcpFallback);
    }

    #[test]
    fn test_from_endpoint_abstract() {
        let endpoint = TransportEndpoint::AbstractSocket {
            name: Arc::from("biomeos_test"),
        };
        let socket = DiscoveredSocket::from_endpoint(endpoint, DiscoveryMethod::AbstractSocket);

        assert!(matches!(
            socket.endpoint,
            TransportEndpoint::AbstractSocket { .. }
        ));
        assert!(socket.path.as_os_str().is_empty());
    }

    #[test]
    fn test_builder_chaining() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::XdgRuntime,
        )
        .with_primal_name("songbird")
        .with_capabilities(vec!["discovery".to_string()])
        .with_primal_name("beardog") // Override
        .with_capabilities(vec!["crypto".to_string()]); // Override

        assert_eq!(socket.primal_name.as_deref(), Some("beardog"));
        assert_eq!(socket.capabilities, vec!["crypto".to_string()]);
    }

    #[test]
    fn test_discovery_method_equality() {
        let method1 = DiscoveryMethod::EnvironmentHint(Arc::from("TEST_VAR"));
        let method2 = DiscoveryMethod::EnvironmentHint(Arc::from("TEST_VAR"));
        let method3 = DiscoveryMethod::EnvironmentHint(Arc::from("OTHER_VAR"));

        assert_eq!(method1, method2);
        assert_ne!(method1, method3);
        assert_ne!(method1, DiscoveryMethod::XdgRuntime);
    }

    #[test]
    fn test_primal_manifest_serde_roundtrip() {
        let manifest = PrimalManifest {
            primal: Arc::from("beardog"),
            socket: Arc::from("/run/user/1000/biomeos/beardog-abc123.sock"),
            capabilities: vec!["security".to_string(), "secrets".to_string()],
            pid: Some(12345),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: PrimalManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.primal.as_ref(), "beardog");
        assert_eq!(parsed.capabilities.len(), 2);
        assert_eq!(parsed.pid, Some(12345));
    }

    #[test]
    fn test_primal_manifest_optional_fields() {
        let json = r#"{"primal":"songbird","socket":"/tmp/songbird.sock"}"#;
        let manifest: PrimalManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.primal.as_ref(), "songbird");
        assert!(manifest.capabilities.is_empty());
        assert_eq!(manifest.pid, None);
    }

    #[test]
    fn test_discovery_method_variants() {
        let methods = vec![
            DiscoveryMethod::EnvironmentHint(Arc::from("VAR")),
            DiscoveryMethod::XdgRuntime,
            DiscoveryMethod::AbstractSocket,
            DiscoveryMethod::FamilyTmp,
            DiscoveryMethod::Manifest,
            DiscoveryMethod::CapabilityRegistry,
            DiscoveryMethod::TcpFallback,
            DiscoveryMethod::SocketScan,
            DiscoveryMethod::Cached,
        ];

        for method in methods {
            // Just verify they can be created and compared
            assert_eq!(method.clone(), method);
        }
    }

    #[test]
    fn test_discovered_socket_empty_capabilities() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        );

        assert!(socket.capabilities.is_empty());
        assert!(socket.primal_name.is_none());
    }

    #[test]
    fn test_discovered_socket_clone() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        )
        .with_primal_name("test")
        .with_capabilities(vec!["cap1".to_string(), "cap2".to_string()]);

        let cloned = socket.clone();
        assert_eq!(socket.path, cloned.path);
        assert_eq!(socket.endpoint, cloned.endpoint);
        assert_eq!(socket.discovered_via, cloned.discovered_via);
        assert_eq!(socket.primal_name.as_deref(), cloned.primal_name.as_deref());
        assert_eq!(socket.capabilities, cloned.capabilities);
    }

    #[test]
    fn test_discovered_socket_debug() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        );

        // Just verify Debug trait works
        let debug_str = format!("{socket:?}");
        assert!(debug_str.contains("DiscoveredSocket"));
    }
}
