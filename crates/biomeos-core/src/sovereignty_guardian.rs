// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Sovereignty Guardian
//!
//! Advanced sovereignty and human dignity protection system that extends beyond
//! the basic crypto-lock system to provide comprehensive safeguards.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Sovereignty Guardian for protecting human dignity and data sovereignty
pub struct SovereigntyGuardian {
    /// Protection policies
    policies: SovereigntyPolicies,
    /// Violation tracking
    violations: HashMap<String, Vec<SovereigntyViolation>>,
    /// Audit trail
    audit_log: Vec<SovereigntyAuditEntry>,
}

/// Comprehensive sovereignty protection policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyPolicies {
    /// Data sovereignty settings
    pub data_sovereignty: DataSovereigntyPolicy,
    /// Human dignity protections
    pub human_dignity: HumanDignityPolicy,
    /// AI interaction policies
    pub ai_interactions: AIInteractionPolicy,
    /// Economic sovereignty policies
    pub economic_sovereignty: EconomicSovereigntyPolicy,
    /// Privacy and surveillance protections
    pub privacy_protection: PrivacyProtectionPolicy,
}

/// Data sovereignty policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSovereigntyPolicy {
    /// Require explicit consent for all data operations
    pub require_explicit_consent: bool,
    /// Prevent unauthorized data extraction
    pub prevent_data_extraction: bool,
    /// Enforce data portability rights
    pub enforce_data_portability: bool,
    /// Geographic data residency requirements
    pub geographic_restrictions: Vec<String>,
    /// Data retention limits
    pub retention_limits: HashMap<String, Duration>,
}

/// Human dignity protection policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDignityPolicy {
    /// Prevent algorithmic discrimination
    pub prevent_discrimination: bool,
    /// Require human oversight for critical decisions
    pub require_human_oversight: bool,
    /// Protect against psychological manipulation
    pub prevent_manipulation: bool,
    /// Ensure right to explanation for automated decisions
    pub right_to_explanation: bool,
    /// Minimum decision deliberation time
    pub minimum_deliberation_time: Duration,
}

/// AI interaction safety policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInteractionPolicy {
    /// Require AI transparency (must identify as AI)
    pub require_ai_identification: bool,
    /// Prevent deceptive AI behavior
    pub prevent_deception: bool,
    /// Limit AI persuasion capabilities
    pub limit_persuasion: bool,
    /// Cost protection thresholds
    pub cost_protection: CostProtectionPolicy,
    /// Model selection constraints
    pub model_constraints: ModelConstraints,
}

/// Cost protection for AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProtectionPolicy {
    /// Daily spending limit
    pub daily_limit: f64,
    /// Monthly spending limit  
    pub monthly_limit: f64,
    /// Warning threshold percentage
    pub warning_threshold: f64,
    /// Auto-stop at limit
    pub auto_stop_at_limit: bool,
}

/// AI model usage constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConstraints {
    /// Allowed model providers
    pub allowed_providers: Vec<String>,
    /// Maximum model capability level
    pub max_capability_level: u8,
    /// Required safety certifications
    pub required_certifications: Vec<String>,
}

/// Economic sovereignty protections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicSovereigntyPolicy {
    /// Prevent vendor lock-in
    pub prevent_vendor_lockin: bool,
    /// Ensure service portability
    pub ensure_portability: bool,
    /// Transparent pricing requirements
    pub transparent_pricing: bool,
    /// Fair competition enforcement
    pub fair_competition: bool,
    /// Local economic priority
    pub local_priority_factor: f64,
}

/// Privacy and surveillance protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyProtectionPolicy {
    /// Block unauthorized tracking
    pub block_tracking: bool,
    /// Prevent behavioral profiling
    pub prevent_profiling: bool,
    /// Minimize data collection
    pub minimize_data_collection: bool,
    /// Anonymous interaction preference
    pub prefer_anonymous: bool,
    /// Surveillance detection and countermeasures
    pub surveillance_detection: bool,
}

