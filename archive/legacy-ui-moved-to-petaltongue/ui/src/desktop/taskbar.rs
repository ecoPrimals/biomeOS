//! Taskbar Rendering

use crate::types::{AppView, SystemNotification, WindowInfo};
use eframe::egui;

/// Render taskbar with launcher and system info
pub fn render_taskbar(
    ui: &mut egui::Ui,
    show_launcher: &mut bool,
    active_windows: &mut [WindowInfo],
    system_notifications: &[SystemNotification],
) {
    ui.horizontal(|ui| {
        // Launcher button
        if ui.button("🌱 Start").clicked() {
            *show_launcher = !*show_launcher;
        }

        ui.separator();

        // Collect window data before mutable borrow
        let window_data: Vec<(String, String, bool)> = active_windows
            .iter()
            .map(|w| {
                (
                    w.id.clone(),
                    format!("{} {}", w.view.icon(), truncate_title(&w.title, 15)),
                    w.focused,
                )
            })
            .collect();

        // Active windows in taskbar
        for (window_id, button_text, is_focused) in window_data {
            if ui.selectable_label(is_focused, button_text).clicked() {
                // Focus window
                for w in active_windows.iter_mut() {
                    w.focused = w.id == window_id;
                }
            }
        }

        // Spacer to push system info to right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // System status indicators
            ui.label("🟢"); // System health
            ui.separator();

            // Time
            let now = chrono::Local::now();
            ui.label(now.format("%H:%M").to_string());

            ui.separator();

            // Notifications
            let unread = system_notifications.iter().filter(|n| !n.dismissed).count();
            if unread > 0 {
                ui.label(format!("🔔 {unread}"));
            }
        });
    });
}

fn truncate_title(title: &str, max_len: usize) -> String {
    if title.len() > max_len {
        format!("{}...", &title[..max_len.saturating_sub(3)])
    } else {
        title.to_string()
    }
}
