//! Main predictive health analytics coordinator

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::config::*;
use super::monitoring::*;
use super::predictions::*;
use super::recommendations::*;
use super::reports::*;
use super::trends::*;
use super::types::*;
use crate::{
    message_utils, BiomeError, BiomeResult, CrossPrimalCoordinator, HealthStatus, NetworkLocation,
    PrimalContext, PrimalHealth, PrimalIdentity, SecurityLevel,
};

/// Health analytics coordinator for predictive monitoring
pub struct PredictiveHealthAnalytics {
    /// Cross-primal coordinator for health queries
    coordinator: Arc<CrossPrimalCoordinator>,
    /// Health history storage
    health_history: Arc<RwLock<HashMap<String, VecDeque<HealthSnapshot>>>>,
    /// Trend analysis engine
    trend_analyzer: Arc<TrendAnalyzer>,
    /// Prediction engine
    prediction_engine: Arc<PredictionEngine>,
    /// Analytics configuration
    config: HealthAnalyticsConfig,
    /// Active health monitoring sessions
    monitoring_sessions: Arc<RwLock<HashMap<String, MonitoringSession>>>,
}

impl PredictiveHealthAnalytics {
    /// Create new predictive health analytics system
    pub fn new(coordinator: Arc<CrossPrimalCoordinator>, config: HealthAnalyticsConfig) -> Self {
        let trend_analyzer = Arc::new(TrendAnalyzer::new(config.analysis_algorithms.clone()));
        let prediction_engine = Arc::new(PredictionEngine::new());

        Self {
            coordinator,
            health_history: Arc::new(RwLock::new(HashMap::new())),
            trend_analyzer,
            prediction_engine,
            config,
            monitoring_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start monitoring session for specific primals
    pub async fn start_monitoring_session(
        &self,
        targets: Vec<PrimalIdentity>,
        config: MonitoringConfig,
    ) -> BiomeResult<String> {
        let session_id = Uuid::new_v4().to_string();

        let session = MonitoringSession {
            session_id: session_id.clone(),
            targets,
            config,
            status: MonitoringStatus::Active,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics: vec!["health_score".to_string(), "cpu_usage".to_string()],
            alert_conditions: vec![],
            analysis_preferences: AnalysisPreferences::default(),
        };

        // Store session
        {
            let mut sessions = self.monitoring_sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        // Start monitoring task
        self.spawn_monitoring_task(session_id.clone()).await?;

        Ok(session_id)
    }

    /// Stop monitoring session
    pub async fn stop_monitoring_session(&self, session_id: &str) -> BiomeResult<()> {
        let mut sessions = self.monitoring_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = MonitoringStatus::Stopped;
        }
        Ok(())
    }

    /// Collect health snapshot for a primal
    pub async fn collect_health_snapshot(
        &self,
        primal_id: &PrimalIdentity,
    ) -> BiomeResult<HealthSnapshot> {
        // Create health query message
        let message = message_utils::create_health_query(
            PrimalIdentity {
                primal_type: "biomeos".to_string(),
                instance_id: "health-analytics".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: HealthStatus::Healthy,
                    health_score: 100.0,
                    last_check: chrono::Utc::now(),
                    details: HashMap::new(),
                    metrics: crate::universal_primal_provider::HealthMetrics {
                        cpu_usage: 0.0,
                        memory_mb: 0.0,
                        response_time_ms: 0.0,
                        error_rate: 0.0,
                        active_connections: 0,
                    },
                },
            },
            primal_id.clone(),
            PrimalContext {
                user_id: "system".to_string(),
                device_id: "health-analytics".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: SecurityLevel::Standard,
                biome_id: None,
                team_id: None,
                metadata: HashMap::new(),
            },
        );

        // Send health query
        let response = self.coordinator.send_message(message).await?;

        // Parse health response
        let health: PrimalHealth = serde_json::from_value(response.payload).map_err(|e| {
            BiomeError::RuntimeError(format!("Failed to parse health response: {}", e))
        })?;

        // Create extended metrics (placeholder - would be collected from actual metrics)
        let extended_metrics = ExtendedHealthMetrics {
            cpu_utilization: 45.0,
            memory_utilization: 60.0,
            network_throughput: 1024.0,
            disk_io_rate: 100.0,
            request_latency: 50.0,
            error_rate: 0.1,
            active_connections: 10,
            queue_depth: 5,
        };

        // Create system context (placeholder - would be collected from system)
        let system_context = SystemContext {
            load_average: 1.5,
            available_memory: 8589934592, // 8GB
            network_conditions: NetworkConditions {
                latency: 10.0,
                packet_loss: 0.01,
                bandwidth: 1000000000, // 1Gbps
                congestion_level: CongestionLevel::Low,
            },
            biome_activity: BiomeActivity {
                active_services: 5,
                request_volume: 100.0,
                data_processing_rate: 1024.0,
                user_activity: ActivityLevel::Medium,
            },
        };

        Ok(HealthSnapshot {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            primal_id: primal_id.instance_id.clone(),
            health,
            extended_metrics,
            system_context,
        })
    }

    /// Analyze health trends for a primal
    pub async fn analyze_trends(&self, primal_id: &str) -> BiomeResult<TrendAnalysis> {
        // Get health history
        let history = self.health_history.read().await;
        let snapshots = history
            .get(primal_id)
            .ok_or_else(|| BiomeError::RuntimeError("No health history found".to_string()))?;

        // Analyze trends using the trend analyzer
        self.trend_analyzer
            .analyze_trends(primal_id, snapshots)
            .await
    }

    /// Generate health predictions
    pub async fn generate_predictions(&self, primal_id: &str) -> BiomeResult<HealthPrediction> {
        // Get health history
        let history = self.health_history.read().await;
        let snapshots = history
            .get(primal_id)
            .ok_or_else(|| BiomeError::RuntimeError("No health history found".to_string()))?;

        // Generate predictions using the prediction engine
        self.prediction_engine
            .generate_predictions(primal_id, snapshots, self.config.prediction_horizon)
            .await
    }

    /// Get comprehensive health report
    pub async fn get_health_report(&self, primal_id: &str) -> BiomeResult<HealthReport> {
        // Collect current health snapshot
        let primal_identity = PrimalIdentity {
            primal_type: "unknown".to_string(),
            instance_id: primal_id.to_string(),
            version: "1.0.0".to_string(),
            endpoint: "localhost".to_string(),
            capabilities: vec![],
            health: PrimalHealth {
                status: HealthStatus::Healthy,
                health_score: 100.0,
                last_check: chrono::Utc::now(),
                details: HashMap::new(),
                metrics: crate::universal_primal_provider::HealthMetrics {
                    cpu_usage: 0.0,
                    memory_mb: 0.0,
                    response_time_ms: 0.0,
                    error_rate: 0.0,
                    active_connections: 0,
                },
            },
        };

        let current_snapshot = self.collect_health_snapshot(&primal_identity).await?;

        // Analyze trends
        let trend_analysis = self.analyze_trends(primal_id).await?;

        // Generate predictions
        let predictions = self.generate_predictions(primal_id).await?;

        // Create comprehensive report
        Ok(HealthReport {
            primal_id: primal_id.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            current_snapshot,
            trend_analysis: trend_analysis.clone(),
            predictions: predictions.clone(),
            overall_status: self.calculate_overall_status(&trend_analysis, &predictions),
            recommendations: self
                .generate_recommendations(&trend_analysis, &predictions)
                .await?,
        })
    }

    /// Calculate overall health status
    fn calculate_overall_status(
        &self,
        trend: &TrendAnalysis,
        predictions: &HealthPrediction,
    ) -> OverallHealthStatus {
        // Simple status calculation based on trends and predictions
        let trend_score = match trend.trend_direction {
            TrendDirection::Improving => 90.0,
            TrendDirection::Stable => 75.0,
            TrendDirection::Degrading => 40.0,
            TrendDirection::Volatile => 50.0,
            TrendDirection::Unknown => 60.0,
        };

        let prediction_score = if let Some(first_prediction) = predictions.predicted_scores.first()
        {
            first_prediction.score
        } else {
            60.0
        };

        let combined_score = (trend_score + prediction_score) / 2.0;

        if combined_score >= 80.0 {
            OverallHealthStatus::Excellent
        } else if combined_score >= 60.0 {
            OverallHealthStatus::Good
        } else if combined_score >= 40.0 {
            OverallHealthStatus::Fair
        } else if combined_score >= 20.0 {
            OverallHealthStatus::Poor
        } else {
            OverallHealthStatus::Critical
        }
    }

    /// Generate health recommendations
    async fn generate_recommendations(
        &self,
        trend: &TrendAnalysis,
        predictions: &HealthPrediction,
    ) -> BiomeResult<Vec<HealthRecommendation>> {
        let mut recommendations = Vec::new();

        // Add trend-based recommendations
        match trend.trend_direction {
            TrendDirection::Degrading => {
                recommendations.push(HealthRecommendation {
                    recommendation_type: RecommendationType::Optimize,
                    priority: RecommendationPriority::High,
                    description: "Health trend is degrading. Consider optimization or maintenance."
                        .to_string(),
                    expected_impact: 25.0,
                    implementation_effort: ImplementationEffort::Medium,
                    timeline: 3600, // 1 hour
                });
            }
            TrendDirection::Volatile => {
                recommendations.push(HealthRecommendation {
                    recommendation_type: RecommendationType::Configuration,
                    priority: RecommendationPriority::Medium,
                    description:
                        "Health is volatile. Review configuration and resource allocation."
                            .to_string(),
                    expected_impact: 15.0,
                    implementation_effort: ImplementationEffort::Low,
                    timeline: 1800, // 30 minutes
                });
            }
            _ => {}
        }

        // Add prediction-based recommendations
        for risk in &predictions.risk_assessments {
            match risk.risk_type {
                RiskType::ResourceExhaustion => {
                    recommendations.push(HealthRecommendation {
                        recommendation_type: RecommendationType::ScaleUp,
                        priority: RecommendationPriority::High,
                        description: "Resource exhaustion predicted. Scale up resources."
                            .to_string(),
                        expected_impact: 30.0,
                        implementation_effort: ImplementationEffort::Medium,
                        timeline: risk.timeline,
                    });
                }
                RiskType::ServiceFailure => {
                    recommendations.push(HealthRecommendation {
                        recommendation_type: RecommendationType::Maintenance,
                        priority: RecommendationPriority::Critical,
                        description: "Service failure risk detected. Perform maintenance."
                            .to_string(),
                        expected_impact: 50.0,
                        implementation_effort: ImplementationEffort::High,
                        timeline: risk.timeline,
                    });
                }
                _ => {}
            }
        }

        Ok(recommendations)
    }

    /// Spawn monitoring task for a session
    async fn spawn_monitoring_task(&self, session_id: String) -> BiomeResult<()> {
        let coordinator = self.coordinator.clone();
        let health_history = self.health_history.clone();
        let monitoring_sessions = self.monitoring_sessions.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                // Check if session is still active
                let session = {
                    let sessions = monitoring_sessions.read().await;
                    if let Some(session) = sessions.get(&session_id) {
                        if matches!(session.status, MonitoringStatus::Active) {
                            session.clone()
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                };

                // Collect health snapshots for all targets
                for target in &session.targets {
                    // Create health query message
                    let message = message_utils::create_health_query(
                        PrimalIdentity {
                            primal_type: "biomeos".to_string(),
                            instance_id: "health-analytics".to_string(),
                            version: "1.0.0".to_string(),
                            endpoint: "localhost".to_string(),
                            capabilities: vec![],
                            health: PrimalHealth {
                                status: HealthStatus::Healthy,
                                health_score: 100.0,
                                last_check: chrono::Utc::now(),
                                details: HashMap::new(),
                                metrics: crate::universal_primal_provider::HealthMetrics {
                                    cpu_usage: 0.0,
                                    memory_mb: 0.0,
                                    response_time_ms: 0.0,
                                    error_rate: 0.0,
                                    active_connections: 0,
                                },
                            },
                        },
                        target.clone(),
                        PrimalContext {
                            user_id: "system".to_string(),
                            device_id: "health-analytics".to_string(),
                            session_id: Uuid::new_v4().to_string(),
                            network_location: NetworkLocation {
                                ip_address: "127.0.0.1".to_string(),
                                subnet: None,
                                network_id: None,
                                geo_location: None,
                            },
                            security_level: SecurityLevel::Standard,
                            biome_id: None,
                            team_id: None,
                            metadata: HashMap::new(),
                        },
                    );

                    // Send health query and collect snapshot
                    if let Ok(response) = coordinator.send_message(message).await {
                        if let Ok(health) = serde_json::from_value::<PrimalHealth>(response.payload)
                        {
                            let snapshot = HealthSnapshot {
                                timestamp: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                primal_id: target.instance_id.clone(),
                                health,
                                extended_metrics: ExtendedHealthMetrics {
                                    cpu_utilization: 45.0,
                                    memory_utilization: 60.0,
                                    network_throughput: 1024.0,
                                    disk_io_rate: 100.0,
                                    request_latency: 50.0,
                                    error_rate: 0.1,
                                    active_connections: 10,
                                    queue_depth: 5,
                                },
                                system_context: SystemContext {
                                    load_average: 1.5,
                                    available_memory: 8589934592,
                                    network_conditions: NetworkConditions {
                                        latency: 10.0,
                                        packet_loss: 0.01,
                                        bandwidth: 1000000000,
                                        congestion_level: CongestionLevel::Low,
                                    },
                                    biome_activity: BiomeActivity {
                                        active_services: 5,
                                        request_volume: 100.0,
                                        data_processing_rate: 1024.0,
                                        user_activity: ActivityLevel::Medium,
                                    },
                                },
                            };

                            // Store snapshot in history
                            {
                                let mut history = health_history.write().await;
                                let entry = history
                                    .entry(target.instance_id.clone())
                                    .or_insert_with(VecDeque::new);
                                entry.push_back(snapshot);

                                // Maintain history size limit
                                while entry.len() > config.history_size {
                                    entry.pop_front();
                                }
                            }
                        }
                    }
                }

                // Wait for next monitoring interval
                tokio::time::sleep(std::time::Duration::from_secs(session.config.interval)).await;
            }
        });

        Ok(())
    }
}