/// Sovereignty violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyViolation {
    /// Violation type
    pub violation_type: ViolationType,
    /// When the violation occurred
    pub timestamp: SystemTime,
    /// Entity responsible for violation
    pub responsible_entity: String,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Description of the violation
    pub description: String,
    /// Remediation taken
    pub remediation: Option<String>,
}

/// Types of sovereignty violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Unauthorized data access
    UnauthorizedDataAccess,
    /// Data extraction without consent
    DataExtraction,
    /// Discrimination in automated decisions
    AlgorithmicDiscrimination,
    /// Deceptive AI behavior
    DeceptiveAI,
    /// Economic exploitation
    EconomicExploitation,
    /// Privacy invasion
    PrivacyInvasion,
    /// Surveillance activity
    UnauthorizedSurveillance,
    /// Human dignity violation
    HumanDignityViolation,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Low severity - warning level
    Low,
    /// Medium severity - action required
    Medium,
    /// High severity - immediate intervention
    High,
    /// Critical severity - emergency response
    Critical,
}

/// Audit trail entry for sovereignty actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyAuditEntry {
    /// Timestamp of the action
    pub timestamp: SystemTime,
    /// Action taken
    pub action: SovereigntyAction,
    /// Entity affected
    pub entity: String,
    /// Outcome of the action
    pub outcome: ActionOutcome,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Sovereignty protection actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyAction {
    /// Access granted with consent
    AccessGranted,
    /// Access denied for policy violation
    AccessDenied,
    /// Data operation blocked
    DataOperationBlocked,
    /// AI interaction regulated
    AIInteractionRegulated,
    /// Economic protection applied
    EconomicProtectionApplied,
    /// Privacy protection activated
    PrivacyProtectionActivated,
}

/// Outcome of sovereignty actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionOutcome {
    /// Action successful
    Success,
    /// Action failed
    Failure(String),
    /// Action partially successful
    Partial(String),
}

impl SovereigntyGuardian {
    /// Create a new sovereignty guardian with default policies
    pub fn new() -> Self {
        Self {
            policies: SovereigntyPolicies::default(),
            violations: HashMap::new(),
            audit_log: Vec::new(),
        }
    }

    /// Create sovereignty guardian with custom policies
    pub fn with_policies(policies: SovereigntyPolicies) -> Self {
        Self {
            policies,
            violations: HashMap::new(),
            audit_log: Vec::new(),
        }
    }

    /// Evaluate a data access request for sovereignty compliance
    pub fn evaluate_data_access(
        &mut self,
        requester: &str,
        data_type: &str,
        purpose: &str,
    ) -> Result<bool> {
        info!(
            "Evaluating data access request from {} for {}",
            requester, data_type
        );

        // Check if explicit consent is required
        if self.policies.data_sovereignty.require_explicit_consent {
            // In a real implementation, this would check for explicit user consent
            debug!("Explicit consent required for data access");
        }

        // Check geographic restrictions
        if !self
            .policies
            .data_sovereignty
            .geographic_restrictions
            .is_empty()
        {
            debug!("Checking geographic restrictions for data access");
        }

        // Check data extraction prevention
        if self.policies.data_sovereignty.prevent_data_extraction
            && (purpose.contains("extract") || purpose.contains("export"))
        {
            self.record_violation(
                requester,
                &ViolationType::DataExtraction,
                &ViolationSeverity::High,
                "Attempted unauthorized data extraction".to_string(),
            );
            return Ok(false);
        }

        // Log successful access
        self.audit_log.push(SovereigntyAuditEntry {
            timestamp: SystemTime::now(),
            action: SovereigntyAction::AccessGranted,
            entity: requester.to_string(),
            outcome: ActionOutcome::Success,
            context: HashMap::from([
                ("data_type".to_string(), data_type.to_string()),
                ("purpose".to_string(), purpose.to_string()),
            ]),
        });

        Ok(true)
    }

