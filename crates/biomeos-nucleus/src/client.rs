//! Unix socket JSON-RPC client and NUCLEUS coordinator
//!
//! Shared utility for talking to primals via Unix sockets.
//! Used by all NUCLEUS layers to avoid code duplication.
//!
//! Also provides the high-level `NucleusClient` that coordinates all 5 layers.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Global atomic counter for JSON-RPC request IDs.
///
/// This ensures unique IDs across concurrent requests, enabling proper
/// request/response correlation in JSON-RPC 2.0.
///
/// # Safety
///
/// This uses `AtomicU64` which is a safe, lock-free atomic operation.
/// `Ordering::Relaxed` is sufficient here since we only need uniqueness,
/// not strict ordering guarantees. This is a zero-cost abstraction over
/// platform-specific atomic instructions.
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Get the next unique request ID.
///
/// # Safety
///
/// This function is safe - it uses atomic operations which are guaranteed
/// to be thread-safe and cannot cause data races. The counter will wrap
/// around after 2^64 requests, which is acceptable for this use case.
#[inline]
fn next_request_id() -> u64 {
    REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

use crate::{
    capability::{CapabilityLayer, CapabilityLayerImpl},
    discovery::{DiscoveryLayer, DiscoveryRequest, PhysicalDiscovery},
    identity::{IdentityLayer, IdentityLayerImpl},
    registry::Registry,
    trust::{TrustLayer, TrustLayerImpl, TrustLevel},
    Error, Result, VerifiedPrimal,
};

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)] // Required by JSON-RPC spec for deserialization
    jsonrpc: String,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
    #[allow(dead_code)] // Required by JSON-RPC spec for deserialization
    id: u64,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// Call a Unix socket JSON-RPC method
///
/// **Deep Debt Principle**: Shared utility, no duplication across layers
///
/// # Errors
///
/// Returns error if:
/// - Unix socket connection fails
/// - JSON-RPC request fails  
/// - Response deserialization fails
pub async fn call_unix_socket_rpc<T: serde::de::DeserializeOwned>(
    socket_path: impl AsRef<Path>,
    method: &str,
    params: serde_json::Value,
) -> Result<T> {
    let socket_path = socket_path.as_ref();

    debug!(
        socket = %socket_path.display(),
        method = %method,
        "Calling Unix socket JSON-RPC"
    );

    // Connect to socket
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| Error::socket_connection_failed(socket_path, e))?;

    // Split stream for concurrent read/write
    let (read_half, mut write_half) = stream.into_split();

    // Create request with unique ID for concurrent request correlation
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: next_request_id(),
    };

    // Serialize and send request
    let request_json = serde_json::to_string(&request)?;
    debug!(request = %request_json, "Sending JSON-RPC request");

    write_half.write_all(request_json.as_bytes()).await?;
    write_half.write_all(b"\n").await?; // Newline delimiter
    write_half.flush().await?;

    // Read response with timeout to prevent hangs
    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();

    // 30 second timeout for socket reads (prevents indefinite hangs)
    timeout(
        Duration::from_secs(30),
        reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| Error::timeout("Socket read", 30))?
    .map_err(|e| Error::discovery_failed(format!("Read error: {e}"), None))?;

    debug!(response = %response_line, "Received JSON-RPC response");

    // Parse response
    let response: JsonRpcResponse = serde_json::from_str(&response_line).map_err(|e| {
        Error::invalid_response(
            socket_path.display().to_string(),
            format!("Invalid JSON-RPC response: {e}"),
        )
    })?;

    // Check for error
    if let Some(error) = response.error {
        return Err(Error::jsonrpc_failed(
            method,
            format!("Code {}: {}", error.code, error.message),
        ));
    }

    // Extract result
    let result = response.result.ok_or_else(|| {
        Error::invalid_response(
            socket_path.display().to_string(),
            "Missing 'result' field in JSON-RPC response",
        )
    })?;

    // Deserialize result
    serde_json::from_value(result).map_err(|e| {
        Error::invalid_response(
            socket_path.display().to_string(),
            format!("Failed to deserialize result: {e}"),
        )
    })
}

//
// ═══════════════════════════════════════════════════════════════════════════
// NUCLEUS CLIENT - High-Level Coordinator
// ═══════════════════════════════════════════════════════════════════════════
//

