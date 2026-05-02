// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph operations — sign and verify deployment graphs.
//!
//! Signing delegates to the security provider via the `crypto.sign` capability
//! routed through the Neural API, keeping biomeOS sovereign (no embedded keys).

use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};
use biomeos_core::atomic_client::AtomicClient;
use biomeos_core::family_discovery::get_family_id;
use biomeos_core::socket_discovery::neural_api::resolve_neural_api_socket;
use biomeos_graph::integrity::compute_content_hash;
use tracing::info;

/// Sign a graph TOML file via security provider delegation.
///
/// Reads the file, computes its BLAKE3 content hash, sends it to
/// `crypto.sign` via the Neural API, and writes the resulting
/// `content_hash`, `signature`, and `signed_by` into `[graph.metadata]`.
pub async fn sign(path: PathBuf) -> Result<()> {
    info!("Signing graph: {}", path.display());

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Cannot read graph: {}", path.display()))?;

    let content_hash = compute_content_hash(&content);
    info!("Content hash: {content_hash}");

    let family_id = get_family_id();
    let socket = resolve_neural_api_socket(&family_id, None, None)
        .context("Neural API socket not found — is the NUCLEUS running?")?;

    let client = AtomicClient::unix(&socket);
    let response = client
        .call(
            "crypto.sign",
            serde_json::json!({
                "message": content_hash,
                "purpose": "coordination",
            }),
        )
        .await
        .context("crypto.sign RPC failed — is the security provider healthy?")?;

    let signature = response
        .get("signature")
        .and_then(|v| v.as_str())
        .context("crypto.sign response missing 'signature' field")?;
    let public_key = response
        .get("public_key")
        .and_then(|v| v.as_str())
        .context("crypto.sign response missing 'public_key' field")?;

    let updated = inject_signing_metadata(&content, &content_hash, signature, public_key);
    std::fs::write(&path, updated)
        .with_context(|| format!("Cannot write signed graph: {}", path.display()))?;

    info!("Graph signed: {}", path.display());
    info!("  content_hash: {content_hash}");
    info!("  signed_by:    {public_key}");
    Ok(())
}

/// Verify a graph file's integrity from the CLI.
pub async fn verify(path: PathBuf) -> Result<()> {
    info!("Verifying graph: {}", path.display());

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Cannot read graph: {}", path.display()))?;

    let (embedded_hash, embedded_sig, embedded_signer) = extract_signing_metadata(&content);

    let report = biomeos_graph::verify_integrity(
        &content,
        embedded_hash.as_deref(),
        embedded_sig.as_deref(),
        embedded_signer.as_deref(),
    );

    info!("Content hash: {}", report.computed_hash);
    match report.hash_match {
        Some(true) => info!("  Hash: MATCH"),
        Some(false) => info!("  Hash: MISMATCH"),
        None => info!("  Hash: (not embedded)"),
    }
    match report.signature_valid {
        Some(true) => info!("  Signature: VALID"),
        Some(false) => info!("  Signature: INVALID"),
        None => info!("  Signature: (unsigned)"),
    }
    if let Some(signer) = &report.signer {
        info!("  Signer: {signer}");
    }

    if report.hash_match == Some(false) {
        anyhow::bail!("Content hash mismatch");
    }
    if report.signature_valid == Some(false) {
        anyhow::bail!("Signature verification failed");
    }

    info!("Graph integrity OK");
    Ok(())
}

