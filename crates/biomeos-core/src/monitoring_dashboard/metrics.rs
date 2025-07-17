//! Metrics processing and aggregation for the monitoring dashboard

use super::collectors::Metric;
use super::types::TrendDirection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metrics processor
pub struct MetricsProcessor {
    /// Metrics storage
    storage: MetricsStorage,
    /// Aggregation engine
    aggregator: MetricsAggregator,
}

/// Metrics storage
pub struct MetricsStorage {
    /// Time series data
    time_series: HashMap<String, Vec<TimeSeriesPoint>>,
    /// Retention policy
    retention_seconds: u64,
}

/// Time series point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// Timestamp
    pub timestamp: u64,
    /// Value
    pub value: f64,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Metrics aggregator
pub struct MetricsAggregator {
    /// Aggregation functions
    functions: HashMap<String, AggregationFunction>,
}

/// Aggregation function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationFunction {
    /// Average
    Average,
    /// Sum
    Sum,
    /// Min
    Min,
    /// Max
    Max,
    /// Count
    Count,
    /// Percentile
    Percentile(f64),
    /// Rate (per second)
    Rate,
    /// Derivative
    Derivative,
}

/// Aggregated metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    /// Metric name
    pub name: String,
    /// Aggregated value
    pub value: f64,
    /// Aggregation function used
    pub function: AggregationFunction,
    /// Time range
    pub time_range: TimeRange,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Time range for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time
    pub start: u64,
    /// End time
    pub end: u64,
}

/// Metrics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQuery {
    /// Metric name pattern
    pub metric_name: String,
    /// Label filters
    pub label_filters: HashMap<String, String>,
    /// Time range
    pub time_range: TimeRange,
    /// Aggregation function
    pub aggregation: Option<AggregationFunction>,
    /// Group by labels
    pub group_by: Vec<String>,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Metric name
    pub metric_name: String,
    /// Time series data
    pub time_series: Vec<TimeSeriesPoint>,
    /// Aggregated value
    pub aggregated_value: Option<f64>,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Metric name
    pub metric_name: String,
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0 to 1.0)
    pub strength: f64,
    /// Change rate
    pub change_rate: f64,
    /// Confidence score
    pub confidence: f64,
}

/// Group key for time series grouping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GroupKey {
    /// Label values
    pub labels: Vec<(String, String)>,
}

impl MetricsProcessor {
    /// Create a new metrics processor
    pub fn new(retention_seconds: u64) -> Self {
        Self {
            storage: MetricsStorage::new(retention_seconds),
            aggregator: MetricsAggregator::new(),
        }
    }

    /// Store metrics
    pub fn store_metrics(&mut self, metrics: Vec<Metric>) {
        for metric in metrics {
            self.storage.store_metric(metric);
        }
    }

    /// Query metrics
    pub fn query_metrics(&self, query: &MetricsQuery) -> Vec<QueryResult> {
        let time_series = self
            .storage
            .get_time_series(&query.metric_name, &query.time_range);

        let mut results = Vec::new();

        if query.group_by.is_empty() {
            // No grouping
            let filtered_series = self.filter_time_series(&time_series, &query.label_filters);
            let aggregated_value = query
                .aggregation
                .as_ref()
                .map(|agg_func| self.aggregator.aggregate(&filtered_series, agg_func));

            results.push(QueryResult {
                metric_name: query.metric_name.clone(),
                time_series: filtered_series,
                aggregated_value,
                labels: HashMap::new(),
            });
        } else {
            // Group by labels
            let grouped_series = self.group_time_series(&time_series, &query.group_by);

            for (group_key, series) in grouped_series {
                let filtered_series = self.filter_time_series(&series, &query.label_filters);
                let aggregated_value = query
                    .aggregation
                    .as_ref()
                    .map(|agg_func| self.aggregator.aggregate(&filtered_series, agg_func));

                let labels: HashMap<String, String> = group_key.labels.into_iter().collect();

                results.push(QueryResult {
                    metric_name: query.metric_name.clone(),
                    time_series: filtered_series,
                    aggregated_value,
                    labels,
                });
            }
        }

        results
    }

    /// Analyze trends
    pub fn analyze_trends(&self, metric_name: &str, time_range: &TimeRange) -> TrendAnalysis {
        let time_series = self.storage.get_time_series(metric_name, time_range);

        if time_series.len() < 2 {
            return TrendAnalysis {
                metric_name: metric_name.to_string(),
                direction: TrendDirection::Unknown,
                strength: 0.0,
                change_rate: 0.0,
                confidence: 0.0,
            };
        }

        let values: Vec<f64> = time_series.iter().map(|p| p.value).collect();
        let (direction, strength, change_rate, confidence) = self.calculate_trend(&values);

        TrendAnalysis {
            metric_name: metric_name.to_string(),
            direction,
            strength,
            change_rate,
            confidence,
        }
    }

