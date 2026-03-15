// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BearDog JWT Client for Neural API
//!
//! Provides orchestrator-managed JWT secret provisioning from BearDog to primals.
//! This is proper separation of concerns - the orchestrator handles integration,
//! primals just receive configuration.
//!
//! **Universal IPC v3.0**: Uses AtomicClient for multi-transport support.

use anyhow::{Context, Result};
use biomeos_core::atomic_client::AtomicClient;
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, info, warn};

/// JWT secret result from BearDog
#[derive(Debug, Deserialize)]
struct JwtSecretResult {
    secret: String,
    #[serde(rename = "purpose")]
    _purpose: String,
    strength: String,
    byte_length: usize,
    #[serde(default, rename = "encoded_length")]
    _encoded_length: usize,
    #[serde(default)]
    algorithm: String,
}

/// Fetch JWT secret from BearDog via AtomicClient (Universal IPC v3.0)
///
/// **Universal IPC v3.0**: Uses `AtomicClient` with automatic transport fallback.
/// This supports Unix sockets, abstract sockets (Android), and TCP (cross-device).
///
/// # Arguments
/// * `socket_path` - Path to BearDog's socket (or use discovery if None)
/// * `purpose` - Purpose of the JWT secret (e.g., "nestgate_authentication")
///
/// # Returns
/// * `Ok(String)` - Base64-encoded JWT secret (512 bits / 88 characters)
/// * `Err` - If BearDog is unavailable or request fails
pub async fn fetch_jwt_secret_from_beardog(socket_path: &str, purpose: &str) -> Result<String> {
    info!("Fetching JWT secret from BearDog at: {}", socket_path);
    info!("   Purpose: {}", purpose);

    // Create AtomicClient with explicit socket path for backwards compatibility
    let client = AtomicClient::unix(socket_path);

    debug!("   Sending JSON-RPC request via AtomicClient...");

    // Call BearDog's JWT secret generation method
    let response = client
        .call(
            "beardog.generate_jwt_secret",
            json!({
                "purpose": purpose,
                "strength": "high" // 512 bits, production-ready
            }),
        )
        .await
        .context(format!(
            "Failed to fetch JWT secret from BearDog at {socket_path}"
        ))?;

    debug!("   Received response from BearDog");

    // Parse the result
    let result: JwtSecretResult =
        serde_json::from_value(response).context("Failed to parse BearDog JWT response")?;

    info!("JWT secret obtained from BearDog");
    info!("   Length: {} characters", result.secret.len());
    info!(
        "   Strength: {} ({} bytes)",
        result.strength, result.byte_length
    );
    info!("   Algorithm: {}", result.algorithm);

    Ok(result.secret)
}

/// Fetch JWT secret from security provider using auto-discovery (Universal IPC v3.0)
///
/// **Universal IPC v3.0**: Uses `AtomicClient::discover()` with automatic
/// transport fallback. Discovers the security provider via environment, XDG,
/// abstract socket, or TCP.
///
/// **Capability-based discovery**: Resolves the security provider via:
/// 1. `BIOMEOS_SECURITY_PROVIDER` env var (explicit override)
/// 2. Capability taxonomy (`CapabilityTaxonomy::resolve_to_primal("security")`)
/// 3. "beardog" bootstrap fallback (only when strict discovery is off)
///
/// # Arguments
/// * `purpose` - Purpose of the JWT secret (e.g., "nestgate_authentication")
///
/// # Returns
/// * `Ok(String)` - Base64-encoded JWT secret
/// * `Err` - If security provider is unavailable or request fails
pub async fn fetch_jwt_secret_with_discovery(purpose: &str) -> Result<String> {
    use biomeos_types::capability_taxonomy::CapabilityTaxonomy;

    // Capability-based provider resolution (3-tier)
    let provider = std::env::var("BIOMEOS_SECURITY_PROVIDER")
        .ok()
        .or_else(|| CapabilityTaxonomy::resolve_to_primal("security").map(String::from))
        .unwrap_or_else(|| "beardog".to_string());

    info!(
        "Discovering security provider '{}' for JWT secret generation...",
        provider
    );
    info!("   Purpose: {}", purpose);

    // Use auto-discovery with fallback (Unix → Abstract → TCP)
    let client = AtomicClient::discover(&provider)
        .await
        .context(format!("Failed to discover security provider '{provider}'"))?;

    info!("   Discovered {} via: {}", provider, client.endpoint());

    // Use capability-namespaced method name for forward compatibility
    // The security provider responds to both "security.generate_jwt_secret"
    // and legacy "beardog.generate_jwt_secret" method names.
    let response = client
        .call(
            "security.generate_jwt_secret",
            json!({
                "purpose": purpose,
                "strength": "high"
            }),
        )
        .await
        .context("Failed to fetch JWT secret from security provider")?;

    // Parse the result
    let result: JwtSecretResult = serde_json::from_value(response)
        .context("Failed to parse security provider JWT response")?;

    info!(
        "JWT secret obtained from security provider '{}' via {}",
        provider,
        client.endpoint()
    );
    info!("   Length: {} characters", result.secret.len());

    Ok(result.secret)
}

