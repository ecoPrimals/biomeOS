//! Layer 4: Trust Evaluation
//!
//! **Delegates to `BearDog`** - No reimplementation!
//!
//! `BearDog` handles:
//! - Genetic lineage verification
//! - Family membership validation
//! - Trust level computation
//! - Cryptographic proof of relationship
//!
//! This layer just coordinates `BearDog`'s existing APIs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{discovery::DiscoveredPrimal, identity::IdentityProof, Error, Result};

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
    pub fn is_sufficient(&self, required: &TrustLevel) -> bool {
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

/// Trust evaluation layer (delegates to `BearDog`)
#[async_trait]
pub trait TrustLayer: Send + Sync {
    /// Evaluate trust for a discovered primal
    ///
    /// Delegates to `BearDog`'s `federation.verify_family_member` API
    async fn evaluate_trust(
        &self,
        discovered: &DiscoveredPrimal,
        identity: &IdentityProof,
        family_seed: &[u8],
    ) -> Result<TrustEvaluation>;
}

/// Trust layer implementation
pub struct TrustLayerImpl {
    /// `BearDog` socket (discovered at runtime)
    beardog_socket: Option<String>,
}

impl TrustLayerImpl {
    /// Create a new trust layer
    ///
    /// **Deep Debt Principle**: Discovers `BearDog` at runtime, no hardcoding!
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Trust Layer (delegating to BearDog)");

        // Discover BearDog socket
        let beardog_socket = Self::discover_beardog_socket().await?;

        Ok(Self {
            beardog_socket: Some(beardog_socket),
        })
    }

    /// Discover `BearDog`'s Unix socket (same logic as identity layer)
    async fn discover_beardog_socket() -> Result<String> {
        // Reuse discovery logic from identity layer
        crate::identity::IdentityLayerImpl::new()
            .await?
            .beardog_socket
            .ok_or_else(|| Error::discovery_failed("BearDog socket not found", None))
    }

    /// Get `BearDog` socket path
    fn beardog_socket(&self) -> Result<&str> {
        self.beardog_socket
            .as_deref()
            .ok_or_else(|| Error::discovery_failed("BearDog socket not initialized", None))
    }
}

#[async_trait]
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
            "Evaluating trust (via BearDog)"
        );

        let beardog_socket = self.beardog_socket()?;

        // Encode family seed as hex
        let seed_hex = hex::encode(family_seed);

        let params = serde_json::json!({
            "seed": seed_hex,
            "reference_seed": identity.family_id,
            "node_id": identity.node_id,
        });

        let response: serde_json::Value = crate::client::call_unix_socket_rpc(
            beardog_socket,
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
}
