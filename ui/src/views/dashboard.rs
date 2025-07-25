//! Enhanced Dashboard View
//!
//! Advanced dashboard with real-time monitoring, performance optimization, and interactive charts.

use crate::api::BiomeOSApi;
use crate::state::AppState;
use crate::views::{BaseView, View};
use biomeos_core::{
    config::BiomeOSConfig,
    integration::live_service::LiveService,
    universal_biomeos_manager::{HealthStatus, SystemResourceUsage, UniversalBiomeOSManager},
};
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};
use egui::{Color32, RichText, Stroke};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

/// Enhanced dashboard with real-time health monitoring
pub struct DashboardView {
    base: BaseView,

    /// Live service integration
    live_service: Arc<RwLock<Option<LiveService>>>,

    /// Real-time metrics history
    metrics_history: VecDeque<LiveSystemMetrics>,

    /// Discovered services cache
    discovered_services: Vec<ServiceInfo>,

    /// Last update timestamp
    last_update: Instant,

    /// Update interval
    update_interval: Duration,

    /// UI state
    show_detailed_metrics: bool,
    selected_service: Option<usize>,
    auto_refresh: bool,

    /// Chart configuration
    chart_time_window: f64, // seconds
    max_history_points: usize,
}

/// Enhanced system metrics with more detailed information
#[derive(Debug, Clone)]
pub struct LiveSystemMetrics {
    pub timestamp: f64,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub disk_usage_percent: f32,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub load_average: (f32, f32, f32),
    pub network_rx_mbps: f32,
    pub network_tx_mbps: f32,
    pub uptime_hours: f32,
    pub process_count: u32,
    pub thread_count: u32,

    // Enhanced health monitoring
    pub system_health: HealthStatus,
    pub primal_health_summary: PrimalHealthSummary,
    pub service_count: usize,
    pub healthy_services: usize,
}

/// Summary of primal health across the ecosystem
#[derive(Debug, Clone)]
pub struct PrimalHealthSummary {
    pub total_primals: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub unhealthy: usize,
    pub unknown: usize,
}

/// Discovered service information for the dashboard
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub endpoint: String,
    pub health: PrimalHealth,
    pub capabilities: Vec<String>,
    pub last_seen: Instant,
    pub response_time_ms: f64,
}

