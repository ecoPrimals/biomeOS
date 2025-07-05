//! Installation View
//! 
//! Guides users through the biomeOS installation and setup process with AI assistance.
//! Follows the sovereignty-first, grandma-safe design principles.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::{AppState, InstallationStatus, InstallationStep};
use crate::api::BiomeOSApi;
use crate::views::{BaseView, View};

pub struct InstallationView {
    base: BaseView,
}

impl InstallationView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
        }
    }
}

impl View for InstallationView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("🚀 Installation");
        ui.separator();
        
        self.base.render_card(ui, "AI-Guided Installation", |ui| {
            ui.label("🤖 Your AI assistant will guide you through the installation process.");
            
            if ui.button("Start Installation").clicked() {
                // Start installation process
            }
        });
    }
} 