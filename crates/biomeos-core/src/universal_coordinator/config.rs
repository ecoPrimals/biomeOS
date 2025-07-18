//! Configuration for the universal biome coordinator
//!
//! This module contains configuration structures and their default implementations
//! for the universal biome coordinator system.

use std::time::Duration;
use super::types::{CoordinatorConfig, MatchingConfig, ScoringWeights, RoutingStrategy};

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: Duration::from_secs(30),
            deployment_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(5),
            auto_discovery: true,
            discovery_refresh_interval: Duration::from_secs(60),
        }
    }
}

impl CoordinatorConfig {
    /// Create a new configuration with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration for development environment
    pub fn development() -> Self {
        Self {
            discovery_timeout: Duration::from_secs(10),
            deployment_timeout: Duration::from_secs(120),
            health_check_interval: Duration::from_secs(15),
            retry_attempts: 5,
            retry_delay: Duration::from_secs(2),
            auto_discovery: true,
            discovery_refresh_interval: Duration::from_secs(30),
        }
    }

    /// Create a configuration for production environment
    pub fn production() -> Self {
        Self {
            discovery_timeout: Duration::from_secs(60),
            deployment_timeout: Duration::from_secs(600),
            health_check_interval: Duration::from_secs(60),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(10),
            auto_discovery: false,
            discovery_refresh_interval: Duration::from_secs(300),
        }
    }

    /// Set discovery timeout
    pub fn with_discovery_timeout(mut self, timeout: Duration) -> Self {
        self.discovery_timeout = timeout;
        self
    }

    /// Set deployment timeout
    pub fn with_deployment_timeout(mut self, timeout: Duration) -> Self {
        self.deployment_timeout = timeout;
        self
    }

    /// Set health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Set retry attempts
    pub fn with_retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Enable or disable auto-discovery
    pub fn with_auto_discovery(mut self, enabled: bool) -> Self {
        self.auto_discovery = enabled;
        self
    }

    /// Set discovery refresh interval
    pub fn with_discovery_refresh_interval(mut self, interval: Duration) -> Self {
        self.discovery_refresh_interval = interval;
        self
    }
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

impl RoutingStrategy {
    /// Get all available routing strategies
    pub fn all() -> Vec<Self> {
        vec![
            Self::RoundRobin,
            Self::LeastConnections,
            Self::LeastLatency,
            Self::Random,
            Self::Weighted,
        ]
    }

    /// Get the strategy name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RoundRobin => "round_robin",
            Self::LeastConnections => "least_connections",
            Self::LeastLatency => "least_latency",
            Self::Random => "random",
            Self::Weighted => "weighted",
        }
    }

    /// Parse a routing strategy from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "round_robin" | "roundrobin" => Some(Self::RoundRobin),
            "least_connections" | "leastconnections" => Some(Self::LeastConnections),
            "least_latency" | "leastlatency" => Some(Self::LeastLatency),
            "random" => Some(Self::Random),
            "weighted" => Some(Self::Weighted),
            _ => None,
        }
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            capability_match: 0.4,
            version_match: 0.2,
            performance_match: 0.2,
            resource_match: 0.1,
            availability_match: 0.1,
        }
    }
}

impl ScoringWeights {
    /// Create balanced weights (all equal)
    pub fn balanced() -> Self {
        Self {
            capability_match: 0.2,
            version_match: 0.2,
            performance_match: 0.2,
            resource_match: 0.2,
            availability_match: 0.2,
        }
    }

    /// Create capability-focused weights
    pub fn capability_focused() -> Self {
        Self {
            capability_match: 0.7,
            version_match: 0.1,
            performance_match: 0.1,
            resource_match: 0.05,
            availability_match: 0.05,
        }
    }

    /// Create performance-focused weights
    pub fn performance_focused() -> Self {
        Self {
            capability_match: 0.3,
            version_match: 0.1,
            performance_match: 0.4,
            resource_match: 0.1,
            availability_match: 0.1,
        }
    }

    /// Validate that weights sum to approximately 1.0
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.capability_match
            + self.version_match
            + self.performance_match
            + self.resource_match
            + self.availability_match;

        if (sum - 1.0).abs() > 0.01 {
            Err(format!("Weights sum to {}, should sum to 1.0", sum))
        } else {
            Ok(())
        }
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(mut self) -> Self {
        let sum = self.capability_match
            + self.version_match
            + self.performance_match
            + self.resource_match
            + self.availability_match;

        if sum > 0.0 {
            self.capability_match /= sum;
            self.version_match /= sum;
            self.performance_match /= sum;
            self.resource_match /= sum;
            self.availability_match /= sum;
        }

        self
    }
}

impl Default for MatchingConfig {
    fn default() -> Self {
        Self {
            min_score: 0.5,
            prefer_exact: true,
            allow_partial: true,
            weights: ScoringWeights::default(),
        }
    }
}

impl MatchingConfig {
    /// Create a strict matching configuration
    pub fn strict() -> Self {
        Self {
            min_score: 0.8,
            prefer_exact: true,
            allow_partial: false,
            weights: ScoringWeights::capability_focused(),
        }
    }

    /// Create a lenient matching configuration
    pub fn lenient() -> Self {
        Self {
            min_score: 0.3,
            prefer_exact: false,
            allow_partial: true,
            weights: ScoringWeights::balanced(),
        }
    }

    /// Create a performance-focused matching configuration
    pub fn performance_focused() -> Self {
        Self {
            min_score: 0.6,
            prefer_exact: false,
            allow_partial: true,
            weights: ScoringWeights::performance_focused(),
        }
    }

    /// Set minimum score threshold
    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_score = score.clamp(0.0, 1.0);
        self
    }

    /// Set preference for exact matches
    pub fn with_prefer_exact(mut self, prefer: bool) -> Self {
        self.prefer_exact = prefer;
        self
    }

    /// Set whether to allow partial matches
    pub fn with_allow_partial(mut self, allow: bool) -> Self {
        self.allow_partial = allow;
        self
    }

    /// Set scoring weights
    pub fn with_weights(mut self, weights: ScoringWeights) -> Self {
        self.weights = weights;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.min_score < 0.0 || self.min_score > 1.0 {
            return Err("min_score must be between 0.0 and 1.0".to_string());
        }

        self.weights.validate()?;

        Ok(())
    }
} 