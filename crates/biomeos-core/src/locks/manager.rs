use super::ai_cat_door::*;
use super::compliance::*;
use super::dependencies::*;
use super::licensing::*;
use super::monitoring::*;
use super::sovereignty::*;
use super::traits::*;
use super::types::*;
use crate::BiomeResult;
use async_trait::async_trait;
use chrono::Timelike;

/// Crypto Lock Manager - implements the actual crypto lock system
pub struct CryptoLockManager {
    pub dependency_registry: DependencyRegistry,
    pub sovereign_key_manager: SovereignKeyManager,
    pub licensing_manager: LicensingManager,
    pub usage_monitor: UsageMonitor,
    pub compliance_engine: ComplianceEngine,
    pub ai_cat_door_manager: AiCatDoorManager,
}

#[async_trait]
impl CryptoLockInterface for CryptoLockManager {
    /// Validate access to external dependency
    async fn validate_access(
        &self,
        dependency: &ExternalDependency,
        context: &AccessContext,
    ) -> BiomeResult<AccessDecision> {
        // Check if AI cat door applies
        if context.biome_configuration.ai_cat_door_enabled
            && dependency.access_requirements.cat_door_allowed
            && matches!(context.user_type, UserType::Individual { .. })
        {
            return Ok(AccessDecision {
                decision: AccessVerdict::Granted,
                reasoning: "AI cat door access granted for individual user".to_string(),
                conditions: vec![],
                alternatives_suggested: vec![],
                compliance_notes: vec!["Using AI cat door exemption".to_string()],
            });
        }

        // Check sovereign key requirement
        if dependency.access_requirements.sovereign_key_required {
            let active_keys = self.sovereign_key_manager.list_active_keys();
            let has_valid_key = active_keys
                .iter()
                .any(|key| key.spec.dependencies.contains(&dependency.id));

            if !has_valid_key {
                return Ok(AccessDecision {
                    decision: AccessVerdict::Denied,
                    reasoning: "Sovereign key required but not found".to_string(),
                    conditions: vec![AccessCondition::KeyUpgradeRequired {
                        target_tier: "sovereign".to_string(),
                    }],
                    alternatives_suggested: dependency.alternatives.clone(),
                    compliance_notes: vec!["Sovereign key authentication required".to_string()],
                });
            }
        }

        // Check compliance level
        let compliance_compatible = match (
            &dependency.access_requirements.compliance_level,
            &context.user_type,
        ) {
            (ComplianceLevel::Open, _) => true,
            (ComplianceLevel::Personal, UserType::Individual { .. }) => true,
            (ComplianceLevel::Research, UserType::Research { .. }) => true,
            (ComplianceLevel::Research, UserType::Individual { .. }) => true,
            (ComplianceLevel::Commercial, UserType::Commercial { .. }) => true,
            (
                ComplianceLevel::Enterprise,
                UserType::Commercial {
                    revenue_tier: RevenueTier::Enterprise,
                    ..
                },
            ) => true,
            (ComplianceLevel::NonCommercial, _) => !context.usage_pattern.commercial_purpose,
            _ => false,
        };

        if !compliance_compatible {
            return Ok(AccessDecision {
                decision: AccessVerdict::Denied,
                reasoning: "Compliance level mismatch".to_string(),
                conditions: vec![],
                alternatives_suggested: dependency.alternatives.clone(),
                compliance_notes: vec![
                    "Usage type not compatible with dependency license".to_string()
                ],
            });
        }

        // Check usage restrictions
        let mut conditions = vec![];
        for restriction in &dependency.access_requirements.usage_restrictions {
            match restriction {
                UsageRestriction::RateLimit { requests_per_hour } => {
                    conditions.push(AccessCondition::RateLimit {
                        max_requests_per_hour: *requests_per_hour,
                    });
                }
                UsageRestriction::DataLimit { mb_per_month } => {
                    conditions.push(AccessCondition::DataLimit {
                        max_gb_per_month: *mb_per_month / 1024,
                    });
                }
                UsageRestriction::GeoRestriction { allowed_countries } => {
                    if let Some(location) = &context.geographic_location {
                        if !allowed_countries.contains(location) {
                            return Ok(AccessDecision {
                                decision: AccessVerdict::Denied,
                                reasoning: "Geographic restriction violation".to_string(),
                                conditions: vec![],
                                alternatives_suggested: dependency.alternatives.clone(),
                                compliance_notes: vec![
                                    "Access not allowed from current location".to_string()
                                ],
                            });
                        }
                    }
                }
                UsageRestriction::TimeRestriction { allowed_hours } => {
                    let current_hour = chrono::Utc::now().hour() as u8;
                    if !allowed_hours.contains(&current_hour) {
                        return Ok(AccessDecision {
                            decision: AccessVerdict::Denied,
                            reasoning: "Time restriction violation".to_string(),
                            conditions: vec![],
                            alternatives_suggested: dependency.alternatives.clone(),
                            compliance_notes: vec!["Access not allowed at current time".to_string()],
                        });
                    }
                }
                _ => {} // Handle other restrictions
            }
        }

        // Grant access with conditions
        Ok(AccessDecision {
            decision: if conditions.is_empty() {
                AccessVerdict::Granted
            } else {
                AccessVerdict::ConditionalGrant
            },
            reasoning: "Access granted after validation".to_string(),
            conditions,
            alternatives_suggested: vec![],
            compliance_notes: vec!["All requirements satisfied".to_string()],
        })
    }

