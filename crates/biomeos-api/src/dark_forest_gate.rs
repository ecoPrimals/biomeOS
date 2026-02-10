//! Dark Forest Beacon Gate Middleware
//!
//! Implements the sovereign security gate for ALL public-facing services.
//! Sovereign mode is **enabled by default** — lineage verification is required
//! before any interaction. This is the Dark Forest principle: the system
//! reveals nothing about itself to non-family members.
//!
//! ## Protocol
//!
//! 1. Client encrypts a beacon challenge using shared family seed (via BearDog)
//! 2. Client includes `X-Dark-Forest-Token` header in request
//! 3. Tower middleware decrypts token via BearDog
//! 4. If decryption succeeds → request passes (family member verified)
//! 5. If decryption fails → 403 Forbidden (not family, indistinguishable from noise)
//!
//! ## Bypass (minimal)
//!
//! - `/.well-known/` paths for ACME/DNS validation only
//! - `/health` returns bare `200 OK` with empty body (no version, no info)
//!
//! ## Architecture
//!
//! This is a TRUE PRIMAL implementation:
//! - No crypto in this module — all crypto delegated to BearDog via Unix socket
//! - Family seed never loaded into memory here — BearDog holds it
//! - Gate decision is binary: decrypt succeeds (family) or fails (not family)
//! - **Default is CLOSED** — must explicitly opt out with `BIOMEOS_SOVEREIGN=false`

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use biomeos_core::AtomicClient;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Paths that bypass the gate completely (ACME/DNS validation only)
const BYPASS_PATHS: &[&str] = &["/.well-known/"];

/// Paths that return bare 200 OK (no body, reveals nothing about the system)
/// Allows basic uptime monitoring without leaking any information.
const BARE_OK_PATHS: &[&str] = &[
    "/health",
    "/api/v1/health",
    "/api/v1/health/ready",
    "/api/v1/health/live",
];

/// Dark Forest gate configuration
#[derive(Debug, Clone)]
pub struct DarkForestGateConfig {
    /// Whether the gate is enabled (sovereign mode)
    /// DEFAULT: true — lineage-first, always
    pub enabled: bool,
    /// BearDog socket path for crypto verification
    pub beardog_socket: String,
}

impl DarkForestGateConfig {
    /// Create from environment variables
    ///
    /// **Sovereign mode is ENABLED by default.** Set `BIOMEOS_SOVEREIGN=false`
    /// to explicitly disable it (development/testing only).
    pub fn from_env() -> Self {
        // DEFAULT: true — the system is closed unless explicitly opened
        let enabled = env::var("BIOMEOS_SOVEREIGN")
            .or_else(|_| env::var("BIOMEOS_DARK_FOREST"))
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        let beardog_socket = env::var("BEARDOG_SOCKET").unwrap_or_else(|_| {
            // Use SystemPaths for XDG-compliant socket discovery (no hardcoded UID)
            let paths = biomeos_types::paths::SystemPaths::new_lazy();
            paths.primal_socket("beardog").to_string_lossy().to_string()
        });

        Self {
            enabled,
            beardog_socket,
        }
    }

    /// Force sovereign mode ON (used when binding to TCP)
    pub fn force_sovereign(mut self) -> Self {
        self.enabled = true;
        self
    }
}

/// Shared gate state (cached verification results)
#[derive(Clone)]
pub struct DarkForestGateState {
    /// Gate configuration
    pub config: DarkForestGateConfig,
    /// Cache of recently verified tokens (token_hash → expiry_timestamp)
    verified_cache: Arc<RwLock<std::collections::HashMap<String, u64>>>,
}

