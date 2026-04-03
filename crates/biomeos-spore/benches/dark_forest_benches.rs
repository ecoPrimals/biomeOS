// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! TRUE Dark Forest Performance Benchmarks
//!
//! Quantifies performance improvements of pure noise beacons vs old format.
//!
//! # Benchmarks
//!
//! 1. **Beacon Generation**
//!    - Old format: JSON + base64 + ChaCha20-Poly1305
//!    - New format: Direct bytes + ChaCha20-Poly1305
//!    - Expected: 20-30% faster
//!
//! 2. **Beacon Decryption (Success)**
//!    - Old format: base64 decode + JSON parse + ChaCha20
//!    - New format: Direct ChaCha20 (no parsing)
//!    - Expected: 15-25% faster
//!
//! 3. **Silent Failure**
//!    - Old format: ChaCha20 fail + error handling + JSON cleanup
//!    - New format: Immediate ChaCha20 fail (no overhead)
//!    - Expected: 40-50% faster
//!
//! 4. **Size Comparison**
//!    - Old format: JSON structure + base64 encoding overhead
//!    - New format: Pure bytes (minimal overhead)
//!    - Expected: 30-40% smaller
//!
//! # Usage
//!
//! ```bash
//! # Requires beardog running
//! cargo bench --bench dark_forest_benches
//! ```

#![allow(clippy::unwrap_used)]

use biomeos_spore::DarkForestBeacon;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

/// Create test seed for benchmarking
async fn setup_test_seed() -> String {
    let seed_path = "/tmp/bench_dark_forest.seed";
    let seed = b"benchmark_seed_32bytes_long!!!!!";
    tokio::fs::write(seed_path, seed).await.unwrap();
    seed_path.to_string()
}

/// Cleanup test seed
async fn cleanup_test_seed(path: &str) {
    tokio::fs::remove_file(path).await.ok();
}

