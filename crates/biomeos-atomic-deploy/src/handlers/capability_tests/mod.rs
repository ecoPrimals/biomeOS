// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability handler tests - extracted to keep capability.rs under 1000 lines

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test assertions")]

use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;

use super::capability::CapabilityHandler;

pub(crate) fn make_handler() -> CapabilityHandler {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    CapabilityHandler::new(router, registry)
}

pub(crate) async fn handler_with_registration() -> CapabilityHandler {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog-test.sock",
        "source": "test"
    }));
    handler.register(&params).await.unwrap();
    handler
}

mod register;
mod discover;
mod route;
mod register_route;
mod resolve;
mod metrics_mcp;
