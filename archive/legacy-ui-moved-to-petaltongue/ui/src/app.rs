//! Main biomeOS Application State and UI Orchestration
//!
//! This module implements the core UI application following biomeOS design principles:
//! - OS-like desktop experience
//! - Universal/recursive patterns  
//! - Sovereignty-first UX
//! - Bootstrap/foundational for developers

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::BiomeOSApi;
use crate::desktop::{self, windows::ViewContext};
use crate::state::AppState;
use crate::types::{AppView, NotificationType, SecureComputeView, SystemNotification, WindowInfo};
use crate::views::{
    byob::ByobView, dashboard::DashboardView, installation::InstallationView,
    iso_creator::IsoCreatorView, niche_manager::NicheManagerView, primals::PrimalsView,
    settings::SettingsView, sovereignty::SovereigntyView, toadstool::ToadStoolView,
    yaml_editor::YamlEditorView, View,
};

/// Main biomeOS UI Application with OS-like desktop interface
pub struct BiomeOSApp {
    /// Application state (shared across views)
    state: Arc<Mutex<AppState>>,

    /// API client for biomeOS core
    api: Arc<BiomeOSApi>,

    /// Desktop interface state
    desktop_mode: bool,
    show_launcher: bool,
    launcher_search: String,
    active_windows: Vec<WindowInfo>,
    taskbar_height: f32,

    /// Current active view (for non-desktop mode)
    current_view: AppView,

    /// View instances
    dashboard_view: DashboardView,
    installation_view: InstallationView,
    primals_view: PrimalsView,
    sovereignty_view: SovereigntyView,
    settings_view: SettingsView,
    yaml_editor_view: YamlEditorView,
    byob_view: ByobView,
    iso_creator_view: IsoCreatorView,
    niche_manager_view: NicheManagerView,
    toadstool_view: ToadStoolView,
    secure_compute_view: SecureComputeView,

    /// UI state
    show_dev_panel: bool,
    show_api_debug: bool,
    system_notifications: Vec<SystemNotification>,
}

