// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph integrity verification.
//!
//! Provides content hashing and signature verification for deployment graphs.
//! Signing is delegated to BearDog (Tower atomic) via `crypto.sign` — biomeOS
//! never holds private keys. Verification uses the public key embedded in
//! `[graph.metadata.signed_by]`.

use crate::error::GraphError;
use crate::graph::GeneticsTier;

/// Result of a graph integrity check.
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// BLAKE3 hash computed from the canonical TOML content.
    pub computed_hash: String,
    /// Whether the embedded `content_hash` matches (None if no hash present).
    pub hash_match: Option<bool>,
    /// Whether the signature is valid (None if unsigned).
    pub signature_valid: Option<bool>,
    /// The public key that signed this graph (if present).
    pub signer: Option<String>,
}

impl IntegrityReport {
    /// Whether this graph passes integrity checks for the given genetics tier.
    ///
    /// - `None` / `Tag`: unsigned graphs accepted with warning
    /// - `MitoBeacon` / `Nuclear`: signature required
    #[must_use]
    pub fn acceptable_for_tier(&self, tier: Option<GeneticsTier>) -> bool {
        match tier {
            None | Some(GeneticsTier::None | GeneticsTier::Tag) => {
                // Hash must match if present
                self.hash_match.unwrap_or(true)
            }
            Some(GeneticsTier::MitoBeacon | GeneticsTier::Nuclear) => {
                // Signature required and must be valid
                self.hash_match.unwrap_or(false) && self.signature_valid.unwrap_or(false)
            }
        }
    }
}

/// Compute a BLAKE3 content hash of a graph TOML string.
///
/// The hash is computed over the TOML with signing-related metadata fields
/// (`content_hash`, `signature`, `signed_by`) stripped from `[graph.metadata]`,
/// ensuring the hash is stable across signing operations.
#[must_use]
pub fn compute_content_hash(toml_content: &str) -> String {
    let canonical = strip_signing_fields(toml_content);
    let hash = blake3::hash(canonical.as_bytes());
    hash.to_hex().to_string()
}

/// Verify graph integrity from raw TOML content and embedded metadata.
///
/// Computes the content hash and compares it against the embedded hash.
/// Signature verification requires the public key from `signed_by`.
pub fn verify_integrity(
    toml_content: &str,
    embedded_hash: Option<&str>,
    embedded_signature: Option<&str>,
    embedded_signer: Option<&str>,
) -> IntegrityReport {
    let computed_hash = compute_content_hash(toml_content);

    let hash_match = embedded_hash.map(|h| h == computed_hash);

    let signature_valid = match (embedded_signature, embedded_signer) {
        (Some(sig_hex), Some(pub_hex)) => {
            Some(verify_ed25519_signature(&computed_hash, sig_hex, pub_hex))
        }
        _ => None,
    };

    IntegrityReport {
        computed_hash,
        hash_match,
        signature_valid,
        signer: embedded_signer.map(String::from),
    }
}

/// Verify an Ed25519 signature over a hex-encoded message.
fn verify_ed25519_signature(message_hex: &str, signature_hex: &str, public_key_hex: &str) -> bool {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let Ok(pub_bytes) = hex_decode_32(public_key_hex) else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pub_bytes) else {
        return false;
    };

    let Ok(sig_bytes) = hex_decode_64(signature_hex) else {
        return false;
    };
    let Ok(signature) = Signature::from_slice(&sig_bytes) else {
        return false;
    };

    verifying_key
        .verify(message_hex.as_bytes(), &signature)
        .is_ok()
}

fn hex_decode_32(hex: &str) -> Result<[u8; 32], GraphError> {
    let bytes = hex_decode(hex)?;
    bytes
        .try_into()
        .map_err(|_| GraphError::Integrity(format!("expected 32 bytes, got {}", hex.len() / 2)))
}

