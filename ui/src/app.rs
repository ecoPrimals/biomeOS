//! Main biomeOS Application State and UI Orchestration
//! 
//! This module implements the core UI application following biomeOS design principles:
//! - API-driven architecture
//! - Universal/recursive patterns  
//! - Sovereignty-first UX
//! - Bootstrap/foundational for developers

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use crate::views::{
    dashboard::DashboardView,
    installation::InstallationView,
    primals::PrimalsView,
    sovereignty::SovereigntyView,
    settings::SettingsView,
    yaml_editor::YamlEditorView,
    View,
};
use crate::api::BiomeOSApi;

/// Main biomeOS UI Application
pub struct BiomeOSApp {
    /// Application state (shared across views)
    state: Arc<Mutex<AppState>>,
    
    /// API client for biomeOS core
    api: Arc<BiomeOSApi>,
    
    /// Current active view
    current_view: AppView,
    
    /// View instances
    dashboard_view: DashboardView,
    installation_view: InstallationView,
    primals_view: PrimalsView,
    sovereignty_view: SovereigntyView,
    settings_view: SettingsView,
    yaml_editor_view: YamlEditorView,
    
    /// UI state
    show_dev_panel: bool,
    show_api_debug: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    Dashboard,
    Installation,
    Primals,
    Sovereignty,
    Settings,
    YamlEditor,
}

impl BiomeOSApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = Arc::new(Mutex::new(AppState::new()));
        let api = Arc::new(BiomeOSApi::new());
        
        Self {
            state: state.clone(),
            api: api.clone(),
            current_view: AppView::Dashboard,
            dashboard_view: DashboardView::new(state.clone(), api.clone()),
            installation_view: InstallationView::new(state.clone(), api.clone()),
            primals_view: PrimalsView::new(state.clone(), api.clone()),
            sovereignty_view: SovereigntyView::new(state.clone(), api.clone()),
            settings_view: SettingsView::new(state.clone(), api.clone()),
            yaml_editor_view: YamlEditorView::new(state.clone(), api.clone()),
            show_dev_panel: false,
            show_api_debug: false,
        }
    }

    /// Render the main navigation bar
    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // biomeOS logo and title
            ui.heading("🌱 biomeOS");
            
            ui.separator();
            
            // Main navigation buttons
            if ui.selectable_label(
                self.current_view == AppView::Dashboard,
                "🏠 Dashboard"
            ).clicked() {
                self.current_view = AppView::Dashboard;
            }
            
            if ui.selectable_label(
                self.current_view == AppView::YamlEditor,
                "📝 YAML Editor"
            ).clicked() {
                self.current_view = AppView::YamlEditor;
            }
            
            if ui.selectable_label(
                self.current_view == AppView::Installation,
                "🚀 Installation"
            ).clicked() {
                self.current_view = AppView::Installation;
            }
            
            if ui.selectable_label(
                self.current_view == AppView::Primals,
                "🧬 Primals"
            ).clicked() {
                self.current_view = AppView::Primals;
            }
            
            if ui.selectable_label(
                self.current_view == AppView::Sovereignty,
                "🔒 Sovereignty"
            ).clicked() {
                self.current_view = AppView::Sovereignty;
            }
            
            // Spacer to push settings to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.selectable_label(
                    self.current_view == AppView::Settings,
                    "⚙️ Settings"
                ).clicked() {
                    self.current_view = AppView::Settings;
                }
                
                ui.separator();
                
                // Developer tools toggle
                ui.checkbox(&mut self.show_dev_panel, "🔧 Dev");
                ui.checkbox(&mut self.show_api_debug, "🌐 API");
            });
        });
    }

    /// Render the current active view
    fn render_current_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        match self.current_view {
            AppView::Dashboard => self.dashboard_view.render(ui, ctx),
            AppView::YamlEditor => self.yaml_editor_view.render(ui, ctx),
            AppView::Installation => self.installation_view.render(ui, ctx),
            AppView::Primals => self.primals_view.render(ui, ctx),
            AppView::Sovereignty => self.sovereignty_view.render(ui, ctx),
            AppView::Settings => self.settings_view.render(ui, ctx),
        }
    }

    /// Render developer panel (when enabled)
    fn render_dev_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_dev_panel {
            return;
        }
        
        ui.collapsing("🔧 Developer Panel", |ui| {
            ui.label("Bootstrap UI Controls:");
            
            ui.horizontal(|ui| {
                if ui.button("Reload State").clicked() {
                    // Trigger state reload
                }
                
                if ui.button("Reset Config").clicked() {
                    // Reset configuration
                }
                
                if ui.button("Export Logs").clicked() {
                    // Export debug logs
                }
            });
            
            ui.separator();
            
            ui.label("View State:");
            ui.label(format!("Current: {:?}", self.current_view));
            
            // Quick navigation for developers
            ui.separator();
            ui.label("Quick Navigation:");
            ui.horizontal(|ui| {
                if ui.small_button("Dashboard").clicked() {
                    self.current_view = AppView::Dashboard;
                }
                if ui.small_button("YAML Editor").clicked() {
                    self.current_view = AppView::YamlEditor;
                }
                if ui.small_button("Installation").clicked() {
                    self.current_view = AppView::Installation;
                }
            });
            
            if self.show_api_debug {
                ui.separator();
                ui.label("API Debug:");
                ui.label("Connection: Active");
                ui.label("Last Request: 2.3s ago");
                ui.label("YAML Validation: Active");
            }
        });
    }

    /// Render status bar
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // System status indicator
            ui.label("🟢 System: Online");
            
            ui.separator();
            
            // API status  
            ui.label("🌐 API: Connected");
            
            ui.separator();
            
            // Sovereignty status
            ui.label("🔒 Sovereignty: 3/3");
            
            ui.separator();
            
            // Current view status
            match self.current_view {
                AppView::YamlEditor => {
                    ui.label("📝 YAML: Ready");
                }
                AppView::Dashboard => {
                    ui.label("📊 Monitoring: Active");
                }
                AppView::Installation => {
                    ui.label("🚀 Installer: Ready");
                }
                _ => {}
            }
            
            // Right-aligned info
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("biomeOS v{}", env!("CARGO_PKG_VERSION")));
            });
        });
    }
}

impl eframe::App for BiomeOSApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle command line arguments for YAML editor mode
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|arg| arg == "--yaml-editor") {
            self.current_view = AppView::YamlEditor;
        }
        
        // Main UI layout
        egui::TopBottomPanel::top("navigation")
            .resizable(false)
            .show(ctx, |ui| {
                self.render_navigation(ui);
            });

        egui::TopBottomPanel::bottom("status")
            .resizable(false)
            .show(ctx, |ui| {
                self.render_status_bar(ui);
            });

        // Developer panel (collapsible)
        if self.show_dev_panel {
            egui::SidePanel::right("dev_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.render_dev_panel(ui);
                });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_current_view(ui, ctx);
        });

        // Auto-refresh the UI for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
} 