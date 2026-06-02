// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use crate::events::UIEvent;
use biomeos_test_utils::ready_signal;

#[tokio::test]
async fn test_orchestrator_creation() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family");
    assert!(orchestrator.is_ok());
}

#[tokio::test]
async fn test_orchestrator_start_graceful_degradation() {
    // Should start even with no primals available
    let mut orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();

    let result = orchestrator.start().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_user_action_assign_device() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();

    let result = orchestrator
        .handle_user_action(UserAction::AssignDevice {
            device_id: "test-device".to_string(),
            primal_id: "test-primal".to_string(),
        })
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_orchestrator_state_and_events_accessors() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let state = orchestrator.state();
    let events = orchestrator.events();

    assert!(state.read().await.devices.is_empty());
    let _rx = events.subscribe();
}

#[tokio::test]
async fn test_handle_primal_event_primal_started() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "type": "primal.started",
        "primal_name": "beardog-1"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let received = rx.recv().await.expect("Should receive event");
    match received {
        UIEvent::PrimalStatusChanged { primal_id, status } => {
            assert_eq!(primal_id, "beardog-1");
            assert_eq!(status, "started");
        }
        _ => panic!("Expected PrimalStatusChanged, got {received:?}"),
    }
}

#[tokio::test]
async fn test_handle_primal_event_primal_stopped() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "type": "primal.stopped",
        "primal_name": "songbird-1"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let received = rx.recv().await.expect("Should receive event");
    match received {
        UIEvent::PrimalStatusChanged { primal_id, status } => {
            assert_eq!(primal_id, "songbird-1");
            assert_eq!(status, "stopped");
        }
        _ => panic!("Expected PrimalStatusChanged, got {received:?}"),
    }
}

#[tokio::test]
async fn test_handle_primal_event_device_connected() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "type": "device.connected",
        "device_id": "gpu-0"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let received = rx.recv().await.expect("Should receive event");
    match received {
        UIEvent::DeviceStatusChanged { device_id, status } => {
            assert_eq!(device_id, "gpu-0");
            assert_eq!(status, "connected");
        }
        _ => panic!("Expected DeviceStatusChanged, got {received:?}"),
    }
}

#[tokio::test]
async fn test_handle_primal_event_device_disconnected() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "type": "device.disconnected",
        "device_id": "gpu-0"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let received = rx.recv().await.expect("Should receive event");
    match received {
        UIEvent::DeviceStatusChanged { device_id, status } => {
            assert_eq!(device_id, "gpu-0");
            assert_eq!(status, "disconnected");
        }
        _ => panic!("Expected DeviceStatusChanged, got {received:?}"),
    }
}

#[tokio::test]
async fn test_handle_primal_event_unknown_type_no_emit() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "type": "unknown.event",
        "data": "ignored"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
    assert!(result.is_err(), "Unknown event type should not emit");
}

#[tokio::test]
async fn test_handle_primal_event_missing_primal_name_ignored() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "type": "primal.started"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
    assert!(result.is_err(), "Event without primal_name should not emit");
}

#[tokio::test]
async fn test_handle_primal_event_missing_device_id_ignored() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "type": "device.connected"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
    assert!(result.is_err(), "Event without device_id should not emit");
}

#[tokio::test]
async fn test_handle_user_action_unassign_device() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let result = orchestrator
        .handle_user_action(UserAction::UnassignDevice {
            device_id: "test-device".to_string(),
        })
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_user_action_refresh() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let result = orchestrator.handle_user_action(UserAction::Refresh).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_user_action_accept_suggestion() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let result = orchestrator
        .handle_user_action(UserAction::AcceptSuggestion {
            suggestion_id: "sug-1".to_string(),
        })
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_user_action_dismiss_suggestion() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let result = orchestrator
        .handle_user_action(UserAction::DismissSuggestion {
            suggestion_id: "sug-2".to_string(),
        })
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_user_action_start_primal_no_toadstool() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let result = orchestrator
        .handle_user_action(UserAction::StartPrimal {
            primal_name: "beardog".to_string(),
        })
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_error());
}

#[tokio::test]
async fn test_handle_user_action_stop_primal() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let result = orchestrator
        .handle_user_action(UserAction::StopPrimal {
            primal_id: "beardog-1".to_string(),
        })
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_error());
}

#[tokio::test]
async fn test_handle_user_action_restart_primal() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let result = orchestrator
        .handle_user_action(UserAction::RestartPrimal {
            primal_id: "songbird-1".to_string(),
        })
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_error());
}

#[tokio::test]
async fn test_handle_primal_event_missing_type_uses_unknown() {
    let orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    let mut rx = orchestrator.events().subscribe();

    let event = serde_json::json!({
        "data": "no type field"
    });
    orchestrator.handle_primal_event_for_test(&event);

    let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "run() blocks indefinitely"]
async fn test_run_subscribes_and_loops() {
    let mut orchestrator = InteractiveUIOrchestrator::new("test-family").unwrap();
    orchestrator.start().await.unwrap();
    let _ =
        tokio::time::timeout(std::time::Duration::from_millis(100), orchestrator.run()).await;
}

#[tokio::test]
async fn test_run_with_registry_subscribe_failure() {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let temp = tempfile::tempdir().expect("temp dir");
    let biomeos_dir = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).expect("create biomeos dir");
    let socket_path = biomeos_dir.join("songbird.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();

    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((stream, _)) = listener.accept().await {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            let _ = reader.read_line(&mut line).await;
            drop(reader);
        }
    });

    ready_rx.wait().await.unwrap();

    let mut orchestrator = InteractiveUIOrchestrator::new_with_runtime_overrides(
        "test-family",
        Some(temp.path().to_path_buf()),
        Some("songbird".to_string()),
    )
    .unwrap();
    orchestrator.start().await.unwrap();

    let result =
        tokio::time::timeout(std::time::Duration::from_millis(500), orchestrator.run()).await;

    server.abort();

    assert!(
        result.is_err(),
        "run() blocks indefinitely; timeout should elapse (Err = timeout fired)"
    );
}

#[tokio::test]
async fn test_run_with_registry_events_poll_non_array() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let temp = tempfile::tempdir().expect("temp dir");
    let biomeos_dir = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).expect("create biomeos dir");
    let socket_path = biomeos_dir.join("songbird.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();

    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        let mut conn_count = 0u32;
        while let Ok((mut stream, _)) = listener.accept().await {
            conn_count += 1;
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let resp = if conn_count == 1 {
                serde_json::json!({"jsonrpc":"2.0","result":{},"id":1})
            } else {
                serde_json::json!({"jsonrpc":"2.0","result":"not_an_array","id":1})
            };
            let line = format!("{}\n", resp);
            let _ = stream.write_all(line.as_bytes()).await;
            let _ = stream.flush().await;
        }
    });

    ready_rx.wait().await.unwrap();

    let mut orchestrator = InteractiveUIOrchestrator::new_with_runtime_overrides(
        "test-family",
        Some(temp.path().to_path_buf()),
        Some("songbird".to_string()),
    )
    .unwrap();
    orchestrator.start().await.unwrap();

    let result =
        tokio::time::timeout(std::time::Duration::from_millis(500), orchestrator.run()).await;

    server.abort();

    assert!(
        result.is_err(),
        "run() blocks indefinitely; timeout should elapse"
    );
}
