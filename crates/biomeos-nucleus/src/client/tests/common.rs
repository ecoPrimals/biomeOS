// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::coordinator::NucleusClient;
use crate::Endpoint;
use crate::EndpointType;
use crate::Error;
use crate::Registry;
use crate::Result;
use crate::capability::{CapabilityInfo, CapabilityLayer, CapabilityVerification};
use crate::discovery::{DiscoveredPrimal, DiscoveryRequest, PhysicalDiscovery};
use crate::identity::{IdentityLayer, IdentityProof, IdentityVerification};
use crate::trust::{TrustEvaluation, TrustLayer, TrustLevel};
use std::sync::Arc;

pub(super) fn sample_endpoint() -> Endpoint {
    Endpoint {
        endpoint_type: EndpointType::UnixSocket,
        address: "/tmp/mock-primal.sock".to_string(),
    }
}

pub(super) fn sample_proof(primal: &str) -> IdentityProof {
    IdentityProof {
        primal_name: primal.to_string(),
        node_id: "node-1".to_string(),
        family_id: "fam-1".to_string(),
        version: "1.0.0".to_string(),
        process_id: 1,
        socket_path: "/tmp/mock-primal.sock".to_string(),
        owner_uid: 1000,
        owner_gid: 1000,
        started_at: "now".to_string(),
        challenge: "c".to_string(),
        signature: "sig".to_string(),
    }
}

pub(super) fn sample_discovered(primal: &str, endpoints: Vec<Endpoint>) -> DiscoveredPrimal {
    DiscoveredPrimal {
        primal: primal.to_string(),
        node_id: "node-1".to_string(),
        family_id: "fam-1".to_string(),
        capabilities: vec!["encryption".to_string()],
        endpoints,
        signature: "sig".to_string(),
        timestamp: "t".to_string(),
    }
}

pub(super) struct MockPhysical {
    pub out: Vec<DiscoveredPrimal>,
}

impl PhysicalDiscovery for MockPhysical {
    async fn discover_by_capability(
        &self,
        _request: &DiscoveryRequest,
    ) -> Result<Vec<DiscoveredPrimal>> {
        Ok(self.out.clone())
    }

    async fn discover_by_family(&self, _family_id: &str) -> Result<Vec<DiscoveredPrimal>> {
        Ok(vec![])
    }

    async fn announce(&self, _primal_info: &DiscoveredPrimal) -> Result<()> {
        Ok(())
    }
}

pub(super) struct MockIdentity {
    pub ok: bool,
    pub proof: IdentityProof,
}

impl IdentityLayer for MockIdentity {
    async fn request_proof(&self, _endpoint: &str, _challenge: &str) -> Result<IdentityProof> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_proof(&self, _proof: &IdentityProof) -> Result<IdentityVerification> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_identity(
        &self,
        _discovered: &DiscoveredPrimal,
    ) -> Result<IdentityVerification> {
        if self.ok {
            Ok(IdentityVerification {
                verified: true,
                proof: self.proof.clone(),
                message: "ok".to_string(),
            })
        } else {
            Err(Error::discovery_failed("identity failed", None))
        }
    }
}

pub(super) struct MockCap {
    pub fail: bool,
}

impl CapabilityLayer for MockCap {
    async fn query_capabilities(&self, _endpoint: &str) -> Result<CapabilityInfo> {
        if self.fail {
            return Err(Error::discovery_failed("cap query", None));
        }
        Ok(CapabilityInfo {
            primal: "p".to_string(),
            version: "1".to_string(),
            family_id: "f".to_string(),
            node_id: "n".to_string(),
            capabilities: vec![],
        })
    }

    async fn verify_capabilities(
        &self,
        _discovered: &DiscoveredPrimal,
        _identity: &IdentityProof,
    ) -> Result<CapabilityVerification> {
        if self.fail {
            return Err(Error::capability_mismatch(vec![], vec![]));
        }
        Ok(CapabilityVerification {
            verified: true,
            expected: vec![],
            actual: vec![],
            message: "mock cap ok".to_string(),
        })
    }
}

pub(super) struct MockIdentityAcceptName {
    pub accept: &'static str,
    pub proof: IdentityProof,
}

impl IdentityLayer for MockIdentityAcceptName {
    async fn request_proof(&self, _endpoint: &str, _challenge: &str) -> Result<IdentityProof> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_proof(&self, _proof: &IdentityProof) -> Result<IdentityVerification> {
        Err(Error::discovery_failed("mock", None))
    }

    async fn verify_identity(&self, discovered: &DiscoveredPrimal) -> Result<IdentityVerification> {
        if discovered.primal == self.accept {
            Ok(IdentityVerification {
                verified: true,
                proof: self.proof.clone(),
                message: "ok".to_string(),
            })
        } else {
            Err(Error::discovery_failed("identity skip", None))
        }
    }
}

pub(super) struct MockTrust {
    pub err: bool,
}

impl TrustLayer for MockTrust {
    async fn evaluate_trust(
        &self,
        _discovered: &DiscoveredPrimal,
        _identity: &IdentityProof,
        _family_seed: &[u8],
    ) -> Result<TrustEvaluation> {
        if self.err {
            Err(Error::discovery_failed("trust err", None))
        } else {
            Ok(TrustEvaluation {
                level: TrustLevel::Verified,
                relationship: None,
                lineage_verified: true,
                message: "trusted".to_string(),
            })
        }
    }
}

pub(super) fn test_client(
    primals: Vec<DiscoveredPrimal>,
    identity_ok: bool,
    primal_for_proof: &str,
    cap_fail: bool,
    trust_err: bool,
) -> NucleusClient<MockPhysical, MockIdentity, MockCap, MockTrust> {
    let proof = sample_proof(primal_for_proof);
    NucleusClient::from_layers_for_test(
        Arc::new(MockPhysical { out: primals }),
        Arc::new(MockIdentity {
            ok: identity_ok,
            proof,
        }),
        Arc::new(MockCap { fail: cap_fail }),
        Arc::new(MockTrust { err: trust_err }),
        Arc::new(Registry::new()),
    )
}
