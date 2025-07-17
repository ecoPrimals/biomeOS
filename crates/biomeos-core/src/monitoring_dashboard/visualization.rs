//! Visualization components for the monitoring dashboard

use super::{
    config::{AggregationInterval, ChartType, ColorScheme, TimeRange},
    metrics::{QueryResult, TimeSeriesPoint},
    types::TrendDirection,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Visualization engine
pub struct VisualizationEngine {
    /// Chart generators
    chart_generators: HashMap<ChartType, Box<dyn ChartGenerator>>,
    /// Color schemes
    color_schemes: HashMap<String, ColorScheme>,
}

/// Chart generator trait
pub trait ChartGenerator: Send + Sync {
    /// Generate chart data
    fn generate_chart(&self, data: &ChartData) -> ChartOutput;
}

/// Chart data input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// Chart title
    pub title: String,
    /// Time series data
    pub time_series: Vec<TimeSeriesPoint>,
    /// Chart configuration
    pub config: ChartConfig,
    /// Color scheme
    pub color_scheme: ColorScheme,
}

/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart type
    pub chart_type: ChartType,
    /// Time range
    pub time_range: TimeRange,
    /// Aggregation interval
    pub aggregation_interval: AggregationInterval,
    /// Show legend
    pub show_legend: bool,
    /// Show grid
    pub show_grid: bool,
    /// Chart height
    pub height: u32,
    /// Chart width
    pub width: u32,
}

/// Chart output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOutput {
    /// Chart type
    pub chart_type: ChartType,
    /// Chart data points
    pub data_points: Vec<ChartDataPoint>,
    /// Chart metadata
    pub metadata: ChartMetadata,
    /// Rendered chart (SVG, PNG, etc.)
    pub rendered_chart: Option<String>,
}

/// Chart data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    /// X coordinate/value
    pub x: f64,
    /// Y coordinate/value
    pub y: f64,
    /// Label
    pub label: Option<String>,
    /// Color
    pub color: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Chart metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartMetadata {
    /// X-axis label
    pub x_label: String,
    /// Y-axis label
    pub y_label: String,
    /// Chart title
    pub title: String,
    /// Min/max values
    pub x_range: (f64, f64),
    /// Y min/max values
    pub y_range: (f64, f64),
    /// Number of data points
    pub data_point_count: usize,
}

/// Time series chart generator
pub struct TimeSeriesChartGenerator;

/// Bar chart generator
pub struct BarChartGenerator;

/// Pie chart generator
pub struct PieChartGenerator;

/// Heatmap chart generator
pub struct HeatmapChartGenerator;

/// Gauge chart generator
pub struct GaugeChartGenerator;

/// Histogram chart generator
pub struct HistogramChartGenerator;

/// Scatter plot chart generator
pub struct ScatterChartGenerator;

/// Dashboard visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardVisualization {
    /// Visualization ID
    pub id: String,
    /// Visualization name
    pub name: String,
    /// Chart type
    pub chart_type: ChartType,
    /// Query configuration
    pub query_config: VisualizationQuery,
    /// Chart configuration
    pub chart_config: ChartConfig,
    /// Generated chart
    pub chart: Option<ChartOutput>,
    /// Last updated
    pub last_updated: u64,
}

/// Visualization query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationQuery {
    /// Metric name
    pub metric_name: String,
    /// Label filters
    pub label_filters: HashMap<String, String>,
    /// Time range
    pub time_range: TimeRange,
    /// Group by labels
    pub group_by: Vec<String>,
}

/// Trend visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendVisualization {
    /// Metric name
    pub metric_name: String,
    /// Trend direction
    pub trend_direction: TrendDirection,
    /// Trend strength
    pub trend_strength: f64,
    /// Trend arrow
    pub trend_arrow: TrendArrow,
    /// Trend color
    pub trend_color: String,
}

/// Trend arrow direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendArrow {
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Flat arrow
    Flat,
    /// Unknown
    Unknown,
}

impl VisualizationEngine {
    /// Create a new visualization engine
    pub fn new() -> Self {
        let mut chart_generators: HashMap<ChartType, Box<dyn ChartGenerator>> = HashMap::new();

        chart_generators.insert(ChartType::TimeSeries, Box::new(TimeSeriesChartGenerator));
        chart_generators.insert(ChartType::Bar, Box::new(BarChartGenerator));
        chart_generators.insert(ChartType::Pie, Box::new(PieChartGenerator));
        chart_generators.insert(ChartType::Heatmap, Box::new(HeatmapChartGenerator));
        chart_generators.insert(ChartType::Gauge, Box::new(GaugeChartGenerator));
        chart_generators.insert(ChartType::Histogram, Box::new(HistogramChartGenerator));
        chart_generators.insert(ChartType::Scatter, Box::new(ScatterChartGenerator));

        Self {
            chart_generators,
            color_schemes: HashMap::new(),
        }
    }

