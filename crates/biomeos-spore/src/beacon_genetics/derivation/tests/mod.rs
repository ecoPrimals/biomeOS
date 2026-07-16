// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Derivation module tests

mod derive;
mod persistence;
mod serialization;

use super::{DeviceLineage, LineageDeriver};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) use base64::engine::general_purpose::STANDARD as BASE64;

use crate::beacon_genetics::capability::CapabilityCaller;

pub(super) struct MockCaller {
    /// `Some(val)` → Ok response, `None` → simulated error.
    responses: Arc<Mutex<HashMap<String, Option<serde_json::Value>>>>,
}

impl MockCaller {
    pub(super) fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(super) async fn set_ok(&self, cap: &str, val: serde_json::Value) {
        self.responses
            .lock()
            .await
            .insert(cap.to_string(), Some(val));
    }

    pub(super) async fn set_err(&self, cap: &str, _msg: &str) {
        self.responses.lock().await.insert(cap.to_string(), None);
    }
}

impl CapabilityCaller for MockCaller {
    async fn call(
        &self,
        capability: &str,
        _params: serde_json::Value,
    ) -> crate::error::SporeResult<serde_json::Value> {
        let responses = self.responses.lock().await;
        match responses.get(capability) {
            Some(Some(val)) => Ok(val.clone()),
            Some(None) => Err(crate::error::SporeError::CapabilityCall(format!(
                "mock error for {capability}"
            ))),
            None => Err(crate::error::SporeError::CapabilityCall(format!(
                "no mock for {capability}"
            ))),
        }
    }
}

pub(super) fn sample_lineage() -> DeviceLineage {
    DeviceLineage {
        device_id: "device-123".to_string(),
        node_id: "tower".to_string(),
        family_id: "1894e909e454".to_string(),
        generation: 1,
        derived_seed: "dGVzdHNlZWQ=".to_string(),
        derived_at: 1_738_726_800,
        derivation_method: "Blake3-Lineage-KDF".to_string(),
        lineage_certificate: None,
    }
}
