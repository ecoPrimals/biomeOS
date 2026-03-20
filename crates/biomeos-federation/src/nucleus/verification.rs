// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Layers 2 & 3: Identity and Capability Verification
//!
//! Identity proof via challenge-response, capability verification via primal query.

use crate::FederationResult;
use crate::beardog_client::BearDogClient;
use crate::capability::{Capability, CapabilitySet};
use crate::discovery::{DiscoveredPrimal, PrimalEndpoint};
use crate::unix_socket_client::{JsonRpcRequest, UnixSocketClient};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Sentinel value for identity proofs where BearDog verification was unavailable
pub const UNVERIFIED_SIGNATURE: &str = "unverified";

/// Identity proof from BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProof {
    /// Node ID
    pub node_id: String,
    /// Family ID (extracted from BearDog lineage verification)
    pub family_id: Option<String>,
    /// Ed25519 signature (`UNVERIFIED_SIGNATURE` when BearDog unavailable)
    pub signature: String,
    /// Challenge that was signed
    pub challenge: String,
    /// Public key
    pub public_key: String,
    /// Timestamp
    pub timestamp: u64,
}

impl IdentityProof {
    /// Returns true if this proof has not been cryptographically verified
    pub fn is_unverified(&self) -> bool {
        self.signature == UNVERIFIED_SIGNATURE
    }
}

/// Primal capability from get_capabilities response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrimalCapabilityInfo {
    #[serde(rename = "type")]
    capability_type: String,
    methods: Vec<String>,
    version: String,
}

/// Get capabilities response from primal
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetCapabilitiesResponse {
    primal: String,
    version: String,
    family_id: Option<String>,
    node_id: String,
    protocols: Vec<String>,
    provided_capabilities: Vec<PrimalCapabilityInfo>,
}

/// Layer 2: Identity Verification via BearDog
#[expect(clippy::expect_used, reason = "system clock before UNIX epoch")]
pub(crate) async fn layer2_identity_verification(
    _beardog: &BearDogClient,
    primal: &DiscoveredPrimal,
) -> FederationResult<IdentityProof> {
    debug!("Layer 2: Identity Verification (BearDog)");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs();

    let socket_path = primal.endpoints.iter().find_map(|ep| {
        if let PrimalEndpoint::UnixSocket { path } = ep {
            Some(path.clone())
        } else {
            None
        }
    });

    if let Some(socket_path) = socket_path {
        let challenge = format!("nucleus-challenge-{}-{}", primal.name, now);

        let client = UnixSocketClient::new(socket_path);
        let request = JsonRpcRequest::new(
            "get_identity",
            serde_json::json!({
                "challenge": challenge
            }),
        );

        match client.call(request).await {
            Ok(response) => {
                let empty_json = serde_json::json!({});
                let result = response.result.as_ref().unwrap_or(&empty_json);
                let node_id = result["node_id"]
                    .as_str()
                    .unwrap_or(&primal.name)
                    .to_string();
                let family_id = result["family_id"]
                    .as_str()
                    .map(std::string::ToString::to_string);
                let signature = result["signature"]
                    .as_str()
                    .unwrap_or(UNVERIFIED_SIGNATURE)
                    .to_string();
                let public_key = result["public_key"].as_str().unwrap_or("none").to_string();

                Ok(IdentityProof {
                    node_id,
                    family_id,
                    signature,
                    challenge,
                    public_key,
                    timestamp: now,
                })
            }
            Err(e) => {
                debug!("get_identity failed: {}, using basic proof", e);
                Ok(IdentityProof {
                    node_id: primal.name.clone(),
                    family_id: None,
                    signature: UNVERIFIED_SIGNATURE.to_string(),
                    challenge,
                    public_key: "none".to_string(),
                    timestamp: now,
                })
            }
        }
    } else {
        Ok(IdentityProof {
            node_id: primal.name.clone(),
            family_id: None,
            signature: UNVERIFIED_SIGNATURE.to_string(),
            challenge: "no-socket".to_string(),
            public_key: "none".to_string(),
            timestamp: now,
        })
    }
}