    /// Generate visualization
    pub fn generate_visualization(
        &self,
        visualization: &mut DashboardVisualization,
        query_result: &QueryResult,
    ) -> Result<(), String> {
        let chart_data = ChartData {
            title: visualization.name.clone(),
            time_series: query_result.time_series.clone(),
            config: visualization.chart_config.clone(),
            color_scheme: self.get_color_scheme("default"),
        };

        if let Some(generator) = self.chart_generators.get(&visualization.chart_type) {
            let chart_output = generator.generate_chart(&chart_data);
            visualization.chart = Some(chart_output);
            visualization.last_updated = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(())
        } else {
            Err(format!(
                "No generator found for chart type: {:?}",
                visualization.chart_type
            ))
        }
    }

    /// Get color scheme
    pub fn get_color_scheme(&self, name: &str) -> ColorScheme {
        self.color_schemes
            .get(name)
            .cloned()
            .unwrap_or_else(|| ColorScheme {
                primary: "#007bff".to_string(),
                secondary: "#6c757d".to_string(),
                success: "#28a745".to_string(),
                warning: "#ffc107".to_string(),
                error: "#dc3545".to_string(),
                info: "#17a2b8".to_string(),
                background: "#ffffff".to_string(),
                text: "#212529".to_string(),
            })
    }

    /// Add color scheme
    pub fn add_color_scheme(&mut self, name: String, color_scheme: ColorScheme) {
        self.color_schemes.insert(name, color_scheme);
    }

    /// Generate trend visualization
    pub fn generate_trend_visualization(
        &self,
        metric_name: &str,
        trend_direction: TrendDirection,
        trend_strength: f64,
    ) -> TrendVisualization {
        let (trend_arrow, trend_color) = match trend_direction {
            TrendDirection::Improving => (TrendArrow::Up, "#28a745".to_string()),
            TrendDirection::Degrading => (TrendArrow::Down, "#dc3545".to_string()),
            TrendDirection::Stable => (TrendArrow::Flat, "#6c757d".to_string()),
            TrendDirection::Unknown => (TrendArrow::Unknown, "#6c757d".to_string()),
        };

        TrendVisualization {
            metric_name: metric_name.to_string(),
            trend_direction,
            trend_strength,
            trend_arrow,
            trend_color,
        }
    }
}

impl ChartGenerator for TimeSeriesChartGenerator {
    fn generate_chart(&self, data: &ChartData) -> ChartOutput {
        let mut data_points = Vec::new();

        for point in &data.time_series {
            data_points.push(ChartDataPoint {
                x: point.timestamp as f64,
                y: point.value,
                label: None,
                color: Some(data.color_scheme.primary.clone()),
                metadata: HashMap::new(),
            });
        }

        let (x_min, x_max) = if data_points.is_empty() {
            (0.0, 0.0)
        } else {
            let x_values: Vec<f64> = data_points.iter().map(|p| p.x).collect();
            (
                *x_values
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *x_values
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            )
        };

        let (y_min, y_max) = if data_points.is_empty() {
            (0.0, 0.0)
        } else {
            let y_values: Vec<f64> = data_points.iter().map(|p| p.y).collect();
            (
                *y_values
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *y_values
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            )
        };

        ChartOutput {
            chart_type: ChartType::TimeSeries,
            data_points,
            metadata: ChartMetadata {
                x_label: "Time".to_string(),
                y_label: "Value".to_string(),
                title: data.title.clone(),
                x_range: (x_min, x_max),
                y_range: (y_min, y_max),
                data_point_count: data.time_series.len(),
            },
            rendered_chart: None,
        }
    }
}

impl ChartGenerator for BarChartGenerator {
    fn generate_chart(&self, data: &ChartData) -> ChartOutput {
        let mut data_points = Vec::new();

        for (i, point) in data.time_series.iter().enumerate() {
            data_points.push(ChartDataPoint {
                x: i as f64,
                y: point.value,
                label: Some(format!("Bar {}", i + 1)),
                color: Some(data.color_scheme.primary.clone()),
                metadata: HashMap::new(),
            });
        }

        ChartOutput {
            chart_type: ChartType::Bar,
            data_points,
            metadata: ChartMetadata {
                x_label: "Category".to_string(),
                y_label: "Value".to_string(),
                title: data.title.clone(),
                x_range: (0.0, data.time_series.len() as f64),
                y_range: (
                    0.0,
                    data.time_series.iter().map(|p| p.value).fold(0.0, f64::max),
                ),
                data_point_count: data.time_series.len(),
            },
            rendered_chart: None,
        }
    }
}

impl ChartGenerator for PieChartGenerator {
    fn generate_chart(&self, data: &ChartData) -> ChartOutput {
        let total: f64 = data.time_series.iter().map(|p| p.value).sum();
        let mut data_points = Vec::new();

        for (i, point) in data.time_series.iter().enumerate() {
            let percentage = if total > 0.0 {
                (point.value / total) * 100.0
            } else {
                0.0
            };
            data_points.push(ChartDataPoint {
                x: i as f64,
                y: percentage,
                label: Some(format!("Slice {}", i + 1)),
                color: Some(data.color_scheme.primary.clone()),
                metadata: HashMap::new(),
            });
        }

        ChartOutput {
            chart_type: ChartType::Pie,
            data_points,
            metadata: ChartMetadata {
                x_label: "Category".to_string(),
                y_label: "Percentage".to_string(),
                title: data.title.clone(),
                x_range: (0.0, data.time_series.len() as f64),
                y_range: (0.0, 100.0),
                data_point_count: data.time_series.len(),
            },
            rendered_chart: None,
        }
    }
}

