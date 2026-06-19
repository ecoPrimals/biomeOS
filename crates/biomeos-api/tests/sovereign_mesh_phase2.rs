// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

mod sovereign_mesh_helpers;
use sovereign_mesh_helpers::*;

/// Phase 2: Pixel and USB post beacons to Tower rendezvous, get matched
#[tokio::test]
async fn e2e_phase2_rendezvous_matching() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");

    let pixel = SimulatedNode::new(
        "pixel8a",
        family_seed.clone(),
        "[2600:1700:b0b0:5b90::80]:9901",
    );
    let usb = SimulatedNode::new("usb", family_seed.clone(), "192.0.2.50:9902");

    let rendezvous = MockRendezvous::new(family_seed);

    let pixel_token = pixel.create_token();
    let pixel_encrypted_beacon = pixel.encrypt_beacon();
    let (accepted, peer_beacon, peers_waiting) = rendezvous
        .post_beacon(&pixel_token, &pixel_encrypted_beacon, "pixel8a")
        .unwrap();

    assert!(accepted, "Pixel beacon should be accepted");
    assert!(peer_beacon.is_none(), "No peer should be waiting yet");
    assert_eq!(peers_waiting, 0, "No peers waiting before Pixel posted");

    let usb_token = usb.create_token();
    let usb_encrypted_beacon = usb.encrypt_beacon();
    let (accepted, peer_beacon, peers_waiting) = rendezvous
        .post_beacon(&usb_token, &usb_encrypted_beacon, "usb")
        .unwrap();

    assert!(accepted, "USB beacon should be accepted");
    assert!(
        peer_beacon.is_some(),
        "USB should get Pixel's beacon (peer match)"
    );
    assert_eq!(peers_waiting, 1, "Pixel was waiting");

    let received_pixel_beacon = peer_beacon.unwrap();
    assert_eq!(
        received_pixel_beacon, pixel_encrypted_beacon,
        "USB should receive Pixel's exact encrypted beacon"
    );

    let pixel_check = rendezvous.check_peer(&pixel_token, "pixel8a").unwrap();
    assert!(
        pixel_check.is_some(),
        "Pixel should find USB's beacon on check"
    );
    let received_usb_beacon = pixel_check.unwrap();
    assert_eq!(
        received_usb_beacon, usb_encrypted_beacon,
        "Pixel should receive USB's exact encrypted beacon"
    );
}
