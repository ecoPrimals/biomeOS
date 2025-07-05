//! biomeOS UI Views Module
//! 
//! This module contains all the UI views for the biomeOS bootstrap interface.
//! Each view is responsible for a specific aspect of the system:
//! - Dashboard: Overview and system status
//! - Installation: Setup and installation process
//! - Primals: Ecosystem component management
//! - Sovereignty: Security and compliance
//! - Settings: Configuration and preferences
//! - YAML Editor: Edit and create biome.yaml files

pub mod dashboard;
pub mod installation;
pub mod primals;
pub mod sovereignty;
pub mod settings;
pub mod yaml_editor;

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use crate::api::BiomeOSApi;

/// Universal View trait for all UI views
pub trait View {
    /// Render the view with UI context and egui context for repaints
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context);
    
    /// Update view state (called on each frame)
    fn update(&mut self) {}
    
    /// Handle view activation (when user switches to this view)
    fn on_activate(&mut self) {}
    
    /// Handle view deactivation (when user switches away from this view)
    fn on_deactivate(&mut self) {}
}

/// Base view struct with common functionality
pub struct BaseView {
    pub state: Arc<Mutex<AppState>>,
    pub api: Arc<BiomeOSApi>,
    pub is_loading: bool,
    pub error_message: Option<String>,
}

impl BaseView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            state,
            api,
            is_loading: false,
            error_message: None,
        }
    }

    /// Render a loading indicator
    pub fn render_loading(&self, ui: &mut egui::Ui, message: &str) {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(message);
        });
    }

    /// Render an error message
    pub fn render_error(&self, ui: &mut egui::Ui, error: &str) {
        ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
    }

    /// Render a success message
    pub fn render_success(&self, ui: &mut egui::Ui, message: &str) {
        ui.colored_label(egui::Color32::GREEN, format!("✅ {}", message));
    }

    /// Render a warning message
    pub fn render_warning(&self, ui: &mut egui::Ui, message: &str) {
        ui.colored_label(egui::Color32::YELLOW, format!("⚠️ {}", message));
    }

    /// Render an info message
    pub fn render_info(&self, ui: &mut egui::Ui, message: &str) {
        ui.colored_label(egui::Color32::LIGHT_BLUE, format!("ℹ️ {}", message));
    }

    /// Render a status badge
    pub fn render_status_badge(&self, ui: &mut egui::Ui, status: &str, color: egui::Color32) {
        ui.colored_label(color, status);
    }

    /// Render a metric value with label
    pub fn render_metric(&self, ui: &mut egui::Ui, label: &str, value: &str, color: Option<egui::Color32>) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            if let Some(color) = color {
                ui.colored_label(color, value);
            } else {
                ui.label(value);
            }
        });
    }

    /// Render a collapsible section
    pub fn render_collapsible<F>(&self, ui: &mut egui::Ui, title: &str, content: F)
    where
        F: FnOnce(&mut egui::Ui),
    {
        ui.collapsing(title, content);
    }

    /// Render a card with title and content
    pub fn render_card<F>(&self, ui: &mut egui::Ui, title: &str, content: F)
    where
        F: FnOnce(&mut egui::Ui),
    {
        egui::Frame::group(ui.style())
            .fill(ui.style().visuals.panel_fill)
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .rounding(4.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(title).heading());
                ui.separator();
                content(ui);
            });
    }
} 