use crate::BiomeResult;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI Cat Door - special allowance for personal AI access
pub struct AiCatDoor {
    pub enabled: bool,
    pub allowed_ai_services: Vec<AiServiceConfig>,
    pub personal_api_keys: HashMap<String, String>,
    pub usage_limits: PersonalAiLimits,
}

/// AI service configuration for cat door
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiServiceConfig {
    pub service_name: String,
    pub api_endpoint: String,
    pub service_type: AiServiceType,
    pub max_requests_per_day: u32,
    pub max_tokens_per_request: u32,
    pub cost_limit_per_month: f64,
}

/// AI service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiServiceType {
    TextGeneration,
    ImageGeneration,
    CodeGeneration,
    Translation,
    Summarization,
    QuestionAnswering,
    Custom { service_type: String },
}

/// Personal AI usage limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAiLimits {
    pub daily_request_limit: u32,
    pub monthly_token_limit: u64,
    pub monthly_cost_limit: f64,
    pub concurrent_request_limit: u32,
}

/// AI usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageTracker {
    pub daily_usage: HashMap<String, DailyAiUsage>,
    pub monthly_usage: HashMap<String, MonthlyAiUsage>,
    pub active_requests: HashMap<String, ActiveAiRequest>,
}

/// Daily AI usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyAiUsage {
    pub date: chrono::NaiveDate,
    pub service_name: String,
    pub requests_made: u32,
    pub tokens_used: u64,
    pub cost_incurred: f64,
}

/// Monthly AI usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAiUsage {
    pub year_month: String,
    pub service_name: String,
    pub total_requests: u32,
    pub total_tokens: u64,
    pub total_cost: f64,
}

/// Active AI request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAiRequest {
    pub request_id: String,
    pub service_name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub estimated_tokens: u32,
    pub estimated_cost: f64,
}

impl AiCatDoor {
    /// Create new AI cat door with grandma-safe defaults
    pub fn new() -> Self {
        Self {
            enabled: true,
            allowed_ai_services: Vec::new(),
            personal_api_keys: HashMap::new(),
            usage_limits: PersonalAiLimits::grandma_safe_defaults(),
        }
    }

    /// Add AI service configuration
    pub fn add_ai_service(&mut self, config: AiServiceConfig) {
        self.allowed_ai_services.push(config);
    }

    /// Remove AI service
    pub fn remove_ai_service(&mut self, service_name: &str) {
        self.allowed_ai_services
            .retain(|config| config.service_name != service_name);
    }

    /// Set personal API key
    pub fn set_api_key(&mut self, service_name: String, api_key: String) {
        self.personal_api_keys.insert(service_name, api_key);
    }

    /// Get AI service configuration
    pub fn get_service_config(&self, service_name: &str) -> Option<&AiServiceConfig> {
        self.allowed_ai_services
            .iter()
            .find(|config| config.service_name == service_name)
    }

    /// Check if AI service is allowed
    pub fn is_service_allowed(&self, service_name: &str) -> bool {
        self.enabled
            && self
                .allowed_ai_services
                .iter()
                .any(|config| config.service_name == service_name)
    }

    /// Get personal API key
    pub fn get_api_key(&self, service_name: &str) -> Option<&String> {
        self.personal_api_keys.get(service_name)
    }

    /// Update usage limits
    pub fn update_limits(&mut self, limits: PersonalAiLimits) {
        self.usage_limits = limits;
    }