    /// Evaluate an AI interaction for human dignity compliance
    pub fn evaluate_ai_interaction(
        &mut self,
        ai_provider: &str,
        interaction_type: &str,
        cost_estimate: f64,
    ) -> Result<bool> {
        info!(
            "Evaluating AI interaction with {} (cost: {})",
            ai_provider, cost_estimate
        );

        // Check cost protection limits
        if cost_estimate > self.policies.ai_interactions.cost_protection.daily_limit {
            self.record_violation(
                ai_provider,
                &ViolationType::EconomicExploitation,
                &ViolationSeverity::Medium,
                format!("Cost estimate {} exceeds daily limit", cost_estimate),
            );
            return Ok(false);
        }

        // Check if AI identification is required
        if self.policies.ai_interactions.require_ai_identification {
            debug!("AI must identify itself in interactions");
        }

        // Check for deceptive behavior prevention
        if self.policies.ai_interactions.prevent_deception {
            debug!("Monitoring for deceptive AI behavior");
        }

        // Log successful interaction
        self.audit_log.push(SovereigntyAuditEntry {
            timestamp: SystemTime::now(),
            action: SovereigntyAction::AIInteractionRegulated,
            entity: ai_provider.to_string(),
            outcome: ActionOutcome::Success,
            context: HashMap::from([
                ("interaction_type".to_string(), interaction_type.to_string()),
                ("cost_estimate".to_string(), cost_estimate.to_string()),
            ]),
        });

        Ok(true)
    }

    /// Monitor for privacy and surveillance violations
    pub fn monitor_privacy_compliance(&mut self, activity: &str, entity: &str) -> Result<()> {
        debug!("Monitoring privacy compliance for activity: {}", activity);

        // Check for unauthorized tracking
        if self.policies.privacy_protection.block_tracking
            && (activity.contains("track") || activity.contains("profile"))
        {
            self.record_violation(
                entity,
                &ViolationType::PrivacyInvasion,
                &ViolationSeverity::High,
                format!("Unauthorized tracking detected: {}", activity),
            );
        }

        // Check for behavioral profiling
        if self.policies.privacy_protection.prevent_profiling
            && (activity.contains("profile") || activity.contains("analyze behavior"))
        {
            self.record_violation(
                entity,
                &ViolationType::PrivacyInvasion,
                &ViolationSeverity::Medium,
                format!("Behavioral profiling detected: {}", activity),
            );
        }

        // Check surveillance detection
        if self.policies.privacy_protection.surveillance_detection {
            // In a real implementation, this would use sophisticated detection algorithms
            debug!("Surveillance detection active");
        }

        Ok(())
    }

    /// Enforce economic sovereignty protections
    pub fn enforce_economic_sovereignty(
        &mut self,
        service_provider: &str,
        service_terms: &str,
    ) -> Result<bool> {
        info!(
            "Evaluating economic sovereignty for provider: {}",
            service_provider
        );

        // Check for vendor lock-in patterns
        if self.policies.economic_sovereignty.prevent_vendor_lockin
            && (service_terms.contains("exclusive") || service_terms.contains("non-transferable"))
        {
            self.record_violation(
                service_provider,
                &ViolationType::EconomicExploitation,
                &ViolationSeverity::High,
                "Vendor lock-in terms detected".to_string(),
            );
            return Ok(false);
        }

        // Check portability requirements
        if self.policies.economic_sovereignty.ensure_portability
            && !service_terms.contains("portable")
            && !service_terms.contains("export")
        {
            warn!("Service may not support required portability");
        }

        // Log economic protection applied
        self.audit_log.push(SovereigntyAuditEntry {
            timestamp: SystemTime::now(),
            action: SovereigntyAction::EconomicProtectionApplied,
            entity: service_provider.to_string(),
            outcome: ActionOutcome::Success,
            context: HashMap::from([("evaluation".to_string(), "passed".to_string())]),
        });

        Ok(true)
    }

