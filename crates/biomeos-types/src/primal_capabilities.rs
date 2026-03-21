// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Primal-specific capability routing types.
//!
//! Defines the JSON-RPC method contracts that biomeOS uses to interact with
//! specific primals. Each type corresponds to a capability domain discovered
//! at runtime through Songbird or manifest-based discovery.
//!
//! These types are *routing contracts*, not implementations. biomeOS discovers
//! which primal provides each capability at runtime and routes requests
//! accordingly — no hardcoded primal references.

use serde::{Deserialize, Serialize};

// =========================================================================
// Relay Authorization (absorbed from BearDog `relay.authorize`)
// =========================================================================

/// Request payload for lineage-gated relay authorization.
///
/// biomeOS routes `relay.authorize` to the security provider (BearDog) at
/// runtime. The relay coordinator (Songbird) uses this to verify that a
/// peer is authorized to use relay resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayAuthorizeRequest {
    /// Peer's lineage certificate (base64-encoded).
    pub lineage_cert: String,
    /// Relay session being requested.
    pub relay_session_id: String,
    /// Maximum bandwidth allocation in bytes/sec (0 = unlimited).
    #[serde(default)]
    pub max_bandwidth_bps: u64,
    /// Requested relay duration in seconds.
    #[serde(default)]
    pub duration_secs: u64,
}

/// Response from relay authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayAuthorizeResponse {
    /// Whether the relay is authorized.
    pub authorized: bool,
    /// Authorization token to present to the relay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Human-readable reason if denied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Expiry timestamp (Unix epoch seconds).
    #[serde(default)]
    pub expires_at: u64,
}

// =========================================================================
// Compute Dispatch (absorbed from ToadStool `compute.dispatch.*`)
// =========================================================================

/// Request to submit a compute job.
///
/// biomeOS routes `compute.dispatch.submit` to the compute provider
/// (ToadStool) at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeSubmitRequest {
    /// Shader or compute kernel identifier.
    pub kernel_id: String,
    /// Input data for the computation (JSON-serialized).
    pub input: serde_json::Value,
    /// Priority (0 = normal, higher = more urgent).
    #[serde(default)]
    pub priority: u32,
    /// Maximum execution time in milliseconds.
    #[serde(default = "default_compute_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_compute_timeout_ms() -> u64 {
    30_000
}

/// Response from a compute dispatch submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeSubmitResponse {
    /// Job identifier for status polling.
    pub job_id: String,
    /// Current status.
    pub status: ComputeJobStatus,
    /// Estimated completion time in milliseconds (if queued).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_ms: Option<u64>,
}

/// Status of a compute job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputeJobStatus {
    /// Queued for execution.
    Queued,
    /// Currently running.
    Running,
    /// Completed successfully.
    Completed,
    /// Failed with an error.
    Failed,
    /// Cancelled by the caller.
    Cancelled,
}

/// Request to check compute job status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeStatusRequest {
    /// Job identifier.
    pub job_id: String,
}

/// Request to cancel a compute job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCancelRequest {
    /// Job identifier.
    pub job_id: String,
}

// =========================================================================
// Model Cache (absorbed from NestGate `model.*`)
// =========================================================================

/// Request to register a model in the cache.
///
/// biomeOS routes `model.register` to the storage provider (NestGate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegisterRequest {
    /// Model identifier (e.g. "llama-3-8b", "whisper-large-v3").
    pub model_id: String,
    /// Storage path or URI.
    pub path: String,
    /// Model size in bytes.
    #[serde(default)]
    pub size_bytes: u64,
    /// Model metadata (format, quantization, etc.).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Request to locate a model in the cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLocateRequest {
    /// Model identifier.
    pub model_id: String,
}

/// Response from model location query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLocateResponse {
    /// Whether the model was found.
    pub found: bool,
    /// Storage path (if found).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Model size in bytes (if found).
    #[serde(default)]
    pub size_bytes: u64,
    /// Metadata (if found).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

// =========================================================================
// Primal Lifecycle Traits (absorbed from sourDough)
// =========================================================================

/// Standard primal lifecycle phases.
///
/// Absorbed from sourDough's `PrimalLifecycle` trait. biomeOS uses these
/// to validate primal compliance and orchestrate startup/shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalLifecyclePhase {
    /// Primal is initializing (loading config, connecting to deps).
    Initializing,
    /// Primal is starting up (binding sockets, registering capabilities).
    Starting,
    /// Primal is fully operational.
    Running,
    /// Primal is shutting down gracefully.
    ShuttingDown,
    /// Primal has stopped.
    Stopped,
    /// Primal is in a degraded state (some capabilities unavailable).
    Degraded,
}

