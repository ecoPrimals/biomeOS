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
//! - BYOB: Build Your Own Biome management
//! - ISO Creator: Create bootable ISO images
//! - Niche Manager: Manage and create niches
//! - ToadStool: Compute orchestration and runtime management

pub mod byob;
pub mod dashboard;
pub mod installation;
pub mod iso_creator;
pub mod niche_manager;
pub mod primals;
pub mod settings;
pub mod sovereignty;
pub mod toadstool;
pub mod yaml_editor;

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::BiomeOSApi;
use crate::state::AppState;

/// Base trait for all UI views
pub trait View {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context);
}

/// Base view struct with common functionality
pub struct BaseView {
    pub state: Arc<Mutex<AppState>>,
    pub api: Arc<BiomeOSApi>,
}

impl BaseView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self { state, api }
    }

    /// Render a collapsible card with title and content
    pub fn render_card<F>(&self, ui: &mut egui::Ui, title: &str, content: F)
    where
        F: FnOnce(&mut egui::Ui),
    {
        egui::Frame::none()
            .fill(ui.visuals().panel_fill)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .rounding(4.0)
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.strong(title);
                ui.separator();
                content(ui);
            });
    }

    /// Render a collapsible section
    pub fn render_collapsible<F>(&self, ui: &mut egui::Ui, title: &str, content: F)
    where
        F: FnOnce(&mut egui::Ui),
    {
        ui.collapsing(title, content);
    }

    /// Render a progress bar with label
    pub fn render_progress(&self, ui: &mut egui::Ui, label: &str, progress: f32) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(egui::ProgressBar::new(progress).show_percentage());
        });
    }

    /// Render a status indicator
    pub fn render_status(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        status: &str,
        color: egui::Color32,
    ) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.colored_label(color, status);
        });
    }

    /// Render a metric display
    pub fn render_metric(&self, ui: &mut egui::Ui, label: &str, value: &str, unit: &str) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.strong(value);
            ui.label(unit);
        });
    }

    /// Render a key-value pair
    pub fn render_kv(&self, ui: &mut egui::Ui, key: &str, value: &str) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", key));
            ui.label(value);
        });
    }

    /// Render a warning message
    pub fn render_warning(&self, ui: &mut egui::Ui, message: &str) {
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::YELLOW, "⚠️");
            ui.label(message);
        });
    }

    /// Render an error message
    pub fn render_error(&self, ui: &mut egui::Ui, message: &str) {
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::RED, "❌");
            ui.label(message);
        });
    }

    /// Render a success message
    pub fn render_success(&self, ui: &mut egui::Ui, message: &str) {
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::GREEN, "✅");
            ui.label(message);
        });
    }

    /// Render an info message
    pub fn render_info(&self, ui: &mut egui::Ui, message: &str) {
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::BLUE, "ℹ️");
            ui.label(message);
        });
    }

    /// Render a loading spinner
    pub fn render_loading(&self, ui: &mut egui::Ui, message: &str) {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(message);
        });
    }

    /// Render a button with icon
    pub fn render_icon_button(&self, ui: &mut egui::Ui, icon: &str, text: &str) -> egui::Response {
        ui.button(format!("{} {}", icon, text))
    }

    /// Render a confirmation dialog
    pub fn render_confirmation_dialog(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        message: &str,
        on_confirm: impl FnOnce(),
    ) {
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(message);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("✅ Confirm").clicked() {
                        on_confirm();
                    }
                    if ui.button("❌ Cancel").clicked() {
                        // Close dialog
                    }
                });
            });
    }
}