fn hex_decode_64(hex: &str) -> Result<Vec<u8>, GraphError> {
    let bytes = hex_decode(hex)?;
    if bytes.len() != 64 {
        return Err(GraphError::Integrity(format!(
            "expected 64 bytes for signature, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, GraphError> {
    if !hex.len().is_multiple_of(2) {
        return Err(GraphError::Integrity(
            "hex string has odd length".to_string(),
        ));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| GraphError::Integrity(format!("invalid hex: {e}")))
        })
        .collect()
}

/// Strip signing-related fields from `[graph.metadata]` so the hash is stable.
fn strip_signing_fields(toml_content: &str) -> String {
    let mut result = String::with_capacity(toml_content.len());
    for line in toml_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("content_hash")
            || trimmed.starts_with("signature")
            || trimmed.starts_with("signed_by")
        {
            // Only skip if it looks like a key = "value" assignment in metadata context
            if trimmed.contains('=') {
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_hash_deterministic() {
        let toml = "[graph]\nid = \"test\"\n";
        let h1 = compute_content_hash(toml);
        let h2 = compute_content_hash(toml);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // BLAKE3 hex = 64 chars
    }

    #[test]
    fn test_compute_content_hash_strips_signing() {
        // Same logical graph: `[graph.metadata]` present in both so stripping
        // signing keys leaves identical canonical TOML.
        let base = "[graph]\nid = \"test\"\n\n[graph.metadata]\n";
        let signed = "[graph]\nid = \"test\"\n\n[graph.metadata]\ncontent_hash = \"abc\"\nsignature = \"def\"\nsigned_by = \"012\"\n";
        assert_eq!(compute_content_hash(base), compute_content_hash(signed));
    }

    #[test]
    fn test_verify_integrity_no_metadata() {
        let toml = "[graph]\nid = \"test\"\n";
        let report = verify_integrity(toml, None, None, None);
        assert!(report.hash_match.is_none());
        assert!(report.signature_valid.is_none());
        assert!(report.acceptable_for_tier(None));
        assert!(report.acceptable_for_tier(Some(GeneticsTier::Tag)));
        assert!(!report.acceptable_for_tier(Some(GeneticsTier::MitoBeacon)));
    }

    #[test]
    fn test_verify_integrity_correct_hash() {
        let toml = "[graph]\nid = \"test\"\n";
        let hash = compute_content_hash(toml);
        let report = verify_integrity(toml, Some(&hash), None, None);
        assert_eq!(report.hash_match, Some(true));
    }

    #[test]
    fn test_verify_integrity_wrong_hash() {
        let toml = "[graph]\nid = \"test\"\n";
        let report = verify_integrity(toml, Some("deadbeef"), None, None);
        assert_eq!(report.hash_match, Some(false));
    }

    #[test]
    fn test_verify_integrity_bad_signature_hex() {
        let toml = "[graph]\nid = \"test\"\n";
        let hash = compute_content_hash(toml);
        let report = verify_integrity(toml, Some(&hash), Some("not-hex"), Some("also-not-hex"));
        assert_eq!(report.signature_valid, Some(false));
    }

    #[test]
    fn test_strip_signing_fields() {
        let input = "id = \"test\"\ncontent_hash = \"abc\"\nname = \"foo\"\nsignature = \"def\"\nsigned_by = \"012\"\nversion = \"1\"\n";
        let stripped = strip_signing_fields(input);
        assert!(!stripped.contains("content_hash"));
        assert!(!stripped.contains("signature"));
        assert!(!stripped.contains("signed_by"));
        assert!(stripped.contains("id = \"test\""));
        assert!(stripped.contains("name = \"foo\""));
        assert!(stripped.contains("version = \"1\""));
    }

    #[test]
    fn test_acceptable_for_tier_none_unsigned() {
        let report = IntegrityReport {
            computed_hash: "abc".into(),
            hash_match: None,
            signature_valid: None,
            signer: None,
        };
        assert!(report.acceptable_for_tier(None));
        assert!(report.acceptable_for_tier(Some(GeneticsTier::None)));
        assert!(report.acceptable_for_tier(Some(GeneticsTier::Tag)));
        assert!(!report.acceptable_for_tier(Some(GeneticsTier::MitoBeacon)));
        assert!(!report.acceptable_for_tier(Some(GeneticsTier::Nuclear)));
    }
}
