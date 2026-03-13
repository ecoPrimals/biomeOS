// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Configuration and types for protocol escalation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::living_graph::ProtocolMode;

/// Configuration for protocol escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Minimum requests before considering escalation
    #[serde(default = "default_min_requests")]
    pub min_requests: u64,

    /// Latency threshold to trigger escalation (μs)
    #[serde(default = "default_latency_threshold")]
    pub latency_threshold_us: u64,

    /// Stable health duration before auto-escalate
    #[serde(default = "default_stable_health_duration")]
    pub stable_health_duration_secs: u64,

    /// tarpc failures before fallback
    #[serde(default = "default_tarpc_failure_threshold")]
    pub tarpc_failure_threshold: u32,

    /// Time between auto-escalation checks
    #[serde(default = "default_check_interval")]
    pub check_interval_secs: u64,

    /// Cooldown after failed escalation attempt
    #[serde(default = "default_escalation_cooldown")]
    pub escalation_cooldown_secs: u64,

    /// Enable auto-escalation
    #[serde(default = "default_auto_escalate")]
    pub auto_escalate: bool,
}

/// Default minimum requests before considering escalation.
pub(crate) fn default_min_requests() -> u64 {
    100
}
/// Default latency threshold (μs) to trigger escalation.
pub(crate) fn default_latency_threshold() -> u64 {
    500
}
/// Default stable health duration (secs) before auto-escalate.
pub(crate) fn default_stable_health_duration() -> u64 {
    30
}
/// Default tarpc failures before fallback to JSON-RPC.
pub(crate) fn default_tarpc_failure_threshold() -> u32 {
    3
}
/// Default interval (secs) between auto-escalation checks.
pub(crate) fn default_check_interval() -> u64 {
    10
}
/// Default cooldown (secs) after failed escalation attempt.
pub(crate) fn default_escalation_cooldown() -> u64 {
    60
}
/// Default for enabling auto-escalation.
pub(crate) fn default_auto_escalate() -> bool {
    true
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            min_requests: default_min_requests(),
            latency_threshold_us: default_latency_threshold(),
            stable_health_duration_secs: default_stable_health_duration(),
            tarpc_failure_threshold: default_tarpc_failure_threshold(),
            check_interval_secs: default_check_interval(),
            escalation_cooldown_secs: default_escalation_cooldown(),
            auto_escalate: default_auto_escalate(),
        }
    }
}

/// Result of an escalation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationResult {
    /// Source primal name
    pub from: String,
    /// Target primal name
    pub to: String,
    /// Protocol mode before escalation
    pub previous_mode: ProtocolMode,
    /// Protocol mode after escalation attempt
    pub current_mode: ProtocolMode,
    /// Path to tarpc socket if available
    pub tarpc_socket: Option<PathBuf>,
    /// Whether escalation succeeded
    pub success: bool,
    /// Human-readable status or error message
    pub message: String,
}

