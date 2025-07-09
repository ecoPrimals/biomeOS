//! ToadStool Compute Orchestration View
//! 
//! Manage and monitor toadStool compute execution, runtime orchestration,
//! and ecosystem integration within the biomeOS interface.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::api::BiomeOSApi;
use crate::views::{BaseView, View};

/// ToadStool orchestration view for compute management
pub struct ToadStoolView {
    pub base: BaseView,
    pub selected_tab: ToadStoolTab,
    pub toadstool_status: ToadStoolStatus,
    pub active_workloads: Vec<WorkloadInfo>,
    pub runtime_metrics: RuntimeMetrics,
    pub available_runtimes: Vec<RuntimeEngine>,
    pub deployment_history: Vec<DeploymentRecord>,
    pub resource_usage: ResourceUsage,
    pub ecosystem_connections: HashMap<String, ConnectionStatus>,
    pub biome_executions: Vec<BiomeExecution>,
    pub show_advanced_config: bool,
    pub auto_refresh: bool,
    pub refresh_interval: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToadStoolTab {
    Overview,
    Workloads,
    Runtimes,
    Resources,
    Ecosystem,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadStoolStatus {
    pub service_id: String,
    pub health: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub active_biomes: usize,
    pub active_workloads: usize,
    pub total_cpu_cores: f64,
    pub total_memory_gb: f64,
    pub available_runtimes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadInfo {
    pub workload_id: String,
    pub name: String,
    pub runtime_type: String,
    pub status: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub started_at: String,
    pub biome_id: String,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub container_runtime: RuntimeStats,
    pub wasm_runtime: RuntimeStats,
    pub native_runtime: RuntimeStats,
    pub python_runtime: RuntimeStats,
    pub gpu_runtime: RuntimeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub enabled: bool,
    pub active_workloads: usize,
    pub total_executions: u64,
    pub avg_startup_time_ms: f64,
    pub success_rate: f64,
    pub resource_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEngine {
    pub name: String,
    pub engine_type: String,
    pub version: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub supported_architectures: Vec<String>,
    pub performance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub deployment_id: String,
    pub biome_name: String,
    pub team_id: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub workload_count: usize,
    pub resource_allocation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_total_bytes: u64,
    pub storage_usage_bytes: u64,
    pub storage_total_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub gpu_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeExecution {
    pub biome_id: String,
    pub team_id: String,
    pub status: String,
    pub workloads: Vec<WorkloadInfo>,
    pub resource_quota: ResourceQuota,
    pub started_at: String,
    pub health_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub cpu_cores: f64,
    pub memory_gb: f64,
    pub storage_gb: f64,
    pub gpu_count: u32,
}

impl ToadStoolView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
            selected_tab: ToadStoolTab::Overview,
            toadstool_status: Self::default_status(),
            active_workloads: Vec::new(),
            runtime_metrics: Self::default_metrics(),
            available_runtimes: Self::default_runtimes(),
            deployment_history: Vec::new(),
            resource_usage: Self::default_resource_usage(),
            ecosystem_connections: Self::default_connections(),
            biome_executions: Vec::new(),
            show_advanced_config: false,
            auto_refresh: true,
            refresh_interval: 5.0,
        }
    }

    fn default_status() -> ToadStoolStatus {
        ToadStoolStatus {
            service_id: "toadstool-primary".to_string(),
            health: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 3600,
            active_biomes: 3,
            active_workloads: 12,
            total_cpu_cores: 16.0,
            total_memory_gb: 64.0,
            available_runtimes: vec![
                "container".to_string(),
                "wasm".to_string(),
                "native".to_string(),
                "python".to_string(),
                "gpu".to_string(),
            ],
        }
    }

    fn default_metrics() -> RuntimeMetrics {
        RuntimeMetrics {
            container_runtime: RuntimeStats {
                enabled: true,
                active_workloads: 8,
                total_executions: 245,
                avg_startup_time_ms: 850.0,
                success_rate: 98.5,
                resource_efficiency: 85.2,
            },
            wasm_runtime: RuntimeStats {
                enabled: true,
                active_workloads: 3,
                total_executions: 89,
                avg_startup_time_ms: 120.0,
                success_rate: 99.8,
                resource_efficiency: 95.1,
            },
            native_runtime: RuntimeStats {
                enabled: true,
                active_workloads: 1,
                total_executions: 34,
                avg_startup_time_ms: 45.0,
                success_rate: 97.1,
                resource_efficiency: 92.3,
            },
            python_runtime: RuntimeStats {
                enabled: true,
                active_workloads: 0,
                total_executions: 12,
                avg_startup_time_ms: 2100.0,
                success_rate: 94.2,
                resource_efficiency: 78.9,
            },
            gpu_runtime: RuntimeStats {
                enabled: false,
                active_workloads: 0,
                total_executions: 0,
                avg_startup_time_ms: 0.0,
                success_rate: 0.0,
                resource_efficiency: 0.0,
            },
        }
    }

    fn default_runtimes() -> Vec<RuntimeEngine> {
        vec![
            RuntimeEngine {
                name: "Container Runtime".to_string(),
                engine_type: "container".to_string(),
                version: "1.6.9".to_string(),
                status: "active".to_string(),
                capabilities: vec!["isolation".to_string(), "networking".to_string(), "volumes".to_string()],
                supported_architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
                performance_score: 8.5,
            },
            RuntimeEngine {
                name: "WASM Runtime".to_string(),
                engine_type: "wasm".to_string(),
                version: "20.0.2".to_string(),
                status: "active".to_string(),
                capabilities: vec!["fast_startup".to_string(), "memory_safe".to_string(), "portable".to_string()],
                supported_architectures: vec!["x86_64".to_string(), "aarch64".to_string(), "riscv64".to_string()],
                performance_score: 9.2,
            },
            RuntimeEngine {
                name: "Native Runtime".to_string(),
                engine_type: "native".to_string(),
                version: "1.0.0".to_string(),
                status: "active".to_string(),
                capabilities: vec!["direct_execution".to_string(), "system_access".to_string(), "performance".to_string()],
                supported_architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
                performance_score: 9.8,
            },
        ]
    }

    fn default_resource_usage() -> ResourceUsage {
        ResourceUsage {
            cpu_usage_percent: 45.2,
            memory_usage_bytes: 12 * 1024 * 1024 * 1024, // 12 GB
            memory_total_bytes: 64 * 1024 * 1024 * 1024, // 64 GB
            storage_usage_bytes: 150 * 1024 * 1024 * 1024, // 150 GB
            storage_total_bytes: 1000 * 1024 * 1024 * 1024, // 1 TB
            network_rx_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            network_tx_bytes: 1024 * 1024 * 1024, // 1 GB
            gpu_usage_percent: None,
        }
    }

    fn default_connections() -> HashMap<String, ConnectionStatus> {
        let mut connections = HashMap::new();
        connections.insert("songbird".to_string(), ConnectionStatus::Connected);
        connections.insert("nestgate".to_string(), ConnectionStatus::Connected);
        connections.insert("beardog".to_string(), ConnectionStatus::Connecting);
        connections.insert("squirrel".to_string(), ConnectionStatus::Disconnected);
        connections
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.selectable_label(self.selected_tab == ToadStoolTab::Overview, "🏠 Overview").clicked() {
                self.selected_tab = ToadStoolTab::Overview;
            }
            if ui.selectable_label(self.selected_tab == ToadStoolTab::Workloads, "⚙️ Workloads").clicked() {
                self.selected_tab = ToadStoolTab::Workloads;
            }
            if ui.selectable_label(self.selected_tab == ToadStoolTab::Runtimes, "🚀 Runtimes").clicked() {
                self.selected_tab = ToadStoolTab::Runtimes;
            }
            if ui.selectable_label(self.selected_tab == ToadStoolTab::Resources, "📊 Resources").clicked() {
                self.selected_tab = ToadStoolTab::Resources;
            }
            if ui.selectable_label(self.selected_tab == ToadStoolTab::Ecosystem, "🌐 Ecosystem").clicked() {
                self.selected_tab = ToadStoolTab::Ecosystem;
            }
            if ui.selectable_label(self.selected_tab == ToadStoolTab::Configuration, "⚙️ Config").clicked() {
                self.selected_tab = ToadStoolTab::Configuration;
            }
        });
    }

