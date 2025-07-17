use super::types::*;
use crate::BiomeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Licensing terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingTerms {
    pub terms_version: String,
    pub effective_date: chrono::DateTime<chrono::Utc>,
    pub personal_use_terms: PersonalUseTerms,
    pub commercial_use_terms: CommercialUseTerms,
    pub compliance_requirements: Vec<ComplianceRequirement>,
}

/// Personal use terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalUseTerms {
    pub ai_cat_door_enabled: bool,
    pub rate_limits: Vec<RateLimit>,
    pub data_limits: Vec<DataLimit>,
    pub feature_restrictions: Vec<String>,
    pub attribution_required: bool,
}

/// Commercial use terms (licensing OR partnership)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommercialUseTerms {
    pub commercial_model: CommercialModel,
    pub pricing_tiers: Vec<PricingTier>,
    pub partnership: Option<PartnershipContribution>,
    pub enterprise_features: Vec<String>,
    pub support_included: bool,
}

/// Commercial models available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommercialModel {
    /// Traditional licensing only
    LicensingOnly,
    /// Partnership contribution only (voluntary)
    PartnershipOnly,
    /// User choice: licensing OR partnership
    UserChoice,
    /// Fully open (no commercial restrictions)
    FullyOpen,
}

/// Rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub resource: String,
    pub limit: u64,
    pub period: String,
}

/// Data limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLimit {
    pub data_type: String,
    pub limit_gb: u64,
    pub period: String,
}

/// Pricing tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub tier_name: String,
    pub price: f64,
    pub currency: String,
    pub billing_period: String,
    pub included_features: Vec<String>,
    pub usage_limits: Vec<UsageLimit>,
}

/// Usage limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimit {
    pub metric: String,
    pub limit: u64,
    pub overage_cost: Option<f64>,
}

/// Partnership contribution (voluntary, sovereign)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipContribution {
    pub enabled: bool,
    pub percentage: Option<f64>,
    pub minimum_threshold: Option<f64>,
    pub payment_frequency: String,
    pub sovereign_wallet: Option<String>, // sweetgrass/rhizoCrypt
    pub benefits: Vec<String>,
}

/// Compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub requirement_id: String,
    pub title: String,
    pub description: String,
    pub mandatory: bool,
    pub verification_method: String,
}

/// Licensing manager
pub struct LicensingManager {
    pub current_terms: LicensingTerms,
    pub terms_history: Vec<LicensingTermsHistory>,
    pub user_agreements: HashMap<String, UserAgreement>,
}

/// Licensing terms history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingTermsHistory {
    pub version: String,
    pub effective_date: chrono::DateTime<chrono::Utc>,
    pub changes: Vec<String>,
    pub migration_notes: Vec<String>,
}

/// User agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgreement {
    pub user_id: String,
    pub agreement_type: AgreementType,
    pub terms_version: String,
    pub agreed_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Agreement types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgreementType {
    PersonalUse,
    CommercialLicense { tier: String },
    Partnership { contribution_percentage: f64 },
    Enterprise { contract_id: String },
}

impl Default for LicensingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LicensingManager {
    /// Create new licensing manager
    pub fn new() -> Self {
        Self {
            current_terms: LicensingTerms::default(),
            terms_history: Vec::new(),
            user_agreements: HashMap::new(),
        }
    }

    /// Update licensing terms
    pub async fn update_terms(&mut self, new_terms: LicensingTerms) -> BiomeResult<()> {
        // Archive current terms
        let history_entry = LicensingTermsHistory {
            version: self.current_terms.terms_version.clone(),
            effective_date: self.current_terms.effective_date,
            changes: Vec::new(), // TODO: Calculate diff
            migration_notes: Vec::new(),
        };

        self.terms_history.push(history_entry);
        self.current_terms = new_terms;

        Ok(())
    }

    /// Record user agreement
    pub async fn record_agreement(
        &mut self,
        user_id: String,
        agreement: UserAgreement,
    ) -> BiomeResult<()> {
        self.user_agreements.insert(user_id, agreement);
        Ok(())
    }

    /// Get user agreement
    pub fn get_user_agreement(&self, user_id: &str) -> Option<&UserAgreement> {
        self.user_agreements.get(user_id)
    }

    /// Check if user has valid agreement
    pub fn has_valid_agreement(&self, user_id: &str) -> bool {
        if let Some(agreement) = self.user_agreements.get(user_id) {
            agreement.terms_version == self.current_terms.terms_version
        } else {
            false
        }
    }

    /// Get applicable terms for user type
    pub fn get_applicable_terms(&self, user_type: &UserType) -> ApplicableTerms {
        match user_type {
            UserType::Individual { .. } => {
                ApplicableTerms::Personal(self.current_terms.personal_use_terms.clone())
            }
            UserType::Commercial { .. } => {
                ApplicableTerms::Commercial(self.current_terms.commercial_use_terms.clone())
            }
            UserType::Research { .. } => {
                ApplicableTerms::Research(self.current_terms.personal_use_terms.clone())
            }
            UserType::Government { .. } => {
                ApplicableTerms::Government(self.current_terms.commercial_use_terms.clone())
            }
            UserType::NonProfit { .. } => {
                ApplicableTerms::NonProfit(self.current_terms.personal_use_terms.clone())
            }
        }
    }