/// tarpc endpoint information from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcEndpoint {
    /// Whether the tarpc endpoint is available
    pub available: bool,
    /// Path to the tarpc Unix socket if available
    pub socket: Option<PathBuf>,
    /// List of service names exposed over tarpc
    pub services: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_escalation_config_defaults() {
        let config = EscalationConfig::default();
        assert_eq!(config.min_requests, 100);
        assert_eq!(config.latency_threshold_us, 500);
        assert!(config.auto_escalate);
    }

    #[tokio::test]
    async fn test_escalation_result_serialization() {
        let result = EscalationResult {
            from: "songbird".to_string(),
            to: "beardog".to_string(),
            previous_mode: ProtocolMode::JsonRpc,
            current_mode: ProtocolMode::Tarpc,
            tarpc_socket: Some(PathBuf::from("/tmp/beardog-tarpc.sock")),
            success: true,
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&result).expect("serialize escalation result");
        assert!(json.contains("songbird"));
        assert!(json.contains("tarpc"));

        let parsed: EscalationResult =
            serde_json::from_str(&json).expect("parse escalation result");
        assert_eq!(parsed.from, "songbird");
        assert_eq!(parsed.to, "beardog");
        assert!(parsed.success);
    }

    #[test]
    fn test_escalation_config_serialization() {
        let config = EscalationConfig {
            min_requests: 200,
            latency_threshold_us: 1000,
            stable_health_duration_secs: 60,
            tarpc_failure_threshold: 5,
            check_interval_secs: 20,
            escalation_cooldown_secs: 120,
            auto_escalate: false,
        };

        let json = serde_json::to_string(&config).expect("serialize config");
        let parsed: EscalationConfig = serde_json::from_str(&json).expect("parse config");
        assert_eq!(parsed.min_requests, 200);
        assert_eq!(parsed.latency_threshold_us, 1000);
        assert!(!parsed.auto_escalate);
        assert_eq!(parsed.tarpc_failure_threshold, 5);
    }

    #[test]
    fn test_tarpc_endpoint_serialization() {
        let endpoint = TarpcEndpoint {
            available: true,
            socket: Some(PathBuf::from("/tmp/beardog-tarpc.sock")),
            services: vec!["health".to_string(), "deploy".to_string()],
        };

        let json = serde_json::to_string(&endpoint).expect("serialize endpoint");
        let parsed: TarpcEndpoint = serde_json::from_str(&json).expect("parse endpoint");
        assert!(parsed.available);
        assert_eq!(parsed.services.len(), 2);
    }

    #[test]
    fn test_tarpc_endpoint_unavailable() {
        let endpoint = TarpcEndpoint {
            available: false,
            socket: None,
            services: vec![],
        };

        let json = serde_json::to_string(&endpoint).expect("serialize");
        let parsed: TarpcEndpoint = serde_json::from_str(&json).expect("parse");
        assert!(!parsed.available);
        assert!(parsed.socket.is_none());
        assert!(parsed.services.is_empty());
    }

    #[tokio::test]
    async fn test_escalation_result_failed() {
        let result = EscalationResult {
            from: "a".to_string(),
            to: "b".to_string(),
            previous_mode: ProtocolMode::JsonRpc,
            current_mode: ProtocolMode::JsonRpc, // stayed the same
            tarpc_socket: None,
            success: false,
            message: "Target does not support tarpc".to_string(),
        };

        assert!(!result.success);
        assert!(result.tarpc_socket.is_none());
        assert_eq!(result.previous_mode, result.current_mode);
    }

    #[test]
    fn test_escalation_config_default_fn_values() {
        assert_eq!(default_min_requests(), 100);
        assert_eq!(default_latency_threshold(), 500);
        assert_eq!(default_stable_health_duration(), 30);
        assert_eq!(default_tarpc_failure_threshold(), 3);
        assert_eq!(default_check_interval(), 10);
        assert_eq!(default_escalation_cooldown(), 60);
        assert!(default_auto_escalate());
    }

    #[test]
    fn test_config_deserialization_empty_json_uses_defaults() {
        let config: EscalationConfig = serde_json::from_str("{}").expect("parse empty json");
        assert_eq!(config.min_requests, 100);
        assert_eq!(config.latency_threshold_us, 500);
        assert_eq!(config.stable_health_duration_secs, 30);
        assert_eq!(config.tarpc_failure_threshold, 3);
        assert_eq!(config.check_interval_secs, 10);
        assert_eq!(config.escalation_cooldown_secs, 60);
        assert!(config.auto_escalate);
    }

    #[test]
    fn test_config_deserialization_partial_json() {
        let json = r#"{"min_requests": 500, "auto_escalate": false}"#;
        let config: EscalationConfig = serde_json::from_str(json).expect("parse partial json");
        assert_eq!(config.min_requests, 500);
        assert!(!config.auto_escalate);
        assert_eq!(config.latency_threshold_us, 500);
        assert_eq!(config.stable_health_duration_secs, 30);
        assert_eq!(config.tarpc_failure_threshold, 3);
        assert_eq!(config.check_interval_secs, 10);
        assert_eq!(config.escalation_cooldown_secs, 60);
    }

    #[test]
    fn test_config_clone() {
        let config = EscalationConfig {
            min_requests: 42,
            latency_threshold_us: 999,
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.min_requests, 42);
        assert_eq!(cloned.latency_threshold_us, 999);
        assert_eq!(cloned.check_interval_secs, config.check_interval_secs);
    }

    #[test]
    fn test_config_debug() {
        let config = EscalationConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("min_requests"));
        assert!(debug_str.contains("auto_escalate"));
    }

    #[test]
    fn test_escalation_result_degraded_mode() {
        let result = EscalationResult {
            from: "songbird".to_string(),
            to: "beardog".to_string(),
            previous_mode: ProtocolMode::Tarpc,
            current_mode: ProtocolMode::Degraded,
            tarpc_socket: None,
            success: true,
            message: "Fell back due to tarpc failure".to_string(),
        };
        assert!(result.success);
        assert_eq!(result.current_mode, ProtocolMode::Degraded);
        assert_eq!(result.previous_mode, ProtocolMode::Tarpc);
    }

    #[test]
    fn test_escalation_result_clone_and_debug() {
        let result = EscalationResult {
            from: "a".to_string(),
            to: "b".to_string(),
            previous_mode: ProtocolMode::JsonRpc,
            current_mode: ProtocolMode::Tarpc,
            tarpc_socket: Some(PathBuf::from("/tmp/test.sock")),
            success: true,
            message: "ok".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.from, result.from);
        assert_eq!(cloned.tarpc_socket, result.tarpc_socket);

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("EscalationResult"));
    }

    #[test]
    fn test_tarpc_endpoint_with_many_services() {
        let endpoint = TarpcEndpoint {
            available: true,
            socket: Some(PathBuf::from("/run/user/1000/biomeos/beardog.sock")),
            services: vec![
                "health".to_string(),
                "deploy".to_string(),
                "crypto.encrypt".to_string(),
                "crypto.decrypt".to_string(),
                "birdsong.verify".to_string(),
            ],
        };
        assert_eq!(endpoint.services.len(), 5);
        assert!(endpoint.services.contains(&"crypto.encrypt".to_string()));

        let json = serde_json::to_string(&endpoint).unwrap();
        let parsed: TarpcEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.services.len(), 5);
        assert_eq!(parsed.socket, endpoint.socket);
    }

    #[test]
    fn test_tarpc_endpoint_debug_and_clone() {
        let endpoint = TarpcEndpoint {
            available: false,
            socket: None,
            services: vec![],
        };
        let cloned = endpoint.clone();
        assert_eq!(cloned.available, endpoint.available);
        let debug_str = format!("{:?}", endpoint);
        assert!(debug_str.contains("TarpcEndpoint"));
    }

    #[test]
    fn test_escalation_result_roundtrip_all_modes() {
        for mode in [
            ProtocolMode::JsonRpc,
            ProtocolMode::Tarpc,
            ProtocolMode::Hybrid,
            ProtocolMode::Degraded,
        ] {
            let result = EscalationResult {
                from: "a".to_string(),
                to: "b".to_string(),
                previous_mode: ProtocolMode::JsonRpc,
                current_mode: mode,
                tarpc_socket: None,
                success: true,
                message: format!("mode: {:?}", mode),
            };
            let json = serde_json::to_string(&result).unwrap();
            let parsed: EscalationResult = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.current_mode, mode);
        }
    }
}
