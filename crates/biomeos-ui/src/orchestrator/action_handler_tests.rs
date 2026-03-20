// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use biomeos_test_utils::ready_signal;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_handle_start_primal_no_toadstool() {
    let result = ActionHandler::handle_start_primal("beardog", None).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_error());
}

#[tokio::test]
async fn test_handle_stop_primal_no_toadstool() {
    let result = ActionHandler::handle_stop_primal("primal-123", None).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_error());
}

#[tokio::test]
async fn test_handle_restart_primal_no_toadstool() {
    let result = ActionHandler::handle_restart_primal("primal-456", None).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_error());
}

#[tokio::test]
async fn test_handle_accept_suggestion_no_squirrel() {
    let result =
        ActionHandler::handle_accept_suggestion("suggestion-001", "family-123", None).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    // Should succeed even without Squirrel (non-critical)
    assert!(action_result.is_success());
}

#[tokio::test]
async fn test_handle_dismiss_suggestion_no_squirrel() {
    let result =
        ActionHandler::handle_dismiss_suggestion("suggestion-002", "family-123", None).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    // Should succeed even without Squirrel (non-critical)
    assert!(action_result.is_success());
}

#[tokio::test]
async fn test_handle_unassign_device_no_clients() {
    let result = ActionHandler::handle_unassign_device("device-123", None, None, None).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_success());
}

#[tokio::test]
async fn test_handle_refresh_no_clients() {
    let result = ActionHandler::handle_refresh(None, None, None).await;

    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_success());
}

#[tokio::test]
async fn test_register_assignment_no_songbird() {
    let result = ActionHandler::register_assignment(None, "device-001", "primal-001").await;

    assert!(result.is_ok());
    let assignment_id = result.unwrap();
    // Should generate local ID
    assert!(assignment_id.starts_with("local-"));
    assert!(assignment_id.contains("device-001"));
    assert!(assignment_id.contains("primal-001"));
}

