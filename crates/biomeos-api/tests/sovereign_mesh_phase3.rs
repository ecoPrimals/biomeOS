// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod sovereign_mesh_helpers;
use serde_json::Value;
use sovereign_mesh_helpers::*;

/// Phase 3: Both nodes decrypt each other's beacons and recover endpoint info
#[tokio::test]
async fn e2e_phase3_mutual_decryption() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");

    let pixel = SimulatedNode::new(
        "pixel8a",
        family_seed.clone(),
        "[2600:1700:b0b0:5b90::80]:9901",
    );
    let usb = SimulatedNode::new("usb", family_seed, "192.0.2.50:9902");

    let pixel_encrypted = pixel.encrypt_beacon();
    let usb_encrypted = usb.encrypt_beacon();

    let usb_info = pixel.decrypt_peer_beacon(&usb_encrypted);
    assert!(
        usb_info.is_some(),
        "Pixel should be able to decrypt USB's beacon (same family)"
    );
    let usb_info = usb_info.unwrap();
    assert_eq!(
        usb_info["node_id"].as_str().unwrap(),
        "usb",
        "Decrypted beacon should reveal USB's identity"
    );
    assert!(
        usb_info["endpoints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str().unwrap().contains("192.0.2.50")),
        "Decrypted beacon should contain USB's endpoint"
    );

    let pixel_info = usb.decrypt_peer_beacon(&pixel_encrypted);
    assert!(
        pixel_info.is_some(),
        "USB should be able to decrypt Pixel's beacon (same family)"
    );
    let pixel_info = pixel_info.unwrap();
    assert_eq!(
        pixel_info["node_id"].as_str().unwrap(),
        "pixel8a",
        "Decrypted beacon should reveal Pixel's identity"
    );
    assert!(
        pixel_info["endpoints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str().unwrap().contains("2600:1700")),
        "Decrypted beacon should contain Pixel's IPv6 endpoint"
    );

    assert!(
        serde_json::from_str::<Value>(&pixel_encrypted).is_err(),
        "Encrypted beacon must not be parseable as JSON"
    );
    assert!(
        serde_json::from_str::<Value>(&usb_encrypted).is_err(),
        "Encrypted beacon must not be parseable as JSON"
    );
}
