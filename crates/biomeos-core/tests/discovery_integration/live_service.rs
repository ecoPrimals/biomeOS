// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Live service discovery + architecture adaptation tests.

use biomeos_core::primal_adapter::discover_primal_interface;
use std::process::{Command, Stdio};

use crate::{find_primal_binary, http_get};

// ============================================================================
// Live Service Discovery
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_running_nestgate() {
    let nestgate_url = "http://localhost:9020/health";

    match http_get(nestgate_url, 2) {
        Ok((status, body)) => {
            if (200..300).contains(&status) {
                println!("✅ NestGate is running and responsive");
                assert!(!body.is_empty(), "Health response should not be empty");
                println!("   Health response: {body}");
            } else {
                println!("⚠️  NestGate returned non-success status: {status}");
            }
        }
        Err(e) => {
            println!("⏭️  NestGate not running (expected if not started): {e}");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_capability_based_discovery() {
    let storage_primals = vec!["nestgate"];
    let encryption_primals = vec!["beardog"];
    let compute_primals = vec!["toadstool"];

    let mut found_storage = false;
    let mut found_encryption = false;
    let mut found_compute = false;

    for name in storage_primals {
        if find_primal_binary(name).is_some() {
            found_storage = true;
            println!("✅ Storage capability available: {name}");
        }
    }

    for name in encryption_primals {
        if find_primal_binary(name).is_some() {
            found_encryption = true;
            println!("✅ Encryption capability available: {name}");
        }
    }

    for name in compute_primals {
        if find_primal_binary(name).is_some() {
            found_compute = true;
            println!("✅ Compute capability available: {name}");
        }
    }

    println!("\n📊 Capability Discovery Summary:");
    println!("   Storage:    {}", if found_storage { "✅" } else { "❌" });
    println!(
        "   Encryption: {}",
        if found_encryption { "✅" } else { "❌" }
    );
    println!("   Compute:    {}", if found_compute { "✅" } else { "❌" });
}

// ============================================================================
// Architecture Adaptation
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_rest_api_architecture() {
    let nestgate_url = "http://localhost:9020/health";

    match http_get(nestgate_url, 2) {
        Ok((status, _)) if (200..300).contains(&status) => {
            println!("✅ REST API primal discovered (NestGate)");
        }
        _ => {
            println!("⏭️  REST API primal not running");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_cli_tool_architecture() {
    let cli_primals = vec!["beardog", "toadstool"];
    let mut found_cli = false;

    for name in cli_primals {
        if let Some(path) = find_primal_binary(name) {
            if let Ok(adapter) = discover_primal_interface(&path).await {
                println!("✅ CLI primal discovered: {name}");
                found_cli = true;

                use biomeos_core::primal_adapter::PrimalInterface;
                match adapter.interface {
                    PrimalInterface::Direct { .. } | PrimalInterface::Subcommand { .. } => {}
                    _ => {
                        println!("   Interface: {:?}", adapter.interface);
                    }
                }
            }
        }
    }

    if !found_cli {
        println!("⏭️  No CLI primals found");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_mdns_architecture() {
    let output = Command::new("pgrep")
        .arg("-f")
        .arg("songbird")
        .stdout(Stdio::null())
        .status();

    match output {
        Ok(status) if status.success() => {
            println!("✅ mDNS primal discovered (Songbird running)");
            println!("   Songbird federation active (mDNS/UDP)");
        }
        _ => {
            println!("⏭️  mDNS primal not running (Songbird)");
        }
    }
}