/// Layer 3: Capability Verification (query primal)
pub(crate) async fn layer3_capability_verification(
    primal: &DiscoveredPrimal,
) -> FederationResult<CapabilitySet> {
    debug!("Layer 3: Capability Verification");

    let socket_path = primal.endpoints.iter().find_map(|ep| {
        if let PrimalEndpoint::UnixSocket { path } = ep {
            Some(path.clone())
        } else {
            None
        }
    });

    if let Some(socket_path) = socket_path {
        let client = UnixSocketClient::new(socket_path);
        let request = JsonRpcRequest::new("get_capabilities", serde_json::json!({}));

        match client.call(request).await {
            Ok(response) => {
                let result_value = response.result.unwrap_or_default();
                match serde_json::from_value::<GetCapabilitiesResponse>(result_value) {
                    Ok(cap_response) => {
                        debug!(
                            "Verified capabilities for {} (v{}): {} capabilities",
                            cap_response.primal,
                            cap_response.version,
                            cap_response.provided_capabilities.len()
                        );

                        let mut capabilities = CapabilitySet::new();
                        for cap_info in cap_response.provided_capabilities {
                            let cap: Capability =
                                cap_info.capability_type.parse().unwrap_or_else(|_| {
                                    Capability::Custom(cap_info.capability_type.clone())
                                });
                            capabilities.add(cap);
                        }

                        if capabilities.is_empty() {
                            warn!(
                                "Primal reported zero capabilities, using discovered capabilities"
                            );
                            Ok(primal.capabilities.clone())
                        } else {
                            Ok(capabilities)
                        }
                    }
                    Err(e) => {
                        debug!("Failed to parse capability response: {}", e);
                        Ok(primal.capabilities.clone())
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to query capabilities: {}, using discovered capabilities",
                    e
                );
                Ok(primal.capabilities.clone())
            }
        }
    } else {
        Ok(primal.capabilities.clone())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_proof_serde_roundtrip() {
        let proof = IdentityProof {
            node_id: "node-1".into(),
            family_id: Some("fam-1".into()),
            signature: "sig-abc".into(),
            challenge: "challenge-xyz".into(),
            public_key: "pk-123".into(),
            timestamp: 42,
        };
        let json = serde_json::to_string(&proof).expect("serialize");
        let restored: IdentityProof = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.node_id, "node-1");
        assert_eq!(restored.family_id, Some("fam-1".into()));
        assert_eq!(restored.timestamp, 42);
    }

    #[test]
    fn test_identity_proof_without_family() {
        let proof = IdentityProof {
            node_id: "solo".into(),
            family_id: None,
            signature: "s".into(),
            challenge: "c".into(),
            public_key: "pk".into(),
            timestamp: 0,
        };
        let json = serde_json::to_string(&proof).expect("serialize");
        assert!(json.contains("\"family_id\":null"));
    }

    #[test]
    fn test_identity_proof_clone() {
        let proof = IdentityProof {
            node_id: "n".into(),
            family_id: None,
            signature: "s".into(),
            challenge: "c".into(),
            public_key: "pk".into(),
            timestamp: 99,
        };
        let cloned = proof.clone();
        assert_eq!(cloned.node_id, proof.node_id);
        assert_eq!(cloned.timestamp, proof.timestamp);
    }

    #[test]
    fn test_primal_capability_info_serde() {
        let json = r#"{"type":"storage","methods":["put","get"],"version":"1.0"}"#;
        let info: PrimalCapabilityInfo = serde_json::from_str(json).expect("deserialize");
        assert_eq!(info.capability_type, "storage");
        assert_eq!(info.methods, vec!["put", "get"]);
    }

    #[test]
    fn test_get_capabilities_response_serde() {
        let json = r#"{
            "primal": "nestgate",
            "version": "2.0",
            "family_id": "fam-1",
            "node_id": "n1",
            "protocols": ["jsonrpc"],
            "provided_capabilities": [
                {"type": "storage", "methods": ["put"], "version": "1.0"}
            ]
        }"#;
        let resp: GetCapabilitiesResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.primal, "nestgate");
        assert_eq!(resp.provided_capabilities.len(), 1);
        assert_eq!(resp.family_id, Some("fam-1".into()));
    }

    #[test]
    fn test_identity_proof_is_unverified() {
        let verified = IdentityProof {
            node_id: "n".into(),
            family_id: None,
            signature: "sig-123".into(),
            challenge: "c".into(),
            public_key: "pk".into(),
            timestamp: 0,
        };
        assert!(!verified.is_unverified());

        let unverified = IdentityProof {
            node_id: "n".into(),
            family_id: None,
            signature: UNVERIFIED_SIGNATURE.into(),
            challenge: "c".into(),
            public_key: "pk".into(),
            timestamp: 0,
        };
        assert!(unverified.is_unverified());
    }

    #[test]
    fn test_identity_proof_debug() {
        let proof = IdentityProof {
            node_id: "debug".into(),
            family_id: Some("fam".into()),
            signature: "s".into(),
            challenge: "c".into(),
            public_key: "pk".into(),
            timestamp: 1,
        };
        let dbg = format!("{proof:?}");
        assert!(dbg.contains("debug"));
        assert!(dbg.contains("fam"));
    }

    use std::collections::HashMap;

    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    use crate::beardog_client::BearDogClient;
    use crate::capability::{Capability, CapabilitySet};
    use crate::discovery::{DiscoveredPrimal, PrimalEndpoint};

    fn test_primal_with_socket(path: std::path::PathBuf) -> DiscoveredPrimal {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::Discovery);
        DiscoveredPrimal {
            name: "test-primal".into(),
            primal_type: "test".into(),
            capabilities: caps,
            endpoints: vec![PrimalEndpoint::UnixSocket { path }],
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn layer2_identity_verification_no_socket_returns_unverified() {
        let primal = DiscoveredPrimal {
            name: "solo".into(),
            primal_type: "t".into(),
            capabilities: CapabilitySet::new(),
            endpoints: vec![PrimalEndpoint::Http {
                url: "http://127.0.0.1:9".into(),
            }],
            metadata: HashMap::new(),
        };
        let beardog =
            BearDogClient::with_endpoint("unix:///tmp/biomeos-unused-beardog-socket".to_string())
                .expect("endpoint");
        let proof = layer2_identity_verification(&beardog, &primal)
            .await
            .expect("layer2");
        assert_eq!(proof.node_id, "solo");
        assert!(proof.is_unverified());
        assert_eq!(proof.challenge, "no-socket");
    }

    #[tokio::test]
    async fn layer2_identity_verification_socket_success_parses_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("identity.sock");
        let listener = UnixListener::bind(&sock_path).expect("bind unix listener");
        let sock_path_clone = sock_path.clone();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut read_half)
                .read_line(&mut line)
                .await
                .expect("read line");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse request");
            let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "node_id": "rpc-node",
                    "family_id": "rpc-family",
                    "signature": "verified-sig",
                    "public_key": "rpc-pk"
                }
            });
            write_half
                .write_all(format!("{body}\n").as_bytes())
                .await
                .expect("write response");
        });

        let primal = test_primal_with_socket(sock_path_clone);
        let beardog =
            BearDogClient::with_endpoint("unix:///tmp/unused".to_string()).expect("endpoint");
        let proof = layer2_identity_verification(&beardog, &primal)
            .await
            .expect("layer2");

        server.await.expect("server task");
        assert_eq!(proof.node_id, "rpc-node");
        assert_eq!(proof.family_id.as_deref(), Some("rpc-family"));
        assert_eq!(proof.signature, "verified-sig");
        assert!(!proof.is_unverified());
    }

    #[tokio::test]
    async fn layer2_identity_verification_socket_error_yields_unverified_proof() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("dead.sock");
        std::fs::write(&sock_path, b"").expect("placeholder path");

        let primal = test_primal_with_socket(sock_path);
        let beardog =
            BearDogClient::with_endpoint("unix:///tmp/unused".to_string()).expect("endpoint");
        let proof = layer2_identity_verification(&beardog, &primal)
            .await
            .expect("layer2");
        assert_eq!(proof.node_id, "test-primal");
        assert!(proof.is_unverified());
        assert!(proof.challenge.contains("nucleus-challenge"));
    }

    #[tokio::test]
    async fn layer3_capability_verification_no_socket_returns_discovered() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::Storage);
        let primal = DiscoveredPrimal {
            name: "n".into(),
            primal_type: "t".into(),
            capabilities: caps.clone(),
            endpoints: vec![],
            metadata: HashMap::new(),
        };
        let got = layer3_capability_verification(&primal)
            .await
            .expect("layer3");
        assert!(got.has(&Capability::Storage));
    }

    #[tokio::test]
    async fn layer3_capability_verification_rpc_error_falls_back() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("dead-cap.sock");
        std::fs::write(&sock_path, b"").expect("file not socket listener");

        let mut caps = CapabilitySet::new();
        caps.add(Capability::Compute);
        let primal = test_primal_with_socket(sock_path);
        let primal = DiscoveredPrimal {
            capabilities: caps.clone(),
            ..primal
        };
        let got = layer3_capability_verification(&primal)
            .await
            .expect("layer3");
        assert!(got.has(&Capability::Compute));
    }

    #[tokio::test]
    async fn layer3_capability_verification_empty_capabilities_falls_back() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("cap.sock");
        let listener = UnixListener::bind(&sock_path).expect("bind");

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut read_half)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "primal": "p",
                    "version": "1",
                    "family_id": null,
                    "node_id": "n1",
                    "protocols": [],
                    "provided_capabilities": []
                }
            });
            write_half
                .write_all(format!("{body}\n").as_bytes())
                .await
                .expect("write");
        });

        let mut caps = CapabilitySet::new();
        caps.add(Capability::Voice);
        let primal = DiscoveredPrimal {
            name: "p".into(),
            primal_type: "t".into(),
            capabilities: caps.clone(),
            endpoints: vec![PrimalEndpoint::UnixSocket {
                path: sock_path.clone(),
            }],
            metadata: HashMap::new(),
        };

        let got = layer3_capability_verification(&primal)
            .await
            .expect("layer3");
        server.await.expect("server");
        assert!(got.has(&Capability::Voice));
    }

    #[tokio::test]
    async fn layer3_capability_verification_parses_custom_capability_type() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("cap2.sock");
        let listener = UnixListener::bind(&sock_path).expect("bind");

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut read_half)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "primal": "p",
                    "version": "1",
                    "family_id": null,
                    "node_id": "n1",
                    "protocols": [],
                    "provided_capabilities": [
                        {"type": "not_a_builtin_cap_xyz", "methods": [], "version": "1"}
                    ]
                }
            });
            write_half
                .write_all(format!("{body}\n").as_bytes())
                .await
                .expect("write");
        });

        let primal = test_primal_with_socket(sock_path.clone());
        let got = layer3_capability_verification(&primal)
            .await
            .expect("layer3");
        server.await.expect("server");
        assert!(got.has(&Capability::Custom("not_a_builtin_cap_xyz".into())));
    }

    #[tokio::test]
    async fn layer3_capability_verification_malformed_result_falls_back() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("cap3.sock");
        let listener = UnixListener::bind(&sock_path).expect("bind");

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut read_half)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": "not-an-object"
            });
            write_half
                .write_all(format!("{body}\n").as_bytes())
                .await
                .expect("write");
        });

        let mut caps = CapabilitySet::new();
        caps.add(Capability::Admin);
        let primal = DiscoveredPrimal {
            name: "p".into(),
            primal_type: "t".into(),
            capabilities: caps.clone(),
            endpoints: vec![PrimalEndpoint::UnixSocket { path: sock_path }],
            metadata: HashMap::new(),
        };

        let got = layer3_capability_verification(&primal)
            .await
            .expect("layer3");
        server.await.expect("server");
        assert!(got.has(&Capability::Admin));
    }
}
