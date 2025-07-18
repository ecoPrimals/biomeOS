//! API contract middleware and enforcement

use super::traits::*;
use super::types::*;
use super::validation::*;
use crate::BiomeResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// API contract middleware for request/response processing
pub struct ApiContractMiddleware {
    /// Validator instance
    validator: Arc<ApiContractValidator>,
    /// Metrics collector
    metrics_collector: Arc<dyn ApiMetricsCollector>,
}

/// Metrics data structure
#[derive(Debug, Clone)]
pub struct ApiMetrics {
    /// Total requests per endpoint
    pub requests_per_endpoint: HashMap<String, u64>,
    /// Total errors per endpoint
    pub errors_per_endpoint: HashMap<String, u64>,
    /// Average response time per endpoint
    pub avg_response_time: HashMap<String, f64>,
    /// Error categories
    pub error_categories: HashMap<String, u64>,
    /// Validation errors
    pub validation_errors: HashMap<String, u64>,
    /// Status code distribution
    pub status_codes: HashMap<u16, u64>,
    /// Request methods distribution
    pub methods: HashMap<String, u64>,
    /// Last update timestamp
    pub last_updated: u64,
}

/// Default metrics collector implementation
pub struct DefaultApiMetricsCollector {
    /// Metrics storage
    metrics: Arc<RwLock<ApiMetrics>>,
    /// Request duration tracking
    duration_tracker: Arc<RwLock<HashMap<String, Vec<u64>>>>,
}

impl DefaultApiMetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ApiMetrics {
                requests_per_endpoint: HashMap::new(),
                errors_per_endpoint: HashMap::new(),
                avg_response_time: HashMap::new(),
                error_categories: HashMap::new(),
                validation_errors: HashMap::new(),
                status_codes: HashMap::new(),
                methods: HashMap::new(),
                last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            })),
            duration_tracker: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> ApiMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) -> BiomeResult<()> {
        let mut metrics = self.metrics.write().await;
        *metrics = ApiMetrics {
            requests_per_endpoint: HashMap::new(),
            errors_per_endpoint: HashMap::new(),
            avg_response_time: HashMap::new(),
            error_categories: HashMap::new(),
            validation_errors: HashMap::new(),
            status_codes: HashMap::new(),
            methods: HashMap::new(),
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        let mut duration_tracker = self.duration_tracker.write().await;
        duration_tracker.clear();
        
        Ok(())
    }

    /// Update average response time
    async fn update_average_response_time(&self, endpoint: &str, duration_ms: u64) {
        let mut duration_tracker = self.duration_tracker.write().await;
        let durations = duration_tracker.entry(endpoint.to_string()).or_insert_with(Vec::new);
        durations.push(duration_ms);
        
        // Keep only last 100 measurements per endpoint
        if durations.len() > 100 {
            durations.remove(0);
        }
        
        // Calculate average
        let average = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
        
        let mut metrics = self.metrics.write().await;
        metrics.avg_response_time.insert(endpoint.to_string(), average);
        metrics.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    }
}

impl ApiMetricsCollector for DefaultApiMetricsCollector {
    async fn record_api_call(
        &self,
        endpoint: &str,
        method: &str,
        status_code: u16,
        duration_ms: u64,
    ) {
        // Update metrics with actual data collection
        {
            let mut metrics = self.metrics.write().await;
            
            // Increment request count per endpoint
            *metrics.requests_per_endpoint.entry(endpoint.to_string()).or_insert(0) += 1;
            
            // Track status code distribution
            *metrics.status_codes.entry(status_code).or_insert(0) += 1;
            
            // Track HTTP method distribution
            *metrics.methods.entry(method.to_string()).or_insert(0) += 1;
            
            // Update last updated timestamp
            metrics.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        }
        
        // Update average response time
        self.update_average_response_time(endpoint, duration_ms).await;
        
        // Log the event
        tracing::info!(
            endpoint = endpoint,
            method = method,
            status_code = status_code,
            duration_ms = duration_ms,
            "API call recorded with metrics"
        );
    }

    async fn record_api_error(&self, endpoint: &str, method: &str, error_category: &ErrorCategory) {
        // Update error metrics with actual data collection
        {
            let mut metrics = self.metrics.write().await;
            
            // Increment error count per endpoint
            *metrics.errors_per_endpoint.entry(endpoint.to_string()).or_insert(0) += 1;
            
            // Track error category distribution
            let category_str = match error_category {
                ErrorCategory::Validation => "validation",
                ErrorCategory::Authentication => "authentication",
                ErrorCategory::Authorization => "authorization",
                ErrorCategory::NotFound => "not_found",
                ErrorCategory::Conflict => "conflict",
                ErrorCategory::RateLimit => "rate_limit",
                ErrorCategory::ServiceUnavailable => "service_unavailable",
                ErrorCategory::InternalError => "internal_error",
                ErrorCategory::NetworkError => "network_error",
                ErrorCategory::Timeout => "timeout",
                ErrorCategory::Configuration => "configuration",
                ErrorCategory::ResourceExhausted => "resource_exhausted",
            };
            
            *metrics.error_categories.entry(category_str.to_string()).or_insert(0) += 1;
            
            // Update last updated timestamp
            metrics.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        }
        
        // Log the error
        tracing::error!(
            endpoint = endpoint,
            method = method,
            error_category = ?error_category,
            "API error recorded with metrics"
        );
    }

    async fn record_validation_error(&self, endpoint: &str, validation_type: &str) {
        // Update validation error metrics with actual data collection
        {
            let mut metrics = self.metrics.write().await;
            
            // Track validation error types
            let validation_key = format!("{}:{}", endpoint, validation_type);
            *metrics.validation_errors.entry(validation_key).or_insert(0) += 1;
            
            // Also count as general error
            *metrics.errors_per_endpoint.entry(endpoint.to_string()).or_insert(0) += 1;
            
            // Update last updated timestamp
            metrics.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        }
        
        // Log the validation error
        tracing::error!(
            endpoint = endpoint,
            validation_type = validation_type,
            "Validation error recorded with metrics"
        );
    }
}
