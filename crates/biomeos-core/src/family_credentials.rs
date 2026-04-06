// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Family Credentials Management
//!
//! This module provides secure handling of family seeds and credentials for
//! genetic lineage authentication in the biomeOS ecosystem.
//!
//! ## Security Model
//!
//! Family seeds are cryptographic secrets that establish genetic lineage.
//! They must be:
//! - Never logged or printed
//! - Zeroized on drop
//! - Loaded from secure sources only
//! - Validated before use
//!
//! ## Usage
//!
//! ```ignore
//! // Load from environment (development/testing)
//! let creds = FamilyCredentials::from_env()?;
//!
//! // Load from encrypted file (production)
//! let key = load_encryption_key()?;
//! let creds = FamilyCredentials::from_encrypted_file("family.enc", &key)?;
//!
//! // Use credentials
//! let family_id = creds.family_id();
//! let seed = creds.seed_ref(); // Temporary reference, never cloned
//! ```

use biomeos_types::identifiers::FamilyId;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use std::io::Write;
use std::path::Path;
use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha256 = Hmac<Sha256>;

/// Error type for credentials operations
#[derive(Debug, thiserror::Error)]
pub enum CredentialsError {
    /// Credentials were present but invalid
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    /// Required credentials were not found
    #[error("Missing required credentials: {0}")]
    MissingCredentials(String),
}

// Alias for backward compatibility
type BirdSongError = CredentialsError;

/// Version 2 credential file format: payload + HMAC for integrity
#[derive(Debug, Serialize, Deserialize)]
struct CredentialFileV2 {
    version: u8,
    payload: String,
    hmac: String,
}

/// Payload structure for version 2 format
#[derive(Debug, Serialize, Deserialize)]
struct CredentialPayload {
    family_id: String,
    family_seed: String,
}

fn load_credential_v2(file: &CredentialFileV2) -> Result<FamilyCredentials, BirdSongError> {
    if file.version != 2 {
        return Err(BirdSongError::InvalidCredentials(format!(
            "Unsupported credential version: {}",
            file.version
        )));
    }

    let payload_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &file.payload).map_err(
            |e| BirdSongError::InvalidCredentials(format!("Invalid payload base64: {e}")),
        )?;

    let payload: CredentialPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid payload JSON: {e}")))?;

    let seed = SecretSeed::new(payload.family_seed.clone())?;
    let seed_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, seed.as_str())
            .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid seed base64: {e}")))?;

    let expected_tag =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &file.hmac)
            .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid HMAC base64: {e}")))?;

    let mut mac = HmacSha256::new_from_slice(&seed_bytes)
        .map_err(|e| BirdSongError::InvalidCredentials(format!("HMAC init failed: {e}")))?;
    mac.update(&payload_bytes);
    mac.verify_slice(&expected_tag).map_err(|_| {
        BirdSongError::InvalidCredentials(
            "Credential file integrity check failed (HMAC mismatch)".to_string(),
        )
    })?;

    let family_id = FamilyId::new(payload.family_id);
    FamilyCredentials::new(family_id, seed)
}

fn load_credential_legacy(contents: &str) -> Result<FamilyCredentials, BirdSongError> {
    #[derive(Deserialize)]
    struct LegacyFormat {
        family_id: String,
        family_seed: String,
    }

    let file: LegacyFormat = serde_json::from_str(contents)
        .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid JSON: {e}")))?;

    let family_id = FamilyId::new(file.family_id);
    let seed = SecretSeed::new(file.family_seed)?;
    FamilyCredentials::new(family_id, seed)
}

/// Secure wrapper for family seed that zeroizes on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretSeed {
    /// Base64-encoded seed bytes
    seed: String,
}

impl SecretSeed {
    /// Create a new secret seed from base64 string
    pub fn new(seed: String) -> Result<Self, BirdSongError> {
        // Validate base64 format
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &seed)
            .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid base64 seed: {e}")))?;

        Ok(Self { seed })
    }

    /// Get a reference to the seed (temporary use only)
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.seed
    }

    /// Validate seed format and length
    pub fn validate(&self) -> Result<(), BirdSongError> {
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &self.seed)
                .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid base64: {e}")))?;

        // Require minimum seed length (32 bytes = 256 bits)
        if decoded.len() < 32 {
            return Err(BirdSongError::InvalidCredentials(format!(
                "Seed too short: {} bytes (minimum 32)",
                decoded.len()
            )));
        }

        Ok(())
    }
}

