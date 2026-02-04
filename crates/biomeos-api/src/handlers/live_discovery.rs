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
// =============================================================================

use anyhow::{Context, Result};
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

/// JSON-RPC 2.0 request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Send a JSON-RPC request over Unix socket
fn send_rpc_request(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    debug!("📡 Sending RPC to {}: {}", socket_path, method);

    let mut stream = UnixStream::connect(socket_path)
        .with_context(|| format!("Failed to connect to socket: {}", socket_path))?;

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method: method.to_string(),
        params,
    };

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
            Err(anyhow::anyhow!("Task join error: {}", e))
        }
    }
}

/// Infer capabilities from primal name (fallback when primal doesn't report)
///
/// This is a FALLBACK only - ideally primals self-report their capabilities.
/// These mappings are based on well-known primal taxonomy patterns.
fn infer_capabilities_from_name(name: &str) -> Vec<String> {
    let name_lower = name.to_lowercase();

    // Security primals (crypto providers)
    if name_lower.contains("security")
        || name_lower.contains("crypto")
        || name_lower.contains("beardog")
    // Known security primal
    {
        vec![
            "security".to_string(),
            "crypto.encrypt".to_string(),
            "crypto.decrypt".to_string(),
            "crypto.sign".to_string(),
            "crypto.verify".to_string(),
        ]
    // Discovery/HTTP primals
    } else if name_lower.contains("discovery")
        || name_lower.contains("http")
        || name_lower.contains("songbird")
    // Known discovery primal
    {
        vec![
            "discovery".to_string(),
            "http.request".to_string(),
            "http.get".to_string(),
            "http.post".to_string(),
        ]
    // Storage primals
    } else if name_lower.contains("storage") || name_lower.contains("toadstool")
    // Known storage primal
    {
        vec![
            "storage".to_string(),
            "storage.get".to_string(),
            "storage.put".to_string(),
        ]
    // Shell/execution primals
    } else if name_lower.contains("nest")
        || name_lower.contains("shell")
        || name_lower.contains("nestgate")
    // Known shell primal
    {
        vec!["shell".to_string(), "shell.execute".to_string()]
    // AI primals
    } else if name_lower.contains("squirrel") || name_lower.contains("ai") {
        vec![
            "ai".to_string(),
            "ai.chat".to_string(),
            "ai.complete".to_string(),
        ]
    } else {
        // Generic capabilities
        vec!["primal".to_string()]
    }
}

/// Infer primary type from primal name (fallback)
fn infer_type_from_name(name: &str) -> String {
    let name_lower = name.to_lowercase();

    if name_lower.contains("security")
        || name_lower.contains("crypto")
        || name_lower.contains("beardog")
    {
        "security".to_string()
    } else if name_lower.contains("discovery")
        || name_lower.contains("http")
        || name_lower.contains("songbird")
    {
        "discovery".to_string()
    } else if name_lower.contains("storage") || name_lower.contains("toadstool") {
        "storage".to_string()
    } else if name_lower.contains("nest")
        || name_lower.contains("shell")
        || name_lower.contains("nestgate")
    {
        "shell".to_string()
    } else if name_lower.contains("squirrel") || name_lower.contains("ai") {
        "ai".to_string()
    } else {
        "primal".to_string()
    }
}

/// Get socket directory from environment (5-tier resolution)
fn get_socket_dir() -> String {
    // Tier 1: Explicit socket dir
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return dir;
    }

    // Tier 2: XDG runtime dir
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        return format!("{}/biomeos/sockets", xdg);
    }

    // Tier 3: Family-specific temp
    if let Ok(family) = std::env::var("BIOMEOS_FAMILY_ID").or_else(|_| std::env::var("FAMILY_ID")) {
        return format!("/tmp/biomeos-{}/sockets", family);
    }

    // Tier 4: Default temp location
    "/tmp/biomeos/sockets".to_string()
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
                .any(|c| c == capability || c.starts_with(&format!("{}.", capability)))
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

// =============================================================================
// Legacy Compatibility (deprecated - use discover_primal instead)
// =============================================================================

/// Query a security primal (capability: crypto.*)
///
/// DEPRECATED: Use `discover_by_capability("crypto")` instead
#[deprecated(
    since = "1.3.0",
    note = "Use discover_by_capability(\"crypto\") instead"
)]
pub async fn discover_beardog(socket_path: &str) -> Result<LivePrimalInfo> {
    discover_primal(socket_path).await
}

/// Query a discovery primal (capability: http.*)
///
/// DEPRECATED: Use `discover_by_capability("http")` instead
#[deprecated(since = "1.3.0", note = "Use discover_by_capability(\"http\") instead")]
pub async fn discover_songbird(socket_path: &str) -> Result<LivePrimalInfo> {
    discover_primal(socket_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_capabilities_beardog() {
        let caps = infer_capabilities_from_name("BearDog");
        assert!(caps.contains(&"security".to_string()));
        assert!(caps.contains(&"crypto.encrypt".to_string()));
    }

    #[test]
    fn test_infer_capabilities_songbird() {
        let caps = infer_capabilities_from_name("Songbird");
        assert!(caps.contains(&"discovery".to_string()));
        assert!(caps.contains(&"http.request".to_string()));
    }

    #[test]
    fn test_infer_capabilities_toadstool() {
        let caps = infer_capabilities_from_name("Toadstool");
        assert!(caps.contains(&"storage".to_string()));
    }

    #[test]
    fn test_infer_capabilities_nestgate() {
        let caps = infer_capabilities_from_name("NestGate");
        assert!(caps.contains(&"shell".to_string()));
    }

    #[test]
    fn test_infer_capabilities_squirrel() {
        let caps = infer_capabilities_from_name("Squirrel");
        assert!(caps.contains(&"ai".to_string()));
    }

    #[test]
    fn test_infer_type() {
        assert_eq!(infer_type_from_name("BearDog"), "security");
        assert_eq!(infer_type_from_name("Songbird"), "discovery");
        assert_eq!(infer_type_from_name("NestGate"), "shell");
        assert_eq!(infer_type_from_name("Toadstool"), "storage");
        assert_eq!(infer_type_from_name("Squirrel"), "ai");
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
        assert_eq!(dir, "/tmp/biomeos/sockets");
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
}
