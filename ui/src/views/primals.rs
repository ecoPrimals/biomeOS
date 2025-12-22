//! Primals View
//!
//! This view provides management and monitoring of discovered primals
//! in the biomeOS ecosystem.

use crate::api::BiomeOSApi;
use crate::state::AppState;
use crate::views::BaseView;
use egui::{Context, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;

/// View for managing and monitoring primals
pub struct PrimalsView {
    base: BaseView,
    discovered_primals: Vec<PrimalInfo>,
    selected_primal: Option<String>,
    refresh_interval: std::time::Duration,
    last_refresh: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct PrimalInfo {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health: String,
    pub last_seen: u64,
}

impl PrimalsView {
    /// Create a new primals view
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
            discovered_primals: Vec::new(),
            selected_primal: None,
            refresh_interval: std::time::Duration::from_secs(5),
            last_refresh: std::time::Instant::now(),
        }
    }

    /// Refresh primals data
    async fn refresh_primals(&mut self) {
        // Simulate primal discovery
        self.discovered_primals = vec![
            PrimalInfo {
                id: "toadstool-1".to_string(),
                name: "ToadStool Compute".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![
                    "container_runtime".to_string(),
                    "manifest_parsing".to_string(),
                    "workload_execution".to_string(),
                ],
                health: "Healthy".to_string(),
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PrimalInfo {
                id: "songbird-1".to_string(),
                name: "Songbird Orchestrator".to_string(),
                primal_type: "Orchestration".to_string(),
                endpoint: "http://localhost:8081".to_string(),
                capabilities: vec![
                    "service_discovery".to_string(),
                    "load_balancing".to_string(),
                    "federation".to_string(),
                ],
                health: "Healthy".to_string(),
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PrimalInfo {
                id: "nestgate-1".to_string(),
                name: "NestGate Storage".to_string(),
                primal_type: "Storage".to_string(),
                endpoint: "http://localhost:8082".to_string(),
                capabilities: vec![
                    "zfs_management".to_string(),
                    "volume_provisioning".to_string(),
                    "backup_services".to_string(),
                ],
                health: "Warning".to_string(),
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PrimalInfo {
                id: "beardog-1".to_string(),
                name: "BearDog Security".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "http://localhost:8083".to_string(),
                capabilities: vec![
                    "authentication".to_string(),
                    "encryption".to_string(),
                    "threat_detection".to_string(),
                ],
                health: "Healthy".to_string(),
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
        ];

        self.last_refresh = std::time::Instant::now();
    }

    /// Render the primals view
    pub fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.show(ctx, ui);
    }

    /// Render the primals view
    pub fn show(&mut self, _ctx: &Context, ui: &mut Ui) {
        // Auto-refresh if needed
        if self.last_refresh.elapsed() >= self.refresh_interval {
            // In a real implementation, this would be async
            // For now, we'll simulate the refresh
            tokio::spawn(async move {
                // self.refresh_primals().await;
            });
        }

        ui.heading("🌐 Discovered Primals");

        ui.separator();

        // Refresh button
        if ui.button("🔄 Refresh Discovery").clicked() {
            tokio::spawn(async move {
                // self.refresh_primals().await;
            });
        }

        ui.add_space(10.0);

        // Primals table
        egui::Grid::new("primals_grid")
            .num_columns(6)
            .striped(true)
            .show(ui, |ui| {
                // Header
                ui.strong("Name");
                ui.strong("Type");
                ui.strong("Endpoint");
                ui.strong("Health");
                ui.strong("Capabilities");
                ui.strong("Actions");
                ui.end_row();

                // Primals data
                for primal in &self.discovered_primals {
                    ui.label(&primal.name);
                    ui.label(&primal.primal_type);
                    ui.label(&primal.endpoint);

                    // Health with color coding
                    let health_color = match primal.health.as_str() {
                        "Healthy" => egui::Color32::GREEN,
                        "Warning" => egui::Color32::YELLOW,
                        "Critical" => egui::Color32::RED,
                        _ => egui::Color32::GRAY,
                    };
                    ui.colored_label(health_color, &primal.health);

                    // Capabilities
                    ui.label(primal.capabilities.join(", "));

                    // Actions
                    ui.horizontal(|ui| {
                        if ui.small_button("📊 Details").clicked() {
                            self.selected_primal = Some(primal.id.clone());
                        }
                        if ui.small_button("🔧 Configure").clicked() {
                            // Open configuration dialog
                        }
                    });

                    ui.end_row();
                }
            });

        // Show details panel if a primal is selected
        if let Some(selected_id) = &self.selected_primal {
            if let Some(primal) = self
                .discovered_primals
                .iter()
                .find(|p| &p.id == selected_id)
            {
                ui.separator();
                ui.heading(format!("📊 {} Details", primal.name));

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("ID: {}", primal.id));
                        ui.label(format!("Type: {}", primal.primal_type));
                        ui.label(format!("Endpoint: {}", primal.endpoint));
                        ui.label(format!("Health: {}", primal.health));
                    });

                    ui.vertical(|ui| {
                        ui.label("Capabilities:");
                        for capability in &primal.capabilities {
                            ui.label(format!("  • {}", capability));
                        }
                    });
                });

                if ui.button("❌ Close Details").clicked() {
                    self.selected_primal = None;
                }
            }
        }
    }
}
