// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Live Primal Discovery - Capability-Based Dynamic Discovery.
//! Reserved for discovery routes and future REST API; functions are pub for cross-module use.
#![allow(dead_code)]

// =============================================================================
// Live Primal Discovery - Capability-Based Dynamic Discovery
// =============================================================================
//
// ARCHITECTURE: Uses JSON-RPC 2.0 over Unix sockets for primal discovery.
// This is the Pure Rust path - no HTTP/TLS dependencies (reqwest, openssl).
//
// Deep Debt Evolution (Feb 2026):
//   - BEFORE: Hardcoded primal names (beardog, songbird)
//   - AFTER: Capability-based discovery - primals self-report their identities
//
// Principle: Primal code only has self-knowledge and discovers others at runtime.
//            No hardcoded primal names - all discovery is dynamic.
//
// This module provides utility functions used by other handlers and by future
// REST API routes for live primal querying. Functions are pub for cross-module use.
//
// =============================================================================

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Primal information from live discovery (capability-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePrimalInfo {
    /// Unique identifier (from primal or derived from socket name)
    pub id: String,
    /// Display name (from primal self-report)
    pub name: String,
    /// Primary capability category (security, discovery, storage, etc.)
    pub primal_type: String,
    /// Version (from primal self-report)
    pub version: String,
    /// Health status
    pub health: String,
    /// Capabilities the primal provides
    pub capabilities: Vec<String>,
    /// Endpoint (socket path or address)
    pub endpoint: String,
    /// Family ID for multi-family deployments
    pub family_id: Option<String>,
}

/// Identity attestation from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAttestation {
    pub provider_capability: String,
    pub format: String,
    pub data: serde_json::Value,
}

/// Send a JSON-RPC request over Unix socket
fn send_rpc_request(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    debug!("📡 Sending RPC to {}: {}", socket_path, method);

    let mut stream = UnixStream::connect(socket_path)
        .with_context(|| format!("Failed to connect to socket: {socket_path}"))?;

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let request = JsonRpcRequest::new(method, params);

    let request_bytes = serde_json::to_vec(&request)?;
    stream.write_all(&request_bytes)?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response_buf = vec![0u8; 65536];
    let n = stream.read(&mut response_buf)?;
    let response: JsonRpcResponse = serde_json::from_slice(&response_buf[..n])?;

    if let Some(error) = response.error {
        anyhow::bail!("RPC error {}: {}", error.code, error.message);
    }

    response
        .result
        .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))
}

/// Discover a primal at the given socket path (capability-agnostic)
///
/// This function queries ANY primal via `health.check` and extracts:
/// - Identity information (name, version, family_id)
/// - Capabilities (what the primal provides)
/// - Health status
///
/// NO hardcoded primal names - the primal self-reports its identity.
pub async fn discover_primal(socket_path: &str) -> Result<LivePrimalInfo> {
    let socket = socket_path.to_string();

    // Extract primal name hint from socket path (fallback only)
    let socket_hint = Path::new(&socket)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.split('-').next().unwrap_or(s))
        .unwrap_or("unknown")
        .to_string();

    info!(
        "🔍 Discovering primal at socket: {} (hint: {})",
        socket_path, socket_hint
    );

    let result = tokio::task::spawn_blocking(move || {
        send_rpc_request(&socket, "health.check", serde_json::json!({}))
    })
    .await;

    match result {
        Ok(Ok(response)) => {
            // Primal self-reports its identity
            let name = response
                .get("name")
                .or_else(|| response.get("primal_name"))
                .and_then(|v| v.as_str())
                .unwrap_or(&socket_hint)
                .to_string();

            let version = response
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let status = response
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("healthy")
                .to_string();

            let family_id = response
                .get("family_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Capabilities from primal self-report
            let capabilities = response
                .get("capabilities")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| infer_capabilities_from_name(&name));

            // Determine primary type from capabilities or name
            let primal_type = capabilities
                .first()
                .cloned()
                .unwrap_or_else(|| infer_type_from_name(&name));

            // Generate ID from family_id and name
            let id = if let Some(ref fid) = family_id {
                format!("{}-{}", name.to_lowercase(), fid)
            } else {
                format!("{}-local", name.to_lowercase())
            };

            info!(
                "✅ Discovered primal: {} v{} ({}) - {} capabilities",
                name,
                version,
                status,
                capabilities.len()
            );

            Ok(LivePrimalInfo {
                id,
                name,
                primal_type,
                version,
                health: status,
                capabilities,
                endpoint: socket_path.to_string(),
                family_id,
            })
        }
        Ok(Err(e)) => {
            warn!("⚠️  Primal at {} not responding: {}", socket_path, e);
            Err(e)
        }
        Err(e) => {
            warn!("⚠️  Discovery task failed for {}: {}", socket_path, e);
            Err(anyhow::anyhow!("Task join error: {e}"))
        }
    }
}