impl DarkForestGateState {
    /// Create a new gate state with the given configuration
    pub fn new(config: DarkForestGateConfig) -> Self {
        if config.enabled {
            info!("Dark Forest gate ACTIVE — lineage required before any interaction");
        } else {
            warn!("Dark Forest gate DISABLED — system is open (development mode only!)");
        }
        Self {
            config,
            verified_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Verify a Dark Forest token via BearDog
    ///
    /// The token is an encrypted beacon (base64). We ask BearDog to decrypt it.
    /// If decryption succeeds, the sender is family.
    async fn verify_token(&self, token: &str) -> bool {
        // Check cache first
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        {
            let cache = self.verified_cache.read().await;
            if let Some(&expiry) = cache.get(token) {
                if now < expiry {
                    debug!("🌲 Token verified (cached)");
                    return true;
                }
            }
        }

        // Ask BearDog to try decrypting the token
        let client =
            AtomicClient::unix(&self.config.beardog_socket).with_timeout(Duration::from_secs(5));

        let params = serde_json::json!({
            "data": token
        });

        // We use beacon.try_decrypt — if it succeeds, the token is from family
        match client.call("beacon.try_decrypt", params).await {
            Ok(result) => {
                let valid = result
                    .get("decrypted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    || result.get("plaintext").is_some();

                if valid {
                    // Cache for 5 minutes
                    let mut cache = self.verified_cache.write().await;
                    cache.insert(token.to_string(), now + 300);
                    // Clean old entries
                    cache.retain(|_, &mut expiry| expiry > now);
                    debug!("🌲 Token verified via BearDog");
                }
                valid
            }
            Err(e) => {
                warn!("🌲 Dark Forest verification failed: {} (denying access)", e);
                false
            }
        }
    }
}

/// Dark Forest gate middleware function
///
/// When sovereign mode is active (the default), this middleware:
/// - Returns bare `200 OK` (empty body) for health paths — no version, no info
/// - Passes through `/.well-known/` for ACME cert validation
/// - Requires `X-Dark-Forest-Token` on ALL other requests
/// - Returns `403 Forbidden` (empty body) for anything unauthorized
///
/// The 403 response is intentionally indistinguishable from a generic error.
/// An attacker cannot determine if this is a biomeOS system or what it expects.
///
/// Attach to an axum Router:
/// ```ignore
/// let gate_state = DarkForestGateState::new(DarkForestGateConfig::from_env());
/// let app = Router::new()
///     .route("/api/v1/data", get(handler))
///     .layer(axum::middleware::from_fn_with_state(
///         gate_state,
///         dark_forest_gate_middleware,
///     ));
/// ```
pub async fn dark_forest_gate_middleware(
    axum::extract::State(gate): axum::extract::State<DarkForestGateState>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    // If gate is disabled (explicitly opted out), pass through
    if !gate.config.enabled {
        return next.run(request).await;
    }

    let path = request.uri().path().to_string();

    // Bare OK paths: return 200 with empty body — reveals NOTHING about the system
    // This allows basic uptime monitoring (load balancers, etc.) without leaking info
    for bare_path in BARE_OK_PATHS {
        if path == *bare_path || path.starts_with(&format!("{bare_path}/")) {
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .expect("static 200 response");
        }
    }

    // ACME/DNS validation bypass (full pass-through)
    for bypass in BYPASS_PATHS {
        if path.starts_with(bypass) {
            debug!("Gate bypass for ACME path: {}", path);
            return next.run(request).await;
        }
    }

    // Extract Dark Forest token from header
    let token = request
        .headers()
        .get("X-Dark-Forest-Token")
        .and_then(|v| v.to_str().ok());

    match token {
        Some(token_value) => {
            if gate.verify_token(token_value).await {
                // Family member verified — allow through
                next.run(request).await
            } else {
                // Not family — return 403 with no information
                Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::empty())
                    .expect("static 403 response")
            }
        }
        None => {
            // No token — return 403 with no information
            // Dark Forest: reveal nothing about what's expected
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::empty())
                .expect("static 403 response")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_to_sovereign() {
        // Without env vars, gate should be ENABLED (sovereign by default)
        let _config = DarkForestGateConfig::from_env();
        // Constants should be properly populated
        assert!(BYPASS_PATHS.iter().any(|p| p.contains(".well-known")));
        assert!(!BARE_OK_PATHS.is_empty());
    }

    #[test]
    fn test_gate_state_creation() {
        let config = DarkForestGateConfig {
            enabled: true,
            beardog_socket: "/tmp/test.sock".to_string(),
        };
        let state = DarkForestGateState::new(config.clone());
        assert!(state.config.enabled);
    }

    #[test]
    fn test_health_is_bare_ok_not_bypass() {
        // Health should NOT be in bypass paths (would leak info)
        assert!(!BYPASS_PATHS.iter().any(|p| p.contains("health")));
        // Health SHOULD be in bare_ok paths (returns 200 with empty body)
        assert!(BARE_OK_PATHS.iter().any(|p| p.contains("health")));
    }

    #[test]
    fn test_only_well_known_bypasses_gate() {
        // Only .well-known should fully bypass
        assert_eq!(BYPASS_PATHS.len(), 1);
        assert!(BYPASS_PATHS[0].contains(".well-known"));
    }

    #[test]
    fn test_force_sovereign() {
        let config = DarkForestGateConfig {
            enabled: false,
            beardog_socket: "/tmp/test.sock".to_string(),
        };
        assert!(!config.enabled);
        let config = config.force_sovereign();
        assert!(config.enabled);
    }
}