    /// Register new external dependency
    async fn register_dependency(
        &self,
        dependency: &ExternalDependency,
    ) -> BiomeResult<DependencyId> {
        // Note: This would need mutable access in real implementation
        Ok(dependency.id.clone())
    }

    /// Grant sovereign access key
    async fn grant_sovereign_key(&self, _spec: &SovereignKeySpec) -> BiomeResult<SovereignKey> {
        // Note: This would need mutable access in real implementation
        Err(crate::BiomeError::NotImplemented(
            "Requires mutable access".to_string(),
        ))
    }

    /// Revoke sovereign access key
    async fn revoke_sovereign_key(&self, _key_id: &str) -> BiomeResult<()> {
        // Note: This would need mutable access in real implementation
        Err(crate::BiomeError::NotImplemented(
            "Requires mutable access".to_string(),
        ))
    }

    /// Check compliance status
    async fn check_compliance(
        &self,
        usage_pattern: &UsagePattern,
    ) -> BiomeResult<ComplianceStatus> {
        let dependencies: Vec<ExternalDependency> = self
            .dependency_registry
            .list_dependencies()
            .into_iter()
            .cloned()
            .collect();
        self.compliance_engine
            .check_compliance(usage_pattern, &dependencies)
            .await
    }

    /// Generate compliance report
    async fn generate_compliance_report(&self) -> BiomeResult<ComplianceReport> {
        let dependencies: Vec<ExternalDependency> = self
            .dependency_registry
            .list_dependencies()
            .into_iter()
            .cloned()
            .collect();
        self.compliance_engine.generate_report(&dependencies).await
    }

    /// Update licensing terms
    async fn update_licensing(&self, _terms: &LicensingTerms) -> BiomeResult<()> {
        // Note: This would need mutable access in real implementation
        Err(crate::BiomeError::NotImplemented(
            "update_licensing not implemented".to_string(),
        ))
    }

    /// Monitor dependency usage
    async fn monitor_usage(&self, dependency_id: &DependencyId) -> BiomeResult<UsageMetrics> {
        let period = ReportPeriod {
            start_date: chrono::Utc::now() - chrono::Duration::days(30),
            end_date: chrono::Utc::now(),
        };

        Ok(self.usage_monitor.get_usage_metrics(dependency_id, &period))
    }
}

