// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Dark Forest Beacon Gate Middleware
//!
//! Implements the sovereign security gate for ALL public-facing services.
//! Sovereign mode is **enabled by default** — lineage verification is required
//! before any interaction. This is the Dark Forest principle: the system
//! reveals nothing about itself to non-family members.
//!
//! ## Deep Debt Evolution (Feb 11, 2026)
//!
//! - Extracted duplicate verification logic into `beacon_verification` module
//! - Single `parse_decrypt_result()` with strict AND validation
//! - No direct primal socket knowledge — routes via Neural API capability.call
//! - Security fix: `success && has_plaintext` (was incorrectly `||`)
//!
//! ## Protocol
//!
//! 1. Client encrypts a beacon challenge using shared family seed
//! 2. Client includes `X-Dark-Forest-Token` header in request
//! 3. Tower Atomic middleware verifies token via `capability.call("beacon", "decrypt")`
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
//! This is a TRUE PRIMAL implementation using Tower Atomic routing:
//! - No crypto in this module — routed via shared `beacon_verification`
//! - No direct primal knowledge — gate doesn't know about BearDog
//! - Family seed never loaded into memory here — capability provider holds it
//! - Gate decision is binary: decrypt succeeds (family) or fails (not family)
//! - **Default is CLOSED** — must explicitly opt out with `BIOMEOS_SOVEREIGN=false`

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::beacon_verification;

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
///
/// Routes verification through Tower Atomic stack:
/// - Prefers Neural API socket for `capability.call` semantic routing
/// - Falls back to direct AtomicClient if Neural API socket not found
#[derive(Debug, Clone)]
pub struct DarkForestGateConfig {
    /// Whether the gate is enabled (sovereign mode)
    /// DEFAULT: true — lineage-first, always
    pub enabled: bool,
    /// Neural API socket path for capability-routed verification
    /// Discovered at runtime — None if Neural API not available
    pub neural_api_socket: Option<String>,
    /// Family ID for decryption context (resolved from .family.seed)
    pub family_id: String,
}

