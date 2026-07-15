// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::beacon_genetics::CapabilityCaller;
use super::super::DarkForestBeacon;

/// Mock capability caller for Dark Forest beacon tests.
/// Returns preset responses keyed by BearDog method name.
pub(super) struct MockDarkForestCaller {
    responses: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl MockDarkForestCaller {
    pub(super) fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(super) async fn set_response(&self, method: &str, result: serde_json::Value) {
        self.responses
            .lock()
            .await
            .insert(method.to_string(), result);
    }

    /// Configure mock for successful beacon generation flow
    pub(super) async fn setup_generate_success(&self, broadcast_key: &str, hash: &str) {
        self.set_response(
            "genetic.derive_lineage_key",
            serde_json::json!({ "key": broadcast_key }),
        )
        .await;
        self.set_response("crypto.blake3_hash", serde_json::json!({ "hash": hash }))
            .await;
        self.set_response(
            "crypto.chacha20_poly1305_encrypt",
            serde_json::json!({
                "ciphertext": "encrypted_payload_b64",
                "nonce": "nonce12bytes==",
                "tag": "auth_tag_16bytes=="
            }),
        )
        .await;
    }

    /// Configure mock for successful beacon decryption
    pub(super) async fn setup_decrypt_success(&self, plaintext_b64: &str) {
        self.set_response(
            "genetic.derive_lineage_key",
            serde_json::json!({ "key": "same_broadcast_key" }),
        )
        .await;
        self.set_response(
            "crypto.chacha20_poly1305_decrypt",
            serde_json::json!({ "plaintext": plaintext_b64 }),
        )
        .await;
    }

    /// Configure mock for lineage verification
    pub(super) async fn setup_verify_lineage(&self, valid: bool) {
        self.set_response(
            "genetic.verify_lineage",
            serde_json::json!({ "valid": valid }),
        )
        .await;
    }

    /// Configure mock for lineage proof generation
    pub(super) async fn setup_generate_lineage_proof(&self, proof: &str) {
        self.set_response(
            "genetic.generate_lineage_proof",
            serde_json::json!({ "proof": proof }),
        )
        .await;
    }

    /// Configure mock for pure noise beacon (derive + encrypt/decrypt)
    pub(super) async fn setup_pure_noise_success(
        &self,
        beacon_key: &str,
        encrypt_result: (String, String, String),
        decrypt_plaintext: Option<&str>,
    ) {
        self.set_response(
            "genetic.derive_lineage_beacon_key",
            serde_json::json!({ "beacon_key": beacon_key }),
        )
        .await;
        self.set_response(
            "crypto.chacha20_poly1305_encrypt",
            serde_json::json!({
                "ciphertext": encrypt_result.0,
                "nonce": encrypt_result.1,
                "tag": encrypt_result.2
            }),
        )
        .await;
        if let Some(pt) = decrypt_plaintext {
            self.set_response(
                "crypto.chacha20_poly1305_decrypt",
                serde_json::json!({ "plaintext": pt }),
            )
            .await;
        }
    }
}

impl CapabilityCaller for MockDarkForestCaller {
    async fn call(
        &self,
        capability: &str,
        _params: serde_json::Value,
    ) -> crate::error::SporeResult<serde_json::Value> {
        let responses = self.responses.lock().await;
        responses.get(capability).cloned().ok_or_else(|| {
            crate::error::SporeError::CapabilityCall(format!("no mock response for {capability}"))
        })
    }
}

pub(super) fn make_beacon(
    caller: MockDarkForestCaller,
    family_seed_b64: &str,
    node_id: &str,
) -> DarkForestBeacon<MockDarkForestCaller> {
    DarkForestBeacon {
        capability_caller: Arc::new(caller),
        family_seed_b64: family_seed_b64.to_string(),
        node_id: node_id.to_string(),
    }
}
