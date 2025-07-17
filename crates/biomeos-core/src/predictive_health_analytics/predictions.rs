//! Prediction engine for health forecasting

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::*;
use crate::BiomeResult;

/// Prediction engine for health forecasting
pub struct PredictionEngine {
    /// Prediction models
    models: Arc<RwLock<HashMap<String, PredictionModel>>>,
    /// Prediction cache
    prediction_cache: Arc<RwLock<HashMap<String, HealthPrediction>>>,
}

/// Prediction model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PredictionModel {
    /// Model identifier
    pub id: String,
    /// Model type
    pub model_type: ModelType,
    /// Model parameters
    pub parameters: HashMap<String, f64>,
    /// Model accuracy metrics
    pub accuracy: ModelAccuracy,
    /// Training data size
    pub training_data_size: usize,
    /// Last update timestamp
    pub last_update: u64,
}

/// Health prediction results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthPrediction {
    /// Primal identifier
    pub primal_id: String,
    /// Prediction timestamp
    pub timestamp: u64,
    /// Predicted health scores
    pub predicted_scores: Vec<PredictedScore>,
    /// Confidence intervals
    pub confidence_intervals: Vec<ConfidenceInterval>,
    /// Risk assessments
    pub risk_assessments: Vec<RiskAssessment>,
    /// Prediction horizon in seconds
    pub prediction_horizon: u64,
    /// Model used for prediction
    pub model_id: String,
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictionEngine {
    /// Create new prediction engine
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            prediction_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate health predictions
    pub async fn generate_predictions(
        &self,
        primal_id: &str,
        snapshots: &VecDeque<HealthSnapshot>,
        horizon: u64,
    ) -> BiomeResult<HealthPrediction> {
        if snapshots.is_empty() {
            return Err(crate::BiomeError::RuntimeError(
                "No health snapshots available".to_string(),
            ));
        }

        // Extract health scores and timestamps
        let scores: Vec<f64> = snapshots.iter().map(|s| s.health.health_score).collect();
        let timestamps: Vec<u64> = snapshots.iter().map(|s| s.timestamp).collect();

        // Get or create prediction model
        let model = self.get_or_create_model(primal_id, &scores).await?;

        // Generate predictions
        let predicted_scores = self
            .predict_scores(&model, &scores, &timestamps, horizon)
            .await?;

        // Calculate confidence intervals
        let confidence_intervals = self.calculate_confidence_intervals(&predicted_scores).await;

        // Assess risks
        let risk_assessments = self.assess_risks(&predicted_scores).await?;

        let prediction = HealthPrediction {
            primal_id: primal_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            predicted_scores,
            confidence_intervals,
            risk_assessments,
            prediction_horizon: horizon,
            model_id: model.id.clone(),
        };

        // Cache the prediction
        {
            let mut cache = self.prediction_cache.write().await;
            cache.insert(primal_id.to_string(), prediction.clone());
        }

        Ok(prediction)
    }

    /// Get or create prediction model for a primal
    async fn get_or_create_model(
        &self,
        primal_id: &str,
        scores: &[f64],
    ) -> BiomeResult<PredictionModel> {
        let mut models = self.models.write().await;

        if let Some(model) = models.get(primal_id) {
            return Ok(model.clone());
        }

        // Create new model
        let model = PredictionModel {
            id: format!("{}_model", primal_id),
            model_type: ModelType::LinearRegression,
            parameters: self.train_linear_model(scores),
            accuracy: self.calculate_model_accuracy(scores),
            training_data_size: scores.len(),
            last_update: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        models.insert(primal_id.to_string(), model.clone());
        Ok(model)
    }

    /// Train linear regression model
    fn train_linear_model(&self, scores: &[f64]) -> HashMap<String, f64> {
        let mut parameters = HashMap::new();

        if scores.len() < 2 {
            parameters.insert("slope".to_string(), 0.0);
            parameters.insert("intercept".to_string(), *scores.first().unwrap_or(&50.0));
            return parameters;
        }

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

        parameters.insert("slope".to_string(), slope);
        parameters.insert("intercept".to_string(), intercept);
        parameters
    }

    /// Calculate model accuracy
    fn calculate_model_accuracy(&self, scores: &[f64]) -> ModelAccuracy {
        if scores.len() < 2 {
            return ModelAccuracy {
                mae: 0.0,
                rmse: 0.0,
                r_squared: 0.0,
                accuracy: 0.5,
            };
        }

        // Simple accuracy calculation for linear model
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance =
            scores.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / scores.len() as f64;

        ModelAccuracy {
            mae: variance.sqrt() * 0.8, // Rough approximation
            rmse: variance.sqrt(),
            r_squared: 0.7, // Placeholder
            accuracy: 0.8,  // Placeholder
        }
    }

    /// Predict health scores
    async fn predict_scores(
        &self,
        model: &PredictionModel,
        scores: &[f64],
        _timestamps: &[u64],
        horizon: u64,
    ) -> BiomeResult<Vec<PredictedScore>> {
        let mut predictions = Vec::new();

        let slope = model.parameters.get("slope").unwrap_or(&0.0);
        let intercept = model.parameters.get("intercept").unwrap_or(&50.0);

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let prediction_points = 10; // Number of prediction points
        let time_step = horizon / prediction_points;

        for i in 1..=prediction_points {
            let future_time = current_time + (i * time_step);
            let x = scores.len() as f64 + i as f64;
            let predicted_score = (slope * x + intercept).clamp(0.0, 100.0);

            // Add some noise based on model accuracy
            let noise = (1.0 - model.accuracy.accuracy) * 10.0;
            let confidence = (model.accuracy.accuracy - noise / 100.0).clamp(0.1, 1.0);

            predictions.push(PredictedScore {
                timestamp: future_time,
                score: predicted_score,
                confidence,
            });
        }

        Ok(predictions)
    }

    /// Calculate confidence intervals
    async fn calculate_confidence_intervals(
        &self,
        predicted_scores: &[PredictedScore],
    ) -> Vec<ConfidenceInterval> {
        let mut intervals = Vec::new();

        for prediction in predicted_scores {
            let margin = (1.0 - prediction.confidence) * 20.0; // Confidence margin

            intervals.push(ConfidenceInterval {
                lower_bound: (prediction.score - margin).max(0.0),
                upper_bound: (prediction.score + margin).min(100.0),
                confidence_level: 95.0,
            });
        }

        intervals
    }

    /// Assess risks based on predictions
    async fn assess_risks(
        &self,
        predicted_scores: &[PredictedScore],
    ) -> BiomeResult<Vec<RiskAssessment>> {
        let mut risks = Vec::new();

        for prediction in predicted_scores {
            // Service failure risk
            if prediction.score < 20.0 {
                risks.push(RiskAssessment {
                    risk_type: RiskType::ServiceFailure,
                    probability: ((20.0 - prediction.score) / 20.0).min(1.0),
                    impact: 90.0,
                    timeline: prediction.timestamp,
                    description: format!(
                        "Service failure risk due to low health score: {:.2}",
                        prediction.score
                    ),
                });
            }

            // Performance degradation risk
            if prediction.score < 50.0 {
                risks.push(RiskAssessment {
                    risk_type: RiskType::PerformanceDegradation,
                    probability: ((50.0 - prediction.score) / 50.0).min(1.0),
                    impact: 60.0,
                    timeline: prediction.timestamp,
                    description: format!("Performance degradation risk: {:.2}", prediction.score),
                });
            }

            // Resource exhaustion risk (example heuristic)
            if prediction.score < 30.0 {
                risks.push(RiskAssessment {
                    risk_type: RiskType::ResourceExhaustion,
                    probability: ((30.0 - prediction.score) / 30.0).min(1.0),
                    impact: 80.0,
                    timeline: prediction.timestamp,
                    description: format!("Resource exhaustion risk: {:.2}", prediction.score),
                });
            }
        }

        Ok(risks)
    }

    /// Update prediction model with new data
    pub async fn update_model(&self, primal_id: &str, new_scores: &[f64]) -> BiomeResult<()> {
        let mut models = self.models.write().await;

        if let Some(model) = models.get_mut(primal_id) {
            model.parameters = self.train_linear_model(new_scores);
            model.accuracy = self.calculate_model_accuracy(new_scores);
            model.training_data_size = new_scores.len();
            model.last_update = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        Ok(())
    }

    /// Get cached prediction
    pub async fn get_cached_prediction(&self, primal_id: &str) -> Option<HealthPrediction> {
        let cache = self.prediction_cache.read().await;
        cache.get(primal_id).cloned()
    }

    /// Clear prediction cache
    pub async fn clear_cache(&self) {
        let mut cache = self.prediction_cache.write().await;
        cache.clear();
    }
}
