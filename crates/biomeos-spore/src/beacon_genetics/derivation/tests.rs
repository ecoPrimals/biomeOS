// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Derivation module tests

#[cfg(test)]
mod derivation_tests {
    use super::super::{DerivationParams, DeviceLineage, LineageDeriver, generate_device_entropy};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

    use crate::beacon_genetics::capability::CapabilityCaller;

    struct MockCaller {
        responses: Arc<Mutex<HashMap<String, Result<serde_json::Value, String>>>>,
    }

    impl MockCaller {
        fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        async fn set_ok(&self, cap: &str, val: serde_json::Value) {
            self.responses.lock().await.insert(cap.to_string(), Ok(val));
        }

        async fn set_err(&self, cap: &str, msg: &str) {
            self.responses
                .lock()
                .await
                .insert(cap.to_string(), Err(msg.to_string()));
        }
    }

    #[async_trait::async_trait]
    impl CapabilityCaller for MockCaller {
        async fn call(
            &self,
            capability: &str,
            _params: serde_json::Value,
        ) -> Result<serde_json::Value, String> {
            let responses = self.responses.lock().await;
            responses
                .get(capability)
                .cloned()
                .unwrap_or_else(|| Err(format!("No mock for {capability}")))
        }
    }

    fn sample_lineage() -> DeviceLineage {
        DeviceLineage {
            device_id: "device-123".to_string(),
            node_id: "tower".to_string(),
            family_id: "1894e909e454".to_string(),
            generation: 1,
            derived_seed: "dGVzdHNlZWQ=".to_string(),
            derived_at: 1738726800,
            derivation_method: "Blake3-Lineage-KDF".to_string(),
            lineage_certificate: None,
        }
    }

    #[test]
    fn test_device_lineage_serialization() {
        let lineage = sample_lineage();
        let json = serde_json::to_string(&lineage).expect("serialize");
        assert!(json.contains("device-123"));
        assert!(json.contains("tower"));
        assert!(json.contains("1894e909e454"));
        let parsed: DeviceLineage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.device_id, "device-123");
        assert_eq!(parsed.node_id, "tower");
    }

    #[test]
    fn test_derivation_params_serialization() {
        let params = DerivationParams {
            family_seed: "c2VlZA==".to_string(),
            device_id: "dev-001".to_string(),
            node_id: "tower".to_string(),
            device_entropy: Some("ZW50cm9weQ==".to_string()),
            purpose: "device-lineage".to_string(),
        };
        let json = serde_json::to_string(&params).expect("serialize");
        assert!(json.contains("family_seed"));
        assert!(json.contains("dev-001"));
    }

    #[test]
    fn test_generate_device_entropy() {
        let entropy1 = generate_device_entropy().expect("device entropy generation");
        let entropy2 = generate_device_entropy().expect("device entropy generation");
        assert_eq!(entropy1.len(), 32);
        assert_eq!(entropy2.len(), 32);
        assert_ne!(entropy1, entropy2);
    }

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
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        let seed_path = tmp.path().join("raw.lineage");
        let seed_bytes = b"raw_seed_bytes_32_bytes!!";
        std::fs::write(&seed_path, seed_bytes).expect("write");

        let loaded = LineageDeriver::<MockCaller>::load_lineage(&seed_path).expect("load");
        assert_eq!(loaded.device_id, "unknown");
        assert_eq!(loaded.node_id, "unknown");
        assert_eq!(loaded.derived_seed, BASE64.encode(seed_bytes));
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

        assert_eq!(result.expect("verify"), true);
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

        assert_eq!(result.expect("verify"), false);
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

        assert_eq!(result.expect("defaults to false"), false);
    }
}