impl ChartGenerator for HeatmapChartGenerator {
    fn generate_chart(&self, data: &ChartData) -> ChartOutput {
        // Simplified heatmap implementation
        let mut data_points = Vec::new();

        for (i, _point) in data.time_series.iter().enumerate() {
            data_points.push(ChartDataPoint {
                x: (i % 10) as f64,
                y: (i / 10) as f64,
                label: None,
                color: Some(data.color_scheme.primary.clone()),
                metadata: HashMap::new(),
            });
        }

        ChartOutput {
            chart_type: ChartType::Heatmap,
            data_points,
            metadata: ChartMetadata {
                x_label: "X".to_string(),
                y_label: "Y".to_string(),
                title: data.title.clone(),
                x_range: (0.0, 10.0),
                y_range: (0.0, (data.time_series.len() / 10) as f64),
                data_point_count: data.time_series.len(),
            },
            rendered_chart: None,
        }
    }
}

impl ChartGenerator for GaugeChartGenerator {
    fn generate_chart(&self, data: &ChartData) -> ChartOutput {
        let current_value = data.time_series.last().map(|p| p.value).unwrap_or(0.0);

        let data_points = vec![ChartDataPoint {
            x: 0.0,
            y: current_value,
            label: Some("Current".to_string()),
            color: Some(data.color_scheme.primary.clone()),
            metadata: HashMap::new(),
        }];

        ChartOutput {
            chart_type: ChartType::Gauge,
            data_points,
            metadata: ChartMetadata {
                x_label: "".to_string(),
                y_label: "Value".to_string(),
                title: data.title.clone(),
                x_range: (0.0, 1.0),
                y_range: (0.0, 100.0),
                data_point_count: 1,
            },
            rendered_chart: None,
        }
    }
}

impl ChartGenerator for HistogramChartGenerator {
    fn generate_chart(&self, data: &ChartData) -> ChartOutput {
        // Simplified histogram implementation
        let values: Vec<f64> = data.time_series.iter().map(|p| p.value).collect();
        let mut data_points = Vec::new();

        // Create 10 bins
        if !values.is_empty() {
            let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let bin_size = (max_val - min_val) / 10.0;

            for i in 0..10 {
                let bin_start = min_val + i as f64 * bin_size;
                let bin_end = bin_start + bin_size;
                let count = values
                    .iter()
                    .filter(|&&v| v >= bin_start && v < bin_end)
                    .count();

                data_points.push(ChartDataPoint {
                    x: bin_start + bin_size / 2.0,
                    y: count as f64,
                    label: Some(format!("Bin {}", i + 1)),
                    color: Some(data.color_scheme.primary.clone()),
                    metadata: HashMap::new(),
                });
            }
        }

        ChartOutput {
            chart_type: ChartType::Histogram,
            data_points,
            metadata: ChartMetadata {
                x_label: "Value".to_string(),
                y_label: "Frequency".to_string(),
                title: data.title.clone(),
                x_range: (0.0, 100.0),
                y_range: (0.0, values.len() as f64),
                data_point_count: 10,
            },
            rendered_chart: None,
        }
    }
}

impl ChartGenerator for ScatterChartGenerator {
    fn generate_chart(&self, data: &ChartData) -> ChartOutput {
        let mut data_points = Vec::new();

        for point in &data.time_series {
            data_points.push(ChartDataPoint {
                x: point.timestamp as f64,
                y: point.value,
                label: None,
                color: Some(data.color_scheme.primary.clone()),
                metadata: HashMap::new(),
            });
        }

        let (x_min, x_max) = if data_points.is_empty() {
            (0.0, 0.0)
        } else {
            let x_values: Vec<f64> = data_points.iter().map(|p| p.x).collect();
            (
                *x_values
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *x_values
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            )
        };

        let (y_min, y_max) = if data_points.is_empty() {
            (0.0, 0.0)
        } else {
            let y_values: Vec<f64> = data_points.iter().map(|p| p.y).collect();
            (
                *y_values
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *y_values
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            )
        };

        ChartOutput {
            chart_type: ChartType::Scatter,
            data_points,
            metadata: ChartMetadata {
                x_label: "X".to_string(),
                y_label: "Y".to_string(),
                title: data.title.clone(),
                x_range: (x_min, x_max),
                y_range: (y_min, y_max),
                data_point_count: data.time_series.len(),
            },
            rendered_chart: None,
        }
    }
}

impl Default for VisualizationEngine {
    fn default() -> Self {
        Self::new()
    }
}
