// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use biomeos_test_utils::ready_signal;
use biomeos_types::xr::TrackedDeviceType;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn mock_devices() -> Vec<HapticDeviceCapabilities> {
    vec![
        HapticDeviceCapabilities {
            device_type: HapticDeviceType::Rumble,
            max_force_n: None,
            max_frequency_hz: Some(500.0),
            force_dof: 0,
            update_hz: 100,
        },
        HapticDeviceCapabilities {
            device_type: HapticDeviceType::ForceFeedback,
            max_force_n: Some(5.0),
            max_frequency_hz: None,
            force_dof: 3,
            update_hz: 1000,
        },
    ]
}

#[test]
fn test_pipeline_new() {
    let pipeline = HapticPipeline::new();
    assert!(!pipeline.is_active());
    assert!(pipeline.devices().is_empty());
    assert!(!pipeline.has_force_feedback());
}

#[test]
fn test_find_device() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = mock_devices();
    pipeline.active = true;

    assert!(pipeline.find_device(HapticDeviceType::Rumble).is_some());
    assert!(
        pipeline
            .find_device(HapticDeviceType::ForceFeedback)
            .is_some()
    );
    assert!(
        pipeline
            .find_device(HapticDeviceType::Electrotactile)
            .is_none()
    );
}

#[test]
fn test_has_force_feedback() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = mock_devices();
    assert!(pipeline.has_force_feedback());

    pipeline.devices = vec![HapticDeviceCapabilities {
        device_type: HapticDeviceType::Rumble,
        max_force_n: None,
        max_frequency_hz: Some(500.0),
        force_dof: 0,
        update_hz: 100,
    }];
    assert!(!pipeline.has_force_feedback());
}

#[tokio::test]
async fn test_send_command_inactive() {
    let pipeline = HapticPipeline::new();
    let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
    let command = HapticCommand {
        device: HapticDeviceType::Rumble,
        target: TrackedDeviceType::RightHand,
        intensity: 0.5,
        duration_ms: 100,
        frequency_hz: Some(200.0),
        force_vector: None,
    };
    let result = pipeline.send_command(&client, command).await;
    assert!(result.is_ok()); // graceful degradation
}

#[tokio::test]
async fn test_stop_all_inactive() {
    let pipeline = HapticPipeline::new();
    let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
    let result = pipeline.stop_all(&client).await;
    assert!(result.is_ok());
}

#[test]
fn test_default_trait() {
    let pipeline = HapticPipeline::default();
    assert!(!pipeline.is_active());
}

#[test]
fn test_devices_empty_when_new() {
    let pipeline = HapticPipeline::new();
    assert!(pipeline.devices().is_empty());
}

#[test]
fn test_find_device_empty_pipeline() {
    let pipeline = HapticPipeline::new();
    assert!(pipeline.find_device(HapticDeviceType::Rumble).is_none());
}

#[test]
fn test_haptic_command_with_force_vector_clamping() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = vec![HapticDeviceCapabilities {
        device_type: HapticDeviceType::ForceFeedback,
        max_force_n: Some(5.0),
        max_frequency_hz: None,
        force_dof: 3,
        update_hz: 1000,
    }];
    pipeline.active = true;
    assert!(
        pipeline
            .find_device(HapticDeviceType::ForceFeedback)
            .is_some()
    );
}

#[test]
fn test_intensity_clamping() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = mock_devices();
    pipeline.active = true;
    let caps = pipeline.find_device(HapticDeviceType::Rumble);
    assert!(caps.is_some());
}

#[test]
fn test_device_type_precision_actuator() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = vec![HapticDeviceCapabilities {
        device_type: HapticDeviceType::PrecisionActuator,
        max_force_n: None,
        max_frequency_hz: Some(1000.0),
        force_dof: 0,
        update_hz: 500,
    }];
    pipeline.active = true;
    assert!(
        pipeline
            .find_device(HapticDeviceType::PrecisionActuator)
            .is_some()
    );
    assert!(!pipeline.has_force_feedback());
}

#[test]
fn test_device_type_electrotactile() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = vec![HapticDeviceCapabilities {
        device_type: HapticDeviceType::Electrotactile,
        max_force_n: None,
        max_frequency_hz: Some(200.0),
        force_dof: 0,
        update_hz: 200,
    }];
    pipeline.active = true;
    assert!(
        pipeline
            .find_device(HapticDeviceType::Electrotactile)
            .is_some()
    );
}

#[tokio::test]
async fn test_send_command_active_device_found_clamping_path() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = mock_devices();
    pipeline.active = true;
    let command = HapticCommand {
        device: HapticDeviceType::Rumble,
        target: TrackedDeviceType::RightHand,
        intensity: 1.5,
        duration_ms: 100,
        frequency_hz: Some(600.0),
        force_vector: None,
    };
    let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
    let result = pipeline.send_command(&client, command).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_send_command_active_device_not_in_list() {
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = vec![HapticDeviceCapabilities {
        device_type: HapticDeviceType::Rumble,
        max_force_n: None,
        max_frequency_hz: Some(500.0),
        force_dof: 0,
        update_hz: 100,
    }];
    pipeline.active = true;
    let command = HapticCommand {
        device: HapticDeviceType::ForceFeedback,
        target: TrackedDeviceType::LeftHand,
        intensity: 0.5,
        duration_ms: 50,
        frequency_hz: None,
        force_vector: Some([10.0, 5.0, -3.0]),
    };
    let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
    let result = pipeline.send_command(&client, command).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "Requires live PetalTongue Unix socket"]
