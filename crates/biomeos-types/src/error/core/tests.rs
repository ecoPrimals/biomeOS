// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::{BiomeError, ValidationError};
use crate::error::ai_context::AIErrorContext;
use crate::error::operations::{
    DataOperation, NetworkOperation, ResourceOperation, SecurityViolationType,
};
use std::error::Error;

fn default_ai_context() -> Box<AIErrorContext> {
    Box::new(AIErrorContext::default())
}

#[test]
fn configuration_variant_can_be_constructed() {
    let _err = BiomeError::Configuration {
        message: "bad config".to_string(),
        key: Some("db.url".to_string()),
        config_path: Some("/etc/config.yaml".to_string()),
        ai_context: default_ai_context(),
    };
}

#[test]
fn invalid_input_variant_can_be_constructed() {
    let _err = BiomeError::InvalidInput {
        message: "invalid value".to_string(),
        field: Some("email".to_string()),
        expected: Some("valid email format".to_string()),
        actual: Some("not-an-email".to_string()),
        ai_context: default_ai_context(),
    };
}

#[test]
fn discovery_variant_can_be_constructed() {
    let _err = BiomeError::Discovery {
        message: "service not found".to_string(),
        endpoint: Some("http://localhost:8080".to_string()),
        status_code: Some(404),
        discovery_method: Some("http".to_string()),
        ai_context: default_ai_context(),
    };
}

#[test]
fn network_variant_can_be_constructed() {
    let _err = BiomeError::Network {
        message: "connection refused".to_string(),
        endpoint: Some("tcp://localhost:5432".to_string()),
        status_code: Some(503),
        timeout_ms: Some(5000),
        operation: Some(NetworkOperation::Connect),
        ai_context: default_ai_context(),
    };
}

#[test]
fn security_variant_can_be_constructed() {
    let _err = BiomeError::Security {
        message: "access denied".to_string(),
        context: Some("api".to_string()),
        auth_method: Some("bearer".to_string()),
        violation_type: Some(SecurityViolationType::AccessDenied),
        ai_context: default_ai_context(),
    };
}

#[test]
fn resource_variant_can_be_constructed() {
    let _err = BiomeError::Resource {
        message: "memory exhausted".to_string(),
        resource_type: Some("memory".to_string()),
        requested: Some("2GB".to_string()),
        available: Some("512MB".to_string()),
        operation: Some(ResourceOperation::Allocate),
        ai_context: default_ai_context(),
    };
}

#[test]
fn integration_variant_can_be_constructed() {
    let _err = BiomeError::Integration {
        message: "component failed".to_string(),
        component: Some("songbird".to_string()),
        integration_type: Some("rpc".to_string()),
        ai_context: default_ai_context(),
    };
}

#[test]
fn internal_variant_can_be_constructed() {
    let _err = BiomeError::Internal {
        message: "unexpected state".to_string(),
        error_code: Some("ERR_001".to_string()),
        stack_trace: Some("at main.rs:42".to_string()),
        ai_context: default_ai_context(),
    };
}

#[test]
fn timeout_variant_can_be_constructed() {
    let _err = BiomeError::Timeout {
        message: "request timed out".to_string(),
        timeout_ms: 30000,
        operation: Some("health_check".to_string()),
        ai_context: default_ai_context(),
    };
}

#[test]
fn authorization_variant_can_be_constructed() {
    let _err = BiomeError::Authorization {
        message: "permission denied".to_string(),
        required_permission: Some("admin".to_string()),
        subject: Some("user-123".to_string()),
        ai_context: default_ai_context(),
    };
}

#[test]
fn validation_variant_can_be_constructed() {
    let _err = BiomeError::Validation {
        message: "validation failed".to_string(),
        field: Some("name".to_string()),
        rule: Some("required".to_string()),
        errors: vec![ValidationError {
            field: "name".to_string(),
            message: "required".to_string(),
            code: "required".to_string(),
            rejected_value: None,
        }],
        ai_context: default_ai_context(),
    };
}

#[test]
fn external_service_variant_can_be_constructed() {
    let _err = BiomeError::ExternalService {
        message: "service unavailable".to_string(),
        service: Some("api.example.com".to_string()),
        endpoint: Some("https://api.example.com".to_string()),
        status_code: Some(503),
        ai_context: default_ai_context(),
    };
}

#[test]
fn data_variant_can_be_constructed() {
    let _err = BiomeError::Data {
        message: "data corruption".to_string(),
        data_type: Some("user".to_string()),
        data_id: Some("user-42".to_string()),
        operation: Some(DataOperation::Read),
        ai_context: default_ai_context(),
    };
}

#[test]
fn unknown_variant_can_be_constructed() {
    let _err = BiomeError::Unknown {
        message: "something went wrong".to_string(),
        ai_context: default_ai_context(),
    };
}