    /// Evaluate an action for human dignity compliance
    pub fn evaluate_human_dignity(
        &mut self,
        actor: &str,
        action: &str,
        context: &HashMap<String, String>,
    ) -> Result<bool> {
        info!("Evaluating human dignity compliance for action: {}", action);

        // Check for algorithmic discrimination patterns
        if self.policies.human_dignity.prevent_discrimination {
            let discrimination_indicators = [
                "differential_treatment",
                "demographic_filter",
                "exclusion_criteria",
                "bias_amplification",
            ];
            if discrimination_indicators.iter().any(|indicator| {
                action.contains(indicator) || context.values().any(|v| v.contains(indicator))
            }) {
                self.record_violation(
                    actor,
                    &ViolationType::AlgorithmicDiscrimination,
                    &ViolationSeverity::Critical,
                    format!("Potential algorithmic discrimination detected in action: {action}"),
                );
                return Ok(false);
            }
        }

        // Enforce human oversight for critical decisions
        if self.policies.human_dignity.require_human_oversight {
            let critical_actions = [
                "account_termination",
                "access_revocation",
                "content_removal",
                "resource_allocation",
                "priority_assignment",
            ];
            if critical_actions.iter().any(|ca| action.contains(ca)) {
                let has_oversight = context
                    .get("human_oversight")
                    .is_some_and(|v| v == "confirmed");
                if !has_oversight {
                    self.record_violation(
                        actor,
                        &ViolationType::HumanDignityViolation,
                        &ViolationSeverity::High,
                        format!("Critical action '{action}' requires human oversight"),
                    );
                    return Ok(false);
                }
            }
        }

        // Protect against psychological manipulation
        if self.policies.human_dignity.prevent_manipulation {
            let manipulation_indicators = [
                "urgency_pressure",
                "dark_pattern",
                "forced_action",
                "guilt_inducement",
                "artificial_scarcity",
            ];
            if manipulation_indicators.iter().any(|indicator| {
                action.contains(indicator) || context.values().any(|v| v.contains(indicator))
            }) {
                self.record_violation(
                    actor,
                    &ViolationType::HumanDignityViolation,
                    &ViolationSeverity::High,
                    format!("Psychological manipulation pattern detected: {action}"),
                );
                return Ok(false);
            }
        }

        // Ensure right to explanation
        if self.policies.human_dignity.right_to_explanation {
            let automated_decisions = [
                "automated_decision",
                "algorithmic_outcome",
                "model_prediction",
            ];
            if automated_decisions.iter().any(|ad| action.contains(ad)) {
                let has_explanation = context.get("explanation_provided").is_some();
                if !has_explanation {
                    warn!(
                        "Automated decision '{}' should provide explanation to affected parties",
                        action
                    );
                }
            }
        }

        self.audit_log.push(SovereigntyAuditEntry {
            timestamp: SystemTime::now(),
            action: SovereigntyAction::AccessGranted,
            entity: actor.to_string(),
            outcome: ActionOutcome::Success,
            context: context.clone(),
        });

        Ok(true)
    }

    /// Record a sovereignty violation
    fn record_violation(
        &mut self,
        entity: &str,
        violation_type: &ViolationType,
        severity: &ViolationSeverity,
        description: String,
    ) {
        let violation = SovereigntyViolation {
            violation_type: violation_type.clone(),
            timestamp: SystemTime::now(),
            responsible_entity: entity.to_string(),
            severity: severity.clone(),
            description: description.clone(),
            remediation: None,
        };

        self.violations
            .entry(entity.to_string())
            .or_default()
            .push(violation);

        // Log the violation
        warn!(
            "Sovereignty violation recorded: {:?} by {} - {}",
            violation_type, entity, description
        );

        // Add to audit trail
        self.audit_log.push(SovereigntyAuditEntry {
            timestamp: SystemTime::now(),
            action: SovereigntyAction::AccessDenied,
            entity: entity.to_string(),
            outcome: ActionOutcome::Failure(description),
            context: HashMap::from([
                (
                    "violation_type".to_string(),
                    format!("{:?}", violation_type),
                ),
                ("severity".to_string(), format!("{:?}", severity)),
            ]),
        });
    }

    /// Get violation history for an entity
    pub fn get_violations(&self, entity: &str) -> Vec<&SovereigntyViolation> {
        self.violations
            .get(entity)
            .map(|violations| violations.iter().collect())
            .unwrap_or_default()
    }

    /// Get full audit trail
    pub fn get_audit_trail(&self) -> &[SovereigntyAuditEntry] {
        &self.audit_log
    }

