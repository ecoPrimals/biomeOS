// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Tests for [`crate::sovereignty_guardian`] (extracted to keep the main module under 800 LOC).

use super::sovereignty_guardian::*;
use std::collections::HashMap;

#[test]
fn test_sovereignty_guardian_creation() {
    let guardian = SovereigntyGuardian::new();
    let report = guardian.generate_compliance_report();
    assert_eq!(report.total_entities, 0);
    assert_eq!(report.total_violations, 0);
    assert!(guardian.get_audit_trail().is_empty());
}

#[tokio::test]
async fn test_data_access_evaluation() {
    let mut guardian = SovereigntyGuardian::new();

    // Test legitimate access
    let result = guardian.evaluate_data_access("trusted-service", "user-data", "analytics");
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Test data extraction attempt
    let result = guardian.evaluate_data_access("suspicious-service", "user-data", "extract-all");
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
    let result = guardian
        .enforce_economic_sovereignty("vendor-x", "This license is exclusive and non-transferable");
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

#[test]
fn test_with_policies_behaves_like_new_with_defaults() {
    let g = SovereigntyGuardian::with_policies(SovereigntyPolicies::default());
    assert_eq!(g.get_audit_trail().len(), 0);
    let r = g.generate_compliance_report();
    assert!((r.compliance_score - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_compliance_report_counts_critical_violations() {
    let mut guardian = SovereigntyGuardian::new();
    let mut ctx = HashMap::new();
    ctx.insert("bias".to_string(), "bias_amplification".to_string());
    let _ = guardian.evaluate_human_dignity("actor", "bias_amplification", &ctx);
    let report = guardian.generate_compliance_report();
    assert_eq!(report.critical_violations, 1);
    assert!(report.total_violations >= 1);
}

#[test]
fn test_compliance_score_mixed_success_and_failure_audit_entries() {
    let mut guardian = SovereigntyGuardian::new();
    guardian
        .evaluate_data_access("ok-service", "data", "read")
        .unwrap();
    guardian
        .evaluate_data_access("bad", "data", "extract-all")
        .unwrap();
    let report = guardian.generate_compliance_report();
    assert!(report.audit_entries >= 2);
    assert!((report.compliance_score - 50.0).abs() < 0.01);
}

#[test]
fn test_evaluate_data_access_geographic_restrictions_debug_path() {
    let mut policies = SovereigntyPolicies::default();
    policies
        .data_sovereignty
        .geographic_restrictions
        .push("EU".to_string());
    let mut guardian = SovereigntyGuardian::with_policies(policies);
    let ok = guardian
        .evaluate_data_access("svc", "personal", "analytics")
        .unwrap();
    assert!(ok);
    assert!(!guardian.get_audit_trail().is_empty());
}

#[test]
fn test_evaluate_human_dignity_discrimination_indicator_in_context_value() {
    let mut guardian = SovereigntyGuardian::new();
    let mut ctx = HashMap::new();
    ctx.insert("signal".to_string(), "demographic_filter".to_string());
    let ok = guardian
        .evaluate_human_dignity("svc", "neutral_action", &ctx)
        .unwrap();
    assert!(!ok);
    assert!(!guardian.get_violations("svc").is_empty());
}

#[test]
fn test_evaluate_human_dignity_automated_decision_without_explanation_still_audited() {
    let mut guardian = SovereigntyGuardian::new();
    let ctx = HashMap::new();
    let ok = guardian
        .evaluate_human_dignity("auto", "automated_decision", &ctx)
        .unwrap();
    assert!(ok);
    let trail = guardian.get_audit_trail();
    assert!(
        trail
            .iter()
            .any(|e| matches!(e.outcome, ActionOutcome::Success)),
        "expected success audit after automated_decision with no blocking branch"
    );
}

#[test]
fn test_monitor_privacy_compliance_surveillance_detection_branch() {
    let mut guardian = SovereigntyGuardian::new();
    guardian
        .monitor_privacy_compliance("routine", "entity")
        .unwrap();
    assert!(guardian.get_violations("entity").is_empty());
}

#[test]
fn test_enforce_economic_portability_warning_path_without_portable_keywords() {
    let mut guardian = SovereigntyGuardian::new();
    let ok = guardian
        .enforce_economic_sovereignty("prov", "opaque terms without keywords")
        .unwrap();
    assert!(ok);
    assert!(guardian.get_violations("prov").is_empty());
}

#[test]
fn test_sovereignty_compliance_report_serde_roundtrip() {
    let mut guardian = SovereigntyGuardian::new();
    guardian.evaluate_ai_interaction("x", "chat", 5.0).unwrap();
    let r = guardian.generate_compliance_report();
    let json = serde_json::to_string(&r).unwrap();
    let _: SovereigntyComplianceReport = serde_json::from_str(&json).unwrap();
}
