//! Primals View
//! 
//! Manages the biomeOS ecosystem components (primals) including discovery,
//! installation, and health monitoring.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use crate::api::BiomeOSApi;
use crate::views::{BaseView, View};

pub struct PrimalsView {
    base: BaseView,
}

impl PrimalsView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
        }
    }
}

impl View for PrimalsView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("🧬 Ecosystem Primals");
        ui.separator();
        
        self.base.render_card(ui, "Available Primals", |ui| {
            ui.label("Discover and manage ecosystem components");
            
            if ui.button("Discover Primals").clicked() {
                // Trigger primal discovery
            }
        });
    }
} 