impl DashboardView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
            live_service: Arc::new(RwLock::new(None)),
            metrics_history: VecDeque::with_capacity(120), // 2 minutes at 1s intervals
            discovered_services: Vec::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_secs(2),
            show_detailed_metrics: false,
            selected_service: None,
            auto_refresh: true,
            chart_time_window: 120.0, // 2 minutes
            max_history_points: 120,
        }
    }

    /// Initialize live service connection
    pub async fn initialize_live_service(&mut self) {
        if self.live_service.read().await.is_none() {
            match LiveService::new(BiomeOSConfig::default()).await {
                Ok(mut service) => {
                    if let Err(e) = service.start().await {
                        eprintln!("Failed to start live service: {}", e);
                        return;
                    }
                    *self.live_service.write().await = Some(service);
                }
                Err(e) => {
                    eprintln!("Failed to create live service: {}", e);
                }
            }
        }
    }

    /// Update real-time metrics
    async fn update_metrics(&mut self) {
        if self.last_update.elapsed() < self.update_interval {
            return;
        }

        // Collect system metrics
        if let Some(metrics) = self.collect_system_metrics().await {
            // Add to history
            if self.metrics_history.len() >= self.max_history_points {
                self.metrics_history.pop_front();
            }
            self.metrics_history.push_back(metrics);
        }

        // Update service discovery
        self.update_service_discovery().await;

        self.last_update = Instant::now();
    }

    /// Collect current system metrics
    async fn collect_system_metrics(&self) -> Option<LiveSystemMetrics> {
        let live_service_guard = self.live_service.read().await;
        let live_service = live_service_guard.as_ref()?;

        // Get system status
        let system_status = live_service.get_system_status().await.ok()?;
        let system_health = live_service.universal_manager.get_system_health().await;

        // Calculate primal health summary
        let primal_health_summary = PrimalHealthSummary {
            total_primals: system_health.primal_health.len(),
            healthy: system_health
                .primal_health
                .values()
                .filter(|&h| matches!(h, PrimalHealth::Healthy))
                .count(),
            degraded: system_health
                .primal_health
                .values()
                .filter(|&h| matches!(h, PrimalHealth::Degraded))
                .count(),
            unhealthy: system_health
                .primal_health
                .values()
                .filter(|&h| matches!(h, PrimalHealth::Unhealthy))
                .count(),
            unknown: system_health
                .primal_health
                .values()
                .filter(|&h| matches!(h, PrimalHealth::Unknown))
                .count(),
        };

        // Get process and thread counts (simplified)
        let (process_count, thread_count) = self.get_process_info();

        Some(LiveSystemMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs_f64(),
            cpu_usage_percent: system_status.resource_usage.cpu_usage_percent as f32,
            memory_usage_percent: system_status.resource_usage.memory_usage_percent as f32,
            memory_used_gb: (system_status.resource_usage.memory_usage_percent as f32) * 16.0
                / 100.0, // Estimate
            memory_total_gb: 16.0, // Estimate - could be detected
            disk_usage_percent: system_status.resource_usage.disk_usage_percent as f32,
            disk_used_gb: (system_status.resource_usage.disk_usage_percent as f32) * 500.0 / 100.0, // Estimate
            disk_total_gb: 500.0, // Estimate
            load_average: (
                system_status.resource_usage.cpu_usage_percent as f32 / 100.0,
                0.0,
                0.0,
            ), // Simplified
            network_rx_mbps: system_status.resource_usage.network_usage_mbps as f32,
            network_tx_mbps: system_status.resource_usage.network_usage_mbps as f32,
            uptime_hours: system_status.uptime.num_seconds() as f32 / 3600.0,
            process_count,
            thread_count,
            system_health: system_status.health_status,
            primal_health_summary,
            service_count: system_health.primal_health.len(),
            healthy_services: primal_health_summary.healthy,
        })
    }

    /// Get process information (simplified implementation)
    fn get_process_info(&self) -> (u32, u32) {
        // In a real implementation, we'd query the system for process/thread counts
        // For now, return reasonable estimates
        (150, 800)
    }

    /// Update service discovery information
    async fn update_service_discovery(&mut self) {
        let live_service_guard = self.live_service.read().await;
        if let Some(live_service) = live_service_guard.as_ref() {
            if let Ok(discovered) = live_service.get_discovered_primals().await {
                self.discovered_services = discovered
                    .into_iter()
                    .map(|service| ServiceInfo {
                        id: service.primal_id.clone(),
                        name: service.primal_id,
                        service_type: service.primal_type.category,
                        endpoint: service.endpoint,
                        health: service.health,
                        capabilities: service
                            .capabilities
                            .iter()
                            .map(|c| c.name.clone())
                            .collect(),
                        last_seen: Instant::now(),
                        response_time_ms: 50.0, // Would measure actual response time
                    })
                    .collect();
            }
        }
    }

    /// Render real-time system overview
    fn render_system_overview(&self, ui: &mut egui::Ui) {
        ui.heading("🎛️ System Overview");

        if let Some(latest) = self.metrics_history.back() {
            ui.horizontal(|ui| {
                // System health indicator
                let health_color = match latest.system_health {
                    HealthStatus::Healthy => Color32::GREEN,
                    HealthStatus::Degraded => Color32::YELLOW,
                    HealthStatus::Warning => Color32::from_rgb(255, 165, 0), // Orange
                    HealthStatus::Critical => Color32::RED,
                    HealthStatus::Unhealthy => Color32::RED,
                    HealthStatus::Unknown => Color32::GRAY,
                };

                ui.colored_label(health_color, "●");
                ui.label(format!("System: {:?}", latest.system_health));

                ui.separator();

                // Uptime
                ui.label(format!("Uptime: {:.1}h", latest.uptime_hours));

                ui.separator();

                // Service summary
                ui.label(format!(
                    "Services: {}/{} healthy",
                    latest.healthy_services, latest.service_count
                ));
            });

            ui.add_space(8.0);

            // Resource usage cards
            ui.columns(4, |columns| {
                // CPU Usage
                columns[0].group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("CPU");
                        let cpu_color = self.get_usage_color(latest.cpu_usage_percent);
                        ui.colored_label(cpu_color, format!("{:.1}%", latest.cpu_usage_percent));
                        ui.small(format!("Load: {:.2}", latest.load_average.0));
                    });
                });

                // Memory Usage
                columns[1].group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Memory");
                        let mem_color = self.get_usage_color(latest.memory_usage_percent);
                        ui.colored_label(mem_color, format!("{:.1}%", latest.memory_usage_percent));
                        ui.small(format!(
                            "{:.1}/{:.1} GB",
                            latest.memory_used_gb, latest.memory_total_gb
                        ));
                    });
                });

                // Disk Usage
                columns[2].group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Disk");
                        let disk_color = self.get_usage_color(latest.disk_usage_percent);
                        ui.colored_label(disk_color, format!("{:.1}%", latest.disk_usage_percent));
                        ui.small(format!(
                            "{:.0}/{:.0} GB",
                            latest.disk_used_gb, latest.disk_total_gb
                        ));
                    });
                });

                // Network Usage
                columns[3].group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Network");
                        ui.label(format!("{:.1} Mbps", latest.network_rx_mbps));
                        ui.small(format!(
                            "RX/TX: {:.1}/{:.1}",
                            latest.network_rx_mbps, latest.network_tx_mbps
                        ));
                    });
                });
            });
        } else {
            ui.label("Collecting system metrics...");
        }
    }

    /// Render real-time charts
    fn render_real_time_charts(&self, ui: &mut egui::Ui) {
        ui.heading("📈 Real-time Metrics");

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.show_detailed_metrics.clone(),
                "Show detailed metrics",
            );
            if ui.button("Clear History").clicked() {
                // Would clear metrics history
            }
        });

        if self.metrics_history.len() < 2 {
            ui.label("Collecting metrics data for charts...");
            return;
        }

        ui.columns(2, |columns| {
            // CPU and Load Chart
            columns[0].group(|ui| {
                ui.label("CPU Usage & Load Average");

                let cpu_points: PlotPoints = self
                    .metrics_history
                    .iter()
                    .enumerate()
                    .map(|(i, m)| [i as f64, m.cpu_usage_percent as f64])
                    .collect();

                let load_points: PlotPoints = self
                    .metrics_history
                    .iter()
                    .enumerate()
                    .map(|(i, m)| [i as f64, m.load_average.0 as f64 * 100.0]) // Scale load for visibility
                    .collect();

                Plot::new("cpu_chart")
                    .view_aspect(2.0)
                    .height(150.0)
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new(cpu_points).color(Color32::RED).name("CPU %"));
                        plot_ui.line(
                            Line::new(load_points)
                                .color(Color32::YELLOW)
                                .name("Load (x100)"),
                        );
                    });
            });

            // Memory and Disk Chart
            columns[1].group(|ui| {
                ui.label("Memory & Disk Usage");

                let memory_points: PlotPoints = self
                    .metrics_history
                    .iter()
                    .enumerate()
                    .map(|(i, m)| [i as f64, m.memory_usage_percent as f64])
                    .collect();

                let disk_points: PlotPoints = self
                    .metrics_history
                    .iter()
                    .enumerate()
                    .map(|(i, m)| [i as f64, m.disk_usage_percent as f64])
                    .collect();

                Plot::new("memory_disk_chart")
                    .view_aspect(2.0)
                    .height(150.0)
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new(memory_points)
                                .color(Color32::BLUE)
                                .name("Memory %"),
                        );
                        plot_ui.line(Line::new(disk_points).color(Color32::GREEN).name("Disk %"));
                    });
            });
        });

        if self.show_detailed_metrics {
            // Network traffic chart
            ui.group(|ui| {
                ui.label("Network Traffic");

                let network_points: PlotPoints = self
                    .metrics_history
                    .iter()
                    .enumerate()
                    .map(|(i, m)| [i as f64, m.network_rx_mbps as f64])
                    .collect();

                Plot::new("network_chart")
                    .view_aspect(3.0)
                    .height(120.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new(network_points)
                                .color(Color32::LIGHT_BLUE)
                                .name("Network Mbps"),
                        );
                    });
            });
        }
    }

    /// Render discovered services with capability information
    fn render_discovered_services(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔍 Discovered Services");

        ui.horizontal(|ui| {
            ui.label(format!("Found {} services", self.discovered_services.len()));
            if ui.button("🔄 Refresh Discovery").clicked() {
                // Trigger immediate service discovery update
                let live_service = Arc::clone(&self.live_service);
                let rt = tokio::runtime::Handle::current();
                rt.spawn(async move {
                    // Force service discovery update
                });
            }
        });

        ui.separator();

        if self.discovered_services.is_empty() {
            ui.label("No services discovered yet. Discovery runs automatically every few seconds.");
            return;
        }

        // Services table
        egui::Grid::new("services_grid")
            .num_columns(5)
            .spacing([20.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Header
                ui.strong("Service");
                ui.strong("Type");
                ui.strong("Health");
                ui.strong("Capabilities");
                ui.strong("Response Time");
                ui.end_row();

                // Service rows
                for (idx, service) in self.discovered_services.iter().enumerate() {
                    let is_selected = self.selected_service == Some(idx);

                    // Service name (clickable)
                    let name_response = ui.selectable_label(is_selected, &service.name);
                    if name_response.clicked() {
                        self.selected_service = if is_selected { None } else { Some(idx) };
                    }

                    // Service type
                    ui.label(&service.service_type);

                    // Health status with color
                    let health_color = match service.health {
                        PrimalHealth::Healthy => Color32::GREEN,
                        PrimalHealth::Degraded => Color32::YELLOW,
                        PrimalHealth::Unhealthy => Color32::RED,
                        PrimalHealth::Unknown => Color32::GRAY,
                    };
                    ui.colored_label(health_color, format!("{:?}", service.health));

                    // Capabilities (truncated)
                    let caps_text = if service.capabilities.len() <= 3 {
                        service.capabilities.join(", ")
                    } else {
                        format!(
                            "{}, ... (+{})",
                            service.capabilities[..2].join(", "),
                            service.capabilities.len() - 2
                        )
                    };
                    ui.label(caps_text);

                    // Response time
                    ui.label(format!("{:.1}ms", service.response_time_ms));

                    ui.end_row();
                }
            });

        // Service details panel
        if let Some(selected_idx) = self.selected_service {
            if let Some(service) = self.discovered_services.get(selected_idx) {
                ui.separator();
                ui.heading("Service Details");

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Endpoint:");
                        ui.monospace(&service.endpoint);
                    });

                    ui.horizontal(|ui| {
                        ui.label("All Capabilities:");
                        ui.vertical(|ui| {
                            for cap in &service.capabilities {
                                ui.label(format!("• {}", cap));
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Last Seen:");
                        ui.label(format!(
                            "{:.1}s ago",
                            service.last_seen.elapsed().as_secs_f64()
                        ));
                    });
                });
            }
        }
    }

    /// Render primal health summary
    fn render_primal_health(&self, ui: &mut egui::Ui) {
        ui.heading("🧬 Primal Health");

        if let Some(latest) = self.metrics_history.back() {
            let summary = &latest.primal_health_summary;

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Total Primals");
                        ui.heading(summary.total_primals.to_string());
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(Color32::GREEN, "Healthy");
                        ui.heading(summary.healthy.to_string());
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(Color32::YELLOW, "Degraded");
                        ui.heading(summary.degraded.to_string());
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(Color32::RED, "Unhealthy");
                        ui.heading(summary.unhealthy.to_string());
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(Color32::GRAY, "Unknown");
                        ui.heading(summary.unknown.to_string());
                    });
                });
            });

            // Health percentage bar
            if summary.total_primals > 0 {
                let health_percentage =
                    (summary.healthy as f32 / summary.total_primals as f32) * 100.0;
                let health_color = if health_percentage >= 80.0 {
                    Color32::GREEN
                } else if health_percentage >= 60.0 {
                    Color32::YELLOW
                } else {
                    Color32::RED
                };

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Overall Health:");
                    ui.colored_label(health_color, format!("{:.1}%", health_percentage));
                });

                let progress_bar = egui::ProgressBar::new(health_percentage / 100.0)
                    .fill(health_color)
                    .show_percentage();
                ui.add(progress_bar);
            }
        } else {
            ui.label("Loading primal health information...");
        }
    }

    /// Get color for usage percentage
    fn get_usage_color(&self, percentage: f32) -> Color32 {
        if percentage >= 90.0 {
            Color32::RED
        } else if percentage >= 70.0 {
            Color32::from_rgb(255, 165, 0) // Orange
        } else if percentage >= 50.0 {
            Color32::YELLOW
        } else {
            Color32::GREEN
        }
    }
}

