// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

mod sovereign_mesh_helpers;
use sovereign_mesh_helpers::*;

/// Phase 5: Non-family attacker is rejected at every step
#[tokio::test]
async fn e2e_phase5_attacker_rejection() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");
    let attacker_seed = FamilySeed::new(b"evil-hacker-different-family-2026");

    let pixel = SimulatedNode::new(
        "pixel8a",
        family_seed.clone(),
        "[2600:1700:b0b0:5b90::80]:9901",
    );
    let attacker = SimulatedNode::new("evil-node", attacker_seed.clone(), "198.51.100.1:6666");

    let rendezvous = MockRendezvous::new(family_seed.clone());

    let attacker_token = attacker.create_token();
    let attacker_beacon = attacker.encrypt_beacon();
    let result = rendezvous.post_beacon(&attacker_token, &attacker_beacon, "evil-node");
    assert!(
        result.is_err(),
        "Attacker's token should be rejected by rendezvous"
    );

    let pixel_encrypted = pixel.encrypt_beacon();
    let stolen_decrypt = attacker.decrypt_peer_beacon(&pixel_encrypted);
    assert!(
        stolen_decrypt.is_none(),
        "Attacker should NOT be able to decrypt Pixel's beacon"
    );

    let pixel_token = pixel.create_token();
    let stolen_token_decrypt = attacker.beardog.try_decrypt_token(&pixel_token);
    assert!(
        stolen_token_decrypt.is_none(),
        "Attacker should NOT be able to decrypt Pixel's Dark Forest token"
    );

    let encrypted_data = pixel.encrypt_data(b"secret family photos");
    let stolen_data = attacker.decrypt_data(&encrypted_data);
    assert!(
        stolen_data.is_none(),
        "Attacker should NOT be able to decrypt transferred data"
    );

    assert_ne!(
        family_seed.lineage_hash(),
        attacker_seed.lineage_hash(),
        "Different families must have different lineage hashes"
    );
}
