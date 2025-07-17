use super::types::*;
use crate::BiomeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sovereign key manager
pub struct SovereignKeyManager {
    pub keys: HashMap<String, SovereignKey>,
    pub key_history: Vec<KeyHistoryEntry>,
}

/// Key history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHistoryEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub key_id: String,
    pub action: KeyAction,
    pub actor: String,
    pub reason: Option<String>,
}

/// Key actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAction {
    Created,
    Activated,
    Suspended,
    Revoked,
    Expired,
    Renewed,
}

impl Default for SovereignKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SovereignKeyManager {
    /// Create new sovereign key manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            key_history: Vec::new(),
        }
    }

    /// Generate sovereign key
    pub async fn generate_key(&mut self, spec: SovereignKeySpec) -> BiomeResult<SovereignKey> {
        let key_id = uuid::Uuid::new_v4().to_string();

        // Generate cryptographic keys
        let (public_key, private_key) = crate::crypto::generate_keypair()?;
        let signature = crate::crypto::sign_data(&private_key, &spec)?;

        let sovereign_key = SovereignKey {
            key_id: key_id.clone(),
            public_key,
            private_key,
            spec,
            signature,
            created_at: chrono::Utc::now(),
            expires_at: None, // TODO: Calculate based on validity period
            status: KeyStatus::PendingActivation,
        };

        self.keys.insert(key_id.clone(), sovereign_key.clone());

        self.log_key_action(key_id, KeyAction::Created, "system".to_string(), None);

        Ok(sovereign_key)
    }

    /// Activate sovereign key
    pub async fn activate_key(&mut self, key_id: &str, actor: &str) -> BiomeResult<()> {
        if let Some(key) = self.keys.get_mut(key_id) {
            key.status = KeyStatus::Active;
            self.log_key_action(
                key_id.to_string(),
                KeyAction::Activated,
                actor.to_string(),
                None,
            );
            Ok(())
        } else {
            Err(crate::BiomeError::NotFound(format!(
                "Key {} not found",
                key_id
            )))
        }
    }

    /// Suspend sovereign key
    pub async fn suspend_key(
        &mut self,
        key_id: &str,
        reason: &str,
        actor: &str,
    ) -> BiomeResult<()> {
        if let Some(key) = self.keys.get_mut(key_id) {
            key.status = KeyStatus::Suspended {
                reason: reason.to_string(),
            };
            self.log_key_action(
                key_id.to_string(),
                KeyAction::Suspended,
                actor.to_string(),
                Some(reason.to_string()),
            );
            Ok(())
        } else {
            Err(crate::BiomeError::NotFound(format!(
                "Key {} not found",
                key_id
            )))
        }
    }

    /// Revoke sovereign key
    pub async fn revoke_key(&mut self, key_id: &str, reason: &str, actor: &str) -> BiomeResult<()> {
        if let Some(key) = self.keys.get_mut(key_id) {
            key.status = KeyStatus::Revoked {
                reason: reason.to_string(),
            };
            self.log_key_action(
                key_id.to_string(),
                KeyAction::Revoked,
                actor.to_string(),
                Some(reason.to_string()),
            );
            Ok(())
        } else {
            Err(crate::BiomeError::NotFound(format!(
                "Key {} not found",
                key_id
            )))
        }
    }

    /// Get sovereign key
    pub fn get_key(&self, key_id: &str) -> Option<&SovereignKey> {
        self.keys.get(key_id)
    }

    /// List active keys
    pub fn list_active_keys(&self) -> Vec<&SovereignKey> {
        self.keys
            .values()
            .filter(|key| matches!(key.status, KeyStatus::Active))
            .collect()
    }

    /// Check key expiration
    pub async fn check_expirations(&mut self) -> BiomeResult<Vec<String>> {
        let now = chrono::Utc::now();
        let mut expired_keys = Vec::new();

        // Collect expired keys first
        let keys_to_expire: Vec<String> = self
            .keys
            .iter()
            .filter_map(|(key_id, key)| {
                if let Some(expires_at) = key.expires_at {
                    if now > expires_at && !matches!(key.status, KeyStatus::Expired) {
                        Some(key_id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Now update the keys
        for key_id in keys_to_expire {
            if let Some(key) = self.keys.get_mut(&key_id) {
                key.status = KeyStatus::Expired;
                expired_keys.push(key_id.clone());
                self.log_key_action(key_id, KeyAction::Expired, "system".to_string(), None);
            }
        }

        Ok(expired_keys)
    }

    /// Log key action
    fn log_key_action(
        &mut self,
        key_id: String,
        action: KeyAction,
        actor: String,
        reason: Option<String>,
    ) {
        self.key_history.push(KeyHistoryEntry {
            timestamp: chrono::Utc::now(),
            key_id,
            action,
            actor,
            reason,
        });
    }

    /// Get key history
    pub fn get_key_history(&self, key_id: &str) -> Vec<&KeyHistoryEntry> {
        self.key_history
            .iter()
            .filter(|entry| entry.key_id == key_id)
            .collect()
    }
}

/// Sovereignty analyzer
pub struct SovereigntyAnalyzer;

impl SovereigntyAnalyzer {
    /// Analyze sovereignty impact of dependencies
    pub fn analyze_sovereignty(&self, dependencies: &[ExternalDependency]) -> SovereigntyAnalysis {
        let mut vendor_lock_risks = Vec::new();
        let data_residency_issues = Vec::new();
        let mut exit_strategies = Vec::new();
        let independence_roadmap = Vec::new();

        let mut total_score = 0.0;
        let mut dependency_count = 0;

        for dependency in dependencies {
            // Calculate sovereignty impact score
            let impact_score = self.calculate_sovereignty_score(&dependency.sovereignty_impact);
            total_score += impact_score;
            dependency_count += 1;

            // Collect vendor lock risks
            vendor_lock_risks.push(dependency.sovereignty_impact.vendor_lock_risk.clone());

            // Generate exit strategy assessment
            exit_strategies.push(ExitStrategyAssessment {
                dependency_id: dependency.id.clone(),
                exit_difficulty: dependency
                    .sovereignty_impact
                    .exit_strategy
                    .estimated_migration_time_weeks
                    .into(),
                estimated_cost: dependency
                    .sovereignty_impact
                    .vendor_lock_risk
                    .cost_to_exit
                    .unwrap_or(0.0),
                estimated_time_weeks: dependency
                    .sovereignty_impact
                    .exit_strategy
                    .estimated_migration_time_weeks,
                major_blockers: dependency
                    .sovereignty_impact
                    .exit_strategy
                    .migration_checklist
                    .clone(),
                recommended_alternatives: dependency.alternatives.clone(),
            });
        }

        let sovereignty_score = if dependency_count > 0 {
            total_score / dependency_count as f64
        } else {
            1.0
        };

        SovereigntyAnalysis {
            sovereignty_score,
            vendor_lock_risks,
            data_residency_issues,
            exit_strategies,
            independence_roadmap,
        }
    }

    /// Calculate sovereignty score for a single dependency
    fn calculate_sovereignty_score(&self, impact: &SovereigntyImpact) -> f64 {
        match impact.impact_level {
            SovereigntyImpactLevel::None => 1.0,
            SovereigntyImpactLevel::Minimal => 0.8,
            SovereigntyImpactLevel::Moderate => 0.6,
            SovereigntyImpactLevel::High => 0.4,
            SovereigntyImpactLevel::Critical => 0.2,
        }
    }
}

impl From<u32> for MigrationDifficulty {
    fn from(weeks: u32) -> Self {
        match weeks {
            0..=1 => MigrationDifficulty::Trivial,
            2..=4 => MigrationDifficulty::Easy,
            5..=12 => MigrationDifficulty::Moderate,
            13..=26 => MigrationDifficulty::Hard,
            _ => MigrationDifficulty::Extreme,
        }
    }
}

/// Exit strategy assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategyAssessment {
    pub dependency_id: DependencyId,
    pub exit_difficulty: MigrationDifficulty,
    pub estimated_cost: f64,
    pub estimated_time_weeks: u32,
    pub major_blockers: Vec<String>,
    pub recommended_alternatives: Vec<AlternativeDependency>,
}

/// Independence step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependenceStep {
    pub step_id: String,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_effort: ImplementationEffort,
    pub dependencies: Vec<String>,
    pub success_criteria: Vec<String>,
}

// RecommendationPriority and ImplementationEffort are imported from types.rs

/// Data residency issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataResidencyIssue {
    pub issue_id: String,
    pub dependency_id: DependencyId,
    pub data_location: String,
    pub required_location: String,
    pub severity: ViolationSeverity,
    pub mitigation_options: Vec<String>,
}

// ViolationSeverity is imported from types.rs

/// Sovereignty analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyAnalysis {
    pub sovereignty_score: f64,
    pub vendor_lock_risks: Vec<VendorLockRisk>,
    pub data_residency_issues: Vec<DataResidencyIssue>,
    pub exit_strategies: Vec<ExitStrategyAssessment>,
    pub independence_roadmap: Vec<IndependenceStep>,
}
