// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use std::io::{BufRead, BufReader, Write};

#[cfg(unix)]
use std::os::unix::net::UnixListener;
#[cfg(unix)]
use std::thread;

#[cfg(unix)]
fn unix_isolated_tempdir(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("temp dir under /tmp")
}

#[test]
fn no_socket_returns_none() {
    assert!(
        NeuralBridge::discover_with(Some("/nonexistent/neural-api.sock"), Some("test-family"))
            .is_none()
    );
}

#[test]
fn parse_success_response() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "et0": 3.88 },
        "id": 1
    });
    let result = parse_response(&resp).unwrap();
    let et0 = result.value["et0"].as_f64().unwrap();
    assert!((et0 - 3.88).abs() < f64::EPSILON);
}

#[test]
fn parse_error_response() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "Method not found" },
        "id": 1
    });
    let err = parse_response(&resp).unwrap_err();
    assert!(matches!(err, NeuralError::Rpc { code: -32601, .. }));
}

#[test]
fn parse_error_response_missing_code_defaults_to_negative_one() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "message": "no code" },
        "id": 1
    });
    let err = parse_response(&resp).unwrap_err();
    if let NeuralError::Rpc { code, message } = err {
        assert_eq!(code, -1);
        assert_eq!(message, "no code");
    } else {
        panic!("expected NeuralError::Rpc");
    }
}

#[test]
fn parse_error_response_missing_message_defaults_to_unknown() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": 42 },
        "id": 1
    });
    let err = parse_response(&resp).unwrap_err();
    if let NeuralError::Rpc { code, message } = err {
        assert_eq!(code, 42);
        assert_eq!(message, "unknown error");
    } else {
        panic!("expected NeuralError::Rpc");
    }
}

#[test]
fn with_timeout_builder() {
    let bridge = NeuralBridge {
        socket_path: PathBuf::from("/tmp/test.sock"),
        timeout: Duration::from_secs(30),
    };
    let bridge = bridge.with_timeout(Duration::from_secs(5));
    assert_eq!(bridge.timeout, Duration::from_secs(5));
}

#[test]
fn socket_path_getter() {
    let path = PathBuf::from("/tmp/neural-api.sock");
    let bridge = NeuralBridge {
        socket_path: path.clone(),
        timeout: Duration::from_secs(30),
    };
    assert_eq!(bridge.socket_path(), path.as_path());
}

#[test]
fn neural_error_display() {
    let err = NeuralError::NotFound("socket not found".to_string());
    assert!(err.to_string().contains("Neural API not found"));
    assert!(err.to_string().contains("socket not found"));

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = NeuralError::Connection(io_err);
    assert!(err.to_string().contains("Connection error"));

    let err = NeuralError::Json("parse error".to_string());
    assert!(err.to_string().contains("JSON error"));

    let err = NeuralError::Rpc {
        code: -32601,
        message: "Method not found".to_string(),
    };
    assert!(err.to_string().contains("RPC error"));
    assert!(err.to_string().contains("-32601"));

    let err = NeuralError::Timeout;
    assert_eq!(err.to_string(), "Timeout");
}

#[test]
fn parse_response_null_result() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": null,
        "id": 1
    });
    let result = parse_response(&resp).unwrap();
    assert!(result.value.is_null());
}

#[test]
fn parse_response_missing_result() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1
    });
    let result = parse_response(&resp).unwrap();
    assert!(result.value.is_null());
}

#[test]
fn resolve_socket_via_env_var() {
    let temp = tempfile::tempdir().expect("temp dir");
    let sock_path = temp.path().join("neural-api.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    let bridge = NeuralBridge::discover_with(
        Some(sock_path.to_string_lossy().as_ref()),
        Some("test-family"),
    );
    assert!(bridge.is_some());
    assert_eq!(bridge.unwrap().socket_path(), sock_path.as_path());
}