/// Generate secure random JWT secret as fallback
///
/// This is used when BearDog is unavailable. Still cryptographically secure,
/// but BearDog is preferred for consistency across NUCLEUS.
///
/// # Arguments
/// * `bytes` - Number of random bytes to generate (default: 64 for 512 bits)
///
/// # Returns
/// * Base64-encoded random secret
pub fn generate_secure_random_jwt(bytes: usize) -> Result<String> {
    use rand::RngCore;

    warn!("⚠️ Generating fallback JWT secret (BearDog unavailable)");
    warn!("   This is cryptographically secure but not coordinated with NUCLEUS");

    let mut rng = rand::thread_rng();
    let mut secret_bytes = vec![0u8; bytes];
    rng.fill_bytes(&mut secret_bytes);

    use base64::Engine;
    let secret = base64::engine::general_purpose::STANDARD.encode(&secret_bytes);

    info!("✅ Secure random JWT secret generated");
    info!(
        "   Length: {} characters ({} bytes, {} bits)",
        secret.len(),
        bytes,
        bytes * 8
    );

    Ok(secret)
}

/// Provision JWT secret for a primal (Universal IPC v3.0)
///
/// **Universal IPC v3.0**: Uses automatic discovery if no socket path provided.
/// Tries BearDog first (preferred), falls back to secure random if unavailable.
///
/// # Arguments
/// * `beardog_socket` - Optional path to BearDog socket (uses discovery if None)
/// * `purpose` - Purpose of the JWT secret
///
/// # Returns
/// * JWT secret (base64-encoded, 512 bits minimum)
pub async fn provision_jwt_secret(beardog_socket: Option<&str>, purpose: &str) -> Result<String> {
    // Try BearDog first (preferred)
    if let Some(socket_path) = beardog_socket {
        // Explicit socket path provided
        match fetch_jwt_secret_from_beardog(socket_path, purpose).await {
            Ok(secret) => {
                info!("Using BearDog-provided JWT secret (explicit socket)");
                return Ok(secret);
            }
            Err(e) => {
                warn!("BearDog JWT fetch failed at {}: {}", socket_path, e);
                warn!("   Trying auto-discovery...");
            }
        }
    }

    // Try auto-discovery (Universal IPC v3.0)
    match fetch_jwt_secret_with_discovery(purpose).await {
        Ok(secret) => {
            info!("Using BearDog-provided JWT secret (auto-discovered)");
            return Ok(secret);
        }
        Err(e) => {
            warn!("BearDog auto-discovery failed: {}", e);
            warn!("   Falling back to secure random generation...");
        }
    }

    // Secure fallback: generate cryptographically strong random
    generate_secure_random_jwt(64) // 64 bytes = 512 bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_random_jwt() {
        let secret = generate_secure_random_jwt(64).expect("64 bytes should succeed");

        // Should be base64-encoded (64 bytes → ~88 characters)
        assert!(
            secret.len() >= 85 && secret.len() <= 90,
            "64 bytes → ~88 chars base64, got {}",
            secret.len()
        );

        // Should be different each time
        let secret2 = generate_secure_random_jwt(64).expect("second call should succeed");
        assert_ne!(secret, secret2);
    }

    #[test]
    fn test_generate_secure_random_jwt_various_sizes() {
        for bytes in [32, 64, 128, 256] {
            let secret = generate_secure_random_jwt(bytes).expect("should succeed");
            let expected_min = (bytes * 4 / 3) - 2; // base64 padding
            let expected_max = (bytes * 4 / 3) + 4;
            assert!(
                secret.len() >= expected_min && secret.len() <= expected_max,
                "{} bytes should produce ~{} chars, got {}",
                bytes,
                expected_min,
                secret.len()
            );
        }
    }

    #[test]
    fn test_generate_secure_random_jwt_valid_base64() {
        use base64::Engine;
        let secret = generate_secure_random_jwt(64).expect("should succeed");
        let decoded = base64::engine::general_purpose::STANDARD.decode(&secret);
        assert!(decoded.is_ok(), "Output should be valid base64: {secret}");
        assert_eq!(decoded.unwrap().len(), 64, "Decoded should be 64 bytes");
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_fallback() {
        let secret = provision_jwt_secret(None, "test_purpose")
            .await
            .expect("Should fall back to secure random when BearDog unavailable");

        assert!(secret.len() >= 85, "Should be 512-bit equivalent");
        assert!(!secret.is_empty());
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_with_failing_socket_falls_back() {
        // Explicit socket that doesn't exist - should try, fail, then fall back
        let secret =
            provision_jwt_secret(Some("/tmp/nonexistent-beardog-xyz.sock"), "test_fallback")
                .await
                .expect("Should fall back to secure random when socket fails");

        assert!(secret.len() >= 85);
    }

    #[tokio::test]
    async fn test_fetch_jwt_secret_from_beardog_unavailable() {
        let result =
            fetch_jwt_secret_from_beardog("/tmp/nonexistent-beardog-xyz.sock", "test_purpose")
                .await;

        let err = result.expect_err("Should fail when BearDog socket does not exist");
        assert!(
            err.to_string().contains("Failed to fetch")
                || err.to_string().contains("BearDog")
                || err.to_string().contains("connect")
                || err.to_string().contains("Connection refused")
                || err.to_string().contains("No such file"),
            "Error should mention BearDog/connection: {err}"
        );
    }

    #[tokio::test]
    async fn test_fetch_jwt_secret_with_discovery_fails_without_provider() {
        // When no BearDog is running, discovery fails at connect or discovery
        let result = fetch_jwt_secret_with_discovery("test_purpose").await;

        if let Err(e) = result {
            assert!(
                e.to_string().contains("discover")
                    || e.to_string().contains("Failed")
                    || e.to_string().contains("connect")
                    || e.to_string().contains("BearDog")
                    || e.to_string().contains("security"),
                "Error should be about discovery/connection: {e}"
            );
        }
        // If it succeeds (e.g. BearDog running in CI), that's also valid
    }
}