    /// Calculate trend from values
    fn calculate_trend(&self, values: &[f64]) -> (TrendDirection, f64, f64, f64) {
        if values.len() < 2 {
            return (TrendDirection::Unknown, 0.0, 0.0, 0.0);
        }

        // Simple linear regression
        let n = values.len() as f64;
        let x_sum: f64 = (0..values.len()).map(|i| i as f64).sum();
        let y_sum: f64 = values.iter().sum();
        let xy_sum: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let x_squared_sum: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x_squared_sum - x_sum.powi(2));

        let direction = if slope > 0.01 {
            TrendDirection::Improving
        } else if slope < -0.01 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        let strength = slope.abs().min(1.0);
        let change_rate = slope;
        let confidence = 0.8; // Simplified confidence calculation

        (direction, strength, change_rate, confidence)
    }

    /// Filter time series by labels
    fn filter_time_series(
        &self,
        time_series: &[TimeSeriesPoint],
        label_filters: &HashMap<String, String>,
    ) -> Vec<TimeSeriesPoint> {
        time_series
            .iter()
            .filter(|point| {
                label_filters
                    .iter()
                    .all(|(key, value)| point.labels.get(key) == Some(value))
            })
            .cloned()
            .collect()
    }

    /// Group time series by labels
    fn group_time_series(
        &self,
        time_series: &[TimeSeriesPoint],
        group_by: &[String],
    ) -> HashMap<GroupKey, Vec<TimeSeriesPoint>> {
        let mut groups: HashMap<GroupKey, Vec<TimeSeriesPoint>> = HashMap::new();

        for point in time_series {
            let group_labels: Vec<(String, String)> = group_by
                .iter()
                .filter_map(|key| {
                    point
                        .labels
                        .get(key)
                        .map(|value| (key.clone(), value.clone()))
                })
                .collect();

            let group_key = GroupKey {
                labels: group_labels,
            };
            groups.entry(group_key).or_default().push(point.clone());
        }

        groups
    }

    /// Clean up old metrics
    pub fn cleanup_old_metrics(&mut self) {
        self.storage.cleanup_old_metrics();
    }
}

impl MetricsStorage {
    /// Create a new metrics storage
    pub fn new(retention_seconds: u64) -> Self {
        Self {
            time_series: HashMap::new(),
            retention_seconds,
        }
    }

    /// Store a metric
    pub fn store_metric(&mut self, metric: Metric) {
        let point = TimeSeriesPoint {
            timestamp: metric.timestamp,
            value: metric.value,
            labels: metric.labels,
        };

        self.time_series.entry(metric.name).or_default().push(point);
    }

    /// Get time series for a metric
    pub fn get_time_series(
        &self,
        metric_name: &str,
        time_range: &TimeRange,
    ) -> Vec<TimeSeriesPoint> {
        self.time_series
            .get(metric_name)
            .map(|series| {
                series
                    .iter()
                    .filter(|point| {
                        point.timestamp >= time_range.start && point.timestamp <= time_range.end
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Clean up old metrics
    pub fn cleanup_old_metrics(&mut self) {
        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.retention_seconds;

        for series in self.time_series.values_mut() {
            series.retain(|point| point.timestamp >= cutoff_time);
        }

        // Remove empty series
        self.time_series.retain(|_, series| !series.is_empty());
    }
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Aggregate time series data
    pub fn aggregate(
        &self,
        time_series: &[TimeSeriesPoint],
        function: &AggregationFunction,
    ) -> f64 {
        if time_series.is_empty() {
            return 0.0;
        }

        let values: Vec<f64> = time_series.iter().map(|p| p.value).collect();

        match function {
            AggregationFunction::Average => values.iter().sum::<f64>() / values.len() as f64,
            AggregationFunction::Sum => values.iter().sum(),
            AggregationFunction::Min => values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            AggregationFunction::Max => values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            AggregationFunction::Count => values.len() as f64,
            AggregationFunction::Percentile(p) => self.calculate_percentile(&values, *p),
            AggregationFunction::Rate => self.calculate_rate(time_series),
            AggregationFunction::Derivative => self.calculate_derivative(time_series),
        }
    }

    /// Calculate percentile
    fn calculate_percentile(&self, values: &[f64], percentile: f64) -> f64 {
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
        sorted_values.get(index).copied().unwrap_or(0.0)
    }

    /// Calculate rate (per second)
    fn calculate_rate(&self, time_series: &[TimeSeriesPoint]) -> f64 {
        if time_series.len() < 2 {
            return 0.0;
        }

        let first = &time_series[0];
        let last = &time_series[time_series.len() - 1];

        if last.timestamp == first.timestamp {
            return 0.0;
        }

        (last.value - first.value) / (last.timestamp - first.timestamp) as f64
    }

    /// Calculate derivative
    fn calculate_derivative(&self, time_series: &[TimeSeriesPoint]) -> f64 {
        if time_series.len() < 2 {
            return 0.0;
        }

        let mut sum = 0.0;
        let mut count = 0;

        for i in 1..time_series.len() {
            let prev = &time_series[i - 1];
            let curr = &time_series[i];

            if curr.timestamp != prev.timestamp {
                sum += (curr.value - prev.value) / (curr.timestamp - prev.timestamp) as f64;
                count += 1;
            }
        }

        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}
