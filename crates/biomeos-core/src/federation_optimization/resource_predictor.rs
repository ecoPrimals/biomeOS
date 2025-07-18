//! Resource Predictor for Federation Optimization
//!
//! This module implements predictive analytics for resource demand forecasting
//! across the federation to enable proactive optimization.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::BiomeResult;
use super::types::*;

/// Resource predictor for optimization
pub struct ResourcePredictor {
    /// Prediction models
    models: Arc<RwLock<HashMap<String, PredictionModel>>>,
    /// Historical data
    history: Arc<RwLock<HashMap<String, Vec<ResourceSnapshot>>>>,
}

/// Resource prediction result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourcePrediction {
    /// Predicted resource utilization
    pub predicted_utilization: ResourceUtilization,
    /// Confidence in prediction (0.0-1.0)
    pub confidence: f64,
    /// Prediction horizon in seconds
    pub prediction_horizon: u64,
}

impl Default for ResourcePredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourcePredictor {
    /// Create new resource predictor
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add resource snapshot to history
    pub async fn add_snapshot(&self, primal_id: &str, snapshot: ResourceSnapshot) -> BiomeResult<()> {
        let mut history = self.history.write().await;
        let primal_history = history
            .entry(primal_id.to_string())
            .or_insert_with(Vec::new);
        primal_history.push(snapshot);

        // Keep only recent snapshots (last 1000)
        if primal_history.len() > 1000 {
            primal_history.remove(0);
        }

        // Update prediction model if we have enough data
        if primal_history.len() >= 10 {
            self.update_prediction_model(primal_id, primal_history).await?;
        }

        Ok(())
    }

    /// Predict resource demand
    pub async fn predict_demand(
        &self,
        primal_id: &str,
        horizon: u64,
    ) -> BiomeResult<ResourcePrediction> {
        let history = self.history.read().await;
        let models = self.models.read().await;

        if let Some(primal_history) = history.get(primal_id) {
            if let Some(model) = models.get(primal_id) {
                return self
                    .apply_prediction_model(model, primal_history, horizon)
                    .await;
            }
        }

        // Default prediction if no history or model
        Ok(ResourcePrediction {
            predicted_utilization: ResourceUtilization {
                cpu_usage: 0.5,
                memory_usage: 0.5,
                storage_usage: 0.5,
                network_usage: 0.5,
                gpu_usage: 0.5,
            },
            confidence: 0.1,
            prediction_horizon: horizon,
        })
    }

    /// Get prediction model for a primal
    pub async fn get_prediction_model(&self, primal_id: &str) -> Option<PredictionModel> {
        let models = self.models.read().await;
        models.get(primal_id).cloned()
    }

    /// Update prediction model for a primal
    pub async fn update_prediction_model(
        &self,
        primal_id: &str,
        history: &[ResourceSnapshot],
    ) -> BiomeResult<()> {
        let model = self.train_prediction_model(history).await?;
        let mut models = self.models.write().await;
        models.insert(primal_id.to_string(), model);
        Ok(())
    }