/// Primal identity metadata.
///
/// Returned by `identity.get` — the standard self-description method
/// that every compliant primal must implement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalIdentityResponse {
    /// Lowercase primal identifier (e.g. "beardog").
    pub id: String,
    /// Display name (e.g. "BearDog").
    pub display_name: String,
    /// Semantic version.
    pub version: String,
    /// Capability domains this primal provides.
    pub capabilities: Vec<String>,
    /// Current lifecycle phase.
    pub phase: PrimalLifecyclePhase,
    /// Family ID this primal belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,
}

/// Primal configuration validation request.
///
/// Used by `biomeos validate primal` to check sourDough compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalComplianceCheck {
    /// UniBin: single binary with subcommands.
    pub unibin_compliant: bool,
    /// ecoBin: pure Rust, no C dependencies.
    pub ecobin_compliant: bool,
    /// Implements `health.liveness` and `health.readiness`.
    pub health_probes: bool,
    /// Implements `identity.get`.
    pub identity_method: bool,
    /// Implements `capability.list`.
    pub capability_list: bool,
    /// Uses semantic method naming (`domain.verb[.variant]`).
    pub semantic_naming: bool,
    /// AGPL-3.0-only license.
    pub agpl_license: bool,
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_authorize_roundtrip() {
        let req = RelayAuthorizeRequest {
            lineage_cert: "base64cert".to_owned(),
            relay_session_id: "sess-1".to_owned(),
            max_bandwidth_bps: 1_000_000,
            duration_secs: 3600,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: RelayAuthorizeRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.relay_session_id, "sess-1");
    }

    #[test]
    fn compute_submit_roundtrip() {
        let req = ComputeSubmitRequest {
            kernel_id: "sha256-batch".to_owned(),
            input: serde_json::json!({"data": [1, 2, 3]}),
            priority: 5,
            timeout_ms: 10_000,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: ComputeSubmitRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.priority, 5);
    }

    #[test]
    fn compute_job_status_variants() {
        let statuses = [
            ComputeJobStatus::Queued,
            ComputeJobStatus::Running,
            ComputeJobStatus::Completed,
            ComputeJobStatus::Failed,
            ComputeJobStatus::Cancelled,
        ];
        for status in &statuses {
            let json = serde_json::to_string(status).expect("serialize");
            let parsed: ComputeJobStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&parsed, status);
        }
    }

    #[test]
    fn model_locate_response_found() {
        let resp = ModelLocateResponse {
            found: true,
            path: Some("/models/llama-3-8b".to_owned()),
            size_bytes: 8_000_000_000,
            metadata: serde_json::json!({"quantization": "Q4_K_M"}),
        };
        assert!(resp.found);
        assert_eq!(resp.path.as_deref(), Some("/models/llama-3-8b"));
    }

    #[test]
    fn primal_identity_roundtrip() {
        let identity = PrimalIdentityResponse {
            id: "beardog".to_owned(),
            display_name: "BearDog".to_owned(),
            version: "0.9.0".to_owned(),
            capabilities: vec!["crypto".to_owned(), "security".to_owned()],
            phase: PrimalLifecyclePhase::Running,
            family_id: Some("default".to_owned()),
        };
        let json = serde_json::to_string(&identity).expect("serialize");
        let parsed: PrimalIdentityResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.display_name, "BearDog");
        assert_eq!(parsed.phase, PrimalLifecyclePhase::Running);
    }

    #[test]
    fn compliance_check_all_true() {
        let check = PrimalComplianceCheck {
            unibin_compliant: true,
            ecobin_compliant: true,
            health_probes: true,
            identity_method: true,
            capability_list: true,
            semantic_naming: true,
            agpl_license: true,
        };
        assert!(check.unibin_compliant);
        assert!(check.ecobin_compliant);
    }

    #[test]
    fn lifecycle_phase_serde() {
        let phases = [
            PrimalLifecyclePhase::Initializing,
            PrimalLifecyclePhase::Starting,
            PrimalLifecyclePhase::Running,
            PrimalLifecyclePhase::ShuttingDown,
            PrimalLifecyclePhase::Stopped,
            PrimalLifecyclePhase::Degraded,
        ];
        for phase in &phases {
            let json = serde_json::to_string(phase).expect("serialize");
            let parsed: PrimalLifecyclePhase = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&parsed, phase);
        }
    }
}