impl std::fmt::Debug for SecretSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretSeed([REDACTED])")
    }
}

/// Family credentials containing ID and seed
#[derive(Clone)]
pub struct FamilyCredentials {
    family_id: FamilyId,
    seed: SecretSeed,
}

impl FamilyCredentials {
    /// Create new family credentials
    pub fn new(family_id: FamilyId, seed: SecretSeed) -> Result<Self, BirdSongError> {
        let creds = Self { family_id, seed };
        creds.validate()?;
        Ok(creds)
    }

    /// Load credentials from environment variables
    ///
    /// Expects:
    /// - `FAMILY_ID` or `BEARDOG_FAMILY_ID`
    /// - `FAMILY_SEED` or `BEARDOG_FAMILY_SEED`
    pub fn from_env() -> Result<Self, BirdSongError> {
        Self::from_env_values(
            std::env::var("FAMILY_ID")
                .ok()
                .or_else(|| std::env::var("BEARDOG_FAMILY_ID").ok())
                .as_deref(),
            std::env::var("FAMILY_SEED")
                .ok()
                .or_else(|| std::env::var("BEARDOG_FAMILY_SEED").ok())
                .as_deref(),
        )
    }

    /// Load credentials from explicit values (same semantics as [`Self::from_env`]).
    pub fn from_env_values(
        family_id: Option<&str>,
        family_seed: Option<&str>,
    ) -> Result<Self, BirdSongError> {
        let family_id_str = family_id.ok_or_else(|| {
            BirdSongError::InvalidCredentials(
                "Missing FAMILY_ID or BEARDOG_FAMILY_ID environment variable".to_string(),
            )
        })?;

        let family_seed = family_seed.ok_or_else(|| {
            BirdSongError::InvalidCredentials(
                "Missing FAMILY_SEED or BEARDOG_FAMILY_SEED environment variable".to_string(),
            )
        })?;

        let family_id = FamilyId::new(family_id_str);

        let seed = SecretSeed::new(family_seed.to_string())?;

        Self::new(family_id, seed)
    }

    /// Load credentials from file.
    ///
    /// Supports two formats:
    /// - **Version 2** (preferred): Structured format with HMAC integrity verification.
    ///   Uses file permissions (0o600 recommended) and HMAC-SHA256 over payload.
    /// - **Version 1** (legacy): Plaintext JSON for backward compatibility.
    ///
    /// The `_encryption_key` parameter is reserved for future age/keychain integration.
    pub fn from_encrypted_file(
        path: impl AsRef<Path>,
        _encryption_key: &[u8],
    ) -> Result<Self, BirdSongError> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|e| BirdSongError::InvalidCredentials(format!("Failed to read file: {e}")))?;

        // Try versioned format first
        if let Ok(v2) = serde_json::from_str::<CredentialFileV2>(&contents) {
            return load_credential_v2(&v2);
        }

        // Fallback to legacy plaintext JSON (version 1)
        load_credential_legacy(&contents)
    }

    /// Save credentials to file with secure storage.
    ///
    /// - Writes with file permissions 0o600 (owner read/write only)
    /// - Uses version 2 format with HMAC-SHA256 integrity verification
    /// - Payload is base64-encoded; HMAC is computed over raw payload bytes using the seed
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), BirdSongError> {
        let payload = CredentialPayload {
            family_id: self.family_id.as_str().to_string(),
            family_seed: self.seed.as_str().to_string(),
        };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| BirdSongError::InvalidCredentials(format!("Failed to serialize: {e}")))?;
        let payload_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            payload_json.as_bytes(),
        );

        let seed_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            self.seed.as_str(),
        )
        .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid seed for HMAC: {e}")))?;

        let mut mac = HmacSha256::new_from_slice(&seed_bytes)
            .map_err(|e| BirdSongError::InvalidCredentials(format!("HMAC init failed: {e}")))?;
        mac.update(payload_json.as_bytes());
        let tag = mac.finalize().into_bytes();
        let hmac_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, tag.as_slice());

        let file = CredentialFileV2 {
            version: 2,
            payload: payload_b64,
            hmac: hmac_b64,
        };
        let contents = serde_json::to_string(&file).map_err(|e| {
            BirdSongError::InvalidCredentials(format!("Failed to serialize file: {e}"))
        })?;

        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
            .map_err(|e| {
                BirdSongError::InvalidCredentials(format!("Failed to create file: {e}"))
            })?;
        f.write_all(contents.as_bytes())
            .map_err(|e| BirdSongError::InvalidCredentials(format!("Failed to write file: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            f.set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(|e| {
                    BirdSongError::InvalidCredentials(format!(
                        "Failed to set file permissions 0o600: {e}"
                    ))
                })?;
        }

        Ok(())
    }

    /// Get the family ID
    #[must_use]
    pub const fn family_id(&self) -> &FamilyId {
        &self.family_id
    }

    /// Get a reference to the seed (temporary use only)
    #[must_use]
    pub fn seed_ref(&self) -> &str {
        self.seed.as_str()
    }

    /// Validate credentials
    pub fn validate(&self) -> Result<(), BirdSongError> {
        // Validate family ID format
        if self.family_id.as_str().is_empty() {
            return Err(BirdSongError::InvalidCredentials(
                "Family ID cannot be empty".to_string(),
            ));
        }

        // Validate seed
        self.seed.validate()?;

        Ok(())
    }
}