/// Resolve security-provider socket path (env override or default).
fn security_socket_path() -> String {
    std::env::var("BEARDOG_SOCKET")
        .unwrap_or_else(|_| "/run/user/1000/biomeos/beardog.sock".to_string())
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 1: Pure Noise Beacon Generation
// ═══════════════════════════════════════════════════════════════════

fn bench_pure_noise_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup
    let seed_path = rt.block_on(setup_test_seed());
    let security_socket = security_socket_path();

    // Check if security provider is available
    if !std::path::Path::new(&security_socket).exists() {
        eprintln!("⚠️  Skipping benchmarks: BearDog not running at {security_socket}");
        return;
    }

    let mgr = rt.block_on(async {
        DarkForestBeacon::from_security_socket(&security_socket, &seed_path, "bench_node")
            .await
            .unwrap()
    });

    c.bench_function("pure_noise_generation", |b| {
        b.to_async(&rt).iter(|| async {
            let beacon = mgr
                .generate_pure_noise_beacon(
                    black_box("/tmp/bench.sock"),
                    black_box(&["test"]),
                    black_box(None),
                )
                .await
                .unwrap();
            black_box(beacon);
        });
    });

    // Cleanup
    rt.block_on(cleanup_test_seed(&seed_path));
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 2: Old Format Beacon Generation (for comparison)
// ═══════════════════════════════════════════════════════════════════

fn bench_old_format_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let seed_path = rt.block_on(setup_test_seed());
    let security_socket = security_socket_path();

    if !std::path::Path::new(&security_socket).exists() {
        return;
    }

    let mgr = rt.block_on(async {
        DarkForestBeacon::from_security_socket(&security_socket, &seed_path, "bench_node")
            .await
            .unwrap()
    });

    c.bench_function("old_format_generation", |b| {
        b.to_async(&rt).iter(|| async {
            let beacon = mgr
                .generate_encrypted_beacon(
                    black_box("/tmp/bench.sock"),
                    black_box(&["test"]),
                    black_box(None),
                )
                .await
                .unwrap();
            black_box(beacon);
        });
    });

    rt.block_on(cleanup_test_seed(&seed_path));
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 3: Pure Noise Successful Decryption
// ═══════════════════════════════════════════════════════════════════

fn bench_pure_noise_decrypt_success(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let seed_path = rt.block_on(setup_test_seed());
    let security_socket = security_socket_path();

    if !std::path::Path::new(&security_socket).exists() {
        return;
    }

    let mgr = rt.block_on(async {
        DarkForestBeacon::from_security_socket(&security_socket, &seed_path, "bench_node")
            .await
            .unwrap()
    });

    // Pre-generate beacon
    let beacon = rt.block_on(async {
        mgr.generate_pure_noise_beacon("/tmp/bench.sock", &["test"], None)
            .await
            .unwrap()
    });

    c.bench_function("pure_noise_decrypt_success", |b| {
        b.to_async(&rt).iter(|| async {
            let result = mgr
                .try_decrypt_pure_noise_beacon(black_box(&beacon))
                .await
                .unwrap();
            black_box(result);
        });
    });

    rt.block_on(cleanup_test_seed(&seed_path));
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 4: Pure Noise Silent Failure (Random Noise)
// ═══════════════════════════════════════════════════════════════════

fn bench_pure_noise_silent_failure(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let seed_path = rt.block_on(setup_test_seed());
    let security_socket = security_socket_path();

    if !std::path::Path::new(&security_socket).exists() {
        return;
    }

    let mgr = rt.block_on(async {
        DarkForestBeacon::from_security_socket(&security_socket, &seed_path, "bench_node")
            .await
            .unwrap()
    });

    // Generate random noise (simulates different family)
    use rand::RngCore;
    let mut random_noise = vec![0u8; 128];
    rand::rng().fill_bytes(&mut random_noise);

    c.bench_function("pure_noise_silent_failure", |b| {
        b.to_async(&rt).iter(|| async {
            let result = mgr
                .try_decrypt_pure_noise_beacon(black_box(&random_noise))
                .await
                .unwrap();
            black_box(result);
        });
    });

    rt.block_on(cleanup_test_seed(&seed_path));
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark 5: Size Comparison
// ═══════════════════════════════════════════════════════════════════

fn bench_size_comparison(_c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let seed_path = rt.block_on(setup_test_seed());
    let security_socket = security_socket_path();

    if !std::path::Path::new(&security_socket).exists() {
        return;
    }

    let mgr = rt.block_on(async {
        DarkForestBeacon::from_security_socket(&security_socket, &seed_path, "bench_node")
            .await
            .unwrap()
    });

    // Generate beacons
    let (old_beacon, pure_noise_beacon) = rt.block_on(async {
        let old = mgr
            .generate_encrypted_beacon("/tmp/bench.sock", &["test"], None)
            .await
            .unwrap();
        let pure = mgr
            .generate_pure_noise_beacon("/tmp/bench.sock", &["test"], None)
            .await
            .unwrap();

        // Serialize old format to bytes for fair comparison
        let old_bytes = serde_json::to_vec(&old).unwrap();

        (old_bytes, pure)
    });

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Size Comparison");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Old format (JSON):     {} bytes", old_beacon.len());
    println!("Pure noise (bytes):    {} bytes", pure_noise_beacon.len());
    println!(
        "Reduction:             {} bytes ({:.1}%)",
        old_beacon.len() - pure_noise_beacon.len(),
        (1.0 - pure_noise_beacon.len() as f64 / old_beacon.len() as f64) * 100.0
    );
    println!("═══════════════════════════════════════════════════════════════════\n");

    rt.block_on(cleanup_test_seed(&seed_path));
}

// ═══════════════════════════════════════════════════════════════════
// Benchmark Group Configuration
// ═══════════════════════════════════════════════════════════════════

criterion_group!(
    benches,
    bench_pure_noise_generation,
    bench_old_format_generation,
    bench_pure_noise_decrypt_success,
    bench_pure_noise_silent_failure,
    bench_size_comparison
);

criterion_main!(benches);