    /// Update sovereignty policies
    pub fn update_policies(&mut self, policies: SovereigntyPolicies) {
        self.policies = policies;
        info!("Sovereignty policies updated");
    }

    /// Generate sovereignty compliance report
    pub fn generate_compliance_report(&self) -> SovereigntyComplianceReport {
        let total_violations = self.violations.values().map(std::vec::Vec::len).sum();
        let critical_violations = self
            .violations
            .values()
            .flatten()
            .filter(|v| matches!(v.severity, ViolationSeverity::Critical))
            .count();

        SovereigntyComplianceReport {
            total_entities: self.violations.len(),
            total_violations,
            critical_violations,
            compliance_score: self.calculate_compliance_score(),
            audit_entries: self.audit_log.len(),
            generated_at: SystemTime::now(),
        }
    }

    /// Calculate overall compliance score (0-100)
    fn calculate_compliance_score(&self) -> f64 {
        if self.audit_log.is_empty() {
            return 100.0;
        }

        let total_actions = self.audit_log.len() as f64;
        let successful_actions = self
            .audit_log
            .iter()
            .filter(|entry| matches!(entry.outcome, ActionOutcome::Success))
            .count() as f64;

        (successful_actions / total_actions) * 100.0
    }
}

/// Sovereignty compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyComplianceReport {
    /// Number of entities monitored
    pub total_entities: usize,
    /// Total violations recorded
    pub total_violations: usize,
    /// Critical violations
    pub critical_violations: usize,
    /// Overall compliance score (0-100)
    pub compliance_score: f64,
    /// Number of audit entries
    pub audit_entries: usize,
    /// When the report was generated
    pub generated_at: SystemTime,
}

impl Default for SovereigntyPolicies {
    fn default() -> Self {
        Self {
            data_sovereignty: DataSovereigntyPolicy {
                require_explicit_consent: true,
                prevent_data_extraction: true,
                enforce_data_portability: true,
                geographic_restrictions: Vec::new(),
                retention_limits: HashMap::new(),
            },
            human_dignity: HumanDignityPolicy {
                prevent_discrimination: true,
                require_human_oversight: true,
                prevent_manipulation: true,
                right_to_explanation: true,
                minimum_deliberation_time: Duration::from_secs(30),
            },
            ai_interactions: AIInteractionPolicy {
                require_ai_identification: true,
                prevent_deception: true,
                limit_persuasion: true,
                cost_protection: CostProtectionPolicy {
                    daily_limit: 20.0,
                    monthly_limit: 100.0,
                    warning_threshold: 0.8,
                    auto_stop_at_limit: true,
                },
                model_constraints: ModelConstraints {
                    allowed_providers: vec!["anthropic".to_string(), "openai".to_string()],
                    max_capability_level: 8,
                    required_certifications: vec!["safety_certified".to_string()],
                },
            },
            economic_sovereignty: EconomicSovereigntyPolicy {
                prevent_vendor_lockin: true,
                ensure_portability: true,
                transparent_pricing: true,
                fair_competition: true,
                local_priority_factor: 1.5,
            },
            privacy_protection: PrivacyProtectionPolicy {
                block_tracking: true,
                prevent_profiling: true,
                minimize_data_collection: true,
                prefer_anonymous: true,
                surveillance_detection: true,
            },
        }
    }
}

impl Default for SovereigntyGuardian {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sovereignty_guardian_creation() {
        let guardian = SovereigntyGuardian::new();
        assert_eq!(guardian.violations.len(), 0);
        assert_eq!(guardian.audit_log.len(), 0);
    }

    #[tokio::test]
    async fn test_data_access_evaluation() {
        let mut guardian = SovereigntyGuardian::new();

        // Test legitimate access
        let result = guardian.evaluate_data_access("trusted-service", "user-data", "analytics");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test data extraction attempt
        let result =
            guardian.evaluate_data_access("suspicious-service", "user-data", "extract-all");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Should have recorded a violation
        assert_eq!(guardian.get_violations("suspicious-service").len(), 1);
    }

