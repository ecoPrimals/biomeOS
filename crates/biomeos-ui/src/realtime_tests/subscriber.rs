// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::sync::Arc;

#[tokio::test]
async fn test_subscriber_creation() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    // Subscriber created with default URLs (none until discover_endpoints)
    let mut rx = subscriber.subscribe();
    assert!(rx.try_recv().is_err()); // No events yet
}
#[tokio::test]
async fn test_event_handler_creation() {
    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let _handler = RealTimeEventHandler::new(subscriber);

    // Handler created successfully — reaching this point validates construction
}

#[tokio::test]
async fn test_subscriber_subscribe() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let _rx1 = subscriber.subscribe();
    let _rx2 = subscriber.subscribe();

    // Both subscriptions created independently — reaching this validates the pattern
}
#[tokio::test]
async fn test_subscribe_sse_no_websocket_returns_ok() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    subscriber.set_urls_for_test(None, Some("http://localhost:9999/sse".to_string()));

    let result = subscriber.subscribe_sse().await;
    assert!(
        result.is_ok(),
        "subscribe_sse should return Ok when only SSE URL is set (graceful degradation)"
    );
}

#[tokio::test]
async fn test_discover_endpoints_no_env() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());

    let result = subscriber.discover_endpoints();
    assert!(result.is_ok());
    // With no env vars, subscribe_websocket should fail
    assert!(subscriber.subscribe_websocket().await.is_err());
}

#[tokio::test]
async fn test_subscribe_websocket_no_url_returns_err() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());

    let result = subscriber.subscribe_websocket().await;
    assert!(
        result.is_err(),
        "subscribe_websocket should fail when URL not discovered"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("WebSocket URL not discovered"),
        "Expected context about URL, got: {err}"
    );
}

#[tokio::test]
async fn test_subscribe_sse_no_url_returns_err() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    // sse_url is None by default
    let result = subscriber.subscribe_sse().await;
    assert!(
        result.is_err(),
        "subscribe_sse should fail when SSE URL not discovered"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("SSE URL not discovered"),
        "Expected context about SSE URL, got: {err}"
    );
}