#[test]
fn resolve_socket_tier_xdg_runtime_dir() {
    let family = "neural-tier-xdg-only-7f3a";
    let temp = tempfile::tempdir().expect("temp dir");
    let biomeos_dir = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).expect("create biomeos dir");
    let expected = biomeos_dir.join(format!("neural-api-{family}.sock"));
    std::fs::write(&expected, "").expect("placeholder socket path");

    let bridge = NeuralBridge::discover_with_env(
        None,
        Some(family),
        &SocketResolveEnv {
            xdg_runtime_dir: Some(temp.path().to_str().expect("utf8 path").to_string()),
            tmpdir: None,
        },
    );
    assert!(bridge.is_some());
    assert_eq!(bridge.expect("bridge").socket_path(), expected.as_path());
}

#[test]
#[cfg(target_os = "linux")]
fn resolve_socket_tier_run_user() {
    let family = "neural-tier-run-user-9c2e";
    let uid = uid_from_runtime_dir();
    let run_biomeos = std::path::PathBuf::from(format!("/run/user/{uid}/biomeos"));
    std::fs::create_dir_all(&run_biomeos).expect("create /run/user/.../biomeos");
    let expected = run_biomeos.join(format!("neural-api-{family}.sock"));
    std::fs::write(&expected, "").expect("placeholder socket path");

    let empty_xdg = tempfile::tempdir().expect("empty xdg");
    let bridge = NeuralBridge::discover_with_env(
        None,
        Some(family),
        &SocketResolveEnv {
            xdg_runtime_dir: Some(empty_xdg.path().to_str().expect("utf8 path").to_string()),
            tmpdir: None,
        },
    );
    assert!(bridge.is_some());
    assert_eq!(bridge.expect("bridge").socket_path(), expected.as_path());
}

#[test]
fn resolve_socket_tier_platform_temp_dir() {
    let family = "neural-tier-tmpdir-b4d1";
    let tmp_root = tempfile::tempdir().expect("tmp root");
    let biomeos_dir = tmp_root.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).expect("create biomeos under TMPDIR");
    let expected = biomeos_dir.join(format!("neural-api-{family}.sock"));
    std::fs::write(&expected, "").expect("placeholder socket path");

    let empty_xdg = tempfile::tempdir().expect("empty xdg");
    let bridge = NeuralBridge::discover_with_env(
        None,
        Some(family),
        &SocketResolveEnv {
            xdg_runtime_dir: Some(empty_xdg.path().to_str().expect("utf8 path").to_string()),
            tmpdir: Some(tmp_root.path().to_str().expect("utf8 path").to_string()),
        },
    );
    assert!(bridge.is_some());
    assert_eq!(bridge.expect("bridge").socket_path(), expected.as_path());
}

#[test]
#[cfg(unix)]
fn send_request_round_trip_reads_json_line() {
    let dir = unix_isolated_tempdir("neural-roundtrip");
    let sock_path = dir.path().join("neural-roundtrip.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind mock socket");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut line = String::new();
        BufReader::new(&mut stream)
            .read_line(&mut line)
            .expect("read request");
        assert!(line.contains("ping.method"));

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": { "pong": true },
            "id": 1
        });
        let mut payload = serde_json::to_string(&response).expect("serialize");
        payload.push('\n');
        stream
            .write_all(payload.as_bytes())
            .expect("write response");
        stream.flush().expect("flush");
    });

    let bridge = NeuralBridge {
        socket_path: sock_path,
        timeout: Duration::from_secs(5),
    };
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "ping.method",
        "params": {},
        "id": 1
    });
    let parsed = bridge.send_request(&request).expect("send_request");
    assert_eq!(parsed["result"]["pong"], serde_json::json!(true));
    server.join().expect("server thread");
}

#[test]
#[cfg(unix)]
fn capability_call_success_via_mock_socket() {
    let dir = unix_isolated_tempdir("neural-cap");
    let sock_path = dir.path().join("neural-cap.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind mock socket");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut line = String::new();
        BufReader::new(&mut stream)
            .read_line(&mut line)
            .expect("read request");
        assert!(line.contains("capability.call"));

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": { "et0": 1.23 },
            "id": 1
        });
        let mut payload = serde_json::to_string(&response).expect("serialize");
        payload.push('\n');
        stream
            .write_all(payload.as_bytes())
            .expect("write response");
        stream.flush().expect("flush");
    });

    let bridge = NeuralBridge {
        socket_path: sock_path,
        timeout: Duration::from_secs(5),
    };
    let out = bridge
        .capability_call("ecology", "et0_pm", &serde_json::json!({}))
        .expect("capability_call");
    assert!((out.value["et0"].as_f64().unwrap() - 1.23).abs() < f64::EPSILON);
    server.join().expect("server thread");
}

