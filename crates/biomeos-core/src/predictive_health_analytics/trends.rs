//! Trend analysis engine for health analytics

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::*;
use super::types::*;
use crate::BiomeResult;

/// Trend analysis engine
pub struct TrendAnalyzer {
    /// Analysis algorithms
    algorithms: Vec<AnalysisAlgorithm>,
    /// Trend cache
    trend_cache: Arc<RwLock<HashMap<String, TrendAnalysis>>>,
}

/// Trend analysis results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrendAnalysis {
    /// Primal identifier
    pub primal_id: String,
    /// Analysis timestamp
    pub timestamp: u64,
    /// Trend direction
    pub trend_direction: TrendDirection,
    /// Trend strength
    pub trend_strength: f64,
    /// Trend confidence
    pub confidence: f64,
    /// Trend components
    pub components: TrendComponents,
    /// Anomalies detected
    pub anomalies: Vec<Anomaly>,
}

impl TrendAnalyzer {
    /// Create new trend analyzer
    pub fn new(algorithms: Vec<AnalysisAlgorithm>) -> Self {
        Self {
            algorithms,
            trend_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze trends for a primal
    pub async fn analyze_trends(
        &self,
        primal_id: &str,
        snapshots: &VecDeque<HealthSnapshot>,
    ) -> BiomeResult<TrendAnalysis> {
        if snapshots.is_empty() {
            return Err(crate::BiomeError::RuntimeError(
                "No health snapshots available".to_string(),
            ));
        }

        // Extract health scores
        let scores: Vec<f64> = snapshots.iter().map(|s| s.health.health_score).collect();

        // Calculate trend direction
        let trend_direction = self.calculate_trend_direction(&scores);

        // Calculate trend strength
        let trend_strength = self.calculate_trend_strength(&scores);

        // Calculate confidence
        let confidence = self.calculate_confidence(&scores);

        // Analyze trend components
        let components = self.analyze_trend_components(&scores);

        // Detect anomalies
        let anomalies = self.detect_anomalies(snapshots).await?;

        let analysis = TrendAnalysis {
            primal_id: primal_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            trend_direction,
            trend_strength,
            confidence,
            components,
            anomalies,
        };

        // Cache the analysis
        {
            let mut cache = self.trend_cache.write().await;
            cache.insert(primal_id.to_string(), analysis.clone());
        }

        Ok(analysis)
    }

    /// Calculate trend direction from scores
    pub fn calculate_trend_direction(&self, scores: &[f64]) -> TrendDirection {
        if scores.len() < 2 {
            return TrendDirection::Unknown;
        }

        let first_half = &scores[..scores.len() / 2];
        let second_half = &scores[scores.len() / 2..];

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let variance = self.calculate_variance(scores);

        // High variance indicates volatility
        if variance > 100.0 {
            return TrendDirection::Volatile;
        }

        let diff = second_avg - first_avg;

        if diff > 5.0 {
            TrendDirection::Improving
        } else if diff < -5.0 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate trend strength
    fn calculate_trend_strength(&self, scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }

        // Simple linear regression slope as strength indicator
        let n = scores.len() as f64;
        let x_sum = (0..scores.len()).map(|i| i as f64).sum::<f64>();
        let y_sum = scores.iter().sum::<f64>();
        let xy_sum = scores
            .iter()
            .enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum::<f64>();
        let x_sq_sum = (0..scores.len()).map(|i| (i as f64).powi(2)).sum::<f64>();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x_sq_sum - x_sum.powi(2));
        slope.abs()
    }

    /// Calculate confidence in trend analysis
    fn calculate_confidence(&self, scores: &[f64]) -> f64 {
        if scores.len() < 3 {
            return 0.5;
        }

        // Use R-squared as confidence measure
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let ss_tot = scores.iter().map(|&x| (x - mean).powi(2)).sum::<f64>();

        if ss_tot == 0.0 {
            return 1.0;
        }

        // Simple linear regression
        let n = scores.len() as f64;
        let x_sum = (0..scores.len()).map(|i| i as f64).sum::<f64>();
        let y_sum = scores.iter().sum::<f64>();
        let xy_sum = scores
            .iter()
            .enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum::<f64>();
        let x_sq_sum = (0..scores.len()).map(|i| (i as f64).powi(2)).sum::<f64>();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x_sq_sum - x_sum.powi(2));
        let intercept = (y_sum - slope * x_sum) / n;

        let ss_res = scores
            .iter()
            .enumerate()
            .map(|(i, &y)| (y - (slope * i as f64 + intercept)).powi(2))
            .sum::<f64>();

        let r_squared = 1.0 - (ss_res / ss_tot);
        r_squared.clamp(0.0, 1.0)
    }

