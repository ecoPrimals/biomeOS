// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod sovereign_mesh_helpers;
use serde_json::Value;
use sovereign_mesh_helpers::*;

/// Phase 1: All 3 nodes generate beacons independently
#[tokio::test]
async fn e2e_phase1_beacon_generation() {
    let family_seed = FamilySeed::new(b"eastgate-family-sovereign-2026");

    let pixel = SimulatedNode::new(
        "pixel8a",
        family_seed.clone(),
        "[2600:1700:b0b0:5b90::80]:9901",
    );
    let usb = SimulatedNode::new("usb", family_seed.clone(), "192.0.2.50:9902");
    let tower = SimulatedNode::new("tower", family_seed, "tower.nestgate.io:3492");

    assert_ne!(pixel.beardog.beacon_id, usb.beardog.beacon_id);
    assert_ne!(pixel.beardog.beacon_id, tower.beardog.beacon_id);
    assert_ne!(usb.beardog.beacon_id, tower.beardog.beacon_id);

    let pixel_token = pixel.create_token();
    let usb_token = usb.create_token();
    let tower_token = tower.create_token();

    assert_ne!(pixel_token, usb_token);
    assert_ne!(pixel_token, tower_token);

    assert!(serde_json::from_str::<Value>(&pixel_token).is_err());
    assert!(serde_json::from_str::<Value>(&usb_token).is_err());
}