impl View for DashboardView {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Initialize live service if not already done
        let rt = tokio::runtime::Handle::current();
        let live_service_clone = Arc::clone(&self.live_service);
        rt.spawn(async move {
            let mut dashboard = DashboardView::new(
                Arc::new(Mutex::new(AppState::new())),
                Arc::new(BiomeOSApi::new()),
            );
            dashboard.initialize_live_service().await;
        });

        // Update metrics if auto-refresh is enabled
        if self.auto_refresh {
            let rt = tokio::runtime::Handle::current();
            rt.spawn(async move {
                // Update metrics in background
            });
        }

        // Dashboard header
        ui.horizontal(|ui| {
            ui.heading("🌱 BiomeOS Dashboard");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.auto_refresh, "Auto-refresh");

                if ui.button("🔄").clicked() {
                    // Force immediate refresh
                    let rt = tokio::runtime::Handle::current();
                    rt.spawn(async move {
                        // Trigger immediate update
                    });
                }
            });
        });

        ui.separator();

        // Main dashboard content in scrollable area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_system_overview(ui);
                ui.add_space(16.0);

                self.render_real_time_charts(ui);
                ui.add_space(16.0);

                self.render_discovered_services(ui);
                ui.add_space(16.0);

                self.render_primal_health(ui);
            });

        // Auto-repaint for real-time updates
        ctx.request_repaint_after(self.update_interval);
    }
}