/// Inject signing metadata into the TOML content.
///
/// If `[graph.metadata]` exists, adds/replaces the signing keys.
/// Otherwise appends a new `[graph.metadata]` section.
fn inject_signing_metadata(
    toml_content: &str,
    content_hash: &str,
    signature: &str,
    signed_by: &str,
) -> String {
    let stripped = strip_old_signing(toml_content);

    if stripped.contains("[graph.metadata]") {
        let mut result = String::with_capacity(stripped.len() + 256);
        let mut injected = false;
        for line in stripped.lines() {
            result.push_str(line);
            result.push('\n');
            if !injected && line.trim() == "[graph.metadata]" {
                let _ = writeln!(result, "content_hash = \"{content_hash}\"");
                let _ = writeln!(result, "signature = \"{signature}\"");
                let _ = writeln!(result, "signed_by = \"{signed_by}\"");
                injected = true;
            }
        }
        result
    } else {
        format!(
            "{stripped}\n[graph.metadata]\ncontent_hash = \"{content_hash}\"\nsignature = \"{signature}\"\nsigned_by = \"{signed_by}\"\n"
        )
    }
}

/// Remove old signing fields so re-signing is idempotent.
fn strip_old_signing(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            let is_signing_field = (trimmed.starts_with("content_hash")
                || trimmed.starts_with("signature")
                || trimmed.starts_with("signed_by"))
                && trimmed.contains('=');
            !is_signing_field
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Best-effort extraction of signing metadata from raw TOML text.
fn extract_signing_metadata(content: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut hash = None;
    let mut sig = None;
    let mut signer = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("content_hash") {
            if let Some(val) = extract_toml_string_value(rest) {
                hash = Some(val);
            }
        } else if let Some(rest) = trimmed.strip_prefix("signed_by") {
            if let Some(val) = extract_toml_string_value(rest) {
                signer = Some(val);
            }
        } else if let Some(rest) = trimmed.strip_prefix("signature") {
            if let Some(val) = extract_toml_string_value(rest) {
                sig = Some(val);
            }
        }
    }

    (hash, sig, signer)
}

fn extract_toml_string_value(after_key: &str) -> Option<String> {
    let after_eq = after_key.trim().strip_prefix('=')?;
    let trimmed = after_eq.trim().strip_prefix('"')?;
    let end = trimmed.find('"')?;
    Some(trimmed[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_signing_with_existing_metadata() {
        let toml = "[graph]\nid = \"test\"\n\n[graph.metadata]\nauthor = \"me\"\n";
        let result = inject_signing_metadata(toml, "abc123", "sig456", "pub789");
        assert!(result.contains("content_hash = \"abc123\""));
        assert!(result.contains("signature = \"sig456\""));
        assert!(result.contains("signed_by = \"pub789\""));
        assert!(result.contains("author = \"me\""));
    }

    #[test]
    fn test_inject_signing_without_metadata() {
        let toml = "[graph]\nid = \"test\"\n";
        let result = inject_signing_metadata(toml, "abc", "sig", "pub");
        assert!(result.contains("[graph.metadata]"));
        assert!(result.contains("content_hash = \"abc\""));
    }

    #[test]
    fn test_strip_old_signing_idempotent() {
        let toml = "[graph.metadata]\ncontent_hash = \"old\"\nsignature = \"old\"\nsigned_by = \"old\"\nauthor = \"me\"\n";
        let stripped = strip_old_signing(toml);
        assert!(!stripped.contains("content_hash"));
        assert!(!stripped.contains("signature"));
        assert!(!stripped.contains("signed_by"));
        assert!(stripped.contains("author = \"me\""));
    }

    #[test]
    fn test_extract_signing_metadata() {
        let toml =
            "[graph.metadata]\ncontent_hash = \"abc\"\nsignature = \"def\"\nsigned_by = \"012\"\n";
        let (h, s, p) = extract_signing_metadata(toml);
        assert_eq!(h.unwrap(), "abc");
        assert_eq!(s.unwrap(), "def");
        assert_eq!(p.unwrap(), "012");
    }

    #[test]
    fn test_extract_signing_metadata_none() {
        let toml = "[graph]\nid = \"test\"\n";
        let (h, s, p) = extract_signing_metadata(toml);
        assert!(h.is_none());
        assert!(s.is_none());
        assert!(p.is_none());
    }
}