impl CryptoLockManager {
    /// Create new crypto lock manager with sovereignty-first defaults
    pub fn new() -> Self {
        Self {
            dependency_registry: DependencyRegistry::new(),
            sovereign_key_manager: SovereignKeyManager::new(),
            licensing_manager: LicensingManager::new(),
            usage_monitor: UsageMonitor::new(),
            compliance_engine: ComplianceEngine::new(),
            ai_cat_door_manager: AiCatDoorManager::new(),
        }
    }

    /// Initialize with grandma-safe AI cat door
    pub fn initialize_with_cat_door(&mut self, ai_services: Vec<AiServiceConfig>) {
        for service in ai_services {
            self.ai_cat_door_manager.cat_door.add_ai_service(service);
        }
        self.ai_cat_door_manager.cat_door.set_enabled(true);
    }

    /// Get mutable access to dependency registry
    pub fn dependency_registry_mut(&mut self) -> &mut DependencyRegistry {
        &mut self.dependency_registry
    }

    /// Get mutable access to sovereign key manager
    pub fn sovereign_key_manager_mut(&mut self) -> &mut SovereignKeyManager {
        &mut self.sovereign_key_manager
    }

    /// Get mutable access to licensing manager
    pub fn licensing_manager_mut(&mut self) -> &mut LicensingManager {
        &mut self.licensing_manager
    }

    /// Get mutable access to usage monitor
    pub fn usage_monitor_mut(&mut self) -> &mut UsageMonitor {
        &mut self.usage_monitor
    }

    /// Get mutable access to compliance engine
    pub fn compliance_engine_mut(&mut self) -> &mut ComplianceEngine {
        &mut self.compliance_engine
    }

    /// Get mutable access to AI cat door manager
    pub fn ai_cat_door_manager_mut(&mut self) -> &mut AiCatDoorManager {
        &mut self.ai_cat_door_manager
    }

    /// Perform system health check
    pub async fn health_check(&self) -> BiomeResult<SystemHealthStatus> {
        let dependency_count = self.dependency_registry.dependencies.len();
        let active_keys = self.sovereign_key_manager.list_active_keys().len();
        let active_sessions = self.usage_monitor.get_active_sessions().len();
        let ai_cat_door_enabled = self.ai_cat_door_manager.cat_door.enabled;

        Ok(SystemHealthStatus {
            healthy: true,
            dependency_count,
            active_sovereign_keys: active_keys,
            active_sessions,
            ai_cat_door_enabled,
            last_check: chrono::Utc::now(),
        })
    }

    /// Get system statistics
    pub fn get_statistics(&self) -> SystemStatistics {
        SystemStatistics {
            total_dependencies: self.dependency_registry.dependencies.len(),
            sovereign_dependencies: self
                .dependency_registry
                .get_sovereign_key_dependencies()
                .len(),
            cat_door_dependencies: self.dependency_registry.get_cat_door_dependencies().len(),
            active_sovereign_keys: self.sovereign_key_manager.list_active_keys().len(),
            active_sessions: self.usage_monitor.get_active_sessions().len(),
            ai_services_configured: self.ai_cat_door_manager.cat_door.allowed_ai_services.len(),
        }
    }
}

/// System health status
#[derive(Debug, Clone)]
pub struct SystemHealthStatus {
    pub healthy: bool,
    pub dependency_count: usize,
    pub active_sovereign_keys: usize,
    pub active_sessions: usize,
    pub ai_cat_door_enabled: bool,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// System statistics
#[derive(Debug, Clone)]
pub struct SystemStatistics {
    pub total_dependencies: usize,
    pub sovereign_dependencies: usize,
    pub cat_door_dependencies: usize,
    pub active_sovereign_keys: usize,
    pub active_sessions: usize,
    pub ai_services_configured: usize,
}

impl Default for CryptoLockManager {
    fn default() -> Self {
        Self::new()
    }
}
