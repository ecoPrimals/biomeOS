// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Layer 4: Trust Evaluation
//!
//! **Delegates to the security provider** (`BearDog` at runtime) — no reimplementation.
//!
//! The security primal handles:
//! - Genetic lineage verification
//! - Family membership validation
//! - Trust level computation
//! - Cryptographic proof of relationship
//!
//! This layer coordinates those APIs over the `"encryption"` capability socket.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{Error, Result, discovery::DiscoveredPrimal, identity::IdentityProof};

/// Trust level (evaluated by `BearDog`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// Fully trusted (same family, verified lineage)
    Verified,
    /// Trusted (sibling family, verified parent)
    Trusted,
    /// Known (announced via Songbird, identity verified)
    Known,
    /// Unknown (no verification)
    Unknown,
}

impl TrustLevel {
    /// Check if this trust level is sufficient
    #[must_use]
    pub const fn is_sufficient(&self, required: &Self) -> bool {
        use TrustLevel::{Known, Trusted, Unknown, Verified};
        matches!(
            (self, required),
            (Verified, _)
                | (Trusted, Trusted | Known | Unknown)
                | (Known, Known | Unknown)
                | (Unknown, Unknown)
        )
    }
}

/// Trust evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluation {
    /// Trust level achieved
    pub level: TrustLevel,
    /// Family relationship (if any)
    pub relationship: Option<String>,
    /// Lineage verification (if applicable)
    pub lineage_verified: bool,
    /// Message
    pub message: String,
}

/// Trust evaluation layer (delegates to the security provider)
pub trait TrustLayer: Send + Sync {
    /// Evaluate trust for a discovered primal
    ///
    /// Delegates to `federation.verify_family_member` on the security provider
    fn evaluate_trust(
        &self,
        discovered: &DiscoveredPrimal,
        identity: &IdentityProof,
        family_seed: &[u8],
    ) -> impl std::future::Future<Output = Result<TrustEvaluation>> + Send;
}

/// Trust layer implementation
pub struct TrustLayerImpl {
    /// Security provider Unix socket (discovered at runtime via `"encryption"` capability)
    security_socket: Option<String>,
}

impl TrustLayerImpl {
    /// Create a new trust layer
    ///
    /// **Deep Debt Principle**: Discovers the security provider at runtime, no hardcoding!
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Security provider socket cannot be discovered (service not running or socket not found)
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Trust Layer (security provider)");

        let security_socket = Self::discover_security_provider().await?;

        Ok(Self {
            security_socket: Some(security_socket),
        })
    }

    /// Discover the security provider Unix socket via [`discover_capability_socket`](biomeos_types::capability_discovery::discover_capability_socket) (`encryption` capability).
    #[expect(
        clippy::unused_async,
        reason = "mirrors IdentityLayerImpl::new — stable async constructor surface"
    )]
    async fn discover_security_provider() -> Result<String> {
        crate::identity::IdentityLayerImpl::discover_provider_socket("encryption", &|k| {
            biomeos_types::capability_discovery::std_env(k)
        })
    }

    fn security_socket(&self) -> Result<&str> {
        self.security_socket.as_deref().ok_or_else(|| {
            Error::discovery_failed("Security provider socket not initialized", None)
        })
    }
}