#[test]
#[cfg(unix)]
fn health_check_success_via_mock_socket() {
    let dir = unix_isolated_tempdir("neural-health");
    let sock_path = dir.path().join("neural-health.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind mock socket");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut line = String::new();
        BufReader::new(&mut stream)
            .read_line(&mut line)
            .expect("read request");
        assert!(line.contains("lifecycle.status"));

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": { "alive": true },
            "id": 1
        });
        let mut payload = serde_json::to_string(&response).expect("serialize");
        payload.push('\n');
        stream
            .write_all(payload.as_bytes())
            .expect("write response");
        stream.flush().expect("flush");
    });

    let bridge = NeuralBridge {
        socket_path: sock_path,
        timeout: Duration::from_secs(5),
    };
    let value = bridge.health_check().expect("health_check");
    assert_eq!(value["alive"], serde_json::json!(true));
    server.join().expect("server thread");
}

#[test]
#[cfg(unix)]
fn discover_capability_success_via_mock_socket() {
    let dir = unix_isolated_tempdir("neural-discover");
    let sock_path = dir.path().join("neural-discover.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind mock socket");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut line = String::new();
        BufReader::new(&mut stream)
            .read_line(&mut line)
            .expect("read request");
        assert!(line.contains("capability.discover"));

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": { "ops": ["a", "b"] },
            "id": 1
        });
        let mut payload = serde_json::to_string(&response).expect("serialize");
        payload.push('\n');
        stream
            .write_all(payload.as_bytes())
            .expect("write response");
        stream.flush().expect("flush");
    });

    let bridge = NeuralBridge {
        socket_path: sock_path,
        timeout: Duration::from_secs(5),
    };
    let value = bridge
        .discover_capability("ecology")
        .expect("discover_capability");
    assert_eq!(value["ops"], serde_json::json!(["a", "b"]));
    server.join().expect("server thread");
}

#[test]
fn capability_call_fails_on_nonexistent_socket() {
    let bridge = NeuralBridge {
        socket_path: PathBuf::from("/nonexistent/path/neural-api-xyz.sock"),
        timeout: Duration::from_millis(100),
    };
    let result = bridge.capability_call("ecology", "et0_pm", &serde_json::json!({}));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, NeuralError::Connection(_)));
}

#[test]
fn discover_capability_fails_on_nonexistent_socket() {
    let bridge = NeuralBridge {
        socket_path: PathBuf::from("/nonexistent/path/neural-api-xyz.sock"),
        timeout: Duration::from_millis(100),
    };
    let result = bridge.discover_capability("ecology");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NeuralError::Connection(_)));
}

#[test]
fn health_check_fails_on_nonexistent_socket() {
    let bridge = NeuralBridge {
        socket_path: PathBuf::from("/nonexistent/path/neural-api-xyz.sock"),
        timeout: Duration::from_millis(100),
    };
    let result = bridge.health_check();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NeuralError::Connection(_)));
}

#[test]
fn uid_from_runtime_dir_returns_valid_uid_on_linux() {
    #[cfg(target_os = "linux")]
    {
        let uid = uid_from_runtime_dir();
        // On Linux, when running as a real user, uid should not be 65534 (nobody)
        // unless the test is actually running as nobody. In CI or normal dev,
        // we expect a real UID. 65534 indicates failure to read /proc/self/status.
        assert_ne!(
            uid, 65534,
            "uid_from_runtime_dir should return real UID on Linux, not nobody (65534)"
        );
    }
    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux, we just verify it returns something (no /proc)
        let uid = uid_from_runtime_dir();
        assert_eq!(uid, 65534, "non-Linux falls back to nobody");
    }
}
