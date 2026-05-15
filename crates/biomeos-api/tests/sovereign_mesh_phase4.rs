// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod sovereign_mesh_helpers;
use sovereign_mesh_helpers::*;

/// Phase 4: Encrypted data transfer through simulated BTSP channel
#[tokio::test]
async fn e2e_phase4_encrypted_data_transfer() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");

    let pixel = SimulatedNode::new(
        "pixel8a",
        family_seed.clone(),
        "[2600:1700:b0b0:5b90::80]:9901",
    );
    let tower = SimulatedNode::new("tower", family_seed, "tower.nestgate.io:3492");

    let original_data = b"Photos from hike - 2026-02-07 - encrypted sovereign transfer";

    let encrypted = pixel.encrypt_data(original_data);

    assert_ne!(
        encrypted.as_bytes(),
        original_data,
        "Encrypted data must differ from original"
    );
    assert!(
        !encrypted.contains("Photos"),
        "Encrypted data must not contain plaintext"
    );

    let decrypted = tower.decrypt_data(&encrypted);
    assert!(
        decrypted.is_some(),
        "Tower should be able to decrypt Pixel's data (same family)"
    );
    assert_eq!(
        decrypted.unwrap(),
        original_data,
        "Decrypted data must match original exactly"
    );

    let large_data: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();
    let encrypted_large = pixel.encrypt_data(&large_data);
    let decrypted_large = tower.decrypt_data(&encrypted_large);
    assert!(decrypted_large.is_some(), "Large data transfer should work");
    assert_eq!(
        decrypted_large.unwrap(),
        large_data,
        "Large data must survive encrypt/decrypt round-trip"
    );
}