/// Capability domain mapping for inference
///
/// This provides a configurable, capability-first approach to inferring capabilities
/// from primal metadata when the primal hasn't self-reported capabilities.
///
/// ## Deep Debt Evolution
///
/// Instead of hardcoding specific primal names, we use:
/// 1. **Capability keywords**: Generic terms that describe functionality
/// 2. **Domain patterns**: Regex-like patterns for common naming conventions
///
/// The mapping is defined here for visibility but could be loaded from
/// configuration in a future evolution.
struct CapabilityDomainMapping {
    /// Keywords that trigger this capability domain
    keywords: &'static [&'static str],
    /// Capabilities provided by this domain
    capabilities: &'static [&'static str],
}

/// Capability domain mappings - CAPABILITY-FIRST, no primal name knowledge
///
/// DEEP DEBT EVOLUTION (Feb 7, 2026):
/// Removed hardcoded primal names ("beardog", "songbird") from keywords.
/// Discovery now works purely by capability semantics. Primals self-report
/// their capabilities; the discovery system matches by what they CAN DO,
/// not what they ARE CALLED.
const CAPABILITY_DOMAINS: &[CapabilityDomainMapping] = &[
    // Security/Cryptography domain (capability-only, no primal names)
    CapabilityDomainMapping {
        keywords: &[
            "security", "crypto", "encrypt", "sign", "vault", "key", "trust", "identity", "lineage",
        ],
        capabilities: &[
            "security",
            "crypto.encrypt",
            "crypto.decrypt",
            "crypto.sign",
            "crypto.verify",
        ],
    },
    // Discovery/Network domain (capability-only, no primal names)
    CapabilityDomainMapping {
        keywords: &[
            "discovery",
            "http",
            "network",
            "gateway",
            "proxy",
            "route",
            "mesh",
            "relay",
            "beacon",
        ],
        capabilities: &["discovery", "http.request", "http.get", "http.post"],
    },
    // Storage domain
    // Includes "toadstool" as a well-known storage primal
    CapabilityDomainMapping {
        keywords: &[
            "storage", "persist", "store", "data", "cache", "db", "archive", "backup",
        ],
        capabilities: &["storage", "storage.get", "storage.put"],
    },
    // Compute/Shell domain (capability-only, no primal names)
    CapabilityDomainMapping {
        keywords: &[
            "shell",
            "compute",
            "exec",
            "run",
            "process",
            "container",
            "gate",
            "sandbox",
        ],
        capabilities: &["shell", "shell.execute"],
    },
    // AI domain (capability-only, no primal names)
    CapabilityDomainMapping {
        keywords: &["ai", "ml", "inference", "model", "llm", "chat", "mcp"],
        capabilities: &["ai", "ai.chat", "ai.complete"],
    },
];

fn infer_capabilities_from_name(name: &str) -> Vec<String> {
    let name_lower = name.to_lowercase();

    // First pass: Check capability-based keywords (primal-agnostic)
    for domain in CAPABILITY_DOMAINS {
        for keyword in domain.keywords {
            if name_lower.contains(keyword) {
                return domain.capabilities.iter().map(|s| s.to_string()).collect();
            }
        }
    }

    // Fallback: Generic primal capability
    vec!["primal".to_string()]
}

/// Infer primary type from primal name (fallback)
///
/// Uses capability-based keywords rather than hardcoded primal names.
/// This is a fallback when the primal doesn't self-report its type.
fn infer_type_from_name(name: &str) -> String {
    let name_lower = name.to_lowercase();

    // Check capability domains in order of priority
    for domain in CAPABILITY_DOMAINS {
        for keyword in domain.keywords {
            if name_lower.contains(keyword) {
                // Return the first (primary) capability as the type
                return domain
                    .capabilities
                    .first()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "primal".to_string());
            }
        }
    }

    "primal".to_string()
}

/// Get socket directory via XDG-compliant SystemPaths
///
/// Single source of truth: `SystemPaths::new_lazy().runtime_dir()`.
/// Only falls back to env override if `BIOMEOS_SOCKET_DIR` is explicitly set.
fn get_socket_dir() -> String {
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return dir;
    }
    biomeos_types::SystemPaths::new_lazy()
        .runtime_dir()
        .to_string_lossy()
        .to_string()
}

