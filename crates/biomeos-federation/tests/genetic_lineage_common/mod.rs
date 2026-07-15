// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shared helpers for genetic lineage integration tests (`genetic_lineage_*.rs`).

#![allow(dead_code, reason = "each integration test binary uses a subset of helpers")]

use anyhow::Result;
use biomeos_federation::security_client::SecurityProviderClient;
use biomeos_types::identifiers::FamilyId;
use sha2::{Digest, Sha256};

/// Create a test BearDog client
pub async fn create_test_client() -> Result<SecurityProviderClient> {
    SecurityProviderClient::with_endpoint("unix:///tmp/beardog-test.sock")
}

/// Generate a test family ID
pub fn test_family_id() -> String {
    FamilyId::generate().to_string()
}

/// Generate a test seed hash (SHA-256 format)
pub fn test_seed_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash_bytes = hasher.finalize();
    // Convert to hex string manually
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let hex_string: String = hash_bytes
        .iter()
        .flat_map(|&b| {
            [
                HEX_DIGITS[(b >> 4) as usize],
                HEX_DIGITS[(b & 0xf) as usize],
            ]
        })
        .map(char::from)
        .collect();
    format!("sha256:{hex_string}")
}
