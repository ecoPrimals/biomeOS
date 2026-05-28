// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

mod sovereign_mesh_helpers;
use sovereign_mesh_helpers::*;

/// Verify crypto properties: same plaintext produces different ciphertext each time
#[tokio::test]
async fn e2e_crypto_nondeterministic() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");
    let node = SimulatedNode::new("pixel8a", family_seed, "1.2.3.4:5678");

    let data = b"same data twice";
    let enc1 = node.encrypt_data(data);
    let enc2 = node.encrypt_data(data);

    assert_ne!(
        enc1, enc2,
        "Same plaintext must produce different ciphertext (nonce randomness)"
    );

    let dec1 = node.decrypt_data(&enc1).unwrap();
    let dec2 = node.decrypt_data(&enc2).unwrap();
    assert_eq!(dec1, data);
    assert_eq!(dec2, data);
}

/// Verify tampered ciphertext is rejected
#[tokio::test]
async fn e2e_tamper_detection() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");
    let node = SimulatedNode::new("pixel8a", family_seed, "1.2.3.4:5678");

    let encrypted = node.encrypt_data(b"sensitive data");

    let mut tampered_bytes = hex::decode(&encrypted).unwrap();
    if tampered_bytes.len() > 20 {
        tampered_bytes[20] ^= 0xFF;
    }
    let tampered = hex::encode(tampered_bytes);

    let result = node.decrypt_data(&tampered);
    assert!(
        result.is_none(),
        "Tampered ciphertext must be rejected (HMAC failure)"
    );
}
