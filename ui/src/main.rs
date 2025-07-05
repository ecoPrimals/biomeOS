//! biomeOS Bootstrap UI
//! 
//! A foundational, API-driven UI for biomeOS installation and primal management.
//! Built with sovereignty-first principles: universal, recursive, agnostic, iterative.

use eframe::egui;

mod app;
mod views;
mod api;
mod state;

use app::BiomeOSApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("🌱 biomeOS - Sovereign Computing Platform")
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "biomeOS Bootstrap UI",
        options,
        Box::new(|cc| {
            // Configure egui visuals for biomeOS theme
            configure_theme(&cc.egui_ctx);

            // Create the main application
            let app = BiomeOSApp::new(cc);
            Box::new(app)
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run native application: {}", e))
}

/// Configure the biomeOS visual theme
fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::default();
    
    // biomeOS color scheme - sovereignty-first theme
    visuals.override_text_color = Some(egui::Color32::from_rgb(240, 240, 240));
    visuals.panel_fill = egui::Color32::from_rgb(30, 35, 40);
    visuals.window_fill = egui::Color32::from_rgb(25, 30, 35);
    visuals.extreme_bg_color = egui::Color32::from_rgb(15, 20, 25);
    
    // biomeOS accent colors - organic/biological
    visuals.selection.bg_fill = egui::Color32::from_rgb(60, 120, 80); // Forest green
    visuals.selection.stroke.color = egui::Color32::from_rgb(80, 160, 100);
    
    // Sovereignty indicators
    visuals.warn_fg_color = egui::Color32::from_rgb(255, 180, 60); // Amber warning
    visuals.error_fg_color = egui::Color32::from_rgb(255, 100, 100); // Coral error
    
    ctx.set_visuals(visuals);
    
    // Custom fonts for biomeOS (using default for now)
    // TODO: Add custom fonts when available
    // let mut fonts = egui::FontDefinitions::default();
    // ctx.set_fonts(fonts);
} 