#[tokio::test]
async fn test_handle_user_action_assign_device() {
    let connections = PrimalConnections::default();
    let action = UserAction::AssignDevice {
        device_id: "dev-123".to_string(),
        primal_id: "primal-456".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_user_action_unassign_device() {
    let connections = PrimalConnections::default();
    let action = UserAction::UnassignDevice {
        device_id: "dev-123".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_user_action_start_primal() {
    let connections = PrimalConnections::default();
    let action = UserAction::StartPrimal {
        primal_name: "beardog".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    // Fails without ToadStool
    assert!(result.unwrap().is_error());
}

#[tokio::test]
async fn test_handle_user_action_stop_primal() {
    let connections = PrimalConnections::default();
    let action = UserAction::StopPrimal {
        primal_id: "primal-123".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_error());
}

#[tokio::test]
async fn test_handle_user_action_restart_primal() {
    let connections = PrimalConnections::default();
    let action = UserAction::RestartPrimal {
        primal_id: "primal-789".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_error());
}

#[tokio::test]
async fn test_handle_user_action_accept_suggestion() {
    let connections = PrimalConnections::default();
    let action = UserAction::AcceptSuggestion {
        suggestion_id: "suggestion-001".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_user_action_dismiss_suggestion() {
    let connections = PrimalConnections::default();
    let action = UserAction::DismissSuggestion {
        suggestion_id: "suggestion-002".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_user_action_refresh() {
    let connections = PrimalConnections::default();
    let action = UserAction::Refresh;
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_start_primal_toadstool_connection_fails() {
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "toadstool",
        ToadStoolClient::with_socket("toadstool", "/nonexistent/toadstool.sock"),
    );
    let action = UserAction::StartPrimal {
        primal_name: "beardog".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_error());
}

#[tokio::test]
async fn test_handle_stop_primal_toadstool_connection_fails() {
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "toadstool",
        ToadStoolClient::with_socket("toadstool", "/nonexistent/toadstool.sock"),
    );
    let action = UserAction::StopPrimal {
        primal_id: "primal-123".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_error());
}

#[tokio::test]
async fn test_handle_refresh_with_failing_clients() {
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "songbird",
        SongbirdClient::with_socket("songbird", "/nonexistent/songbird.sock"),
    );
    connections.add_client(
        "toadstool",
        ToadStoolClient::with_socket("toadstool", "/nonexistent/toadstool.sock"),
    );
    let action = UserAction::Refresh;
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    let action_result = result.unwrap();
    assert!(action_result.is_success());
    if let ActionResult::Success { message } = &action_result {
        assert!(message.contains("0 sources"));
    }
}

#[tokio::test]
async fn test_register_assignment_songbird_returns_fallback_id() {
    let result = ActionHandler::register_assignment(None, "device-001", "primal-001").await;
    assert!(result.is_ok());
    let id = result.unwrap();
    assert!(id.starts_with("local-"));
    assert!(id.contains("device-001"));
    assert!(id.contains("primal-001"));
}

#[tokio::test]
async fn test_handle_unassign_device_with_songbird_failing() {
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "songbird",
        SongbirdClient::with_socket("songbird", "/nonexistent/songbird.sock"),
    );
    let action = UserAction::UnassignDevice {
        device_id: "dev-123".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_success());
}

#[tokio::test]
async fn test_handle_assign_device_success_with_empty_connections() {
    let connections = PrimalConnections::default();
    let action = UserAction::AssignDevice {
        device_id: "gpu-0".to_string(),
        primal_id: "toadstool".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "family-1", &connections).await;
    assert!(result.is_ok());
    let ar = result.unwrap();
    assert!(ar.is_success());
    if let ActionResult::Success { message } = &ar {
        assert!(message.contains("gpu-0"));
        assert!(message.contains("toadstool"));
    }
}

#[tokio::test]
async fn test_register_assignment_songbird_returns_id() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("songbird.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let _req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"assignment_id": "songbird-abc-123"},
                "id": 1
            });
            let line = format!("{resp}\n");
            stream.write_all(line.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    });
    ready_rx.wait().await.unwrap();
    let songbird = SongbirdClient::with_socket("songbird", &socket_path);
    let result = ActionHandler::register_assignment(Some(&songbird), "device-1", "primal-1").await;
    assert!(result.is_ok());
    let id = result.unwrap();
    assert_eq!(id, "songbird-abc-123");
    server.abort();
}

#[tokio::test]
async fn test_handle_assign_device_beardog_denied() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("beardog.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        let mut conn_count = 0u32;
        while let Ok((mut stream, _)) = listener.accept().await {
            conn_count += 1;
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = if conn_count == 1 {
                serde_json::json!({"jsonrpc":"2.0","result":{"user_id":"u1"},"id":req["id"]})
            } else {
                serde_json::json!({"jsonrpc":"2.0","result":{"authorized":false,"reason":"Policy denied"},"id":req["id"]})
            };
            let line = format!("{resp}\n");
            let _ = stream.write_all(line.as_bytes()).await;
            let _ = stream.flush().await;
        }
    });
    ready_rx.wait().await.unwrap();
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "beardog",
        BearDogClient::with_socket("beardog", &socket_path),
    );
    let action = UserAction::AssignDevice {
        device_id: "dev-1".to_string(),
        primal_id: "primal-1".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "fam", &connections).await;
    assert!(result.is_ok());
    let ar = result.unwrap();
    assert!(ar.is_error());
    if let ActionResult::Error { message } = &ar {
        assert!(message.contains("Authorization denied"));
    }
    server.abort();
}

