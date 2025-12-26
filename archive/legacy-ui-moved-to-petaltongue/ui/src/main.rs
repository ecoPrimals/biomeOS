//! biomeOS Bootstrap UI - Unified Architecture
//!
//! A unified UI application that can operate in multiple modes:
//! - Full: Complete feature set with all views
//! - Minimal: Clean desktop focused on orchestration
//! - Hybrid: Best of both worlds

use eframe::egui;

// Use the library modules
use biomeos_ui_app::app::BiomeOSApp;

/// Application mode selection
#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Minimal,
    Full,
    Hybrid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    println!("🌱 Starting BiomeOS Unified Interface...");

    // Determine mode from command line arguments or environment
    let mode = determine_app_mode();

    println!("🔧 Running in {:?} mode", mode);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(match mode {
                AppMode::Minimal => [1200.0, 800.0],
                AppMode::Full => [1400.0, 900.0],
                AppMode::Hybrid => [1300.0, 850.0],
            })
            .with_title(match mode {
                AppMode::Minimal => "🌱 BiomeOS - Desktop",
                AppMode::Full => "🌱 BiomeOS - Control Center",
                AppMode::Hybrid => "🌱 BiomeOS - Hybrid Interface",
            })
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "BiomeOS Unified Interface",
        options,
        Box::new(move |cc| {
            // Create the unified application in the specified mode
            let app = match mode {
                AppMode::Minimal => BiomeOSApp::new_minimal(cc),
                AppMode::Full => BiomeOSApp::new_full(cc),
                AppMode::Hybrid => BiomeOSApp::new_hybrid(cc),
            };

            Box::new(app)
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run BiomeOS interface: {}", e))
}

/// Determine application mode from environment or arguments
fn determine_app_mode() -> AppMode {
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();

    for arg in &args {
        match arg.as_str() {
            "--minimal" => return AppMode::Minimal,
            "--full" => return AppMode::Full,
            "--hybrid" => return AppMode::Hybrid,
            _ => {}
        }
    }

    // Check environment variable
    if let Ok(mode_str) = std::env::var("BIOMEOS_UI_MODE") {
        match mode_str.to_lowercase().as_str() {
            "minimal" => return AppMode::Minimal,
            "full" => return AppMode::Full,
            "hybrid" => return AppMode::Hybrid,
            _ => {}
        }
    }

    // Default to hybrid mode for best of both worlds
    AppMode::Hybrid
}
