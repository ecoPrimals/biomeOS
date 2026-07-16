// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::{BASE64, MockCaller, sample_lineage};
use super::super::{DeviceLineage, LineageDeriver};
use base64::Engine;

#[test]
fn test_save_and_load_lineage_roundtrip() {
    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let seed_path = tmp.path().join("device.lineage");

    let mock = MockCaller::new();
    let deriver = LineageDeriver::new(mock);

    let lineage = DeviceLineage {
        derived_seed: BASE64.encode(b"32-bytes-of-derived-seed-data!!"),
        ..sample_lineage()
    };

    deriver
        .save_lineage(&lineage, &seed_path)
        .expect("save should succeed");

    assert!(seed_path.exists());
    assert!(seed_path.with_extension("json").exists());

    let loaded = LineageDeriver::<MockCaller>::load_lineage(&seed_path).expect("load");
    assert_eq!(loaded.device_id, "device-123");
    assert_eq!(loaded.node_id, "tower");
}

#[test]
fn test_has_lineage() {
    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let path = tmp.path().join("exists.lineage");
    assert!(!LineageDeriver::<MockCaller>::has_lineage(&path));
    std::fs::write(&path, b"data").expect("write");
    assert!(LineageDeriver::<MockCaller>::has_lineage(&path));
}

#[tokio::test]
async fn test_enroll_device_success() {
    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let family_seed_path = tmp.path().join(".family.seed");
    let lineage_seed_path = tmp.path().join("device.lineage");
    std::fs::write(&family_seed_path, b"family_seed_bytes_32!!").expect("write");

    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.derive_lineage_key",
        serde_json::json!({
            "key": "ZGV2aWNlLWRlcml2ZWQtc2VlZA==",
            "method": "Blake3"
        }),
    )
    .await;
    mock.set_err("crypto.sign", "n/a").await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .enroll_device(
            &family_seed_path,
            &lineage_seed_path,
            "fam-1",
            "dev-1",
            "tower",
        )
        .await;

    let enrollment = result.expect("enroll");
    assert_eq!(enrollment.lineage.device_id, "dev-1");
    assert_eq!(enrollment.lineage.node_id, "tower");
    assert!(lineage_seed_path.exists());
    assert!(lineage_seed_path.with_extension("json").exists());
}

#[tokio::test]
async fn test_enroll_device_family_seed_not_found() {
    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let mock = MockCaller::new();
    let deriver = LineageDeriver::new(mock);

    let result = deriver
        .enroll_device(
            &tmp.path().join("nonexistent.seed"),
            &tmp.path().join("out.lineage"),
            "fam",
            "dev",
            "node",
        )
        .await;

    assert!(result.is_err());
}

#[test]
fn test_save_lineage_invalid_base64_seed() {
    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let path = tmp.path().join("bad.lineage");
    let mock = MockCaller::new();
    let deriver = LineageDeriver::new(mock);

    let lineage = DeviceLineage {
        derived_seed: "!!!invalid!!!base64!!!".to_string(),
        ..sample_lineage()
    };

    let result = deriver.save_lineage(&lineage, &path);
    assert!(result.is_err());
}

#[test]
fn test_load_lineage_from_raw_seed_no_metadata() {
    use sha2::{Digest, Sha256};

    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let seed_path = tmp.path().join("raw.lineage");
    let seed_bytes = b"raw_seed_bytes_32_bytes!!";
    std::fs::write(&seed_path, seed_bytes).expect("write");

    let loaded = LineageDeriver::<MockCaller>::load_lineage(&seed_path).expect("load");

    let mut hasher = Sha256::new();
    hasher.update(seed_bytes);
    let expected_device_id = hex::encode(hasher.finalize());

    assert_eq!(loaded.device_id, expected_device_id);
    assert_eq!(loaded.node_id, format!("raw-{}", &expected_device_id[..12]));
    assert_eq!(loaded.family_id, hex::encode(&seed_bytes[..8]));
    assert_eq!(loaded.derivation_method, "raw_seed");
    assert_eq!(loaded.derived_seed, BASE64.encode(seed_bytes));
    assert!(loaded.derived_at > 0);

    // Deterministic: same bytes → same identity
    let loaded_again =
        LineageDeriver::<MockCaller>::load_lineage(&seed_path).expect("reload");
    assert_eq!(loaded_again.device_id, loaded.device_id);
    assert_eq!(loaded_again.family_id, loaded.family_id);
}

#[test]
fn test_load_lineage_nonexistent() {
    let result = LineageDeriver::<MockCaller>::load_lineage(std::path::Path::new(
        "/nonexistent/path.lineage",
    ));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_lineage_proof_success() {
    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.generate_lineage_proof",
        serde_json::json!({"proof": "proof-base64-string"}),
    )
    .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .generate_lineage_proof(&sample_lineage(), "peer-family")
        .await;

    assert_eq!(result.expect("proof"), "proof-base64-string");
}

#[tokio::test]
async fn test_generate_lineage_proof_fails() {
    let mock = MockCaller::new();
    mock.set_err("genetic.generate_lineage_proof", "timeout")
        .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .generate_lineage_proof(&sample_lineage(), "peer")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_lineage_proof_missing_proof_field() {
    let mock = MockCaller::new();
    mock.set_ok("genetic.generate_lineage_proof", serde_json::json!({}))
        .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .generate_lineage_proof(&sample_lineage(), "peer")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_lineage_proof_valid() {
    let mock = MockCaller::new();
    mock.set_ok("genetic.verify_lineage", serde_json::json!({"valid": true}))
        .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .verify_lineage_proof(&sample_lineage(), "peer", "proof-str")
        .await;

    assert!(result.expect("verify"));
}

#[tokio::test]
async fn test_verify_lineage_proof_invalid() {
    let mock = MockCaller::new();
    mock.set_ok(
        "genetic.verify_lineage",
        serde_json::json!({"valid": false}),
    )
    .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .verify_lineage_proof(&sample_lineage(), "peer", "bad-proof")
        .await;

    assert!(!result.expect("verify"));
}

#[tokio::test]
async fn test_verify_lineage_proof_no_valid_field() {
    let mock = MockCaller::new();
    mock.set_ok("genetic.verify_lineage", serde_json::json!({}))
        .await;

    let deriver = LineageDeriver::new(mock);
    let result = deriver
        .verify_lineage_proof(&sample_lineage(), "peer", "proof")
        .await;

    assert!(!result.expect("defaults to false"));
}