    /// Calculate pricing for user
    pub fn calculate_pricing(
        &self,
        user_type: &UserType,
        usage_pattern: &UsagePattern,
    ) -> PricingCalculation {
        match user_type {
            UserType::Individual { .. } => PricingCalculation::Free,
            UserType::Commercial { revenue_tier, .. } => {
                let base_price = match revenue_tier {
                    RevenueTier::Startup => 100.0,
                    RevenueTier::SmallBiz => 500.0,
                    RevenueTier::MidMarket => 2000.0,
                    RevenueTier::Enterprise => 10000.0,
                };

                PricingCalculation::Commercial {
                    base_price,
                    currency: "USD".to_string(),
                    billing_period: "monthly".to_string(),
                    usage_multiplier: self.calculate_usage_multiplier(usage_pattern),
                }
            }
            _ => PricingCalculation::Free,
        }
    }

    /// Calculate usage multiplier
    fn calculate_usage_multiplier(&self, usage_pattern: &UsagePattern) -> f64 {
        let mut multiplier = 1.0;

        // Scale multiplier
        multiplier *= match usage_pattern.scale {
            UsageScale::Individual => 1.0,
            UsageScale::Team => 1.5,
            UsageScale::Department => 2.0,
            UsageScale::Organization => 3.0,
            UsageScale::PublicService => 4.0,
        };

        // Frequency multiplier
        multiplier *= match usage_pattern.frequency {
            UsageFrequency::Occasional => 1.0,
            UsageFrequency::Regular => 1.2,
            UsageFrequency::Heavy => 1.5,
            UsageFrequency::Continuous => 2.0,
        };

        multiplier
    }

    /// Check compliance with terms
    pub fn check_compliance(&self, user_id: &str, usage_pattern: &UsagePattern) -> ComplianceCheck {
        if let Some(agreement) = self.user_agreements.get(user_id) {
            if agreement.terms_version != self.current_terms.terms_version {
                return ComplianceCheck::NonCompliant {
                    reason: "Terms version mismatch".to_string(),
                    required_action: "Update agreement".to_string(),
                };
            }

            // Check usage against agreed terms
            match &agreement.agreement_type {
                AgreementType::PersonalUse => {
                    if usage_pattern.commercial_purpose {
                        ComplianceCheck::NonCompliant {
                            reason: "Commercial use not allowed under personal terms".to_string(),
                            required_action: "Upgrade to commercial license".to_string(),
                        }
                    } else {
                        ComplianceCheck::Compliant
                    }
                }
                AgreementType::CommercialLicense { .. } => ComplianceCheck::Compliant,
                AgreementType::Partnership { .. } => ComplianceCheck::Compliant,
                AgreementType::Enterprise { .. } => ComplianceCheck::Compliant,
            }
        } else {
            ComplianceCheck::NoAgreement
        }
    }
}

/// Applicable terms for user
#[derive(Debug, Clone)]
pub enum ApplicableTerms {
    Personal(PersonalUseTerms),
    Commercial(CommercialUseTerms),
    Research(PersonalUseTerms),
    Government(CommercialUseTerms),
    NonProfit(PersonalUseTerms),
}

/// Pricing calculation
#[derive(Debug, Clone)]
pub enum PricingCalculation {
    Free,
    Commercial {
        base_price: f64,
        currency: String,
        billing_period: String,
        usage_multiplier: f64,
    },
    Partnership {
        contribution_percentage: f64,
        minimum_threshold: f64,
    },
    Custom {
        pricing_model: String,
        parameters: HashMap<String, f64>,
    },
}

/// Compliance check result
#[derive(Debug, Clone)]
pub enum ComplianceCheck {
    Compliant,
    NonCompliant {
        reason: String,
        required_action: String,
    },
    NoAgreement,
}

impl Default for LicensingTerms {
    fn default() -> Self {
        Self {
            terms_version: "1.0".to_string(),
            effective_date: chrono::Utc::now(),
            personal_use_terms: PersonalUseTerms::default(),
            commercial_use_terms: CommercialUseTerms::default(),
            compliance_requirements: vec![],
        }
    }
}

impl Default for PersonalUseTerms {
    fn default() -> Self {
        Self {
            ai_cat_door_enabled: true, // Grandma-safe default
            rate_limits: vec![],
            data_limits: vec![],
            feature_restrictions: vec![],
            attribution_required: false,
        }
    }
}

impl Default for CommercialUseTerms {
    fn default() -> Self {
        Self {
            commercial_model: CommercialModel::UserChoice, // Sovereignty-respecting choice
            pricing_tiers: vec![],
            partnership: None, // Opt-in, not extractive
            enterprise_features: vec![],
            support_included: false,
        }
    }
}