#[tokio::test]
async fn test_handle_assign_device_songbird_validation_invalid() {
    let temp = tempfile::tempdir().expect("temp dir");
    let beardog_path = temp.path().join("beardog.sock");
    let songbird_path = temp.path().join("songbird.sock");
    let bp = beardog_path.clone();
    let sp = songbird_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let bl = tokio::net::UnixListener::bind(&bp).expect("bind beardog");
        let sl = tokio::net::UnixListener::bind(&sp).expect("bind songbird");
        ready_tx.signal();
        if let Ok((mut s, _)) = bl.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = s.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let r1 = serde_json::json!({"jsonrpc":"2.0","result":{"user_id":"u1"},"id":req["id"]});
            let _ = s.write_all(format!("{r1}\n").as_bytes()).await;
            let _ = s.flush().await;
        }
        if let Ok((mut s, _)) = bl.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = s.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let r2 =
                serde_json::json!({"jsonrpc":"2.0","result":{"authorized":true},"id":req["id"]});
            let _ = s.write_all(format!("{r2}\n").as_bytes()).await;
            let _ = s.flush().await;
        }
        if let Ok((mut s, _)) = sl.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = s.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let r = serde_json::json!({"jsonrpc":"2.0","result":{"valid":false,"reason":"Device already assigned"},"id":req["id"]});
            let _ = s.write_all(format!("{r}\n").as_bytes()).await;
            let _ = s.flush().await;
        }
    });
    ready_rx.wait().await.unwrap();
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "beardog",
        BearDogClient::with_socket("beardog", &beardog_path),
    );
    connections.add_client(
        "songbird",
        SongbirdClient::with_socket("songbird", &songbird_path),
    );
    let action = UserAction::AssignDevice {
        device_id: "dev-1".to_string(),
        primal_id: "primal-1".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "fam", &connections).await;
    assert!(result.is_ok());
    let ar = result.unwrap();
    assert!(ar.is_error());
    if let ActionResult::Error { message } = &ar {
        assert!(message.contains("Validation failed"));
    }
    server.abort();
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_handle_assign_device_toadstool_insufficient_capacity() {
    let temp = tempfile::tempdir().expect("temp dir");
    let beardog_path = temp.path().join("beardog.sock");
    let songbird_path = temp.path().join("songbird.sock");
    let toadstool_path = temp.path().join("toadstool.sock");
    let bp = beardog_path.clone();
    let sp = songbird_path.clone();
    let tp = toadstool_path.clone();
    let (ready_tx, ready_rx) = ready_signal();
    let bind_count = std::sync::Arc::new(AtomicUsize::new(0));
    let bind_count1 = bind_count.clone();
    let bind_count2 = bind_count.clone();
    let bind_count3 = bind_count.clone();
    let ready_tx_cell = std::sync::Arc::new(std::sync::Mutex::new(Some(ready_tx)));
    let ready_tx1 = ready_tx_cell.clone();
    let s1 = tokio::spawn(async move {
        let l = tokio::net::UnixListener::bind(&bp).expect("bind");
        if bind_count1.fetch_add(1, Ordering::SeqCst) == 2 {
            let mut guard = ready_tx1.lock().unwrap();
            if let Some(mut tx) = guard.take() {
                tx.signal();
            }
        }
        for _ in 0..2 {
            if let Ok((mut s, _)) = l.accept().await {
                let mut buf = vec![0u8; 4096];
                let n = s.read(&mut buf).await.expect("read");
                let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                let resp = if req.get("method").and_then(|m| m.as_str())
                    == Some("auth.get_current_user")
                {
                    serde_json::json!({"jsonrpc":"2.0","result":{"user_id":"u1"},"id":req["id"]})
                } else {
                    serde_json::json!({"jsonrpc":"2.0","result":{"authorized":true},"id":req["id"]})
                };
                let _ = s.write_all(format!("{resp}\n").as_bytes()).await;
                let _ = s.flush().await;
            }
        }
    });
    let ready_tx2 = ready_tx_cell.clone();
    let s2 = tokio::spawn(async move {
        let l = tokio::net::UnixListener::bind(&sp).expect("bind");
        if bind_count2.fetch_add(1, Ordering::SeqCst) == 2 {
            let mut guard = ready_tx2.lock().unwrap();
            if let Some(mut tx) = guard.take() {
                tx.signal();
            }
        }
        if let Ok((mut s, _)) = l.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = s.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{"valid":true},"id":req["id"]});
            let _ = s.write_all(format!("{resp}\n").as_bytes()).await;
            let _ = s.flush().await;
        }
    });
    let ready_tx3 = ready_tx_cell.clone();
    let s3 = tokio::spawn(async move {
        let l = tokio::net::UnixListener::bind(&tp).expect("bind");
        if bind_count3.fetch_add(1, Ordering::SeqCst) == 2 {
            let mut guard = ready_tx3.lock().unwrap();
            if let Some(mut tx) = guard.take() {
                tx.signal();
            }
        }
        if let Ok((mut s, _)) = l.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = s.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{"available":false,"reason":"GPU memory full"},"id":req["id"]});
            let _ = s.write_all(format!("{resp}\n").as_bytes()).await;
            let _ = s.flush().await;
        }
    });
    ready_rx.wait().await.unwrap();
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "beardog",
        BearDogClient::with_socket("beardog", &beardog_path),
    );
    connections.add_client(
        "songbird",
        SongbirdClient::with_socket("songbird", &songbird_path),
    );
    connections.add_client(
        "toadstool",
        ToadStoolClient::with_socket("toadstool", &toadstool_path),
    );
    let action = UserAction::AssignDevice {
        device_id: "dev-1".to_string(),
        primal_id: "primal-1".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "fam", &connections).await;
    assert!(result.is_ok());
    let ar = result.unwrap();
    assert!(ar.is_error());
    if let ActionResult::Error { message } = &ar {
        assert!(message.contains("Insufficient capacity"));
    }
    s1.abort();
    s2.abort();
    s3.abort();
}

