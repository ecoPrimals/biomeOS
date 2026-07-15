// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::gate_disabled;
use crate::*;
use biomeos_test_utils::ready_signal;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[tokio::test]
async fn events_ws_welcome_and_subscribe_roundtrip() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = axum::serve(listener, app);
    let (mut ready_tx, ready_rx) = ready_signal();
    let join = tokio::spawn(async move {
        ready_tx.signal();
        server.await.expect("serve");
    });
    ready_rx.wait().await.expect("server ready");
    let url = format!("ws://{addr}/api/v1/events/ws");
    let (ws, _) = tokio_tungstenite::connect_async(url.as_str())
        .await
        .expect("ws connect");
    let (mut write, mut read) = ws.split();
    let welcome = read.next().await.expect("welcome").expect("msg");
    let WsMessage::Text(text) = welcome else {
        panic!("expected text welcome");
    };
    assert!(text.contains("connection.established"));
    let sub = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "events.subscribe",
        "params": { "graph_id": "g1" },
        "id": 7
    });
    write
        .send(WsMessage::Text(sub.to_string().into()))
        .await
        .expect("send");
    let reply = read.next().await.expect("reply").expect("ok");
    let WsMessage::Text(reply_text) = reply else {
        panic!("expected text reply");
    };
    let v: serde_json::Value = serde_json::from_str(&reply_text).expect("json");
    assert!(v.get("result").is_some());
    let unsub = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "events.unsubscribe",
        "params": { "subscription_id": "sub_1" },
        "id": 8
    });
    write
        .send(WsMessage::Text(unsub.to_string().into()))
        .await
        .expect("unsub");
    let list = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "events.list_subscriptions",
        "id": 9
    });
    write
        .send(WsMessage::Text(list.to_string().into()))
        .await
        .expect("list");
    join.abort();
}

#[tokio::test]
async fn router_events_ws_invalid_json_and_unknown_method() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = axum::serve(listener, app);
    let (mut ready_tx, ready_rx) = ready_signal();
    let join = tokio::spawn(async move {
        ready_tx.signal();
        server.await.expect("serve");
    });
    ready_rx.wait().await.expect("server ready");
    let url = format!("ws://{addr}/api/v1/events/ws");
    let (ws, _) = tokio_tungstenite::connect_async(url.as_str())
        .await
        .expect("ws connect");
    let (mut write, mut read) = ws.split();
    let _welcome = read.next().await.expect("welcome").expect("msg");

    write
        .send(WsMessage::Text("not json".into()))
        .await
        .expect("send bad json");
    let parse_reply = read.next().await.expect("parse reply").expect("ok");
    let WsMessage::Text(parse_text) = parse_reply else {
        panic!("expected text");
    };
    let v: serde_json::Value = serde_json::from_str(&parse_text).expect("json");
    assert_eq!(v["error"]["code"], -32700);

    let unknown = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 99,
        "method": "unknown.custom",
        "params": {}
    });
    write
        .send(WsMessage::Text(unknown.to_string().into()))
        .await
        .expect("send unknown");
    let method_reply = read.next().await.expect("method reply").expect("ok");
    let WsMessage::Text(method_text) = method_reply else {
        panic!("expected text");
    };
    let v2: serde_json::Value = serde_json::from_str(&method_text).expect("json");
    assert_eq!(v2["error"]["code"], -32601);

    join.abort();
}

#[tokio::test]
async fn router_events_ws_binary_message_ignored_no_reply() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = axum::serve(listener, app);
    let (mut ready_tx, ready_rx) = ready_signal();
    let join = tokio::spawn(async move {
        ready_tx.signal();
        server.await.expect("serve");
    });
    ready_rx.wait().await.expect("server ready");
    let url = format!("ws://{addr}/api/v1/events/ws");
    let (ws, _) = tokio_tungstenite::connect_async(url.as_str())
        .await
        .expect("ws connect");
    let (mut write, mut read) = ws.split();
    let _welcome = read.next().await.expect("welcome").expect("msg");

    write
        .send(WsMessage::Binary(vec![1, 2, 3].into()))
        .await
        .expect("binary");
    let next = tokio::time::timeout(std::time::Duration::from_millis(200), read.next()).await;
    assert!(
        next.is_err(),
        "binary frames are ignored; no JSON-RPC reply expected"
    );

    let ping = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "events.list_subscriptions",
    });
    write
        .send(WsMessage::Text(ping.to_string().into()))
        .await
        .expect("list");
    let after = read.next().await.expect("after binary").expect("ok");
    let WsMessage::Text(t) = after else {
        panic!("expected text");
    };
    let v: serde_json::Value = serde_json::from_str(&t).expect("json");
    assert!(v.get("result").is_some());

    join.abort();
}
