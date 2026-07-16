// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::MockCaller;
use super::super::LineageDeriver;

#[tokio::test]
async fn test_derive_device_seed_success() {
    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.derive_lineage_key",
        serde_json::json!({
            "key": "ZGVyaXZlZC1zZWVkLWRhdGE=",
            "method": "Blake3-KDF"
        }),
    )
    .await;
    mock.set_err("crypto.sign", "not available").await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .derive_device_seed("family-seed-b64", "family-01", "dev-001", "tower", None)
        .await;

    let lineage = result.expect("derivation should succeed");
    assert_eq!(lineage.device_id, "dev-001");
    assert_eq!(lineage.node_id, "tower");
    assert_eq!(lineage.derived_seed, "ZGVyaXZlZC1zZWVkLWRhdGE=");
}

#[tokio::test]
async fn test_derive_device_seed_derive_fails() {
    let mock = MockCaller::new();
    mock.set_err("genetic.derive_lineage_key", "connection refused")
        .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .derive_device_seed("seed", "fam", "dev", "node", None)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("derive"));
}

#[tokio::test]
async fn test_derive_device_seed_missing_key_in_response() {
    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.derive_lineage_key",
        serde_json::json!({"method": "kdf"}),
    )
    .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .derive_device_seed("seed", "fam", "dev", "node", None)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("key"));
}

#[tokio::test]
async fn test_derive_device_seed_with_entropy_mix_success() {
    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.derive_lineage_key",
        serde_json::json!({"key": "base64derived", "method": "Blake3"}),
    )
    .await;
    mock.set_ok(
        "genetic.mix_entropy",
        serde_json::json!({"mixed_seed": "mixed_result_seed"}),
    )
    .await;
    mock.set_err("crypto.sign", "unavailable").await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .derive_device_seed("seed", "fam", "dev", "node", Some(b"entropy_bytes"))
        .await;

    let lineage = result.expect("should succeed");
    assert_eq!(lineage.derived_seed, "mixed_result_seed");
}

#[tokio::test]
async fn test_derive_device_seed_entropy_mix_fallback_to_derived() {
    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.derive_lineage_key",
        serde_json::json!({"key": "derived_key", "method": "KDF"}),
    )
    .await;
    mock.set_err("genetic.mix_entropy", "mix failed").await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .derive_device_seed("seed", "fam", "dev", "node", Some(b"entropy"))
        .await;

    let lineage = result.expect("graceful fallback");
    assert_eq!(lineage.derived_seed, "derived_key");
}

#[tokio::test]
async fn test_derive_device_seed_with_certificate() {
    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.derive_lineage_key",
        serde_json::json!({"key": "ZGVyaXZlZA==", "method": "Blake3"}),
    )
    .await;
    mock.set_ok("crypto.sign", serde_json::json!({"signature": "sig123"}))
        .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .derive_device_seed("seed", "fam", "dev", "node", None)
        .await;

    let lineage = result.expect("success");
    assert_eq!(lineage.lineage_certificate, Some("sig123".to_string()));
}