#[tokio::test]
async fn test_handle_start_primal_success() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("toadstool.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{"pid":9999},"id":req["id"]});
            let _ = stream.write_all(format!("{resp}\n").as_bytes()).await;
            let _ = stream.flush().await;
        }
    });
    ready_rx.wait().await.unwrap();
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "toadstool",
        ToadStoolClient::with_socket("toadstool", &socket_path),
    );
    let action = UserAction::StartPrimal {
        primal_name: "beardog".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "fam", &connections).await;
    assert!(result.is_ok());
    let ar = result.unwrap();
    assert!(ar.is_success());
    if let ActionResult::Success { message } = &ar {
        assert!(message.contains("PID: 9999"));
    }
    server.abort();
}

#[tokio::test]
async fn test_handle_stop_primal_success() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("toadstool.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{},"id":req["id"]});
            let _ = stream.write_all(format!("{resp}\n").as_bytes()).await;
            let _ = stream.flush().await;
        }
    });
    ready_rx.wait().await.unwrap();
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "toadstool",
        ToadStoolClient::with_socket("toadstool", &socket_path),
    );
    let action = UserAction::StopPrimal {
        primal_id: "primal-123".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "fam", &connections).await;
    assert!(result.is_ok());
    let ar = result.unwrap();
    assert!(ar.is_success());
    server.abort();
}

#[tokio::test]
async fn test_handle_restart_primal_success() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("toadstool.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{"pid":7777},"id":req["id"]});
            let _ = stream.write_all(format!("{resp}\n").as_bytes()).await;
            let _ = stream.flush().await;
        }
    });
    ready_rx.wait().await.unwrap();
    let mut connections = PrimalConnections::default();
    connections.add_client(
        "toadstool",
        ToadStoolClient::with_socket("toadstool", &socket_path),
    );
    let action = UserAction::RestartPrimal {
        primal_id: "primal-456".to_string(),
    };
    let result = ActionHandler::handle_user_action(action, "fam", &connections).await;
    assert!(result.is_ok());
    let ar = result.unwrap();
    assert!(ar.is_success());
    if let ActionResult::Success { message } = &ar {
        assert!(message.contains("7777"));
    }
    server.abort();
}