impl BiomeOSApp {
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self::new_hybrid(_cc)
    }

    #[must_use]
    pub fn new_minimal(_cc: &eframe::CreationContext) -> Self {
        let state = Arc::new(Mutex::new(AppState::new()));
        let api = Arc::new(BiomeOSApi::new());

        Self {
            state: state.clone(),
            api: api.clone(),
            desktop_mode: false,
            show_launcher: false,
            launcher_search: String::new(),
            active_windows: Vec::new(),
            taskbar_height: 32.0,
            current_view: AppView::Dashboard,
            dashboard_view: DashboardView::new(state.clone(), api.clone()),
            installation_view: InstallationView::new(state.clone(), api.clone()),
            primals_view: PrimalsView::new(state.clone(), api.clone()),
            sovereignty_view: SovereigntyView::new(state.clone(), api.clone()),
            settings_view: SettingsView::new(state.clone(), api.clone()),
            yaml_editor_view: YamlEditorView::new(state.clone(), api.clone()),
            byob_view: ByobView::new(),
            iso_creator_view: IsoCreatorView::new(state.clone(), api.clone()),
            niche_manager_view: NicheManagerView::new(state.clone(), api.clone()),
            toadstool_view: ToadStoolView::new(state.clone(), api.clone()),
            secure_compute_view: SecureComputeView::default(),
            show_dev_panel: false,
            show_api_debug: false,
            system_notifications: Vec::new(),
        }
    }

    #[must_use]
    pub fn new_full(_cc: &eframe::CreationContext) -> Self {
        let state = Arc::new(Mutex::new(AppState::new()));
        let api = Arc::new(BiomeOSApi::new());

        Self {
            state: state.clone(),
            api: api.clone(),
            desktop_mode: true,
            show_launcher: false,
            launcher_search: String::new(),
            active_windows: Vec::new(),
            taskbar_height: 56.0,
            current_view: AppView::Dashboard,
            dashboard_view: DashboardView::new(state.clone(), api.clone()),
            installation_view: InstallationView::new(state.clone(), api.clone()),
            primals_view: PrimalsView::new(state.clone(), api.clone()),
            sovereignty_view: SovereigntyView::new(state.clone(), api.clone()),
            settings_view: SettingsView::new(state.clone(), api.clone()),
            yaml_editor_view: YamlEditorView::new(state.clone(), api.clone()),
            byob_view: ByobView::new(),
            iso_creator_view: IsoCreatorView::new(state.clone(), api.clone()),
            niche_manager_view: NicheManagerView::new(state.clone(), api.clone()),
            toadstool_view: ToadStoolView::new(state.clone(), api.clone()),
            secure_compute_view: SecureComputeView::default(),
            show_dev_panel: true,
            show_api_debug: true,
            system_notifications: Vec::new(),
        }
    }

    #[must_use]
    pub fn new_hybrid(_cc: &eframe::CreationContext) -> Self {
        let state = Arc::new(Mutex::new(AppState::new()));
        let api = Arc::new(BiomeOSApi::new());

        Self {
            state: state.clone(),
            api: api.clone(),
            desktop_mode: true,
            show_launcher: false,
            launcher_search: String::new(),
            active_windows: Vec::new(),
            taskbar_height: 48.0,
            current_view: AppView::Dashboard,
            dashboard_view: DashboardView::new(state.clone(), api.clone()),
            installation_view: InstallationView::new(state.clone(), api.clone()),
            primals_view: PrimalsView::new(state.clone(), api.clone()),
            sovereignty_view: SovereigntyView::new(state.clone(), api.clone()),
            settings_view: SettingsView::new(state.clone(), api.clone()),
            yaml_editor_view: YamlEditorView::new(state.clone(), api.clone()),
            byob_view: ByobView::new(),
            iso_creator_view: IsoCreatorView::new(state.clone(), api.clone()),
            niche_manager_view: NicheManagerView::new(state.clone(), api.clone()),
            toadstool_view: ToadStoolView::new(state.clone(), api.clone()),
            secure_compute_view: SecureComputeView::default(),
            show_dev_panel: false,
            show_api_debug: false,
            system_notifications: Vec::new(),
        }
    }

    /// Render OS-like desktop interface
    fn render_desktop(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Desktop wallpaper/background
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 25, 30)))
            .show(ctx, |ui| {
                desktop::render_desktop_background(ui);
                self.render_active_windows(ctx);
            });

        // Taskbar at bottom
        egui::TopBottomPanel::bottom("taskbar")
            .exact_height(self.taskbar_height)
            .resizable(false)
            .show(ctx, |ui| {
                desktop::render_taskbar(
                    ui,
                    &mut self.show_launcher,
                    &mut self.active_windows,
                    &self.system_notifications,
                );
            });

        // Launcher overlay (when active)
        if self.show_launcher {
            desktop::render_launcher(
                ctx,
                self.taskbar_height,
                &mut self.launcher_search,
                &mut self.show_launcher,
                &mut self.active_windows,
            );
        }

        // System notifications
        desktop::render_notifications(ctx, &mut self.system_notifications);
    }

    /// Render active windows - delegates to desktop module but provides view callbacks
    fn render_active_windows(&mut self, ctx: &egui::Context) {
        // We need to manually handle this due to borrow checker limitations
        // with closures capturing multiple fields
        let windows_to_render: Vec<_> = self.active_windows.clone();

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

            window.show(ctx, |ui| match window_info.view {
                AppView::Dashboard => self.dashboard_view.render(ui, ctx),
                AppView::Byob => self.byob_view.render(ui, ctx),
                AppView::IsoCreator => self.iso_creator_view.render(ui, ctx),
                AppView::SecureCompute => self.render_secure_compute_view(ui, ctx),
                AppView::SystemSpinup => self.render_system_spinup_view(ui, ctx),
                AppView::Primals => self.primals_view.render(ui, ctx),
                AppView::ToadStool => self.toadstool_view.render(ui, ctx),
                AppView::Sovereignty => self.sovereignty_view.render(ui, ctx),
                AppView::Settings => self.settings_view.render(ui, ctx),
                _ => {
                    ui.centered_and_justified(|ui| {
                        ui.label("Coming Soon!");
                    });
                }
            });

            if let Some(window) = self.active_windows.get_mut(i) {
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    window.focused = false;
                }
            }
        }
    }

    /// Render secure compute view - orchestrates primals
    fn render_secure_compute_view(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("🔐 Primal Orchestration Center");
        ui.separator();

        ui.horizontal(|ui| {
            self.render_beardog_panel(ui);
            ui.separator();
            self.render_toadstool_panel(ui);
        });

        ui.separator();
        self.render_orchestration_controls(ui);
    }

    fn render_beardog_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("🐕 Beardog - Encryption Services");

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::GREEN, "●");
                    ui.label("Beardog Primal");
                    ui.label("Status: Ready");
                });

                if ui.button("🔒 Request Encryption").clicked() {
                    self.add_notification(
                        "Beardog",
                        "Encryption request sent",
                        NotificationType::Info,
                    );
                }

                if ui.button("🔑 Key Management").clicked() {
                    self.add_notification(
                        "Beardog",
                        "Opening key management",
                        NotificationType::Info,
                    );
                }

                if ui.button("🛡️ Security Policies").clicked() {
                    self.add_notification(
                        "Beardog",
                        "Loading security policies",
                        NotificationType::Info,
                    );
                }
            });
        });
    }

    fn render_toadstool_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("🍄 ToadStool - Computation Services");

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::GREEN, "●");
                    ui.label("ToadStool Primal");
                    ui.label("Status: Computing");
                });

                if ui.button("⚡ Submit Job").clicked() {
                    self.add_notification("ToadStool", "Job submitted", NotificationType::Success);
                }

                if ui.button("📊 View Results").clicked() {
                    self.add_notification("ToadStool", "Fetching results", NotificationType::Info);
                }

                if ui.button("🎛️ Runtime Config").clicked() {
                    self.add_notification(
                        "ToadStool",
                        "Opening configuration",
                        NotificationType::Info,
                    );
                }
            });
        });
    }

    fn render_orchestration_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("🎭 Orchestration Controls");

                if ui.button("🔄 Sync All Primals").clicked() {
                    self.add_notification("System", "Syncing primals", NotificationType::System);
                }

                if ui.button("🌐 Ecosystem Health").clicked() {
                    self.add_notification("System", "Checking health", NotificationType::System);
                }

                if ui.button("📋 Job Pipeline").clicked() {
                    self.add_notification("System", "Opening pipeline", NotificationType::Info);
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("🔗 Active Connections");

                ui.group(|ui| {
                    ui.label("🐕 Beardog: Connected");
                    ui.label("🍄 ToadStool: Active (3 jobs)");
                    ui.label("🦜 Songbird: Discovering");
                    ui.label("🐿️ Squirrel: Connected");
                    ui.label("🦆 NestGate: Ready");
                });
            });
        });
    }

    /// Render system spinup view
    fn render_system_spinup_view(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("🚀 System Spinup & Deployment");
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Quick Launch");

                if ui.button("🖥️ Development Environment").clicked() {}
                if ui.button("🏢 Production Cluster").clicked() {}
                if ui.button("🧪 Testing Sandbox").clicked() {}
                if ui.button("🔒 Secure Workspace").clicked() {}
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Active Systems");

                ui.group(|ui| {
                    ui.label("🟢 dev-env-001 (Running)");
                    ui.label("CPU: 45% | Memory: 2.1GB | Uptime: 2h 15m");
                });

                ui.group(|ui| {
                    ui.label("🟡 prod-cluster-001 (Starting)");
                    ui.add(egui::ProgressBar::new(0.75).show_percentage());
                });
            });
        });
    }

    /// Traditional navigation for non-desktop mode
    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🌱 biomeOS");
            ui.separator();

            if ui
                .button(if self.desktop_mode {
                    "📱 Classic"
                } else {
                    "🖥️ Desktop"
                })
                .clicked()
            {
                self.desktop_mode = !self.desktop_mode;
            }

            if !self.desktop_mode {
                ui.separator();
                self.render_nav_buttons(ui);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .selectable_label(self.current_view == AppView::Settings, "⚙️ Settings")
                    .clicked()
                {
                    self.current_view = AppView::Settings;
                }
                ui.separator();
                ui.checkbox(&mut self.show_dev_panel, "🔧 Dev");
                ui.checkbox(&mut self.show_api_debug, "🌐 API");
            });
        });
    }

    fn render_nav_buttons(&mut self, ui: &mut egui::Ui) {
        let nav_items = [
            (AppView::Dashboard, "🏠 Dashboard"),
            (AppView::YamlEditor, "📝 YAML Editor"),
            (AppView::Byob, "🧬 BYOB"),
            (AppView::NicheManager, "🎭 Niches"),
            (AppView::IsoCreator, "💿 ISO Creator"),
            (AppView::SecureCompute, "🔐 Secure Compute"),
        ];

        for (view, label) in nav_items {
            if ui
                .selectable_label(self.current_view == view, label)
                .clicked()
            {
                self.current_view = view;
            }
        }
    }

    /// Render the current active view (classic mode)
    fn render_current_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        match self.current_view {
            AppView::Dashboard => self.dashboard_view.render(ui, ctx),
            AppView::YamlEditor => self.yaml_editor_view.render(ui, ctx),
            AppView::Byob => self.byob_view.render(ui, ctx),
            AppView::NicheManager => self.niche_manager_view.render(ui, ctx),
            AppView::IsoCreator => self.iso_creator_view.render(ui, ctx),
            AppView::SecureCompute => self.render_secure_compute_view(ui, ctx),
            AppView::Installation => self.installation_view.render(ui, ctx),
            AppView::Primals => self.primals_view.render(ui, ctx),
            AppView::Sovereignty => self.sovereignty_view.render(ui, ctx),
            AppView::Settings => self.settings_view.render(ui, ctx),
            AppView::ToadStool => self.toadstool_view.render(ui, ctx),
            _ => {
                ui.centered_and_justified(|ui| {
                    ui.label("View not implemented in classic mode");
                });
            }
        }
    }

    /// Render developer panel (when enabled)
    fn render_dev_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_dev_panel {
            return;
        }

        ui.collapsing("🔧 Developer Tools", |ui| {
            ui.label("Environment: Development");
            ui.label(format!("Active Windows: {}", self.active_windows.len()));
            ui.label(format!("Desktop Mode: {}", self.desktop_mode));

            if ui.button("Clear Notifications").clicked() {
                self.system_notifications.clear();
            }

            if ui.button("Test Notification").clicked() {
                self.add_notification(
                    "Test",
                    "This is a test notification",
                    NotificationType::Info,
                );
            }
        });
    }

    /// Add system notification
    fn add_notification(
        &mut self,
        title: &str,
        message: &str,
        notification_type: NotificationType,
    ) {
        let id = format!("notif_{}", self.system_notifications.len());
        self.system_notifications.push(SystemNotification::new(
            id,
            title.to_string(),
            message.to_string(),
            notification_type,
        ));
    }

    /// Render status bar
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Status: Ready");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💬 Feedback").clicked() {}
                ui.separator();
                ui.label("biomeOS v1.0");
            });
        });
    }
}

