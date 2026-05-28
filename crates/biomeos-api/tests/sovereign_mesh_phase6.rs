// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

mod sovereign_mesh_helpers;
use serde_json::Value;
use sovereign_mesh_helpers::*;

/// Phase 6: Full flow — all 3 nodes, rendezvous, decryption, data transfer
#[tokio::test]
async fn e2e_phase6_full_sovereign_mesh_flow() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");

    let pixel = SimulatedNode::new(
        "pixel8a",
        family_seed.clone(),
        "[2600:1700:b0b0:5b90::80]:9901",
    );
    let usb = SimulatedNode::new("usb", family_seed.clone(), "192.0.2.50:9902");
    let tower = SimulatedNode::new("tower", family_seed.clone(), "tower.nestgate.io:3492");

    let rendezvous = MockRendezvous::new(family_seed);

    let pixel_token = pixel.create_token();
    let pixel_beacon = pixel.encrypt_beacon();
    let (accepted, _, _) = rendezvous
        .post_beacon(&pixel_token, &pixel_beacon, "pixel8a")
        .unwrap();
    assert!(accepted);

    let usb_token = usb.create_token();
    let usb_beacon = usb.encrypt_beacon();
    let (accepted, peer_beacon, _) = rendezvous
        .post_beacon(&usb_token, &usb_beacon, "usb")
        .unwrap();
    assert!(accepted);
    let pixel_beacon_received = peer_beacon.expect("USB should get Pixel's beacon");

    let usb_beacon_received = rendezvous
        .check_peer(&pixel_token, "pixel8a")
        .unwrap()
        .expect("Pixel should get USB's beacon");

    let usb_info = pixel
        .decrypt_peer_beacon(&usb_beacon_received)
        .expect("Pixel decrypts USB's beacon");
    let pixel_info = usb
        .decrypt_peer_beacon(&pixel_beacon_received)
        .expect("USB decrypts Pixel's beacon");

    assert_eq!(usb_info["node_id"], "usb");
    assert_eq!(pixel_info["node_id"], "pixel8a");

    let photo_data = b"[encrypted sovereign photo data - 2MB simulated]";
    let encrypted_payload = pixel.encrypt_data(photo_data);

    assert!(!encrypted_payload.contains("photo"));
    assert!(!encrypted_payload.contains("sovereign"));

    let received = tower
        .decrypt_data(&encrypted_payload)
        .expect("Tower decrypts Pixel's data");
    assert_eq!(received, photo_data, "Data survives full mesh flow");

    let response_data = b"ACK: photos received and stored securely";
    let encrypted_response = tower.encrypt_data(response_data);
    let received_response = pixel
        .decrypt_data(&encrypted_response)
        .expect("Pixel decrypts Tower's response");
    assert_eq!(received_response, response_data);

    let usb_decrypted = usb.decrypt_data(&encrypted_payload);
    assert!(
        usb_decrypted.is_some(),
        "USB can also decrypt (same family seed)"
    );

    assert!(serde_json::from_str::<Value>(&pixel_token).is_err());
    assert!(serde_json::from_str::<Value>(&usb_token).is_err());

    assert!(serde_json::from_str::<Value>(&pixel_beacon).is_err());
    assert!(serde_json::from_str::<Value>(&usb_beacon).is_err());

    assert!(!encrypted_payload.contains("photo"));
    assert!(!encrypted_response.contains("ACK"));

    let attacker_seed = FamilySeed::new(b"not-your-family");
    let attacker = SimulatedNode::new("attacker", attacker_seed, "evil.com:666");
    assert!(attacker.decrypt_peer_beacon(&pixel_beacon).is_none());
    assert!(attacker.decrypt_data(&encrypted_payload).is_none());
    assert!(
        rendezvous
            .post_beacon(
                &attacker.create_token(),
                &attacker.encrypt_beacon(),
                "attacker"
            )
            .is_err()
    );
}
