// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BearDog integration for sub-federation cryptographic operations
//!
//! Lineage verification and key derivation via JSON-RPC.

use crate::{FederationError, FederationResult};
use biomeos_types::JsonRpcRequest;
use tracing::{debug, info};

/// Discover BearDog socket path via XDG-compliant SystemPaths
pub fn discover_beardog_socket() -> FederationResult<String> {
    if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
        return Ok(socket);
    }

    let paths = biomeos_types::SystemPaths::new_lazy();
    let security_provider = biomeos_types::CapabilityTaxonomy::resolve_to_primal("security")
        .unwrap_or(biomeos_types::primal_names::BEARDOG);
    let socket = paths.primal_socket(security_provider);
    if socket.exists() {
        return Ok(socket.to_string_lossy().to_string());
    }

    if let Ok(family_id) = std::env::var("BIOMEOS_FAMILY_ID") {
        let family_socket = paths.primal_socket(&format!("{}-{}", security_provider, family_id));
        if family_socket.exists() {
            return Ok(family_socket.to_string_lossy().to_string());
        }
    }

    Err(FederationError::Generic(
        "BearDog socket not found. Ensure BearDog is running.".to_string(),
    ))
}

/// Verify that all members share genetic lineage with the parent family
pub async fn verify_member_lineage(
    parent_family: &str,
    members: &[String],
) -> FederationResult<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let beardog_socket = discover_beardog_socket()?;

    let stream = UnixStream::connect(&beardog_socket)
        .await
        .map_err(|e| FederationError::Generic(format!("BearDog connection failed: {}", e)))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = JsonRpcRequest::new(
        "lineage.verify_members",
        serde_json::json!({
            "family_id": parent_family,
            "member_patterns": members
        }),
    );

    let request_str = serde_json::to_string(&request)
        .map_err(|e| FederationError::Generic(format!("JSON error: {}", e)))?
        + "\n";

    writer
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| FederationError::Generic(format!("Write error: {}", e)))?;
    writer
        .flush()
        .await
        .map_err(|e| FederationError::Generic(format!("Flush error: {}", e)))?;

    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| FederationError::Generic(format!("Read error: {}", e)))?;

    let response: serde_json::Value = serde_json::from_str(response_line.trim())
        .map_err(|e| FederationError::Generic(format!("JSON parse error: {}", e)))?;

    if let Some(error) = response.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown");
        return Err(FederationError::Generic(format!(
            "Lineage verification failed: {}",
            msg
        )));
    }

    let all_verified = response
        .get("result")
        .and_then(|r| r.get("all_verified"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if all_verified {
        info!("✅ Lineage verified for {} members", members.len());
        Ok(())
    } else {
        let failed = response
            .get("result")
            .and_then(|r| r.get("failed_members"))
            .and_then(|f| f.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        Err(FederationError::Generic(format!(
            "Lineage verification failed for: {}",
            failed
        )))
    }
}

/// Request a derived encryption key for this sub-federation
pub async fn request_subfederation_key(
    parent_family: &str,
    subfed_name: &str,
) -> FederationResult<String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let beardog_socket = discover_beardog_socket()?;

    let stream = UnixStream::connect(&beardog_socket)
        .await
        .map_err(|e| FederationError::Generic(format!("BearDog connection failed: {}", e)))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = JsonRpcRequest::new(
        "crypto.derive_subfederation_key",
        serde_json::json!({
            "family_id": parent_family,
            "subfederation_name": subfed_name,
            "purpose": "subfederation-encryption-v1"
        }),
    );

    let request_str = serde_json::to_string(&request)
        .map_err(|e| FederationError::Generic(format!("JSON error: {}", e)))?
        + "\n";

    writer
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| FederationError::Generic(format!("Write error: {}", e)))?;
    writer
        .flush()
        .await
        .map_err(|e| FederationError::Generic(format!("Flush error: {}", e)))?;

    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| FederationError::Generic(format!("Read error: {}", e)))?;

    let response: serde_json::Value = serde_json::from_str(response_line.trim())
        .map_err(|e| FederationError::Generic(format!("JSON parse error: {}", e)))?;

    if let Some(error) = response.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown");
        return Err(FederationError::Generic(format!(
            "Key derivation failed: {}",
            msg
        )));
    }

    let key_ref = response
        .get("result")
        .and_then(|r| r.get("key_ref"))
        .and_then(|k| k.as_str())
        .ok_or_else(|| FederationError::Generic("Missing key_ref in response".to_string()))?;

    debug!(
        "Derived key for sub-federation '{}': {}",
        subfed_name, key_ref
    );
    Ok(key_ref.to_string())
}
