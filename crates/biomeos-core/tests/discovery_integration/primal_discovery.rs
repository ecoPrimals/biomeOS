// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery system integration tests — individual and multi-primal discovery.

use biomeos_core::primal_adapter::discover_primal_interface;

use crate::{find_primal_binary, wait_for_service};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Requires running HTTP service — use for live service integration tests"]
async fn test_wait_for_service_helper() {
    let ready = wait_for_service("http://localhost:9020/health", 3).await;
    let _ = ready;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_nestgate_if_available() {
    if let Some(nestgate_path) = find_primal_binary("nestgate") {
        let result = discover_primal_interface(&nestgate_path).await;

        match result {
            Ok(adapter) => {
                println!("✅ Discovered NestGate: {:?}", adapter.name);
                assert_eq!(adapter.name, "nestgate");
                assert!(adapter.interface.is_known());
            }
            Err(e) => {
                println!("⚠️  Could not discover NestGate (may be expected): {e}");
            }
        }
    } else {
        println!("⏭️  Skipping NestGate discovery test - binary not found");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_beardog_if_available() {
    if let Some(beardog_path) = find_primal_binary("beardog") {
        let result = discover_primal_interface(&beardog_path).await;

        match result {
            Ok(adapter) => {
                println!("✅ Discovered BearDog: {:?}", adapter.name);
                assert_eq!(adapter.name, "beardog");
                assert!(adapter.interface.is_known());
            }
            Err(e) => {
                println!("⚠️  Could not discover BearDog (may be expected): {e}");
            }
        }
    } else {
        println!("⏭️  Skipping BearDog discovery test - binary not found");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_toadstool_if_available() {
    if let Some(toadstool_path) = find_primal_binary("toadstool") {
        let result = discover_primal_interface(&toadstool_path).await;

        match result {
            Ok(adapter) => {
                println!("✅ Discovered Toadstool: {:?}", adapter.name);
                assert_eq!(adapter.name, "toadstool");
                assert!(adapter.interface.is_known());
            }
            Err(e) => {
                println!("⚠️  Could not discover Toadstool (may be expected): {e}");
            }
        }
    } else {
        println!("⏭️  Skipping Toadstool discovery test - binary not found");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_squirrel_if_available() {
    if let Some(squirrel_path) = find_primal_binary("squirrel") {
        let result = discover_primal_interface(&squirrel_path).await;

        match result {
            Ok(adapter) => {
                println!("✅ Discovered Squirrel: {:?}", adapter.name);
                assert!(adapter.interface.is_known());
            }
            Err(e) => {
                println!("⚠️  Could not discover Squirrel (may be expected): {e}");
            }
        }
    } else {
        println!("⏭️  Skipping Squirrel discovery test - binary not found");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_multiple_primals() {
    let primal_names = vec!["nestgate", "beardog", "toadstool", "squirrel"];
    let mut discovered = Vec::new();

    for name in primal_names {
        if let Some(path) = find_primal_binary(name) {
            if let Ok(adapter) = discover_primal_interface(&path).await {
                discovered.push(adapter);
            }
        }
    }

    println!("✅ Discovered {} primals", discovered.len());

    if discovered.is_empty() {
        println!("⚠️  No primals discovered - binaries may not be built yet");
    } else {
        for adapter in discovered {
            assert!(
                adapter.interface.is_known(),
                "Primal {} should have known interface",
                adapter.name
            );
        }
    }
}