/// NUCLEUS Client - Coordinates all 5 discovery layers
///
/// **Deep Debt Principles Applied**:
/// - No hardcoding: Discovers all primals at runtime
/// - No reimplementation: Delegates to `BearDog` and Songbird
/// - Fast AND safe: Zero unsafe code, async throughout
/// - Capability-based: Selects by what primals can do
pub struct NucleusClient {
    /// Layer 1: Physical discovery (Songbird)
    discovery: Arc<dyn PhysicalDiscovery>,
    /// Layer 2: Identity verification (`BearDog`)
    identity: Arc<dyn IdentityLayer>,
    /// Layer 3: Capability verification
    capability: Arc<dyn CapabilityLayer>,
    /// Layer 4: Trust evaluation (`BearDog`)
    trust: Arc<dyn TrustLayer>,
    /// Layer 5: Registry and tracking
    registry: Arc<Registry>,
}

impl NucleusClient {
    /// Create a new NUCLEUS client
    ///
    /// Initializes all 5 layers
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Discovery layer fails to initialize (Songbird socket discovery fails)
    /// - Identity layer fails to initialize (`BearDog` socket discovery fails)
    /// - Trust layer fails to initialize (system paths or credentials unavailable)
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Client (5-layer secure discovery)");

        let discovery = Arc::new(DiscoveryLayer::new().await?) as Arc<dyn PhysicalDiscovery>;
        let identity = Arc::new(IdentityLayerImpl::new().await?) as Arc<dyn IdentityLayer>;
        let capability = Arc::new(CapabilityLayerImpl::new()) as Arc<dyn CapabilityLayer>;
        let trust = Arc::new(TrustLayerImpl::new().await?) as Arc<dyn TrustLayer>;
        let registry = Arc::new(Registry::new());

        info!("✅ NUCLEUS Client initialized successfully");