    /// Train prediction model
    async fn train_prediction_model(&self, history: &[ResourceSnapshot]) -> BiomeResult<PredictionModel> {
        // Simple moving average model for now
        let mut parameters = HashMap::new();
        
        if history.len() >= 5 {
            // Calculate moving average parameters
            let window_size = 5.0;
            let recent_snapshots = &history[history.len() - 5..];
            
            let avg_cpu: f64 = recent_snapshots.iter().map(|s| s.utilization.cpu_usage).sum::<f64>() / window_size;
            let avg_memory: f64 = recent_snapshots.iter().map(|s| s.utilization.memory_usage).sum::<f64>() / window_size;
            let avg_storage: f64 = recent_snapshots.iter().map(|s| s.utilization.storage_usage).sum::<f64>() / window_size;
            let avg_network: f64 = recent_snapshots.iter().map(|s| s.utilization.network_usage).sum::<f64>() / window_size;
            let avg_gpu: f64 = recent_snapshots.iter().map(|s| s.utilization.gpu_usage).sum::<f64>() / window_size;
            
            parameters.insert("avg_cpu".to_string(), avg_cpu);
            parameters.insert("avg_memory".to_string(), avg_memory);
            parameters.insert("avg_storage".to_string(), avg_storage);
            parameters.insert("avg_network".to_string(), avg_network);
            parameters.insert("avg_gpu".to_string(), avg_gpu);
        }

        Ok(PredictionModel {
            model_type: ModelType::MovingAverage,
            parameters,
            accuracy: 0.8, // TODO: Calculate real accuracy
            last_training: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Apply prediction model
    async fn apply_prediction_model(
        &self,
        model: &PredictionModel,
        history: &[ResourceSnapshot],
        horizon: u64,
    ) -> BiomeResult<ResourcePrediction> {
        match &model.model_type {
            ModelType::MovingAverage => {
                self.apply_moving_average_model(model, history, horizon).await
            }
            ModelType::LinearRegression => {
                self.apply_linear_regression_model(model, history, horizon).await
            }
            ModelType::ExponentialSmoothing => {
                self.apply_exponential_smoothing_model(model, history, horizon).await
            }
            _ => {
                // Default prediction for unsupported models
                Ok(ResourcePrediction {
                    predicted_utilization: ResourceUtilization {
                        cpu_usage: 0.5,
                        memory_usage: 0.5,
                        storage_usage: 0.5,
                        network_usage: 0.5,
                        gpu_usage: 0.5,
                    },
                    confidence: 0.5,
                    prediction_horizon: horizon,
                })
            }
        }
    }

    /// Apply moving average model
    async fn apply_moving_average_model(
        &self,
        model: &PredictionModel,
        _history: &[ResourceSnapshot],
        horizon: u64,
    ) -> BiomeResult<ResourcePrediction> {
        let predicted_utilization = ResourceUtilization {
            cpu_usage: model.parameters.get("avg_cpu").copied().unwrap_or(0.5),
            memory_usage: model.parameters.get("avg_memory").copied().unwrap_or(0.5),
            storage_usage: model.parameters.get("avg_storage").copied().unwrap_or(0.5),
            network_usage: model.parameters.get("avg_network").copied().unwrap_or(0.5),
            gpu_usage: model.parameters.get("avg_gpu").copied().unwrap_or(0.5),
        };

        Ok(ResourcePrediction {
            predicted_utilization,
            confidence: model.accuracy,
            prediction_horizon: horizon,
        })
    }

    /// Apply linear regression model
    async fn apply_linear_regression_model(
        &self,
        model: &PredictionModel,
        history: &[ResourceSnapshot],
        horizon: u64,
    ) -> BiomeResult<ResourcePrediction> {
        if history.len() < 2 {
            return Ok(ResourcePrediction {
                predicted_utilization: ResourceUtilization {
                    cpu_usage: 0.5,
                    memory_usage: 0.5,
                    storage_usage: 0.5,
                    network_usage: 0.5,
                    gpu_usage: 0.5,
                },
                confidence: 0.3,
                prediction_horizon: horizon,
            });
        }

        // Prepare time series data (use indices as time points)
        let n = history.len();
        let x_values: Vec<f64> = (0..n).map(|i| i as f64).collect();
        
        // Calculate linear regression for each resource type
        let cpu_prediction = self.calculate_linear_regression(
            &x_values,
            &history.iter().map(|s| s.utilization.cpu_usage).collect::<Vec<f64>>(),
            n as f64 + horizon as f64,
        );
        
        let memory_prediction = self.calculate_linear_regression(
            &x_values,
            &history.iter().map(|s| s.utilization.memory_usage).collect::<Vec<f64>>(),
            n as f64 + horizon as f64,
        );
        
        let storage_prediction = self.calculate_linear_regression(
            &x_values,
            &history.iter().map(|s| s.utilization.storage_usage).collect::<Vec<f64>>(),
            n as f64 + horizon as f64,
        );
        
        let network_prediction = self.calculate_linear_regression(
            &x_values,
            &history.iter().map(|s| s.utilization.network_usage).collect::<Vec<f64>>(),
            n as f64 + horizon as f64,
        );
        
        let gpu_prediction = self.calculate_linear_regression(
            &x_values,
            &history.iter().map(|s| s.utilization.gpu_usage).collect::<Vec<f64>>(),
            n as f64 + horizon as f64,
        );

        // Calculate confidence based on r-squared of the linear regression
        let cpu_r_squared = self.calculate_r_squared(
            &x_values,
            &history.iter().map(|s| s.utilization.cpu_usage).collect::<Vec<f64>>(),
        );
        
        let memory_r_squared = self.calculate_r_squared(
            &x_values,
            &history.iter().map(|s| s.utilization.memory_usage).collect::<Vec<f64>>(),
        );
        
        let average_r_squared = (cpu_r_squared + memory_r_squared) / 2.0;
        let confidence = (average_r_squared * model.accuracy).clamp(0.0, 1.0);

        Ok(ResourcePrediction {
            predicted_utilization: ResourceUtilization {
                cpu_usage: cpu_prediction.clamp(0.0, 1.0),
                memory_usage: memory_prediction.clamp(0.0, 1.0),
                storage_usage: storage_prediction.clamp(0.0, 1.0),
                network_usage: network_prediction.clamp(0.0, 1.0),
                gpu_usage: gpu_prediction.clamp(0.0, 1.0),
            },
            confidence,
            prediction_horizon: horizon,
        })
    }

    /// Apply exponential smoothing model
    async fn apply_exponential_smoothing_model(
        &self,
        model: &PredictionModel,
        history: &[ResourceSnapshot],
        horizon: u64,
    ) -> BiomeResult<ResourcePrediction> {
        if history.is_empty() {
            return Ok(ResourcePrediction {
                predicted_utilization: ResourceUtilization {
                    cpu_usage: 0.5,
                    memory_usage: 0.5,
                    storage_usage: 0.5,
                    network_usage: 0.5,
                    gpu_usage: 0.5,
                },
                confidence: 0.3,
                prediction_horizon: horizon,
            });
        }

        // Exponential smoothing parameters
        let alpha = model.parameters.get("alpha").unwrap_or(&0.3); // Smoothing parameter
        let beta = model.parameters.get("beta").unwrap_or(&0.1);   // Trend parameter
        let gamma = model.parameters.get("gamma").unwrap_or(&0.1); // Seasonality parameter

        // Extract resource utilization series
        let cpu_series: Vec<f64> = history.iter().map(|s| s.utilization.cpu_usage).collect();
        let memory_series: Vec<f64> = history.iter().map(|s| s.utilization.memory_usage).collect();
        let storage_series: Vec<f64> = history.iter().map(|s| s.utilization.storage_usage).collect();
        let network_series: Vec<f64> = history.iter().map(|s| s.utilization.network_usage).collect();
        let gpu_series: Vec<f64> = history.iter().map(|s| s.utilization.gpu_usage).collect();

        // Apply Holt-Winters exponential smoothing
        let cpu_prediction = self.holt_winters_smoothing(&cpu_series, *alpha, *beta, *gamma, horizon);
        let memory_prediction = self.holt_winters_smoothing(&memory_series, *alpha, *beta, *gamma, horizon);
        let storage_prediction = self.holt_winters_smoothing(&storage_series, *alpha, *beta, *gamma, horizon);
        let network_prediction = self.holt_winters_smoothing(&network_series, *alpha, *beta, *gamma, horizon);
        let gpu_prediction = self.holt_winters_smoothing(&gpu_series, *alpha, *beta, *gamma, horizon);

        // Calculate confidence based on prediction variance
        let cpu_variance = self.calculate_variance(&cpu_series);
        let memory_variance = self.calculate_variance(&memory_series);
        let average_variance = (cpu_variance + memory_variance) / 2.0;
        
        // Lower variance = higher confidence
        let confidence = ((1.0 - average_variance.sqrt()) * model.accuracy).clamp(0.0, 1.0);

        Ok(ResourcePrediction {
            predicted_utilization: ResourceUtilization {
                cpu_usage: cpu_prediction.clamp(0.0, 1.0),
                memory_usage: memory_prediction.clamp(0.0, 1.0),
                storage_usage: storage_prediction.clamp(0.0, 1.0),
                network_usage: network_prediction.clamp(0.0, 1.0),
                gpu_usage: gpu_prediction.clamp(0.0, 1.0),
            },
            confidence,
            prediction_horizon: horizon,
        })
    }

    /// Calculate linear regression prediction
    fn calculate_linear_regression(&self, x: &[f64], y: &[f64], predict_x: f64) -> f64 {
        let n = x.len() as f64;
        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
        let sum_x_squared: f64 = x.iter().map(|xi| xi * xi).sum();

        // Calculate slope (m) and intercept (b) for y = mx + b
        let denominator = n * sum_x_squared - sum_x * sum_x;
        if denominator.abs() < f64::EPSILON {
            return y.last().unwrap_or(&0.0).clone(); // Return last value if no trend
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        // Predict value at predict_x
        slope * predict_x + intercept
    }

    /// Calculate R-squared for linear regression
    fn calculate_r_squared(&self, x: &[f64], y: &[f64]) -> f64 {
        let n = x.len() as f64;
        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
        let sum_x_squared: f64 = x.iter().map(|xi| xi * xi).sum();
        let sum_y_squared: f64 = y.iter().map(|yi| yi * yi).sum();

        let y_mean = sum_y / n;
        let denominator = n * sum_x_squared - sum_x * sum_x;
        
        if denominator.abs() < f64::EPSILON {
            return 0.0;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let ss_res: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| {
            let predicted = slope * xi + intercept;
            (yi - predicted).powi(2)
        }).sum();

        let ss_tot: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();

        if ss_tot.abs() < f64::EPSILON {
            return 1.0; // Perfect fit if no variance
        }

        (1.0 - (ss_res / ss_tot)).clamp(0.0, 1.0)
    }

    /// Holt-Winters exponential smoothing
    fn holt_winters_smoothing(&self, series: &[f64], alpha: f64, beta: f64, gamma: f64, horizon: u64) -> f64 {
        if series.is_empty() {
            return 0.5;
        }

        if series.len() < 3 {
            return series.last().unwrap_or(&0.5).clone();
        }

        // Initialize level, trend, and seasonality
        let mut level = series[0];
        let mut trend = series[1] - series[0];
        let season_length = (series.len() / 4).max(1); // Assume quarterly seasonality
        let mut seasonality = vec![0.0; season_length];

        // Initialize seasonality components
        for i in 0..season_length {
            if i < series.len() {
                seasonality[i] = series[i] - level;
            }
        }

        // Apply Holt-Winters algorithm
        for (t, &value) in series.iter().enumerate().skip(1) {
            let season_index = t % season_length;
            let old_level = level;
            
            // Update level
            level = alpha * (value - seasonality[season_index]) + (1.0 - alpha) * (level + trend);
            
            // Update trend
            trend = beta * (level - old_level) + (1.0 - beta) * trend;
            
            // Update seasonality
            seasonality[season_index] = gamma * (value - level) + (1.0 - gamma) * seasonality[season_index];
        }

        // Forecast
        let season_index = (series.len() + horizon as usize - 1) % season_length;
        let forecast = level + trend * horizon as f64 + seasonality[season_index];

        forecast
    }

    /// Calculate variance of a series
    fn calculate_variance(&self, series: &[f64]) -> f64 {
        if series.len() < 2 {
            return 0.0;
        }

        let mean = series.iter().sum::<f64>() / series.len() as f64;
        let variance = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (series.len() - 1) as f64;
        variance
    }

    /// Get resource history for a primal
    pub async fn get_resource_history(&self, primal_id: &str) -> Option<Vec<ResourceSnapshot>> {
        let history = self.history.read().await;
        history.get(primal_id).cloned()
    }

    /// Get historical average for a primal
    pub async fn get_historical_average(&self, primal_id: &str) -> Option<ResourceUtilization> {
        let history = self.history.read().await;
        let primal_history = history.get(primal_id)?;
        
        if primal_history.is_empty() {
            return None;
        }

        let count = primal_history.len() as f64;
        let total_cpu: f64 = primal_history.iter().map(|s| s.utilization.cpu_usage).sum();
        let total_memory: f64 = primal_history.iter().map(|s| s.utilization.memory_usage).sum();
        let total_storage: f64 = primal_history.iter().map(|s| s.utilization.storage_usage).sum();
        let total_network: f64 = primal_history.iter().map(|s| s.utilization.network_usage).sum();
        let total_gpu: f64 = primal_history.iter().map(|s| s.utilization.gpu_usage).sum();

        Some(ResourceUtilization {
            cpu_usage: total_cpu / count,
            memory_usage: total_memory / count,
            storage_usage: total_storage / count,
            network_usage: total_network / count,
            gpu_usage: total_gpu / count,
        })
    }

    /// Clear history for a primal
    pub async fn clear_primal_history(&self, primal_id: &str) {
        let mut history = self.history.write().await;
        let mut models = self.models.write().await;
        history.remove(primal_id);
        models.remove(primal_id);
    }

    /// Clear all history
    pub async fn clear_all_history(&self) {
        let mut history = self.history.write().await;
        let mut models = self.models.write().await;
        history.clear();
        models.clear();
    }

    /// Get prediction accuracy for a primal
    pub async fn get_prediction_accuracy(&self, primal_id: &str) -> Option<f64> {
        let models = self.models.read().await;
        models.get(primal_id).map(|model| model.accuracy)
    }

    /// Update model accuracy
    pub async fn update_model_accuracy(&self, primal_id: &str, accuracy: f64) -> BiomeResult<()> {
        let mut models = self.models.write().await;
        if let Some(model) = models.get_mut(primal_id) {
            model.accuracy = accuracy;
        }
        Ok(())
    }
} 