    #[tokio::test]
    async fn test_ai_interaction_evaluation() {
        let mut guardian = SovereigntyGuardian::new();

        // Test normal cost interaction
        let result = guardian.evaluate_ai_interaction("anthropic", "question", 5.0);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test excessive cost interaction
        let result = guardian.evaluate_ai_interaction("expensive-ai", "analysis", 50.0);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Should have recorded a violation
        assert_eq!(guardian.get_violations("expensive-ai").len(), 1);
    }

    #[test]
    fn test_compliance_report_generation() {
        let guardian = SovereigntyGuardian::new();
        let report = guardian.generate_compliance_report();

        assert_eq!(report.total_entities, 0);
        assert_eq!(report.total_violations, 0);
        assert!((report.compliance_score - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_human_dignity_evaluation() {
        let mut guardian = SovereigntyGuardian::new();

        // Legitimate action passes
        let ctx = HashMap::new();
        let result = guardian.evaluate_human_dignity("service-a", "status_check", &ctx);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Discrimination detected
        let ctx = HashMap::new();
        let result =
            guardian.evaluate_human_dignity("service-b", "differential_treatment_applied", &ctx);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Critical action without oversight
        let ctx = HashMap::new();
        let result = guardian.evaluate_human_dignity("service-c", "account_termination", &ctx);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Critical action WITH oversight
        let mut ctx = HashMap::new();
        ctx.insert("human_oversight".to_string(), "confirmed".to_string());
        let result = guardian.evaluate_human_dignity("service-d", "account_termination", &ctx);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Manipulation detected
        let mut ctx = HashMap::new();
        ctx.insert("tactic".to_string(), "urgency_pressure".to_string());
        let result = guardian.evaluate_human_dignity("service-e", "purchase_prompt", &ctx);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_monitor_privacy_compliance_tracking() {
        let mut guardian = SovereigntyGuardian::new();
        guardian
            .monitor_privacy_compliance("track user behavior", "tracker-svc")
            .unwrap();
        assert_eq!(guardian.get_violations("tracker-svc").len(), 1);
    }

    #[test]
    fn test_monitor_privacy_compliance_profiling() {
        let mut guardian = SovereigntyGuardian::new();
        guardian
            .monitor_privacy_compliance("analyze behavior patterns", "analytics-svc")
            .unwrap();
        assert_eq!(guardian.get_violations("analytics-svc").len(), 1);
    }

    #[test]
    fn test_enforce_economic_sovereignty_vendor_lockin() {
        let mut guardian = SovereigntyGuardian::new();
        let result = guardian.enforce_economic_sovereignty(
            "vendor-x",
            "This license is exclusive and non-transferable",
        );
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert_eq!(guardian.get_violations("vendor-x").len(), 1);
    }

    #[test]
    fn test_enforce_economic_sovereignty_ok() {
        let mut guardian = SovereigntyGuardian::new();
        let result =
            guardian.enforce_economic_sovereignty("vendor-y", "Terms support portable data export");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_get_violations_empty() {
        let guardian = SovereigntyGuardian::new();
        assert!(guardian.get_violations("nonexistent").is_empty());
    }

    #[test]
    fn test_update_policies() {
        let mut guardian = SovereigntyGuardian::new();
        let mut policies = SovereigntyPolicies::default();
        policies.data_sovereignty.require_explicit_consent = false;
        guardian.update_policies(policies);
        let result = guardian.evaluate_data_access("x", "y", "z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compliance_report_with_violations() {
        let mut guardian = SovereigntyGuardian::new();
        guardian
            .evaluate_data_access("bad", "data", "extract-all")
            .unwrap();
        let report = guardian.generate_compliance_report();
        assert_eq!(report.total_entities, 1);
        assert!(report.total_violations >= 1);
    }

    #[test]
    fn test_violation_types_serde() {
        let vt = ViolationType::DataExtraction;
        let json = serde_json::to_string(&vt).unwrap();
        let _: ViolationType = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_action_outcome_serde() {
        let o = ActionOutcome::Partial("reason".into());
        let json = serde_json::to_string(&o).unwrap();
        let _: ActionOutcome = serde_json::from_str(&json).unwrap();
    }
}