#[test]
fn display_configuration_shows_meaningful_message() {
    let err = BiomeError::Configuration {
        message: "invalid database URL".to_string(),
        key: None,
        config_path: None,
        ai_context: default_ai_context(),
    };
    let s = err.to_string();
    assert!(s.contains("Configuration error"));
    assert!(s.contains("invalid database URL"));
}

#[test]
fn display_network_shows_meaningful_message() {
    let err = BiomeError::Network {
        message: "connection refused".to_string(),
        endpoint: None,
        status_code: None,
        timeout_ms: None,
        operation: None,
        ai_context: default_ai_context(),
    };
    let s = err.to_string();
    assert!(s.contains("Network error"));
    assert!(s.contains("connection refused"));
}

#[test]
fn display_validation_shows_meaningful_message() {
    let err = BiomeError::Validation {
        message: "invalid input".to_string(),
        field: None,
        rule: None,
        errors: vec![],
        ai_context: default_ai_context(),
    };
    let s = err.to_string();
    assert!(s.contains("Validation error"));
    assert!(s.contains("invalid input"));
}

#[test]
fn display_unknown_shows_meaningful_message() {
    let err = BiomeError::Unknown {
        message: "unexpected failure".to_string(),
        ai_context: default_ai_context(),
    };
    let s = err.to_string();
    assert!(s.contains("Unknown error"));
    assert!(s.contains("unexpected failure"));
}

#[test]
fn error_trait_source_returns_none() {
    let err = BiomeError::Internal {
        message: "internal".to_string(),
        error_code: None,
        stack_trace: None,
        ai_context: default_ai_context(),
    };
    assert!(err.source().is_none());
}

#[test]
fn error_trait_implemented() {
    fn assert_error<E: Error>() {}
    assert_error::<BiomeError>();
}

#[test]
fn from_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: BiomeError = io_err.into();
    assert!(matches!(err, BiomeError::Internal { .. }));
    assert!(err.to_string().contains("IO error"));
}

#[test]
fn from_serde_json_error_conversion() {
    let json_str = "{ invalid }";
    let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let err: BiomeError = json_err.into();
    assert!(matches!(err, BiomeError::Validation { .. }));
    assert!(err.to_string().contains("JSON"));
}

#[test]
fn from_uuid_error_conversion() {
    let uuid_str = "not-a-uuid";
    let uuid_err: Result<uuid::Uuid, _> = uuid_str.parse();
    let err: BiomeError = uuid_err.unwrap_err().into();
    assert!(matches!(err, BiomeError::Validation { .. }));
    assert!(err.to_string().contains("UUID"));
}

#[test]
fn error_matching_configuration() {
    let err = BiomeError::Configuration {
        message: "msg".to_string(),
        key: Some("k".to_string()),
        config_path: None,
        ai_context: default_ai_context(),
    };
    match &err {
        BiomeError::Configuration { message, key, .. } => {
            assert_eq!(message, "msg");
            assert_eq!(key.as_deref(), Some("k"));
        }
        _ => panic!("expected Configuration variant"),
    }
}

#[test]
fn error_matching_network() {
    let err = BiomeError::Network {
        message: "timeout".to_string(),
        endpoint: Some("http://x".to_string()),
        status_code: Some(504),
        timeout_ms: Some(5000),
        operation: Some(NetworkOperation::Connect),
        ai_context: default_ai_context(),
    };
    match &err {
        BiomeError::Network {
            message,
            status_code,
            operation,
            ..
        } => {
            assert_eq!(message, "timeout");
            assert_eq!(*status_code, Some(504));
            assert!(operation.is_some());
            assert!(matches!(
                operation.as_ref(),
                Some(NetworkOperation::Connect)
            ));
        }
        _ => panic!("expected Network variant"),
    }
}

#[test]
fn error_matching_validation_with_errors() {
    let validation_errors = vec![ValidationError {
        field: "x".to_string(),
        message: "bad".to_string(),
        code: "E001".to_string(),
        rejected_value: None,
    }];
    let err = BiomeError::Validation {
        message: "failed".to_string(),
        field: Some("x".to_string()),
        rule: Some("required".to_string()),
        errors: validation_errors,
        ai_context: default_ai_context(),
    };
    match &err {
        BiomeError::Validation {
            message,
            field,
            errors,
            ..
        } => {
            assert_eq!(message, "failed");
            assert_eq!(field.as_deref(), Some("x"));
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].field, "x");
            assert_eq!(errors[0].code, "E001");
        }
        _ => panic!("expected Validation variant"),
    }
}

#[test]
fn biome_error_clone() {
    let err = BiomeError::Unknown {
        message: "test".to_string(),
        ai_context: default_ai_context(),
    };
    let cloned = err.clone();
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn biome_error_debug() {
    let err = BiomeError::Internal {
        message: "debug".to_string(),
        error_code: None,
        stack_trace: None,
        ai_context: default_ai_context(),
    };
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("Internal"));
    assert!(debug_str.contains("debug"));
}
