//! Window Management

use crate::types::{AppView, WindowInfo};
use eframe::egui;

/// Context for rendering view content within windows
pub struct ViewContext<'a> {
    pub dashboard: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub byob: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub iso_creator: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub secure_compute: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub system_spinup: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub primals: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub toadstool: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub sovereignty: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
    pub settings: &'a mut dyn FnMut(&mut egui::Ui, &egui::Context),
}

/// Render active windows on the desktop
pub fn render_active_windows(
    ctx: &egui::Context,
    active_windows: &mut Vec<WindowInfo>,
    view_ctx: &mut ViewContext<'_>,
) {
    let windows_to_render: Vec<_> = active_windows.clone();

    for (i, window_info) in windows_to_render.iter().enumerate() {
        if window_info.minimized {
            continue;
        }

        let mut window = egui::Window::new(&window_info.title)
            .id(egui::Id::new(&window_info.id))
            .default_pos(window_info.position)
            .default_size(window_info.size)
            .resizable(true)
            .collapsible(false);

        if window_info.maximized {
            window = window.fixed_size(ctx.screen_rect().size());
            window = window.fixed_pos(egui::pos2(0.0, 0.0));
        }

        window.show(ctx, |ui| {
            // Window content based on view type
            match window_info.view {
                AppView::Dashboard => (view_ctx.dashboard)(ui, ctx),
                AppView::Byob => (view_ctx.byob)(ui, ctx),
                AppView::IsoCreator => (view_ctx.iso_creator)(ui, ctx),
                AppView::SecureCompute => (view_ctx.secure_compute)(ui, ctx),
                AppView::SystemSpinup => (view_ctx.system_spinup)(ui, ctx),
                AppView::Primals => (view_ctx.primals)(ui, ctx),
                AppView::ToadStool => (view_ctx.toadstool)(ui, ctx),
                AppView::Sovereignty => (view_ctx.sovereignty)(ui, ctx),
                AppView::Settings => (view_ctx.settings)(ui, ctx),
                _ => {
                    ui.centered_and_justified(|ui| {
                        ui.label("Coming Soon!");
                    });
                }
            }
        });

        // Update window info if needed
        if let Some(window) = active_windows.get_mut(i) {
            // Handle window interactions
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                window.focused = false;
            }
        }
    }
}
