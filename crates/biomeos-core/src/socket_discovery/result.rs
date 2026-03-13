// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Discovery Result Types
//!
//! Types representing the results of socket discovery operations.

use std::path::PathBuf;

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
    pub primal_name: Option<String>,

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
    pub fn with_primal_name(mut self, name: impl Into<String>) -> Self {
        self.primal_name = Some(name.into());
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
    EnvironmentHint(String),

    /// Via XDG runtime directory
    XdgRuntime,

    /// Via abstract socket (Linux/Android)
    AbstractSocket,

    /// Via family-scoped /tmp
    FamilyTmp,

    /// Via capability registry query
    CapabilityRegistry,

    /// Via TCP fallback
    TcpFallback,

    /// Via socket scanning
    SocketScan,

    /// Cached from previous discovery
    Cached,
}

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
            host: "127.0.0.1".to_string(),
            port: 9100,
        };
        let socket =
            DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::TcpFallback);

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

        assert_eq!(socket.primal_name, Some("beardog".to_string()));
        assert_eq!(socket.capabilities.len(), 2);
        assert!(socket.capabilities.contains(&"crypto".to_string()));
    }

    #[test]
    fn test_from_endpoint_tcp() {
        let endpoint = TransportEndpoint::TcpSocket {
            host: "127.0.0.1".to_string(),
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
            name: "biomeos_test".to_string(),
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

        assert_eq!(socket.primal_name, Some("beardog".to_string()));
        assert_eq!(socket.capabilities, vec!["crypto".to_string()]);
    }

    #[test]
    fn test_discovery_method_equality() {
        let method1 = DiscoveryMethod::EnvironmentHint("TEST_VAR".to_string());
        let method2 = DiscoveryMethod::EnvironmentHint("TEST_VAR".to_string());
        let method3 = DiscoveryMethod::EnvironmentHint("OTHER_VAR".to_string());

        assert_eq!(method1, method2);
        assert_ne!(method1, method3);
        assert_ne!(method1, DiscoveryMethod::XdgRuntime);
    }

    #[test]
    fn test_discovery_method_variants() {
        let methods = vec![
            DiscoveryMethod::EnvironmentHint("VAR".to_string()),
            DiscoveryMethod::XdgRuntime,
            DiscoveryMethod::AbstractSocket,
            DiscoveryMethod::FamilyTmp,
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
        assert_eq!(socket.primal_name, cloned.primal_name);
        assert_eq!(socket.capabilities, cloned.capabilities);
    }

    #[test]
    fn test_discovered_socket_debug() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        );

        // Just verify Debug trait works
        let debug_str = format!("{:?}", socket);
        assert!(debug_str.contains("DiscoveredSocket"));
    }
}