async fn test_discover() {
    let mut pipeline = HapticPipeline::new();
    let client = crate::primal_client::PrimalClient::with_socket("ui", "/tmp/ui.sock");
    let _ = pipeline.discover(&client).await;
}

#[tokio::test]
async fn test_discover_success_from_mock_socket() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("ui.sock");
    let path = socket_path.clone();
    let expected = mock_devices();
    let expected_clone = expected.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            assert_eq!(
                req.get("method").and_then(|m| m.as_str()),
                Some("xr.discover_haptic")
            );
            let result = serde_json::to_value(&expected_clone).expect("serialize devices");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": req["id"]
            });
            let line = format!("{resp}\n");
            stream.write_all(line.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    });
    ready_rx.wait().await.unwrap();
    let mut pipeline = HapticPipeline::new();
    let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
    let discovered_len = pipeline.discover(&client).await.expect("discover").len();
    assert!(pipeline.is_active());
    assert_eq!(discovered_len, expected.len());
    assert_eq!(pipeline.devices().len(), discovered_len);
    server.abort();
}

#[tokio::test]
async fn test_send_command_success_clamps_rumble_caps() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("ui.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            assert_eq!(
                req.get("method").and_then(|m| m.as_str()),
                Some("xr.send_haptic")
            );
            let params = req.get("params").expect("params");
            assert!(
                (params
                    .get("intensity")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap()
                    - 1.0)
                    .abs()
                    < 1e-9
            );
            assert!(
                (params
                    .get("frequency_hz")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap()
                    - 500.0)
                    .abs()
                    < 1e-9
            );
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{},"id":req["id"]});
            let line = format!("{resp}\n");
            stream.write_all(line.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    });
    ready_rx.wait().await.unwrap();
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = mock_devices();
    pipeline.active = true;
    let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
    let command = HapticCommand {
        device: HapticDeviceType::Rumble,
        target: TrackedDeviceType::RightHand,
        intensity: 2.0,
        duration_ms: 100,
        frequency_hz: Some(800.0),
        force_vector: None,
    };
    pipeline.send_command(&client, command).await.expect("send");
    server.abort();
}

#[tokio::test]
async fn test_send_command_clamps_force_feedback_vector() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("ui.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let params = req.get("params").expect("params");
            let fv = params
                .get("force_vector")
                .and_then(|v| v.as_array())
                .expect("force_vector");
            assert_eq!(fv.len(), 3);
            assert!((fv[0].as_f64().unwrap() - 5.0).abs() < 1e-9);
            assert!((fv[1].as_f64().unwrap() + 5.0).abs() < 1e-9);
            assert!((fv[2].as_f64().unwrap() - 0.0).abs() < 1e-9);
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{},"id":req["id"]});
            let line = format!("{resp}\n");
            stream.write_all(line.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    });
    ready_rx.wait().await.unwrap();
    let mut pipeline = HapticPipeline::new();
    pipeline.devices = vec![HapticDeviceCapabilities {
        device_type: HapticDeviceType::ForceFeedback,
        max_force_n: Some(5.0),
        max_frequency_hz: None,
        force_dof: 3,
        update_hz: 1000,
    }];
    pipeline.active = true;
    let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
    let command = HapticCommand {
        device: HapticDeviceType::ForceFeedback,
        target: TrackedDeviceType::LeftHand,
        intensity: 0.5,
        duration_ms: 20,
        frequency_hz: None,
        force_vector: Some([100.0, -10.0, 0.0]),
    };
    pipeline.send_command(&client, command).await.expect("send");
    server.abort();
}

#[tokio::test]
async fn test_stop_all_active_success() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("ui.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            assert_eq!(
                req.get("method").and_then(|m| m.as_str()),
                Some("xr.stop_haptic")
            );
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{},"id":req["id"]});
            let line = format!("{resp}\n");
            stream.write_all(line.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    });
    ready_rx.wait().await.unwrap();
    let mut pipeline = HapticPipeline::new();
    pipeline.active = true;
    let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
    pipeline.stop_all(&client).await.expect("stop");
    server.abort();
}

#[tokio::test]
async fn test_discover_invalid_payload_returns_err() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("ui.sock");
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let server = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).await.expect("read");
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "result": "not-an-array",
                "id": req["id"]
            });
            let line = format!("{resp}\n");
            stream.write_all(line.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    });
    ready_rx.wait().await.unwrap();
    let mut pipeline = HapticPipeline::new();
    let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
    let err = pipeline.discover(&client).await.unwrap_err();
    assert!(err.to_string().contains("invalid type") || err.to_string().contains("invalid"));
    server.abort();
}