impl eframe::App for BiomeOSApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Handle command line arguments for specific modes
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|arg| arg == "--yaml-editor") {
            self.current_view = AppView::YamlEditor;
        } else if args.iter().any(|arg| arg == "--byob") {
            self.current_view = AppView::Byob;
        } else if args.iter().any(|arg| arg == "--iso-creator") {
            self.current_view = AppView::IsoCreator;
        } else if args.iter().any(|arg| arg == "--niche-manager") {
            self.current_view = AppView::NicheManager;
        } else if args.iter().any(|arg| arg == "--desktop") {
            self.desktop_mode = true;
        } else if args.iter().any(|arg| arg == "--classic") {
            self.desktop_mode = false;
        }

        if self.desktop_mode {
            self.render_desktop(ctx, frame);
        } else {
            egui::TopBottomPanel::top("navigation")
                .resizable(false)
                .show(ctx, |ui| {
                    self.render_navigation(ui);
                });

            egui::TopBottomPanel::bottom("status")
                .resizable(false)
                .show(ctx, |ui| {
                    self.render_status_bar(ui);
                });

            if self.show_dev_panel {
                egui::SidePanel::right("dev_panel")
                    .resizable(true)
                    .default_width(300.0)
                    .show(ctx, |ui| {
                        self.render_dev_panel(ui);
                    });
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                self.render_current_view(ui, ctx);
            });
        }

        // Auto-refresh the UI for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
