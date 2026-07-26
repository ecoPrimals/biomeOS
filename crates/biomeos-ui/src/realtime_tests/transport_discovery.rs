// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_subscribe_sse_with_websocket_upgrades_to_websocket() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    subscriber.set_urls_for_test(
        Some("ws://localhost:9999/ws".to_string()),
        Some("http://localhost:9999/sse".to_string()),
    );
    let result = subscriber.subscribe_sse().await;
    assert!(
        result.is_err(),
        "subscribe_sse with both URLs delegates to subscribe_websocket which fails on invalid URL"
    );
}

#[tokio::test]
async fn test_discover_endpoints_with_ws_env() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result = subscriber.discover_endpoints_with(Some("ws://test.example/ws"), None, None, None);
    assert!(result.is_ok());
    assert!(subscriber.subscribe_websocket().await.is_err());
}

#[tokio::test]
async fn test_discover_endpoints_with_sse_env() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result =
        subscriber.discover_endpoints_with(None, Some("http://test.example/sse"), None, None);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_endpoints_with_both_env_vars() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result = subscriber.discover_endpoints_with(
        Some("ws://test.example/ws"),
        Some("http://test.example/sse"),
        None,
        None,
    );
    assert!(result.is_ok());
}
#[tokio::test]
async fn test_discover_endpoints_biomeos_api_ws_fallback() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result =
        subscriber.discover_endpoints_with(None, None, Some("ws://fallback.example/ws"), None);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_endpoints_biomeos_api_sse_fallback() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result =
        subscriber.discover_endpoints_with(None, None, None, Some("http://fallback.example/sse"));
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_sse_derives_ws_url_from_http() {
    let mut subscriber = RealTimeEventSubscriber::new("family".to_string());
    subscriber.set_urls_for_test(None, Some("http://host:9000/events".to_string()));
    let result = subscriber.subscribe_sse().await;
    assert!(result.is_ok(), "SSE with no WS should gracefully degrade");
}

#[tokio::test]
async fn test_subscribe_sse_derives_ws_url_from_https() {
    let mut subscriber = RealTimeEventSubscriber::new("family".to_string());
    subscriber.set_urls_for_test(None, Some("https://host:9000/sse".to_string()));
    let result = subscriber.subscribe_sse().await;
    assert!(result.is_ok(), "SSE with HTTPS should gracefully degrade");
}

#[tokio::test]
async fn test_subscribe_websocket_success_path() {
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;

    let tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = tcp.local_addr().unwrap();
    let ws_url = format!("ws://127.0.0.1:{}", addr.port());

    let server = tokio::spawn(async move {
        let (stream, _) = tcp.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut write, mut read) = ws.split();

        if let Some(Ok(msg)) = read.next().await {
            assert!(msg.is_text(), "expected subscribe JSON-RPC");
        }

        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "event.notify",
            "params": {
                "event": {
                    "type": "heartbeat",
                    "timestamp": 99,
                    "primals_count": 3,
                    "healthy_count": 3
                }
            }
        });
        let _ = write
            .send(tokio_tungstenite::tungstenite::Message::Text(
                serde_json::to_string(&notification).unwrap().into(),
            ))
            .await;

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    });

    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    subscriber.set_urls_for_test(Some(ws_url), None);

    let mut rx = subscriber.subscribe();

    subscriber
        .subscribe_websocket()
        .await
        .expect("websocket connection should succeed with local server");

    let event = tokio::time::timeout(std::time::Duration::from_secs(3), rx.recv()).await;
    assert!(event.is_ok(), "should receive event within timeout");
    let event = event.unwrap().unwrap();
    assert!(matches!(
        event,
        RealTimeEvent::Heartbeat { timestamp: 99, .. }
    ));

    server.abort();
}
