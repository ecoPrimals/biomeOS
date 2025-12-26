//! Desktop Background Rendering

use eframe::egui;

/// Render desktop background with grid pattern
pub fn render_desktop_background(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();

    // Subtle grid pattern
    let painter = ui.painter();
    let grid_size = 64.0;
    let grid_color = egui::Color32::from_rgba_premultiplied(40, 45, 50, 32);

    // Vertical lines
    let mut x = 0.0;
    while x < rect.width() {
        painter.line_segment(
            [
                egui::pos2(rect.left() + x, rect.top()),
                egui::pos2(rect.left() + x, rect.bottom()),
            ],
            egui::Stroke::new(0.5, grid_color),
        );
        x += grid_size;
    }

    // Horizontal lines
    let mut y = 0.0;
    while y < rect.height() {
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.top() + y),
                egui::pos2(rect.right(), rect.top() + y),
            ],
            egui::Stroke::new(0.5, grid_color),
        );
        y += grid_size;
    }

    // BiomeOS logo watermark
    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.label(
                egui::RichText::new("🌱 BiomeOS")
                    .size(24.0)
                    .color(egui::Color32::from_rgba_unmultiplied(100, 120, 100, 128)),
            );
        });
    });
}
