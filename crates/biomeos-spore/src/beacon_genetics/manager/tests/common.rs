// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::beacon_genetics::capability::CapabilityCaller;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct MockCapabilityCaller {
    pub responses: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl MockCapabilityCaller {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn set_response(&self, capability: &str, response: serde_json::Value) {
        self.responses
            .lock()
            .await
            .insert(capability.to_string(), response);
    }
}

impl CapabilityCaller for MockCapabilityCaller {
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
