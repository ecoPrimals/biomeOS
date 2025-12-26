//! System Notifications

use crate::types::SystemNotification;
use eframe::egui;

/// Render system notifications overlay
pub fn render_notifications(ctx: &egui::Context, notifications: &mut Vec<SystemNotification>) {
    let screen_rect = ctx.screen_rect();
    let mut y_offset = 20.0;

    // Remove old notifications (auto-dismiss after 10 seconds)
    notifications.retain(|n| n.timestamp.elapsed().as_secs() < 10 || !n.dismissed);

    for notification in notifications.iter_mut() {
        if notification.dismissed {
            continue;
        }

        let notification_size = egui::vec2(300.0, 80.0);
        let notification_pos = egui::pos2(
            screen_rect.right() - notification_size.x - 20.0,
            screen_rect.top() + y_offset,
        );

        egui::Window::new(&notification.id)
            .fixed_pos(notification_pos)
            .fixed_size(notification_size)
            .title_bar(false)
            .resizable(false)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(notification.notification_type.background_color()),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(notification.notification_type.icon());
                    ui.vertical(|ui| {
                        ui.label(&notification.title);
                        ui.label(&notification.message);
                    });

                    if ui.button("✕").clicked() {
                        notification.dismissed = true;
                    }
                });
            });

        y_offset += notification_size.y + 10.0;
    }
}
