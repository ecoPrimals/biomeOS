// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Layer 4: Trust Evaluation
//!
//! Trust/attestation logic via BearDog lineage verification.

use super::verification::IdentityProof;
use crate::FederationResult;
use crate::beardog_client::BearDogClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Trust level for a verified primal
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Unknown/unverified primal
    Unknown = 0,
    /// Basic trust (discovered, identity verified)
    Basic = 1,
    /// Elevated trust (capabilities verified)
    Elevated = 2,
    /// High trust (same family)
    High = 3,
    /// Highest trust (sibling node)
    Highest = 4,
}

/// Layer 4: Trust Evaluation via BearDog
pub(crate) async fn layer4_trust_evaluation(
    beardog: &BearDogClient,
    identity_proof: &IdentityProof,
    family_id: Option<&str>,
) -> FederationResult<TrustLevel> {
    debug!("Layer 4: Trust Evaluation (BearDog)");

    if let Some(ref peer_family_id) = identity_proof.family_id {
        if let Some(our_family_id) = family_id {
            match beardog
                .verify_same_family(our_family_id, peer_family_id, &identity_proof.node_id)
                .await
            {
                Ok(lineage) => {
                    let trust_level = match lineage.relationship.as_str() {
                        "parent" | "child" => TrustLevel::High,
                        "sibling" => TrustLevel::Highest,
                        _ if lineage.is_family_member => TrustLevel::Elevated,
                        _ => TrustLevel::Basic,
                    };

                    info!(
                        "   Trust evaluation: {} → {:?} (relationship: {})",
                        identity_proof.node_id, trust_level, lineage.relationship
                    );

                    Ok(trust_level)
                }
                Err(e) => {
                    warn!("Failed to verify lineage: {}, defaulting to Basic trust", e);
                    Ok(TrustLevel::Basic)
                }
            }
        } else {
            debug!("No family_id set, cannot verify lineage");
            Ok(TrustLevel::Basic)
        }
    } else {
        debug!("Primal has no family_id, cannot verify lineage");
        Ok(TrustLevel::Basic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Unknown < TrustLevel::Basic);
        assert!(TrustLevel::Basic < TrustLevel::Elevated);
        assert!(TrustLevel::Elevated < TrustLevel::High);
        assert!(TrustLevel::High < TrustLevel::Highest);
    }

    #[test]
    fn test_trust_level_clone_eq() {
        let t = TrustLevel::High;
        let t2 = t;
        assert_eq!(t, t2);
    }

    #[test]
    fn test_trust_level_debug() {
        let dbg = format!("{:?}", TrustLevel::Elevated);
        assert!(dbg.contains("Elevated"));
    }

    #[test]
    fn test_trust_level_serde_roundtrip() {
        let original = TrustLevel::Highest;
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: TrustLevel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }
}