    /// Enable/disable cat door
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for AiUsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl AiUsageTracker {
    /// Create new usage tracker
    pub fn new() -> Self {
        Self {
            daily_usage: HashMap::new(),
            monthly_usage: HashMap::new(),
            active_requests: HashMap::new(),
        }
    }

    /// Start tracking AI request
    pub fn start_request(
        &mut self,
        service_name: String,
        estimated_tokens: u32,
        estimated_cost: f64,
    ) -> String {
        let request_id = uuid::Uuid::new_v4().to_string();
        let request = ActiveAiRequest {
            request_id: request_id.clone(),
            service_name,
            started_at: chrono::Utc::now(),
            estimated_tokens,
            estimated_cost,
        };

        self.active_requests.insert(request_id.clone(), request);
        request_id
    }

    /// Complete AI request tracking
    pub fn complete_request(
        &mut self,
        request_id: &str,
        actual_tokens: u64,
        actual_cost: f64,
    ) -> BiomeResult<()> {
        if let Some(request) = self.active_requests.remove(request_id) {
            let today = chrono::Utc::now().date_naive();
            let month_key = format!("{}-{:02}", today.year(), today.month());

            // Update daily usage
            let daily_key = format!("{}:{}", request.service_name, today);
            let daily_usage = self.daily_usage.entry(daily_key).or_insert(DailyAiUsage {
                date: today,
                service_name: request.service_name.clone(),
                requests_made: 0,
                tokens_used: 0,
                cost_incurred: 0.0,
            });

            daily_usage.requests_made += 1;
            daily_usage.tokens_used += actual_tokens;
            daily_usage.cost_incurred += actual_cost;

            // Update monthly usage
            let monthly_key = format!("{}:{}", request.service_name, month_key);
            let monthly_usage = self
                .monthly_usage
                .entry(monthly_key)
                .or_insert(MonthlyAiUsage {
                    year_month: month_key,
                    service_name: request.service_name.clone(),
                    total_requests: 0,
                    total_tokens: 0,
                    total_cost: 0.0,
                });

            monthly_usage.total_requests += 1;
            monthly_usage.total_tokens += actual_tokens;
            monthly_usage.total_cost += actual_cost;

            Ok(())
        } else {
            Err(crate::BiomeError::NotFound(format!(
                "Request {} not found",
                request_id
            )))
        }
    }

    /// Check if request is within limits
    pub fn check_limits(&self, service_name: &str, limits: &PersonalAiLimits) -> LimitCheckResult {
        let today = chrono::Utc::now().date_naive();
        let month_key = format!("{}-{:02}", today.year(), today.month());

        // Check daily limits
        let daily_key = format!("{}:{}", service_name, today);
        if let Some(daily_usage) = self.daily_usage.get(&daily_key) {
            if daily_usage.requests_made >= limits.daily_request_limit {
                return LimitCheckResult::DailyLimitExceeded;
            }
        }

        // Check monthly limits
        let monthly_key = format!("{}:{}", service_name, month_key);
        if let Some(monthly_usage) = self.monthly_usage.get(&monthly_key) {
            if monthly_usage.total_tokens >= limits.monthly_token_limit {
                return LimitCheckResult::MonthlyTokenLimitExceeded;
            }
            if monthly_usage.total_cost >= limits.monthly_cost_limit {
                return LimitCheckResult::MonthlyCostLimitExceeded;
            }
        }

        // Check concurrent requests
        let concurrent_count = self
            .active_requests
            .values()
            .filter(|req| req.service_name == service_name)
            .count();

        if concurrent_count >= limits.concurrent_request_limit as usize {
            return LimitCheckResult::ConcurrentLimitExceeded;
        }

        LimitCheckResult::WithinLimits
    }

    /// Get daily usage for service
    pub fn get_daily_usage(
        &self,
        service_name: &str,
        date: chrono::NaiveDate,
    ) -> Option<&DailyAiUsage> {
        let key = format!("{}:{}", service_name, date);
        self.daily_usage.get(&key)
    }

    /// Get monthly usage for service
    pub fn get_monthly_usage(
        &self,
        service_name: &str,
        year_month: &str,
    ) -> Option<&MonthlyAiUsage> {
        let key = format!("{}:{}", service_name, year_month);
        self.monthly_usage.get(&key)
    }

    /// Get active request count for service
    pub fn get_active_request_count(&self, service_name: &str) -> usize {
        self.active_requests
            .values()
            .filter(|req| req.service_name == service_name)
            .count()
    }

    /// Clean up old usage data
    pub fn cleanup_old_data(&mut self, retention_days: u32) {
        let cutoff_date =
            chrono::Utc::now().date_naive() - chrono::Duration::days(retention_days as i64);

        self.daily_usage
            .retain(|_, usage| usage.date >= cutoff_date);

        // Keep last 12 months of monthly data
        let cutoff_month = chrono::Utc::now().date_naive() - chrono::Duration::days(365);
        let cutoff_month_key = format!("{}-{:02}", cutoff_month.year(), cutoff_month.month());

        self.monthly_usage
            .retain(|_, usage| usage.year_month >= cutoff_month_key);
    }
}

impl PersonalAiLimits {
    /// Create grandma-safe defaults
    pub fn grandma_safe_defaults() -> Self {
        Self {
            daily_request_limit: 100,       // Reasonable daily limit
            monthly_token_limit: 1_000_000, // Generous but not unlimited
            monthly_cost_limit: 20.0,       // $20/month limit
            concurrent_request_limit: 3,    // Prevent abuse
        }
    }

    /// Create developer-friendly limits
    pub fn developer_defaults() -> Self {
        Self {
            daily_request_limit: 1000,
            monthly_token_limit: 10_000_000,
            monthly_cost_limit: 100.0,
            concurrent_request_limit: 10,
        }
    }

