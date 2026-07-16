// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Additional routing coverage for uncovered dispatch paths and route arms.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

mod core_routes;
mod dispatch;
mod extended_routes;

use crate::neural_api_server::NeuralApiServer;
use serde_json::json;

pub(super) async fn rpc(
    server: &NeuralApiServer,
    method: &str,
    params: serde_json::Value,
    id: u64,
) -> serde_json::Value {
    let req = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    })
    .to_string();
    server.handle_request_json(&req).await
}
