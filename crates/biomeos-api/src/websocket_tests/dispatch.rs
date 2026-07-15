// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use tokio::sync::RwLock;

use super::super::*;

#[tokio::test]
async fn test_dispatch_ws_method_subscribe_unsubscribe_list() {
    let subs: Arc<RwLock<HashMap<String, SubscriptionFilter>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let next = AtomicU64::new(1);
    let r1 = dispatch_ws_method(
        "events.subscribe",
        Some(serde_json::json!({"graph_id": "g9"})),
        Some(serde_json::json!(1)),
        &subs,
        &next,
    )
    .await;
    assert!(r1.error.is_none());
    let sid = r1
        .result
        .as_ref()
        .and_then(|x| x.get("subscription_id"))
        .and_then(|v| v.as_str())
        .expect("sub id");
    let r2 = dispatch_ws_method(
        "events.list_subscriptions",
        None,
        Some(serde_json::json!(2)),
        &subs,
        &next,
    )
    .await;
    assert!(r2.error.is_none());
    assert_eq!(
        r2.result
            .as_ref()
            .and_then(|x| x.get("count"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    let r3 = dispatch_ws_method(
        "events.unsubscribe",
        Some(serde_json::json!({"subscription_id": sid})),
        Some(serde_json::json!(3)),
        &subs,
        &next,
    )
    .await;
    assert!(r3.error.is_none());
    assert_eq!(
        r3.result
            .as_ref()
            .and_then(|x| x.get("success"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

#[tokio::test]
async fn test_dispatch_ws_method_unknown_method() {
    let subs: Arc<RwLock<HashMap<String, SubscriptionFilter>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let next = AtomicU64::new(0);
    let r = dispatch_ws_method(
        "events.not_a_method",
        None,
        Some(serde_json::json!("x")),
        &subs,
        &next,
    )
    .await;
    assert_eq!(r.error.as_ref().map(|e| e.code), Some(-32601));
}

#[tokio::test]
async fn test_dispatch_ws_method_subscribe_invalid_params_uses_default_filter() {
    let subs: Arc<RwLock<HashMap<String, SubscriptionFilter>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let next = AtomicU64::new(0);
    let r = dispatch_ws_method(
        "events.subscribe",
        Some(serde_json::json!({"graph_id": []})),
        None,
        &subs,
        &next,
    )
    .await;
    assert!(r.error.is_none());
    assert!(
        r.result
            .as_ref()
            .and_then(|x| x.get("subscription_id"))
            .is_some()
    );
}

#[tokio::test]
async fn test_dispatch_ws_method_unsubscribe_missing_id() {
    let subs: Arc<RwLock<HashMap<String, SubscriptionFilter>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let next = AtomicU64::new(0);
    let r = dispatch_ws_method(
        "events.unsubscribe",
        Some(serde_json::json!({})),
        Some(serde_json::json!(0)),
        &subs,
        &next,
    )
    .await;
    assert!(r.error.is_none());
    assert_eq!(
        r.result
            .as_ref()
            .and_then(|x| x.get("success"))
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
}