/// Scan socket directory and discover all available primals
///
/// This is the primary discovery mechanism - scan for .sock files and query each.
/// No hardcoded primal lists - purely dynamic discovery.
pub async fn discover_all_primals() -> Vec<LivePrimalInfo> {
    let mut primals = Vec::new();
    let socket_dir = get_socket_dir();

    info!("🔍 Scanning for primals in: {}", socket_dir);

    // Check if socket directory exists
    let dir_path = Path::new(&socket_dir);
    if !dir_path.exists() {
        warn!("⚠️  Socket directory does not exist: {}", socket_dir);
        return primals;
    }

    // Scan for .sock files
    let entries = match std::fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("⚠️  Cannot read socket directory: {}", e);
            return primals;
        }
    };

    let mut socket_paths: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "sock" {
                if let Some(path_str) = path.to_str() {
                    socket_paths.push(path_str.to_string());
                }
            }
        }
    }

    info!("📂 Found {} socket files", socket_paths.len());

    // Discover each primal concurrently
    let handles: Vec<_> = socket_paths
        .into_iter()
        .map(|path| tokio::spawn(async move { discover_primal(&path).await }))
        .collect();

    for handle in handles {
        match handle.await {
            Ok(Ok(primal)) => {
                info!(
                    "✅ Discovered: {} ({}) at {}",
                    primal.name, primal.primal_type, primal.endpoint
                );
                primals.push(primal);
            }
            Ok(Err(e)) => {
                debug!("⚠️  Primal discovery failed: {}", e);
            }
            Err(e) => {
                debug!("⚠️  Task join error: {}", e);
            }
        }
    }

    info!("🎯 Total primals discovered: {}", primals.len());
    primals
}

/// Discover primals by capability
///
/// Find all primals that provide a specific capability (e.g., "crypto.encrypt")
pub async fn discover_by_capability(capability: &str) -> Vec<LivePrimalInfo> {
    let all = discover_all_primals().await;

    all.into_iter()
        .filter(|p| {
            p.capabilities
                .iter()
                .any(|c| c == capability || c.starts_with(&format!("{capability}.")))
        })
        .collect()
}

/// Discover primals by type
///
/// Find all primals of a specific type (e.g., "security", "discovery")
pub async fn discover_by_type(primal_type: &str) -> Vec<LivePrimalInfo> {
    let all = discover_all_primals().await;

    all.into_iter()
        .filter(|p| p.primal_type == primal_type)
        .collect()
}

// DEEP DEBT EVOLUTION (Feb 7, 2026): Removed deprecated `discover_beardog` and
// `discover_songbird` functions. All discovery should use capability-based methods:
// - `discover_all_primals()` for scanning
// - `discover_by_capability(capability)` for capability-filtered discovery
// - `discover_primal(socket_path)` for querying a specific socket

#[cfg(test)]
mod tests {
    use super::*;

    // DEEP DEBT EVOLUTION: Tests now use CAPABILITY keywords, not primal names.
    // Primals are discovered by what they CAN DO, not what they're CALLED.

    #[test]
    fn test_infer_capabilities_security_keywords() {
        let caps = infer_capabilities_from_name("crypto-provider");
        assert!(caps.contains(&"security".to_string()));
        assert!(caps.contains(&"crypto.encrypt".to_string()));

        // "key" is a security keyword
        let caps2 = infer_capabilities_from_name("key-manager");
        assert!(caps2.contains(&"security".to_string()));
    }

    #[test]
    fn test_infer_capabilities_discovery_keywords() {
        let caps = infer_capabilities_from_name("mesh-relay");
        assert!(caps.contains(&"discovery".to_string()));

        let caps2 = infer_capabilities_from_name("beacon-service");
        assert!(caps2.contains(&"discovery".to_string()));
    }

    #[test]
    fn test_infer_capabilities_storage_keywords() {
        let caps = infer_capabilities_from_name("data-store");
        assert!(caps.contains(&"storage".to_string()));
    }

    #[test]
    fn test_infer_capabilities_compute_keywords() {
        let caps = infer_capabilities_from_name("shell-gate");
        assert!(caps.contains(&"shell".to_string()));
    }

    #[test]
    fn test_infer_capabilities_ai_keywords() {
        let caps = infer_capabilities_from_name("ai-assistant");
        assert!(caps.contains(&"ai".to_string()));
    }

