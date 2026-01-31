//! BearDog JWT Client for Neural API
//!
//! Provides orchestrator-managed JWT secret provisioning from BearDog to primals.
//! This is proper separation of concerns - the orchestrator handles integration,
//! primals just receive configuration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{info, warn};

/// Request for JWT secret generation
#[derive(Debug, Serialize)]
struct JwtSecretRequest {
    jsonrpc: String,
    method: String,
    params: JwtSecretParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct JwtSecretParams {
    purpose: String,
    strength: String,
}

/// Response from BearDog JWT secret generation
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used in deserialization
struct JwtSecretResponse {
    jsonrpc: String,
    result: JwtSecretResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JwtSecretResult {
    secret: String,
    #[allow(dead_code)] // Part of BearDog API response, may be used for validation
    purpose: String,
    #[allow(dead_code)] // Part of BearDog API response
    strength: String,
    #[allow(dead_code)] // Part of BearDog API response
    byte_length: usize,
    #[serde(default)]
    #[allow(dead_code)] // Part of BearDog API response
    encoded_length: usize,
    #[serde(default)]
    #[allow(dead_code)] // Part of BearDog API response
    algorithm: String,
}

/// Fetch JWT secret from BearDog via JSON-RPC over Unix socket
///
/// # Arguments
/// * `socket_path` - Path to BearDog's Unix socket
/// * `purpose` - Purpose of the JWT secret (e.g., "nestgate_authentication")
///
/// # Returns
/// * `Ok(String)` - Base64-encoded JWT secret (512 bits / 88 characters)
/// * `Err` - If BearDog is unavailable or request fails
pub async fn fetch_jwt_secret_from_beardog(socket_path: &str, purpose: &str) -> Result<String> {
    info!("🔐 Fetching JWT secret from BearDog at: {}", socket_path);
    info!("   Purpose: {}", purpose);

    // Connect to BearDog via Unix socket
    let mut stream = UnixStream::connect(socket_path)
        .await
        .context(format!("Failed to connect to BearDog at {}", socket_path))?;

    // Create JSON-RPC request
    let request = JwtSecretRequest {
        jsonrpc: "2.0".to_string(),
        method: "beardog.generate_jwt_secret".to_string(),
        params: JwtSecretParams {
            purpose: purpose.to_string(),
            strength: "high".to_string(), // 512 bits, production-ready
        },
        id: 1,
    };

    // Serialize request
    let request_json =
        serde_json::to_string(&request).context("Failed to serialize JWT request")?;

    info!("   📤 Sending JSON-RPC request...");

    // Send request (JSON-RPC over Unix socket)
    stream
        .write_all(request_json.as_bytes())
        .await
        .context("Failed to write to BearDog socket")?;

    stream
        .write_all(b"\n")
        .await
        .context("Failed to write newline to BearDog socket")?;

    // Read response
    let mut response_buffer = Vec::new();
    let mut read_buffer = [0u8; 4096];

    loop {
        match stream.read(&mut read_buffer).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                response_buffer.extend_from_slice(&read_buffer[..n]);
                // Check if we have a complete JSON response
                if response_buffer.contains(&b'\n') {
                    break;
                }
            }
            Err(e) => return Err(e).context("Failed to read from BearDog socket"),
        }
    }

    let response_str =
        String::from_utf8(response_buffer).context("BearDog response was not valid UTF-8")?;

    info!("   📥 Received response from BearDog");

    // Parse JSON-RPC response
    let response: JwtSecretResponse = serde_json::from_str(response_str.trim())
        .context("Failed to parse BearDog JWT response")?;

    // Extract secret
    let secret = response.result.secret;

    info!("✅ JWT secret obtained from BearDog");
    info!("   Length: {} characters", secret.len());
    info!(
        "   Strength: {} ({} bytes)",
        response.result.strength, response.result.byte_length
    );
    info!("   Algorithm: {}", response.result.algorithm);

    Ok(secret)
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

    let secret = base64::encode(&secret_bytes);

    info!("✅ Secure random JWT secret generated");
    info!(
        "   Length: {} characters ({} bytes, {} bits)",
        secret.len(),
        bytes,
        bytes * 8
    );

    Ok(secret)
}

/// Provision JWT secret for a primal
///
/// Tries BearDog first (preferred), falls back to secure random if unavailable.
///
/// # Arguments
/// * `beardog_socket` - Optional path to BearDog socket
/// * `purpose` - Purpose of the JWT secret
///
/// # Returns
/// * JWT secret (base64-encoded, 512 bits minimum)
pub async fn provision_jwt_secret(beardog_socket: Option<&str>, purpose: &str) -> Result<String> {
    // Try BearDog first (preferred)
    if let Some(socket_path) = beardog_socket {
        match fetch_jwt_secret_from_beardog(socket_path, purpose).await {
            Ok(secret) => {
                info!("✅ Using BearDog-provided JWT secret (preferred)");
                return Ok(secret);
            }
            Err(e) => {
                warn!("⚠️ BearDog JWT fetch failed: {}", e);
                warn!("   Falling back to secure random generation...");
            }
        }
    } else {
        warn!("⚠️ No BearDog socket provided, using secure random JWT");
    }

    // Secure fallback: generate cryptographically strong random
    generate_secure_random_jwt(64) // 64 bytes = 512 bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_random_jwt() {
        let secret = generate_secure_random_jwt(64).unwrap();

        // Should be base64-encoded (64 bytes → ~88 characters)
        assert!(secret.len() >= 85 && secret.len() <= 90);

        // Should be different each time
        let secret2 = generate_secure_random_jwt(64).unwrap();
        assert_ne!(secret, secret2);
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_fallback() {
        // No BearDog available, should fall back to secure random
        let secret = provision_jwt_secret(None, "test_purpose").await.unwrap();

        assert!(secret.len() >= 85);
        assert!(!secret.is_empty());
    }
}
