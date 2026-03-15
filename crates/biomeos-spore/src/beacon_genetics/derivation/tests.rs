// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Derivation module tests

#[cfg(test)]
mod derivation_tests {
    use super::super::{generate_device_entropy, DerivationParams, DeviceLineage, LineageDeriver};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

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
}
