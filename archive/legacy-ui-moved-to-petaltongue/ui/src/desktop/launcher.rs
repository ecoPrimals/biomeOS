//! Application Launcher

use crate::types::{AppView, LauncherApp, WindowInfo};
use eframe::egui;

/// Render application launcher overlay
pub fn render_launcher(
    ctx: &egui::Context,
    taskbar_height: f32,
    launcher_search: &mut String,
    show_launcher: &mut bool,
    active_windows: &mut Vec<WindowInfo>,
) {
    let screen_rect = ctx.screen_rect();
    let launcher_size = egui::vec2(400.0, 600.0);
    let launcher_pos = egui::pos2(
        screen_rect.left() + 20.0,
        screen_rect.bottom() - taskbar_height - launcher_size.y - 20.0,
    );

    egui::Window::new("BiomeOS Launcher")
        .fixed_pos(launcher_pos)
        .fixed_size(launcher_size)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::from_rgb(25, 30, 35)))
        .show(ctx, |ui| {
            // Search bar
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(launcher_search);
            });

            ui.separator();

            // Application grid
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.columns(3, |columns| {
                    let apps = get_filtered_apps(launcher_search);
                    for (i, app) in apps.iter().enumerate() {
                        let col = i % 3;
                        if columns[col]
                            .button(format!("{}\n{}", app.icon, app.name))
                            .clicked()
                        {
                            launch_app(active_windows, app.view);
                            *show_launcher = false;
                        }
                    }
                });
            });
        });
}

fn get_filtered_apps(search: &str) -> Vec<LauncherApp> {
    let apps = LauncherApp::defaults();
    if search.is_empty() {
        apps
    } else {
        let search_lower = search.to_lowercase();
        apps.into_iter()
            .filter(|app| app.name.to_lowercase().contains(&search_lower))
            .collect()
    }
}

fn launch_app(active_windows: &mut Vec<WindowInfo>, view: AppView) {
    let window_id = format!("window_{}", active_windows.len());
    let offset = active_windows.len();

    // Unfocus other windows
    for w in active_windows.iter_mut() {
        w.focused = false;
    }

    active_windows.push(WindowInfo::new(
        window_id,
        view.title().to_string(),
        view,
        offset,
    ));
}