        Ok(Self {
            discovery,
            identity,
            capability,
            trust,
            registry,
        })
    }

    /// Discover and verify primals
    ///
    /// Runs all 5 NUCLEUS layers:
    /// 1. Physical discovery (Songbird)
    /// 2. Identity verification (`BearDog`)
    /// 3. Capability verification
    /// 4. Trust evaluation (`BearDog`)
    /// 5. Registry and tracking
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Physical discovery fails (Songbird unreachable or returns error)
    /// - Identity verification fails (invalid signatures or `BearDog` unreachable)
    /// - Capability verification fails (primal doesn't match requested capability)
    /// - Trust evaluation fails (lineage verification or trust score below threshold)
    pub async fn discover(&self, request: DiscoveryRequest) -> Result<Vec<VerifiedPrimal>> {
        info!(
            capability = %request.capability,
            family = ?request.family,
            "Starting NUCLEUS 5-layer discovery"
        );

        // Layer 1: Physical Discovery (Songbird)
        let discovered = self.discovery.discover_by_capability(&request).await?;
        info!(
            count = discovered.len(),
            "Layer 1: Discovered {} primals",
            discovered.len()
        );

        let mut verified_primals = Vec::new();

        for primal in discovered {
            info!(primal = %primal.primal, "Verifying primal through remaining layers");

            // Layer 2: Identity Verification (BearDog)
            let identity = match self.identity.verify_identity(&primal).await {
                Ok(id) => {
                    info!(primal = %primal.primal, "Layer 2: Identity verified ✓");
                    id
                }
                Err(e) => {
                    warn!(primal = %primal.primal, error = %e, "Layer 2: Identity verification failed, skipping");
                    continue;
                }
            };

            // Layer 3: Capability Verification
            let _capability = match self
                .capability
                .verify_capabilities(&primal, &identity.proof)
                .await
            {
                Ok(cap) => {
                    info!(primal = %primal.primal, "Layer 3: Capabilities verified ✓");
                    cap
                }
                Err(e) => {
                    warn!(primal = %primal.primal, error = %e, "Layer 3: Capability verification failed, skipping");
                    continue;
                }
            };

            // Layer 4: Trust Evaluation (BearDog)
            // EVOLVED (Jan 27, 2026): Get family seed from secure storage
            // Priority: 1) Environment variable (for bootstrap)
            //           2) Default empty (graceful degradation - Known trust level)
            let family_seed = Self::get_family_seed_from_storage();
            let trust = match self
                .trust
                .evaluate_trust(&primal, &identity.proof, &family_seed)
                .await
            {
                Ok(trust) => {
                    info!(primal = %primal.primal, level = ?trust.level, "Layer 4: Trust evaluated ✓");
                    trust
                }
                Err(e) => {
                    warn!(primal = %primal.primal, error = %e, "Layer 4: Trust evaluation failed, using 'Known' level");
                    crate::trust::TrustEvaluation {
                        level: TrustLevel::Known,
                        relationship: None,
                        lineage_verified: false,
                        message: "Trust evaluation failed, defaulting to Known".to_string(),
                    }
                }
            };

            // Create verified primal
            let verified = VerifiedPrimal {
                name: primal.primal,
                node_id: primal.node_id,
                family_id: primal.family_id,
                capabilities: primal.capabilities,
                endpoint: primal.endpoints.first().cloned().ok_or_else(|| {
                    Error::invalid_response(&identity.proof.primal_name, "No endpoints")
                })?,
                trust_level: trust.level,
                version: identity.proof.version,
            };

            // Layer 5: Register
            self.registry.register(verified.clone()).await;
            info!(primal = %verified.name, "Layer 5: Registered ✓");

            verified_primals.push(verified);
        }

        info!(
            verified = verified_primals.len(),
            "NUCLEUS discovery complete: {} verified primals",
            verified_primals.len()
        );

        Ok(verified_primals)
    }

    /// Get registry for direct access
    #[must_use]
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }

    /// Get family seed from secure storage
    ///
    /// EVOLVED (Jan 27, 2026): Capability-based secure storage access
    ///
    /// # Priority Sources
    /// 1. `BIOMEOS_FAMILY_SEED` environment variable (base64-encoded)
    /// 2. XDG runtime dir: `$XDG_RUNTIME_DIR/biomeos/family.seed`
    /// 3. Empty (graceful degradation - results in Known trust level)
    ///
    /// # Deep Debt Principle
    /// Family seed is NOT hardcoded. It's discovered from the environment
    /// or secure runtime storage. Missing seed results in reduced trust
    /// rather than failure.
    fn get_family_seed_from_storage() -> Vec<u8> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        // Priority 1: Environment variable (for bootstrap/testing)
        if let Ok(seed_b64) = std::env::var("BIOMEOS_FAMILY_SEED") {
            if let Ok(seed) = STANDARD.decode(&seed_b64) {
                debug!("Family seed loaded from BIOMEOS_FAMILY_SEED environment");
                return seed;
            }
            warn!("BIOMEOS_FAMILY_SEED set but invalid base64, ignoring");
        }

        // Priority 2: XDG runtime directory (secure runtime storage)
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let seed_path = std::path::Path::new(&runtime_dir)
                .join("biomeos")
                .join("family.seed");
            if let Ok(seed) = std::fs::read(&seed_path) {
                debug!(
                    "Family seed loaded from XDG runtime dir: {}",
                    seed_path.display()
                );
                return seed;
            }
        }

        // Priority 3: User-specific runtime dir (fallback)
        if let Ok(uid) = std::env::var("UID").or_else(|_| {
            // Try to get UID from /proc on Linux
            std::fs::read_to_string("/proc/self/loginuid")
                .map(|s| s.trim().to_string())
                .or_else(|_| Ok::<_, std::io::Error>("1000".to_string()))
        }) {
            let seed_path =
                std::path::PathBuf::from(format!("/run/user/{uid}/biomeos/family.seed"));
            if let Ok(seed) = std::fs::read(&seed_path) {
                debug!(
                    "Family seed loaded from user runtime dir: {}",
                    seed_path.display()
                );
                return seed;
            }
        }

        // Graceful degradation: no seed available, trust evaluation will use Known level
        debug!("No family seed available - trust evaluation will use Known level");
        Vec::new()
    }
}

/// NUCLEUS Client Builder (for customization)
pub struct NucleusClientBuilder {
    // Future: Allow custom layer implementations
}

impl NucleusClientBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Build the client
    ///
    /// # Errors
    ///
    /// Returns an error if `NucleusClient::new()` fails. See [`NucleusClient::new`] for details.
    pub async fn build(self) -> Result<NucleusClient> {
        NucleusClient::new().await
    }
}

impl Default for NucleusClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test_method".to_string(),
            params: serde_json::json!({"key": "value"}),
            id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test_method"));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_jsonrpc_response_deserialization() {
        let json = r#"{"jsonrpc":"2.0","result":{"success":true},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error_response() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid request");
    }
}