    fn render_overview_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Status card
            self.base.render_card(ui, "🍄 ToadStool Status", |ui| {
                self.base.render_status(ui, "Health", &self.toadstool_status.health, 
                    if self.toadstool_status.health == "healthy" { 
                        egui::Color32::GREEN 
                    } else { 
                        egui::Color32::RED 
                    }
                );
                self.base.render_kv(ui, "Version", &self.toadstool_status.version);
                self.base.render_kv(ui, "Service ID", &self.toadstool_status.service_id);
                self.base.render_kv(ui, "Uptime", &format!("{}h {}m", 
                    self.toadstool_status.uptime_seconds / 3600,
                    (self.toadstool_status.uptime_seconds % 3600) / 60));
            });

            ui.add_space(10.0);

            // Quick stats
            self.base.render_card(ui, "📊 Quick Stats", |ui| {
                self.base.render_metric(ui, "Active Biomes", 
                    &self.toadstool_status.active_biomes.to_string(), "");
                self.base.render_metric(ui, "Active Workloads", 
                    &self.toadstool_status.active_workloads.to_string(), "");
                self.base.render_metric(ui, "CPU Cores", 
                    &self.toadstool_status.total_cpu_cores.to_string(), "");
                self.base.render_metric(ui, "Memory", 
                    &self.toadstool_status.total_memory_gb.to_string(), "GB");
            });
        });

        ui.add_space(15.0);

        // Resource usage overview
        self.base.render_card(ui, "📈 Resource Usage", |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("CPU Usage");
                    self.base.render_progress(ui, "", self.resource_usage.cpu_usage_percent / 100.0);
                    ui.label(format!("{:.1}%", self.resource_usage.cpu_usage_percent));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Memory Usage");
                    let memory_percent = (self.resource_usage.memory_usage_bytes as f64 / 
                        self.resource_usage.memory_total_bytes as f64) as f32;
                    self.base.render_progress(ui, "", memory_percent);
                    ui.label(format!("{:.1} GB / {:.1} GB", 
                        self.resource_usage.memory_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                        self.resource_usage.memory_total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Storage Usage");
                    let storage_percent = (self.resource_usage.storage_usage_bytes as f64 / 
                        self.resource_usage.storage_total_bytes as f64) as f32;
                    self.base.render_progress(ui, "", storage_percent);
                    ui.label(format!("{:.0} GB / {:.0} GB", 
                        self.resource_usage.storage_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                        self.resource_usage.storage_total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)));
                });
            });
        });

        ui.add_space(15.0);

        // Runtime overview
        self.base.render_card(ui, "🚀 Runtime Overview", |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.strong("Container Runtime");
                    ui.label(format!("Active: {}", self.runtime_metrics.container_runtime.active_workloads));
                    ui.label(format!("Success Rate: {:.1}%", self.runtime_metrics.container_runtime.success_rate));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.strong("WASM Runtime");
                    ui.label(format!("Active: {}", self.runtime_metrics.wasm_runtime.active_workloads));
                    ui.label(format!("Success Rate: {:.1}%", self.runtime_metrics.wasm_runtime.success_rate));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.strong("Native Runtime");
                    ui.label(format!("Active: {}", self.runtime_metrics.native_runtime.active_workloads));
                    ui.label(format!("Success Rate: {:.1}%", self.runtime_metrics.native_runtime.success_rate));
                });
            });
        });
    }

    fn render_workloads_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("⚙️ Active Workloads");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    // Refresh workloads
                }
                if ui.button("📊 Export Metrics").clicked() {
                    // Export metrics
                }
            });
        });

        ui.add_space(10.0);

        // Simulated workloads for demo
        let demo_workloads = vec![
            WorkloadInfo {
                workload_id: "wl-001".to_string(),
                name: "web-frontend".to_string(),
                runtime_type: "container".to_string(),
                status: "running".to_string(),
                cpu_usage: 2.5,
                memory_usage: 512 * 1024 * 1024,
                started_at: "2025-01-01T10:30:00Z".to_string(),
                biome_id: "team-alpha-dev".to_string(),
                endpoints: vec!["http://localhost:8080".to_string()],
            },
            WorkloadInfo {
                workload_id: "wl-002".to_string(),
                name: "data-processor".to_string(),
                runtime_type: "wasm".to_string(),
                status: "running".to_string(),
                cpu_usage: 1.2,
                memory_usage: 128 * 1024 * 1024,
                started_at: "2025-01-01T11:15:00Z".to_string(),
                biome_id: "team-beta-prod".to_string(),
                endpoints: vec![],
            },
            WorkloadInfo {
                workload_id: "wl-003".to_string(),
                name: "ml-inference".to_string(),
                runtime_type: "python".to_string(),
                status: "starting".to_string(),
                cpu_usage: 0.1,
                memory_usage: 64 * 1024 * 1024,
                started_at: "2025-01-01T12:00:00Z".to_string(),
                biome_id: "team-gamma-test".to_string(),
                endpoints: vec!["http://localhost:8081/predict".to_string()],
            },
        ];

        for workload in &demo_workloads {
            self.base.render_card(ui, &format!("⚙️ {}", workload.name), |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.base.render_kv(ui, "ID", &workload.workload_id);
                        self.base.render_kv(ui, "Runtime", &workload.runtime_type);
                        self.base.render_kv(ui, "Biome", &workload.biome_id);
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        let status_color = match workload.status.as_str() {
                            "running" => egui::Color32::GREEN,
                            "starting" => egui::Color32::YELLOW,
                            "stopping" => egui::Color32::ORANGE,
                            "error" => egui::Color32::RED,
                            _ => egui::Color32::GRAY,
                        };
                        self.base.render_status(ui, "Status", &workload.status, status_color);
                        self.base.render_metric(ui, "CPU", &format!("{:.1}", workload.cpu_usage), "cores");
                        self.base.render_metric(ui, "Memory", 
                            &format!("{:.0}", workload.memory_usage as f64 / (1024.0 * 1024.0)), "MB");
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        if !workload.endpoints.is_empty() {
                            ui.label("Endpoints:");
                            for endpoint in &workload.endpoints {
                                ui.small(endpoint);
                            }
                        }
                        
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            if ui.button("📊 Logs").clicked() {
                                // View logs
                            }
                            if ui.button("⏹️ Stop").clicked() {
                                // Stop workload
                            }
                            if ui.button("🔄 Restart").clicked() {
                                // Restart workload
                            }
                        });
                    });
                });
            });
            
            ui.add_space(10.0);
        }
    }

    fn render_ecosystem_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌐 Ecosystem Integration");
        ui.label("ToadStool's connections to other biomeOS Primals");
        ui.add_space(10.0);

        for (primal, status) in &self.ecosystem_connections {
            self.base.render_card(ui, &format!("🔗 {}", primal), |ui| {
                let (status_text, status_color) = match status {
                    ConnectionStatus::Connected => ("Connected", egui::Color32::GREEN),
                    ConnectionStatus::Connecting => ("Connecting", egui::Color32::YELLOW),
                    ConnectionStatus::Disconnected => ("Disconnected", egui::Color32::RED),
                    ConnectionStatus::Error(msg) => (msg.as_str(), egui::Color32::RED),
                };
                
                self.base.render_status(ui, "Status", status_text, status_color);
                
                // Add primal-specific information
                match primal.as_str() {
                    "songbird" => {
                        ui.label("Service mesh coordination and API gateway");
                        if matches!(status, ConnectionStatus::Connected) {
                            ui.label("✅ Service registration active");
                            ui.label("✅ Message routing operational");
                        }
                    }
                    "nestgate" => {
                        ui.label("Storage orchestration and volume management");
                        if matches!(status, ConnectionStatus::Connected) {
                            ui.label("✅ Volume mounting available");
                            ui.label("✅ Storage allocation active");
                        }
                    }
                    "beardog" => {
                        ui.label("Security validation and cryptographic services");
                        if matches!(status, ConnectionStatus::Connecting) {
                            ui.label("🔄 Establishing secure channel");
                            ui.label("🔄 Exchanging certificates");
                        }
                    }
                    "squirrel" => {
                        ui.label("AI agent orchestration and optimization");
                        if matches!(status, ConnectionStatus::Disconnected) {
                            ui.label("⏸️ Service not available");
                            ui.label("⏸️ Manual operation mode");
                        }
                    }
                    _ => {}
                }

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui.button("🔄 Reconnect").clicked() {
                        // Reconnect to primal
                    }
                    if ui.button("📊 Test Connection").clicked() {
                        // Test connection
                    }
                    if ui.button("⚙️ Configure").clicked() {
                        // Configure connection
                    }
                });
            });
            
            ui.add_space(10.0);
        }
    }
}

impl View for ToadStoolView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.heading("🍄 ToadStool Compute Orchestration");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.auto_refresh, "Auto Refresh");
                if self.auto_refresh {
                    ui.add(egui::Slider::new(&mut self.refresh_interval, 1.0..=30.0)
                        .suffix("s").text("Interval"));
                }
            });
        });
        
        ui.label("Universal runtime environment for sovereign compute execution");
        ui.separator();

        self.render_tab_bar(ui);
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            match self.selected_tab {
                ToadStoolTab::Overview => self.render_overview_tab(ui),
                ToadStoolTab::Workloads => self.render_workloads_tab(ui),
                ToadStoolTab::Runtimes => {
                    ui.heading("🚀 Runtime Engines");
                    ui.label("Runtime engine management coming soon...");
                }
                ToadStoolTab::Resources => {
                    ui.heading("📊 Resource Management");
                    ui.label("Resource monitoring and allocation coming soon...");
                }
                ToadStoolTab::Ecosystem => self.render_ecosystem_tab(ui),
                ToadStoolTab::Configuration => {
                    ui.heading("⚙️ Configuration");
                    ui.label("ToadStool configuration management coming soon...");
                }
            }
        });
    }
} 