impl std::fmt::Debug for FamilyCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FamilyCredentials")
            .field("family_id", &self.family_id)
            .field("seed", &"[REDACTED]")
            .finish()
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_seed_validation() {
        // Valid seed (32+ bytes when decoded)
        let valid_seed = "iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=";
        let seed = SecretSeed::new(valid_seed.to_string()).unwrap();
        assert!(seed.validate().is_ok());

        // Invalid base64
        let invalid_seed = "not-valid-base64!!!";
        assert!(SecretSeed::new(invalid_seed.to_string()).is_err());

        // Too short (less than 32 bytes)
        let short_seed =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"short");
        let seed = SecretSeed::new(short_seed).unwrap();
        assert!(seed.validate().is_err());
    }

    #[test]
    fn test_family_credentials_validation() {
        let family_id = FamilyId::new("test-family");
        let valid_seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();

        let creds = FamilyCredentials::new(family_id, valid_seed).unwrap();
        assert_eq!(creds.family_id().as_str(), "test-family");
        assert!(creds.validate().is_ok());
    }

    #[test]
    fn test_secret_seed_debug() {
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let debug_str = format!("{seed:?}");
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("iIDn"));
    }

    #[test]
    fn test_family_credentials_debug() {
        let family_id = FamilyId::new("test-family");
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let creds = FamilyCredentials::new(family_id, seed).unwrap();

        let debug_str = format!("{creds:?}");
        assert!(debug_str.contains("test-family"));
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("iIDn"));
    }

    #[test]
    fn test_from_env_missing() {
        assert!(FamilyCredentials::from_env_values(None, None).is_err());
    }

    #[test]
    fn test_from_env_success() {
        let creds = FamilyCredentials::from_env_values(
            Some("test-family"),
            Some("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="),
        )
        .unwrap();
        assert_eq!(creds.family_id().as_str(), "test-family");
    }

    #[test]
    fn test_from_encrypted_file_legacy_format() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let json = r#"{"family_id":"legacy-family","family_seed":"iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="}"#;
        std::fs::write(temp.path(), json).unwrap();

        let creds =
            FamilyCredentials::from_encrypted_file(temp.path(), b"").expect("load legacy format");
        assert_eq!(creds.family_id().as_str(), "legacy-family");
    }

    #[test]
    fn test_save_and_load_v2_format() {
        let family_id = FamilyId::new("save-test-family");
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let creds = FamilyCredentials::new(family_id, seed).unwrap();

        let temp = tempfile::NamedTempFile::new().unwrap();
        creds.save_to_file(temp.path()).expect("save");

        let loaded =
            FamilyCredentials::from_encrypted_file(temp.path(), b"").expect("load v2 format");
        assert_eq!(loaded.family_id().as_str(), "save-test-family");
    }

    #[test]
    fn test_v2_hmac_tamper_detection() {
        let family_id = FamilyId::new("tamper-test");
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let creds = FamilyCredentials::new(family_id, seed).unwrap();

        let temp = tempfile::NamedTempFile::new().unwrap();
        creds.save_to_file(temp.path()).expect("save");

        // Tamper with the payload: decode, modify family_id, re-encode (invalidates HMAC)
        let mut file: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path()).unwrap()).unwrap();
        let payload_b64 = file["payload"].as_str().unwrap();
        let mut payload_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, payload_b64)
                .unwrap();
        let mut payload: serde_json::Value = serde_json::from_slice(&payload_bytes).unwrap();
        payload["family_id"] = serde_json::Value::String("evil-family".to_string());
        payload_bytes = serde_json::to_vec(&payload).unwrap();
        file["payload"] = serde_json::Value::String(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &payload_bytes,
        ));
        std::fs::write(temp.path(), serde_json::to_string(&file).unwrap()).unwrap();

        let result = FamilyCredentials::from_encrypted_file(temp.path(), b"");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("integrity check failed")
        );
    }

    #[test]
    fn test_credentials_error_display_variants() {
        let invalid = CredentialsError::InvalidCredentials("bad seed".to_string());
        assert!(invalid.to_string().contains("Invalid credentials"));
        assert!(invalid.to_string().contains("bad seed"));

        let missing = CredentialsError::MissingCredentials("FAMILY_ID".to_string());
        assert!(missing.to_string().contains("Missing required credentials"));
        assert!(missing.to_string().contains("FAMILY_ID"));
    }

    #[test]
    fn test_v2_file_rejects_unsupported_version() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let file = r#"{"version":1,"payload":"e30=","hmac":"AAAA"}"#;
        std::fs::write(temp.path(), file).unwrap();
        let err = FamilyCredentials::from_encrypted_file(temp.path(), b"").unwrap_err();
        assert!(err.to_string().contains("Unsupported credential version"));
    }

    #[test]
    fn test_v2_file_rejects_invalid_payload_base64() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let file = r#"{"version":2,"payload":"@@@not-base64@@@","hmac":"AAAA"}"#;
        std::fs::write(temp.path(), file).unwrap();
        let err = FamilyCredentials::from_encrypted_file(temp.path(), b"").unwrap_err();
        assert!(
            err.to_string().contains("base64") || err.to_string().contains("Invalid"),
            "{err}"
        );
    }

    #[test]
    fn test_v2_file_rejects_invalid_hmac_base64() {
        let family_id = FamilyId::new("hmac-b64-test");
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let creds = FamilyCredentials::new(family_id, seed).unwrap();
        let temp = tempfile::NamedTempFile::new().unwrap();
        creds.save_to_file(temp.path()).unwrap();
        let mut v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path()).unwrap()).unwrap();
        v["hmac"] = serde_json::Value::String("not-valid-base64!!!".to_string());
        std::fs::write(temp.path(), serde_json::to_string(&v).unwrap()).unwrap();
        let err = FamilyCredentials::from_encrypted_file(temp.path(), b"").unwrap_err();
        assert!(
            err.to_string().contains("HMAC") || err.to_string().contains("base64"),
            "{err}"
        );
    }

    #[test]
    fn test_family_credentials_rejects_empty_family_id() {
        let family_id = FamilyId::new("");
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let err = FamilyCredentials::new(family_id, seed).unwrap_err();
        assert!(err.to_string().contains("empty") || err.to_string().contains("Family ID"));
    }

    #[test]
    fn test_secret_seed_validate_wrong_length_after_decode() {
        let short_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [0u8; 31]);
        let seed = SecretSeed::new(short_b64).unwrap();
        let err = seed.validate().unwrap_err();
        assert!(err.to_string().contains("32") || err.to_string().contains("short"));
    }

    #[test]
    fn test_legacy_json_rejects_malformed() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "{ not json").unwrap();
        let err = FamilyCredentials::from_encrypted_file(temp.path(), b"").unwrap_err();
        assert!(err.to_string().contains("JSON") || err.to_string().contains("Invalid"));
    }
}