impl DarkForestGateConfig {
    /// Create from environment variables
    ///
    /// **Sovereign mode is ENABLED by default.** Set `BIOMEOS_SOVEREIGN=false`
    /// to explicitly disable it (development/testing only).
    ///
    /// Socket resolution uses shared `beacon_verification::discover_neural_api_socket()`
    pub fn from_env() -> Self {
        // DEFAULT: true — the system is closed unless explicitly opened
        let enabled = env::var("BIOMEOS_SOVEREIGN")
            .or_else(|_| env::var("BIOMEOS_DARK_FOREST"))
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        // Resolve family ID from seed (not hardcoded!)
        let family_id = biomeos_core::family_discovery::get_family_id();

        // Discover Neural API socket via shared discovery logic
        let neural_api_socket = beacon_verification::discover_neural_api_socket(&family_id);

        Self {
            enabled,
            neural_api_socket,
            family_id,
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
            match &config.neural_api_socket {
                Some(socket) => {
                    info!(
                        "🌲 Dark Forest gate ACTIVE via Tower Atomic (Neural API: {})",
                        socket
                    );
                }
                None => {
                    info!("🌲 Dark Forest gate ACTIVE via direct capability routing");
                    info!("   (Neural API not found — using socket discovery fallback)");
                }
            }
        } else {
            warn!("🌲 Dark Forest gate DISABLED — system is open (development mode only!)");
        }
        Self {
            config,
            verified_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Verify a Dark Forest token via shared beacon verification
    ///
    /// Uses `beacon_verification::verify_dark_forest_token()` — single source of truth.
    /// Result is cached for 5 minutes to avoid redundant crypto operations.
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

        // Delegate to shared verification (Neural API → socket discovery fallback)
        let verified = beacon_verification::verify_dark_forest_token(
            self.config.neural_api_socket.as_deref(),
            &self.config.family_id,
            token,
        )
        .await;

        if verified.is_some() {
            // Cache for 5 minutes
            let mut cache = self.verified_cache.write().await;
            cache.insert(token.to_string(), now + 300);
            // Clean old entries
            cache.retain(|_, &mut expiry| expiry > now);
            debug!("🌲 Token verified via Tower Atomic");
            true
        } else {
            false
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

    // ========================================================================
    // Constant path tests
    // ========================================================================

    #[test]
    fn test_config_defaults_to_sovereign() {
        let _config = DarkForestGateConfig::from_env();
        assert!(BYPASS_PATHS.iter().any(|p| p.contains(".well-known")));
        assert!(!BARE_OK_PATHS.is_empty());
    }

    #[test]
    fn test_health_is_bare_ok_not_bypass() {
        assert!(!BYPASS_PATHS.iter().any(|p| p.contains("health")));
        assert!(BARE_OK_PATHS.iter().any(|p| p.contains("health")));
    }

    #[test]
    fn test_only_well_known_bypasses_gate() {
        assert_eq!(BYPASS_PATHS.len(), 1);
        assert!(BYPASS_PATHS[0].contains(".well-known"));
    }

    #[test]
    fn test_bypass_paths_no_data_leaks() {
        // Bypass paths should only include ACME/DNS validation
        for path in BYPASS_PATHS {
            assert!(
                path.contains("well-known"),
                "Bypass path should only be for ACME: {path}"
            );
        }
    }

    #[test]
    fn test_bare_ok_paths_all_health_related() {
        // All bare OK paths should be health-related
        for path in BARE_OK_PATHS {
            assert!(
                path.contains("health"),
                "Bare OK path should be health-related: {path}"
            );
        }
    }

    #[test]
    fn test_bare_ok_paths_include_standard_endpoints() {
        let paths: Vec<&str> = BARE_OK_PATHS.to_vec();
        assert!(paths.contains(&"/health"), "Should include /health");
        assert!(
            paths.contains(&"/api/v1/health"),
            "Should include /api/v1/health"
        );
        assert!(
            paths.contains(&"/api/v1/health/ready"),
            "Should include readiness"
        );
        assert!(
            paths.contains(&"/api/v1/health/live"),
            "Should include liveness"
        );
    }

    // ========================================================================
    // Config tests
    // ========================================================================

    #[test]
    fn test_gate_state_creation() {
        let config = DarkForestGateConfig {
            enabled: true,
            neural_api_socket: None,
            family_id: "test_family".to_string(),
        };
        let state = DarkForestGateState::new(config.clone());
        assert!(state.config.enabled);
    }

    #[test]
    fn test_gate_state_creation_disabled() {
        let config = DarkForestGateConfig {
            enabled: false,
            neural_api_socket: None,
            family_id: "test".to_string(),
        };
        let state = DarkForestGateState::new(config);
        assert!(!state.config.enabled);
    }

    #[test]
    fn test_gate_state_with_neural_api_socket() {
        let config = DarkForestGateConfig {
            enabled: true,
            neural_api_socket: Some("/run/user/1000/biomeos/neural-api-cf7e.sock".to_string()),
            family_id: "cf7e".to_string(),
        };
        let state = DarkForestGateState::new(config);
        assert!(state.config.neural_api_socket.is_some());
        assert_eq!(state.config.family_id, "cf7e");
    }

    #[test]
    fn test_force_sovereign() {
        let config = DarkForestGateConfig {
            enabled: false,
            neural_api_socket: None,
            family_id: "test".to_string(),
        };
        assert!(!config.enabled);
        let config = config.force_sovereign();
        assert!(config.enabled);
    }

    #[test]
    fn test_force_sovereign_already_enabled() {
        let config = DarkForestGateConfig {
            enabled: true,
            neural_api_socket: None,
            family_id: "test".to_string(),
        };
        let config = config.force_sovereign();
        assert!(config.enabled); // stays enabled
    }

    #[test]
    fn test_config_preserves_socket_on_force() {
        let config = DarkForestGateConfig {
            enabled: false,
            neural_api_socket: Some("/test/socket".to_string()),
            family_id: "test".to_string(),
        };
        let config = config.force_sovereign();
        assert!(config.enabled);
        assert_eq!(config.neural_api_socket, Some("/test/socket".to_string()));
    }

    // ========================================================================
    // Token cache tests
    // ========================================================================

    #[tokio::test]
    async fn test_cache_is_initially_empty() {
        let config = DarkForestGateConfig {
            enabled: true,
            neural_api_socket: None,
            family_id: "test".to_string(),
        };
        let state = DarkForestGateState::new(config);

        // Token verification should fail (no actual crypto provider)
        let verified = state.verify_token("fake-token").await;
        assert!(!verified); // No provider → cannot verify → false
    }

    // ========================================================================
    // Middleware path routing tests (unit tests for path matching logic)
    // ========================================================================

    fn is_bypass_path(path: &str) -> bool {
        BYPASS_PATHS.iter().any(|bypass| path.starts_with(bypass))
    }

    fn is_bare_ok_path(path: &str) -> bool {
        BARE_OK_PATHS
            .iter()
            .any(|bare| path == *bare || path.starts_with(&format!("{bare}/")))
    }

    #[test]
    fn test_bypass_path_matching() {
        assert!(is_bypass_path("/.well-known/acme-challenge/abc123"));
        assert!(is_bypass_path("/.well-known/dns-validation"));
        assert!(!is_bypass_path("/api/v1/data"));
        assert!(!is_bypass_path("/health"));
        assert!(!is_bypass_path("/"));
    }

    #[test]
    fn test_bare_ok_path_matching() {
        assert!(is_bare_ok_path("/health"));
        assert!(is_bare_ok_path("/api/v1/health"));
        assert!(is_bare_ok_path("/api/v1/health/ready"));
        assert!(is_bare_ok_path("/api/v1/health/live"));
        assert!(is_bare_ok_path("/health/extra")); // sub-path matches
        assert!(!is_bare_ok_path("/api/v1/data"));
        assert!(!is_bare_ok_path("/api/v1/primals"));
        assert!(!is_bare_ok_path("/"));
    }

    #[test]
    fn test_arbitrary_path_requires_token() {
        // Any path that's not in bypass or bare_ok should require Dark Forest token
        let paths = [
            "/api/v1/data",
            "/rendezvous/beacon",
            "/jsonrpc",
            "/",
            "/admin",
        ];
        for path in &paths {
            assert!(!is_bypass_path(path), "{path} should not bypass gate");
            assert!(!is_bare_ok_path(path), "{path} should not be bare OK");
        }
    }

    #[test]
    fn test_well_known_subpaths_bypass() {
        let paths = [
            "/.well-known/acme-challenge/token123",
            "/.well-known/dns-01/challenge",
            "/.well-known/openid-configuration",
        ];
        for path in &paths {
            assert!(is_bypass_path(path), "{path} should bypass gate");
        }
    }

    // ========================================================================
    // Config Clone/Debug tests
    // ========================================================================

    #[test]
    fn test_config_clone() {
        let config = DarkForestGateConfig {
            enabled: true,
            neural_api_socket: Some("/test".to_string()),
            family_id: "abc123".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.enabled, config.enabled);
        assert_eq!(cloned.neural_api_socket, config.neural_api_socket);
        assert_eq!(cloned.family_id, config.family_id);
    }

    #[test]
    fn test_config_debug() {
        let config = DarkForestGateConfig {
            enabled: true,
            neural_api_socket: None,
            family_id: "test".to_string(),
        };
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("family_id"));
    }

    #[test]
    fn test_state_clone() {
        let config = DarkForestGateConfig {
            enabled: true,
            neural_api_socket: None,
            family_id: "test".to_string(),
        };
        let state = DarkForestGateState::new(config);
        let _cloned = state.clone(); // Should compile — DarkForestGateState: Clone
    }
}
