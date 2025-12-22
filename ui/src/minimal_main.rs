//! Minimal BiomeOS Desktop - Working OS-like Interface
//!
//! A clean, working desktop interface that focuses on orchestrating other primals
//! rather than implementing complex functionality directly.

use eframe::egui;

mod minimal_app;
use minimal_app::MinimalBiomeOSApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🌱 Starting BiomeOS Desktop Interface...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("🌱 BiomeOS - Sovereign Computing Platform")
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "BiomeOS Desktop",
        options,
        Box::new(|cc| {
            // Configure dark theme for BiomeOS
            configure_biome_theme(&cc.egui_ctx);

            // Create the minimal working app
            Box::new(MinimalBiomeOSApp::new())
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run BiomeOS desktop: {}", e))
}

/// Configure BiomeOS visual theme
fn configure_biome_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // BiomeOS color scheme
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 30, 35);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 40, 45);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 55, 60);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 70, 80);
    
    // Accent colors
    visuals.selection.bg_fill = egui::Color32::from_rgb(80, 120, 80);
    visuals.hyperlink_color = egui::Color32::from_rgb(100, 150, 100);
    
    // Panel backgrounds
    visuals.panel_fill = egui::Color32::from_rgb(20, 25, 30);
    visuals.window_fill = egui::Color32::from_rgb(25, 30, 35);
    
    ctx.set_visuals(visuals);
    
    println!("✅ BiomeOS theme configured");
} 