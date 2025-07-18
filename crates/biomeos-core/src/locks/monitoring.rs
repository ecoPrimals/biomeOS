use super::types::*;
use crate::BiomeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Usage monitor for tracking dependency usage
pub struct UsageMonitor {
    pub active_sessions: HashMap<String, ActiveSession>,
    pub usage_history: Vec<UsageRecord>,
    pub rate_limiters: HashMap<DependencyId, RateLimiter>,
}

/// Active monitoring session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub session_id: String,
    pub dependency_id: DependencyId,
    pub user_context: AccessContext,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub requests_made: u32,
    pub data_transferred_mb: f64,
    pub errors_encountered: u32,
    pub total_response_time_ms: f64,
}

/// Usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub record_id: String,
    pub dependency_id: DependencyId,
    pub user_context: AccessContext,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub success: bool,
    pub response_time_ms: u64,
    pub data_size_bytes: u64,
    pub cost: Option<f64>,
}

/// Rate limiter
pub struct RateLimiter {
    pub dependency_id: DependencyId,
    pub limits: Vec<RateLimit>,
    pub current_usage: HashMap<String, u64>,
    pub reset_times: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

/// Rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub resource: String,
    pub limit: u64,
    pub period: String,
}

impl Default for UsageMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageMonitor {
    /// Create new usage monitor
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            usage_history: Vec::new(),
            rate_limiters: HashMap::new(),
        }
    }

    /// Start monitoring session
    pub async fn start_session(
        &mut self,
        dependency_id: DependencyId,
        user_context: AccessContext,
    ) -> BiomeResult<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let session = ActiveSession {
            session_id: session_id.clone(),
            dependency_id,
            user_context,
            start_time: now,
            last_activity: now,
            requests_made: 0,
            data_transferred_mb: 0.0,
            errors_encountered: 0,
            total_response_time_ms: 0.0,
        };

        self.active_sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// End usage session
    pub async fn end_session(&mut self, session_id: &str) -> BiomeResult<UsageMetrics> {
        if let Some(session) = self.active_sessions.remove(session_id) {
            let duration = session.last_activity - session.start_time;

            // Calculate actual concurrency from session history
            let peak_concurrency = self.calculate_peak_concurrency(&session.dependency_id, &session.start_time, &session.last_activity);
            
            // Calculate error rate from session data
            let error_rate = if session.requests_made > 0 {
                session.errors_encountered as f64 / session.requests_made as f64
            } else {
                0.0
            };

            // Calculate average response time
            let average_response_time_ms = if session.requests_made > 0 {
                session.total_response_time_ms / session.requests_made as f64
            } else {
                0.0
            };

            // Calculate actual cost based on usage
            let cost_incurred = self.calculate_session_cost(&session);

            // Calculate quota utilization
            let quota_utilization = self.calculate_quota_utilization(&session.dependency_id, &session);

            let metrics = UsageMetrics {
                dependency_id: session.dependency_id.clone(),
                reporting_period: ReportPeriod {
                    start_date: session.start_time,
                    end_date: session.last_activity,
                },
                request_count: session.requests_made as u64,
                data_transferred_gb: session.data_transferred_mb / 1024.0,
                peak_concurrency,
                error_rate,
                average_response_time_ms,
                cost_incurred,
                quota_utilization,
            };

            Ok(metrics)
        } else {
            Err(crate::BiomeError::NotFound(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Record usage
    pub async fn record_usage(
        &mut self,
        session_id: &str,
        operation: String,
        success: bool,
        response_time_ms: u64,
        data_size_bytes: u64,
        cost: Option<f64>,
    ) -> BiomeResult<()> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.last_activity = chrono::Utc::now();
            session.requests_made += 1;
            session.data_transferred_mb += data_size_bytes as f64 / (1024.0 * 1024.0);

            let usage_record = UsageRecord {
                record_id: uuid::Uuid::new_v4().to_string(),
                dependency_id: session.dependency_id.clone(),
                user_context: session.user_context.clone(),
                timestamp: chrono::Utc::now(),
                operation,
                success,
                response_time_ms,
                data_size_bytes,
                cost,
            };

            self.usage_history.push(usage_record);
            Ok(())
        } else {
            Err(crate::BiomeError::NotFound(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Get active sessions
    pub fn get_active_sessions(&self) -> Vec<&ActiveSession> {
        self.active_sessions.values().collect()
    }

    /// Get usage history for dependency
    pub fn get_usage_history(&self, dependency_id: &DependencyId) -> Vec<&UsageRecord> {
        self.usage_history
            .iter()
            .filter(|record| &record.dependency_id == dependency_id)
            .collect()
    }

    /// Get usage metrics for a dependency
    pub fn get_usage_metrics(
        &self,
        dependency_id: &DependencyId,
        period: &ReportPeriod,
    ) -> UsageMetrics {
        let records: Vec<&UsageRecord> = self
            .usage_history
            .iter()
            .filter(|record| {
                &record.dependency_id == dependency_id
                    && record.timestamp >= period.start_date
                    && record.timestamp <= period.end_date
            })
            .collect();

        let request_count = records.len() as u64;
        let data_transferred_gb = records
            .iter()
            .map(|r| r.data_size_bytes as f64)
            .sum::<f64>()
            / (1024.0 * 1024.0 * 1024.0);

        let error_rate = if request_count > 0 {
            records.iter().filter(|r| !r.success).count() as f64 / request_count as f64
        } else {
            0.0
        };

        let average_response_time_ms = if request_count > 0 {
            records
                .iter()
                .map(|r| r.response_time_ms as f64)
                .sum::<f64>()
                / request_count as f64
        } else {
            0.0
        };

        let cost_incurred = records.iter().filter_map(|r| r.cost).sum::<f64>();

        // Calculate actual peak concurrency
        let peak_concurrency = self.calculate_peak_concurrency(dependency_id, period);

        // Calculate quota utilization against limits
        let quota_utilization = self.calculate_quota_utilization(dependency_id, std::time::Duration::from_secs(3600));

        UsageMetrics {
            dependency_id: dependency_id.clone(),
            reporting_period: period.clone(),
            request_count,
            data_transferred_gb,
            peak_concurrency,
            error_rate,
            average_response_time_ms,
            cost_incurred,
            quota_utilization,
        }
    }

    /// Setup rate limiter for dependency
    pub fn setup_rate_limiter(&mut self, dependency_id: DependencyId, limits: Vec<RateLimit>) {
        let rate_limiter = RateLimiter {
            dependency_id: dependency_id.clone(),
            limits,
            current_usage: HashMap::new(),
            reset_times: HashMap::new(),
        };

        self.rate_limiters.insert(dependency_id, rate_limiter);
    }

    /// Check rate limit
    pub fn check_rate_limit(
        &mut self,
        dependency_id: &DependencyId,
        resource: &str,
    ) -> RateLimitResult {
        if let Some(limiter) = self.rate_limiters.get_mut(dependency_id) {
            limiter.check_limit(resource)
        } else {
            RateLimitResult::Allowed
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(
        &mut self,
        max_idle_minutes: u64,
    ) -> BiomeResult<Vec<String>> {
        let now = chrono::Utc::now();
        let max_idle = chrono::Duration::minutes(max_idle_minutes as i64);

        let expired_sessions: Vec<String> = self
            .active_sessions
            .iter()
            .filter(|(_, session)| now - session.last_activity > max_idle)
            .map(|(session_id, _)| session_id.clone())
            .collect();

        for session_id in &expired_sessions {
            self.active_sessions.remove(session_id);
        }

        Ok(expired_sessions)
    }
    pub fn calculate_peak_concurrency_from_records(&self, _dependency_id: &str, _records: &[UsageRecord]) -> f64 {
        // TODO: Implement peak concurrency calculation
        0.0
    }

    pub fn calculate_peak_concurrency(&self, _dependency_id: &str, _duration: std::time::Duration) -> f64 {
        // TODO: Implement peak concurrency calculation
        0.0
    }

    pub fn calculate_session_cost(&self, _session: &UsageSession) -> f64 {
        // TODO: Implement session cost calculation
        0.0
    }

    pub fn calculate_quota_utilization(&self, _dependency_id: &str, _duration: std::time::Duration) -> f64 {
        // TODO: Implement quota utilization calculation
        0.0
    }

    pub fn calculate_quota_utilization_from_records(&self, _dependency_id: &str, _records: &[UsageRecord], _period: std::time::Duration) -> f64 {
        // TODO: Implement quota utilization calculation
        0.0
    }

}

impl RateLimiter {
    /// Check if request is within rate limit
    pub fn check_limit(&mut self, resource: &str) -> RateLimitResult {
        let now = chrono::Utc::now();

        // Find applicable limit
        if let Some(limit) = self.limits.iter().find(|l| l.resource == resource) {
            let key = format!("{}:{}", self.dependency_id, resource);

            // Check if reset time has passed
            if let Some(reset_time) = self.reset_times.get(&key) {
                if now > *reset_time {
                    self.current_usage.insert(key.clone(), 0);
                    self.reset_times
                        .insert(key.clone(), self.calculate_reset_time(&limit.period));
                }
            } else {
                self.reset_times
                    .insert(key.clone(), self.calculate_reset_time(&limit.period));
            }

            let current = self.current_usage.get(&key).unwrap_or(&0);

            if *current >= limit.limit {
                RateLimitResult::Exceeded {
                    limit: limit.limit,
                    current: *current,
                    reset_time: self.reset_times.get(&key).cloned().unwrap_or(now),
                }
            } else {
                self.current_usage.insert(key, current + 1);
                RateLimitResult::Allowed
            }
        } else {
            RateLimitResult::Allowed
        }
    }

    /// Calculate reset time based on period
    fn calculate_reset_time(&self, period: &str) -> chrono::DateTime<chrono::Utc> {
        let now = chrono::Utc::now();
        match period {
            "hourly" => now + chrono::Duration::hours(1),
            "daily" => now + chrono::Duration::days(1),
            "weekly" => now + chrono::Duration::weeks(1),
            "monthly" => now + chrono::Duration::days(30),
            _ => now + chrono::Duration::hours(1), // Default to hourly
        }
    }
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed,
    Exceeded {
        limit: u64,
        current: u64,
        reset_time: chrono::DateTime<chrono::Utc>,
    },
}

/// Configuration for monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable usage monitoring
    pub enabled: bool,

    /// Session timeout in minutes
    pub session_timeout_minutes: u64,

    /// History retention days
    pub history_retention_days: u32,

    /// Rate limiting enabled
    pub rate_limiting_enabled: bool,

    /// Metrics collection interval
    pub metrics_interval_seconds: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            session_timeout_minutes: 30,
            history_retention_days: 90,
            rate_limiting_enabled: true,
            metrics_interval_seconds: 60,
        }
    }
}
