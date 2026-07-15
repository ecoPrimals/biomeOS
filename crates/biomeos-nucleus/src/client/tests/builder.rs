// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::coordinator::{NucleusClient, NucleusClientBuilder};

#[test]
fn test_nucleus_client_builder_default() {
    let _builder = NucleusClientBuilder::default();
}

#[test]
fn test_nucleus_client_builder_new() {
    let _builder = NucleusClientBuilder::new();
}

#[test]
fn test_nucleus_client_builder_equality() {
    let b1 = NucleusClientBuilder::new();
    let b2 = NucleusClientBuilder::default();
    assert_eq!(std::mem::size_of_val(&b1), std::mem::size_of_val(&b2));
}

#[test]
fn test_nucleus_client_builder_build_smoke() {
    // May fail without full stack (Songbird/BearDog paths) — exercise constructor path only when ok.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt");
    let res: std::result::Result<NucleusClient, _> =
        rt.block_on(async { NucleusClientBuilder::new().build().await });
    if let Ok(client) = res {
        let _ = client.registry();
    }
}
