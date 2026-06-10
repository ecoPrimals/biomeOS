// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use tokio::io::AsyncBufReadExt;

#[test]
fn test_json_rpc_request_deserialize() {
    let json = r#"{"jsonrpc":"2.0","method":"device.list","params":null,"id":1}"#;
    let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "device.list");
    assert_eq!(req.id, serde_json::json!(1));
}

#[test]
fn test_json_rpc_response_serialize() {
    let response = JsonRpcResponse {
        jsonrpc: JsonRpcVersion,
        result: Some(json!({"devices": []})),
        error: None,
        id: json!(1),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"result\""));
    assert!(json.contains("\"devices\""));
}

#[test]
fn test_json_rpc_error_serialize() {
    let response = JsonRpcResponse {
        jsonrpc: JsonRpcVersion,
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
        id: json!(1),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"error\""));
    assert!(json.contains("Method not found"));
    assert!(json.contains("-32601"));
}

#[test]
fn test_json_rpc_error_with_data_serialize() {
    let response = JsonRpcResponse {
        jsonrpc: JsonRpcVersion,
        result: None,
        error: Some(JsonRpcError {
            code: -32000,
            message: "App error".to_string(),
            data: Some(json!({"hint": "retry"})),
        }),
        id: json!(2),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"hint\""));
    assert!(json.contains("retry"));
}

#[test]
fn test_json_rpc_request_roundtrip_with_params_object() {
    let json = r#"{"jsonrpc":"2.0","method":"device.assign","params":{"device_id":"a","primal_id":"b"},"id":"rid"}"#;
    let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.method, "device.assign");
    let params = req.params.expect("params");
    assert_eq!(params["device_id"], "a");
    assert_eq!(params["primal_id"], "b");
    assert_eq!(req.id, json!("rid"));
}

#[tokio::test]
async fn test_handle_method_unknown_method() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "unknown_method".to_string(),
        params: None,
        id: json!(1),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert_eq!(err.code, -32601);
    assert!(err.message.contains("Method not found"));
}

#[tokio::test]
async fn test_handle_method_get_devices() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-get-devices.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "device.list".to_string(),
        params: None,
        id: json!(1),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_handle_method_get_primals_extended() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-primals.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "primal.list".to_string(),
        params: None,
        id: json!(2),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
    let result = response.result.unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_handle_method_get_niche_templates() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-templates.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "niche.list_templates".to_string(),
        params: None,
        id: json!(3),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
    let result = response.result.unwrap();
    let arr = result.as_array().unwrap();
    assert!(!arr.is_empty());
}

#[tokio::test]
async fn test_handle_method_assign_device_with_params() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-assign.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "device.assign".to_string(),
        params: Some(json!({
            "device_id": "gpu-0",
            "primal_id": "toadstool-1"
        })),
        id: json!(4),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert_eq!(err.code, -32603);
    assert!(err.message.contains("Internal error"));
}

#[tokio::test]
async fn test_handle_method_assign_device_without_params() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-assign-empty.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "device.assign".to_string(),
        params: None,
        id: json!(5),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_some());
}

#[tokio::test]
async fn test_handle_method_assign_device_empty_string_ids() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-assign-empty-ids.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "device.assign".to_string(),
        params: Some(json!({"device_id":"","primal_id":""})),
        id: json!(51),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_some());
}

#[tokio::test]
async fn test_handle_method_validate_niche_success() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-validate.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "niche.validate".to_string(),
        params: Some(json!({"template_id": "tower"})),
        id: json!(6),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
    let result = response.result.unwrap();
    assert!(result.get("valid").is_some());
}

#[tokio::test]
async fn test_handle_method_validate_niche_template_not_found() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-validate-nf.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "niche.validate".to_string(),
        params: Some(json!({"template_id": "nonexistent_template"})),
        id: json!(7),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("Template not found"));
}

#[tokio::test]
async fn test_handle_method_deploy_niche() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-deploy.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "niche.deploy".to_string(),
        params: Some(json!({"config": {"template": "tower"}})),
        id: json!(8),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert_eq!(err.code, -32603);
}

#[tokio::test]
async fn test_handle_method_deploy_niche_without_config() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-deploy-empty.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "niche.deploy".to_string(),
        params: Some(json!({})),
        id: json!(9),
    };
    let response = handle_method(request, &provider).await;
    assert!(response.error.is_some());
}

#[tokio::test]
async fn test_handle_method_result_success_structure() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-struct.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "device.list".to_string(),
        params: None,
        id: json!("string-id"),
    };
    let response = handle_method(request, &provider).await;
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.error.is_none());
    assert_eq!(response.id, json!("string-id"));
}

#[tokio::test]
async fn test_handle_method_result_error_structure() {
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-err-struct.sock",
    )));
    let request = JsonRpcRequest {
        jsonrpc: JsonRpcVersion,
        method: "bogus".to_string(),
        params: None,
        id: json!(null),
    };
    let response = handle_method(request, &provider).await;
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.id, json!(null));
}

#[tokio::test]
async fn test_handle_connection_valid_request_returns_response() {
    let (client_stream, server_stream) = tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-conn.sock",
    )));

    let provider_clone = provider.clone();
    let handle =
        tokio::spawn(async move { handle_connection(server_stream, provider_clone).await });

    let (reader, mut writer) = client_stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = r#"{"jsonrpc":"2.0","method":"device.list","params":null,"id":1}"#;
    writer
        .write_all((request.to_string() + "\n").as_bytes())
        .await
        .unwrap();
    writer.flush().await.unwrap();

    let mut response_line = String::new();
    reader.read_line(&mut response_line).await.unwrap();
    assert!(!response_line.is_empty());
    let response: JsonRpcResponse = serde_json::from_str(response_line.trim()).unwrap();
    assert!(response.error.is_none());
    assert!(response.result.is_some());

    drop(writer);
    let _ = handle.await;
}

#[tokio::test]
async fn test_handle_connection_invalid_json_continues_then_valid_works() {
    let (client_stream, server_stream) = tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-invalid.sock",
    )));

    let provider_clone = provider.clone();
    let handle =
        tokio::spawn(async move { handle_connection(server_stream, provider_clone).await });

    let (reader, mut writer) = client_stream.into_split();
    let mut reader = BufReader::new(reader);

    writer.write_all(b"{invalid json\n").await.unwrap();
    writer.flush().await.unwrap();

    let valid_request = r#"{"jsonrpc":"2.0","method":"unknown_method","params":null,"id":1}"#;
    writer
        .write_all((valid_request.to_string() + "\n").as_bytes())
        .await
        .unwrap();
    writer.flush().await.unwrap();

    let mut response_line = String::new();
    reader.read_line(&mut response_line).await.unwrap();
    let response: JsonRpcResponse = serde_json::from_str(response_line.trim()).unwrap();
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32601);

    drop(writer);
    let _ = handle.await;
}

#[tokio::test]
async fn test_handle_connection_closed_on_eof() {
    let (client_stream, server_stream) = tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
        "/tmp/test-device-mgmt-eof.sock",
    )));

    let provider_clone = provider.clone();
    let handle =
        tokio::spawn(async move { handle_connection(server_stream, provider_clone).await });

    drop(client_stream);

    let result = handle.await.unwrap();
    assert!(result.is_ok());
}