    /// Calculate variance
    pub fn calculate_variance(&self, scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }

        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance =
            scores.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (scores.len() - 1) as f64;
        variance
    }

    /// Analyze trend components
    fn analyze_trend_components(&self, scores: &[f64]) -> TrendComponents {
        if scores.len() < 4 {
            return TrendComponents {
                long_term: 0.0,
                seasonal: 0.0,
                cyclical: 0.0,
                irregular: 0.0,
            };
        }

        // Simple trend decomposition
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;

        // Long-term trend (linear regression slope)
        let n = scores.len() as f64;
        let x_sum = (0..scores.len()).map(|i| i as f64).sum::<f64>();
        let y_sum = scores.iter().sum::<f64>();
        let xy_sum = scores
            .iter()
            .enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum::<f64>();
        let x_sq_sum = (0..scores.len()).map(|i| (i as f64).powi(2)).sum::<f64>();

        let long_term = (n * xy_sum - x_sum * y_sum) / (n * x_sq_sum - x_sum.powi(2));

        // Seasonal component (simplified)
        let seasonal = if scores.len() >= 12 {
            let period = 12.min(scores.len() / 2);
            let seasonal_sum: f64 = (0..period)
                .map(|i| {
                    let indices: Vec<usize> = (i..scores.len()).step_by(period).collect();
                    let seasonal_values: Vec<f64> =
                        indices.iter().map(|&idx| scores[idx]).collect();
                    let seasonal_mean =
                        seasonal_values.iter().sum::<f64>() / seasonal_values.len() as f64;
                    (seasonal_mean - mean).abs()
                })
                .sum();
            seasonal_sum / period as f64
        } else {
            0.0
        };

        // Cyclical component (placeholder)
        let cyclical = 0.0;

        // Irregular component (residual variance)
        let irregular = self.calculate_variance(scores).sqrt();

        TrendComponents {
            long_term,
            seasonal,
            cyclical,
            irregular,
        }
    }

    /// Detect anomalies in health snapshots
    async fn detect_anomalies(
        &self,
        snapshots: &VecDeque<HealthSnapshot>,
    ) -> BiomeResult<Vec<Anomaly>> {
        let mut anomalies = Vec::new();

        if snapshots.len() < 10 {
            return Ok(anomalies);
        }

        let scores: Vec<f64> = snapshots.iter().map(|s| s.health.health_score).collect();
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let std_dev = self.calculate_variance(&scores).sqrt();

        // Z-score based anomaly detection
        for snapshot in snapshots.iter() {
            let z_score = (snapshot.health.health_score - mean) / std_dev;

            if z_score.abs() > 2.0 {
                let anomaly_type = if z_score > 0.0 {
                    AnomalyType::Spike
                } else {
                    AnomalyType::Drop
                };

                anomalies.push(Anomaly {
                    timestamp: snapshot.timestamp,
                    anomaly_type,
                    severity: z_score.abs() / 3.0, // Normalize to 0-1 range
                    affected_metrics: vec!["health_score".to_string()],
                    description: format!(
                        "Health score anomaly detected: {:.2} (z-score: {:.2})",
                        snapshot.health.health_score, z_score
                    ),
                });
            }
        }

        Ok(anomalies)
    }

    /// Get cached trend analysis
    pub async fn get_cached_analysis(&self, primal_id: &str) -> Option<TrendAnalysis> {
        let cache = self.trend_cache.read().await;
        cache.get(primal_id).cloned()
    }

    /// Clear trend cache
    pub async fn clear_cache(&self) {
        let mut cache = self.trend_cache.write().await;
        cache.clear();
    }

    /// Get available analysis algorithms
    pub fn get_available_algorithms(&self) -> &[AnalysisAlgorithm] {
        &self.algorithms
    }

    /// Add analysis algorithm
    pub fn add_algorithm(&mut self, algorithm: AnalysisAlgorithm) {
        self.algorithms.push(algorithm);
    }

    /// Apply specific algorithm to analyze trends
    pub async fn apply_algorithm(&self, algorithm_index: usize, snapshots: &[HealthSnapshot]) -> Option<TrendAnalysis> {
        if let Some(algorithm) = self.algorithms.get(algorithm_index) {
            // Apply the specific algorithm to analyze trends
            match algorithm {
                AnalysisAlgorithm::LinearRegression => self.apply_linear_regression(snapshots).await,
                AnalysisAlgorithm::MovingAverage => self.apply_moving_average(snapshots).await,
                AnalysisAlgorithm::ExponentialSmoothing => self.apply_exponential_analysis(snapshots).await,
                AnalysisAlgorithm::SeasonalDecomposition => self.apply_seasonal_analysis(snapshots).await,
                AnalysisAlgorithm::AnomalyDetection => self.apply_anomaly_analysis(snapshots).await,
                AnalysisAlgorithm::ResourceCorrelation => self.apply_correlation_analysis(snapshots).await,
            }
        } else {
            None
        }
    }

    async fn apply_linear_regression(&self, _snapshots: &[HealthSnapshot]) -> Option<TrendAnalysis> {
        // Mock implementation
        Some(TrendAnalysis {
            primal_id: "default".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            trend_direction: TrendDirection::Stable,
            trend_strength: 0.5,
            components: crate::predictive_health_analytics::types::TrendComponents {
                long_term: 0.0,
                seasonal: 0.0,
                cyclical: 0.0,
                irregular: 0.0,
            },
            anomalies: Vec::new(),
            confidence: 0.7,
        })
    }

    async fn apply_moving_average(&self, _snapshots: &[HealthSnapshot]) -> Option<TrendAnalysis> {
        // Mock implementation
        Some(TrendAnalysis {
            primal_id: "default".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            trend_direction: TrendDirection::Improving,
            trend_strength: 0.3,
            components: crate::predictive_health_analytics::types::TrendComponents {
                long_term: 0.1,
                seasonal: 0.0,
                cyclical: 0.0,
                irregular: 0.0,
            },
            anomalies: Vec::new(),
            confidence: 0.6,
        })
    }

    async fn apply_exponential_analysis(&self, _snapshots: &[HealthSnapshot]) -> Option<TrendAnalysis> {
        // Mock implementation
        Some(TrendAnalysis {
            primal_id: "default".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            trend_direction: TrendDirection::Degrading,
            trend_strength: 0.4,
            components: crate::predictive_health_analytics::types::TrendComponents {
                long_term: -0.1,
                seasonal: 0.0,
                cyclical: 0.0,
                irregular: 0.0,
            },
            anomalies: Vec::new(),
            confidence: 0.8,
        })
    }

    async fn apply_seasonal_analysis(&self, _snapshots: &[HealthSnapshot]) -> Option<TrendAnalysis> {
        // Mock implementation
        Some(TrendAnalysis {
            primal_id: "default".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            trend_direction: TrendDirection::Stable,
            trend_strength: 0.2,
            components: crate::predictive_health_analytics::types::TrendComponents {
                long_term: 0.0,
                seasonal: 0.2,
                cyclical: 0.0,
                irregular: 0.0,
            },
            anomalies: Vec::new(),
            confidence: 0.5,
        })
    }

    async fn apply_anomaly_analysis(&self, _snapshots: &[HealthSnapshot]) -> Option<TrendAnalysis> {
        // Mock implementation
        Some(TrendAnalysis {
            primal_id: "default".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            trend_direction: TrendDirection::Stable,
            trend_strength: 0.1,
            components: crate::predictive_health_analytics::types::TrendComponents {
                long_term: 0.0,
                seasonal: 0.0,
                cyclical: 0.0,
                irregular: 0.1,
            },
            anomalies: Vec::new(),
            confidence: 0.9,
        })
    }

    async fn apply_correlation_analysis(&self, _snapshots: &[HealthSnapshot]) -> Option<TrendAnalysis> {
        // Mock implementation
        Some(TrendAnalysis {
            primal_id: "default".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            trend_direction: TrendDirection::Improving,
            trend_strength: 0.6,
            components: crate::predictive_health_analytics::types::TrendComponents {
                long_term: 0.3,
                seasonal: 0.0,
                cyclical: 0.2,
                irregular: 0.1,
            },
            anomalies: Vec::new(),
            confidence: 0.7,
        })
    }
}