impl TrustLayer for TrustLayerImpl {
    async fn evaluate_trust(
        &self,
        discovered: &DiscoveredPrimal,
        identity: &IdentityProof,
        family_seed: &[u8],
    ) -> Result<TrustEvaluation> {
        info!(
            primal = %discovered.primal,
            family = %discovered.family_id,
            "Evaluating trust (via security provider)"
        );

        let security_socket = self.security_socket()?;

        // Encode family seed as hex
        let seed_hex = hex::encode(family_seed);

        let params = serde_json::json!({
            "seed": seed_hex,
            "reference_seed": identity.family_id,
            "node_id": identity.node_id,
        });

        let response: serde_json::Value = crate::client::call_unix_socket_rpc(
            security_socket,
            "federation.verify_family_member",
            params,
        )
        .await?;

        // Parse trust evaluation
        let is_family_member = response
            .get("is_family_member")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let relationship = response
            .get("relationship")
            .and_then(|v| v.as_str())
            .map(String::from);

        let message = response
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message")
            .to_string();

        // Determine trust level
        let level = if is_family_member {
            if relationship.as_deref() == Some("sibling")
                || relationship.as_deref() == Some("child")
            {
                TrustLevel::Verified
            } else {
                TrustLevel::Trusted
            }
        } else {
            TrustLevel::Known
        };

        info!(
            primal = %discovered.primal,
            trust_level = ?level,
            "Trust evaluation complete"
        );

        Ok(TrustEvaluation {
            level,
            relationship,
            lineage_verified: is_family_member,
            message,
        })
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_sufficiency() {
        use TrustLevel::*;

        // Verified is sufficient for all
        assert!(Verified.is_sufficient(&Verified));
        assert!(Verified.is_sufficient(&Trusted));
        assert!(Verified.is_sufficient(&Known));
        assert!(Verified.is_sufficient(&Unknown));

        // Trusted is not sufficient for Verified
        assert!(!Trusted.is_sufficient(&Verified));
        assert!(Trusted.is_sufficient(&Trusted));
        assert!(Trusted.is_sufficient(&Known));

        // Known is not sufficient for Verified or Trusted
        assert!(!Known.is_sufficient(&Verified));
        assert!(!Known.is_sufficient(&Trusted));
        assert!(Known.is_sufficient(&Known));
    }

    #[test]
    fn trust_level_unknown_only_satisfies_unknown() {
        use TrustLevel::*;
        assert!(Unknown.is_sufficient(&Unknown));
        assert!(!Unknown.is_sufficient(&Known));
        assert!(!Unknown.is_sufficient(&Trusted));
        assert!(!Unknown.is_sufficient(&Verified));
    }

    #[test]
    fn trust_level_known_and_trusted_cover_unknown_requirement() {
        use TrustLevel::*;
        assert!(Known.is_sufficient(&Unknown));
        assert!(Trusted.is_sufficient(&Unknown));
    }

    #[test]
    fn test_trust_evaluation_parsing() {
        let json = r#"{
            "level": "verified",
            "relationship": "sibling",
            "lineage_verified": true,
            "message": "Family member verified"
        }"#;

        let eval: TrustEvaluation = serde_json::from_str(json).unwrap();
        assert_eq!(eval.level, TrustLevel::Verified);
        assert_eq!(eval.relationship, Some("sibling".to_string()));
        assert!(eval.lineage_verified);
    }

    #[test]
    fn trust_level_serde_snake_case_roundtrip() {
        for level in [
            TrustLevel::Verified,
            TrustLevel::Trusted,
            TrustLevel::Known,
            TrustLevel::Unknown,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let back: TrustLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(back, level);
        }
    }

    #[test]
    fn trust_evaluation_serde_roundtrip() {
        let eval = TrustEvaluation {
            level: TrustLevel::Trusted,
            relationship: Some("child".to_string()),
            lineage_verified: false,
            message: "m".to_string(),
        };
        let json = serde_json::to_string(&eval).unwrap();
        let back: TrustEvaluation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.level, eval.level);
        assert_eq!(back.relationship, eval.relationship);
        assert_eq!(back.lineage_verified, eval.lineage_verified);
        assert_eq!(back.message, eval.message);
    }

    #[test]
    fn trusted_is_sufficient_for_known_requirement() {
        assert!(TrustLevel::Trusted.is_sufficient(&TrustLevel::Known));
    }

    #[test]
    fn verified_is_sufficient_for_trusted_requirement() {
        assert!(TrustLevel::Verified.is_sufficient(&TrustLevel::Trusted));
    }
}