    /// Create enterprise limits
    pub fn enterprise_defaults() -> Self {
        Self {
            daily_request_limit: 10000,
            monthly_token_limit: 100_000_000,
            monthly_cost_limit: 1000.0,
            concurrent_request_limit: 50,
        }
    }
}

impl Default for AiCatDoor {
    fn default() -> Self {
        Self::new()
    }
}

/// Limit check result
#[derive(Debug, Clone, PartialEq)]
pub enum LimitCheckResult {
    WithinLimits,
    DailyLimitExceeded,
    MonthlyTokenLimitExceeded,
    MonthlyCostLimitExceeded,
    ConcurrentLimitExceeded,
}

/// AI cat door manager
pub struct AiCatDoorManager {
    pub cat_door: AiCatDoor,
    pub usage_tracker: AiUsageTracker,
    pub config: AiCatDoorConfig,
}

/// AI cat door configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCatDoorConfig {
    pub enabled: bool,
    pub auto_cleanup_days: u32,
    pub alert_on_limit_approach: bool,
    pub limit_warning_threshold: f64,
}

impl Default for AiCatDoorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AiCatDoorManager {
    /// Create new AI cat door manager
    pub fn new() -> Self {
        Self {
            cat_door: AiCatDoor::new(),
            usage_tracker: AiUsageTracker::new(),
            config: AiCatDoorConfig::default(),
        }
    }

    /// Request AI service access
    pub async fn request_access(
        &mut self,
        service_name: &str,
        estimated_tokens: u32,
        estimated_cost: f64,
    ) -> BiomeResult<AiAccessResult> {
        if !self.cat_door.enabled {
            return Ok(AiAccessResult::Disabled);
        }

        if !self.cat_door.is_service_allowed(service_name) {
            return Ok(AiAccessResult::ServiceNotAllowed);
        }

        let limit_check = self
            .usage_tracker
            .check_limits(service_name, &self.cat_door.usage_limits);

        match limit_check {
            LimitCheckResult::WithinLimits => {
                let request_id = self.usage_tracker.start_request(
                    service_name.to_string(),
                    estimated_tokens,
                    estimated_cost,
                );
                Ok(AiAccessResult::Granted { request_id })
            }
            _ => Ok(AiAccessResult::LimitExceeded {
                limit_type: limit_check,
            }),
        }
    }

    /// Complete AI request
    pub async fn complete_request(
        &mut self,
        request_id: &str,
        actual_tokens: u64,
        actual_cost: f64,
    ) -> BiomeResult<()> {
        self.usage_tracker
            .complete_request(request_id, actual_tokens, actual_cost)
    }

    /// Get usage summary
    pub fn get_usage_summary(&self, service_name: &str) -> AiUsageSummary {
        let today = chrono::Utc::now().date_naive();
        let month_key = format!("{}-{:02}", today.year(), today.month());

        let daily_usage = self.usage_tracker.get_daily_usage(service_name, today);
        let monthly_usage = self
            .usage_tracker
            .get_monthly_usage(service_name, &month_key);
        let active_requests = self.usage_tracker.get_active_request_count(service_name);

        AiUsageSummary {
            service_name: service_name.to_string(),
            daily_requests: daily_usage.map(|u| u.requests_made).unwrap_or(0),
            daily_tokens: daily_usage.map(|u| u.tokens_used).unwrap_or(0),
            daily_cost: daily_usage.map(|u| u.cost_incurred).unwrap_or(0.0),
            monthly_requests: monthly_usage.map(|u| u.total_requests).unwrap_or(0),
            monthly_tokens: monthly_usage.map(|u| u.total_tokens).unwrap_or(0),
            monthly_cost: monthly_usage.map(|u| u.total_cost).unwrap_or(0.0),
            active_requests: active_requests as u32,
            limits: self.cat_door.usage_limits.clone(),
        }
    }
}

impl Default for AiCatDoorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_cleanup_days: 90,
            alert_on_limit_approach: true,
            limit_warning_threshold: 0.8,
        }
    }
}

/// AI access result
#[derive(Debug, Clone)]
pub enum AiAccessResult {
    Granted { request_id: String },
    Disabled,
    ServiceNotAllowed,
    LimitExceeded { limit_type: LimitCheckResult },
}

/// AI usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageSummary {
    pub service_name: String,
    pub daily_requests: u32,
    pub daily_tokens: u64,
    pub daily_cost: f64,
    pub monthly_requests: u32,
    pub monthly_tokens: u64,
    pub monthly_cost: f64,
    pub active_requests: u32,
    pub limits: PersonalAiLimits,
}
