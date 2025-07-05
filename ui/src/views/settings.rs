//! Settings View
//! 
//! System configuration, preferences, and advanced options.
//! Provides both basic and developer-focused settings.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use crate::api::BiomeOSApi;
use crate::views::{BaseView, View};

pub struct SettingsView {
    base: BaseView,
}

impl SettingsView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
        }
    }
}

impl View for SettingsView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("⚙️ Settings");
        ui.separator();
        
        self.base.render_card(ui, "System Configuration", |ui| {
            ui.label("Configure biomeOS settings and preferences");
            
            ui.checkbox(&mut true, "Enable AI assistance");
            ui.checkbox(&mut true, "Auto-discover primals");
            ui.checkbox(&mut false, "Developer mode");
        });
    }
} 