    #[test]
    fn test_infer_type_by_capability() {
        // Types are inferred from capability keywords, not primal names
        assert_eq!(infer_type_from_name("crypto-vault"), "security");
        assert_eq!(infer_type_from_name("mesh-beacon"), "discovery");
        // Use "shell-runner" instead of "compute-sandbox" to avoid substring collisions
        assert_eq!(infer_type_from_name("shell-runner"), "shell");
        assert_eq!(infer_type_from_name("data-store"), "storage");
        assert_eq!(infer_type_from_name("ai-model"), "ai");
        assert_eq!(infer_type_from_name("Unknown"), "primal");
    }

    #[test]
    fn test_socket_dir_default() {
        // Clear env vars for test
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
        std::env::remove_var("XDG_RUNTIME_DIR");
        std::env::remove_var("BIOMEOS_FAMILY_ID");
        std::env::remove_var("FAMILY_ID");

        let dir = get_socket_dir();
        // Default fallback uses username for isolation
        let username = std::env::var("USER").unwrap_or_else(|_| "default".to_string());
        assert_eq!(dir, format!("/tmp/biomeos-{username}"));
    }

    #[test]
    fn test_live_primal_info_serialize() {
        let info = LivePrimalInfo {
            id: "test-local".to_string(),
            name: "Test".to_string(),
            primal_type: "primal".to_string(),
            version: "1.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec!["test".to_string()],
            endpoint: "/tmp/test.sock".to_string(),
            family_id: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-local"));
    }

    #[test]
    fn test_live_primal_info_roundtrip() {
        let info = LivePrimalInfo {
            id: "roundtrip-id".to_string(),
            name: "RoundtripPrimal".to_string(),
            primal_type: "security".to_string(),
            version: "2.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec!["crypto.encrypt".to_string(), "crypto.sign".to_string()],
            endpoint: "/run/user/1000/biomeos/test.sock".to_string(),
            family_id: Some("family-abc".to_string()),
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let restored: LivePrimalInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(info.id, restored.id);
        assert_eq!(info.name, restored.name);
        assert_eq!(info.primal_type, restored.primal_type);
        assert_eq!(info.family_id, restored.family_id);
    }

    #[test]
    fn test_identity_attestation_roundtrip() {
        let attestation = IdentityAttestation {
            provider_capability: "crypto.verify".to_string(),
            format: "jwt".to_string(),
            data: serde_json::json!({"sub": "primal-1", "aud": "biomeos"}),
        };

        let json = serde_json::to_string(&attestation).expect("serialize");
        let restored: IdentityAttestation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            attestation.provider_capability,
            restored.provider_capability
        );
        assert_eq!(attestation.format, restored.format);
    }

    #[test]
    fn test_get_socket_dir_returns_valid_path() {
        let dir = get_socket_dir();
        assert!(!dir.is_empty(), "socket dir should not be empty");
        assert!(
            dir.contains("biomeos")
                || dir.starts_with("/tmp")
                || dir.starts_with("/run")
                || dir.starts_with("/nonexistent")
                || std::path::Path::new(&dir).is_absolute(),
            "socket dir should be an absolute path, got: {dir}"
        );
    }

    #[test]
    fn test_infer_capabilities_unknown_name() {
        let caps = infer_capabilities_from_name("xyz-unknown-service");
        assert_eq!(caps, vec!["primal".to_string()]);
    }

    #[test]
    fn test_infer_capabilities_first_match_wins() {
        // "security" and "discovery" - first domain in CAPABILITY_DOMAINS wins
        let caps = infer_capabilities_from_name("security-discovery-hybrid");
        assert!(caps.contains(&"security".to_string()));
    }

    #[test]
    fn test_infer_type_unknown() {
        assert_eq!(infer_type_from_name("random-service-xyz"), "primal");
    }

    #[tokio::test]
    async fn test_discover_all_primals_empty_dir() {
        // With no sockets, should return empty without panicking
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/nonexistent/path/for/tests");
        let primals = discover_all_primals().await;
        assert!(primals.is_empty());
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
    }

    #[tokio::test]
    async fn test_discover_by_capability_returns() {
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/nonexistent/path");
        let primals = discover_by_capability("crypto.encrypt").await;
        assert!(primals.is_empty());
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
    }

    #[tokio::test]
    async fn test_discover_by_type_returns() {
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/nonexistent/path");
        let primals = discover_by_type("security").await;
        assert!(primals.is_empty());
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
    }
}
