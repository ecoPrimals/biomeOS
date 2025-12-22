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
use crate::state::AppState;
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

/// Window information for desktop mode
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub view: AppView,
    pub position: egui::Pos2,
    pub size: egui::Vec2,
    pub minimized: bool,
    pub maximized: bool,
    pub focused: bool,
}

/// System notifications
#[derive(Debug, Clone)]
pub struct SystemNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: std::time::Instant,
    pub dismissed: bool,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    System,
}

/// New Secure Compute View for orchestrating primals (simplified)
pub struct SecureComputeView {
    pub connected_primals: Vec<PrimalConnection>,
    pub last_sync: Option<std::time::Instant>,
}

#[derive(Debug, Clone)]
pub struct PrimalConnection {
    pub name: String,
    pub primal_type: String, // "beardog", "toadstool", "songbird", etc.
    pub status: ConnectionStatus,
    pub endpoint: String,
    pub last_heartbeat: Option<std::time::Instant>,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

impl Default for SecureComputeView {
    fn default() -> Self {
        Self {
            connected_primals: vec![
                PrimalConnection {
                    name: "Beardog Encryption".to_string(),
                    primal_type: "beardog".to_string(),
                    status: ConnectionStatus::Connected,
                    endpoint: "beardog://localhost:8080".to_string(),
                    last_heartbeat: Some(std::time::Instant::now()),
                },
                PrimalConnection {
                    name: "ToadStool Compute".to_string(),
                    primal_type: "toadstool".to_string(),
                    status: ConnectionStatus::Connected,
                    endpoint: "toadstool://localhost:8081".to_string(),
                    last_heartbeat: Some(std::time::Instant::now()),
                },
                PrimalConnection {
                    name: "Songbird Discovery".to_string(),
                    primal_type: "songbird".to_string(),
                    status: ConnectionStatus::Connecting,
                    endpoint: "songbird://localhost:8082".to_string(),
                    last_heartbeat: None,
                },
            ],
            last_sync: Some(std::time::Instant::now()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    Dashboard,
    Installation,
    Primals,
    Sovereignty,
    Settings,
    YamlEditor,
    Byob,
    IsoCreator,
    NicheManager,
    ToadStool,
    SecureCompute,
    SystemSpinup,
    DesktopLauncher,
}

impl BiomeOSApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self::new_hybrid(_cc)
    }

    pub fn new_minimal(_cc: &eframe::CreationContext) -> Self {
        let state = Arc::new(Mutex::new(AppState::new()));
        let api = Arc::new(BiomeOSApi::new());

        Self {
            state: state.clone(),
            api: api.clone(),
            desktop_mode: false, // Minimal mode - no desktop
            show_launcher: false,
            launcher_search: String::new(),
            active_windows: Vec::new(),
            taskbar_height: 32.0, // Smaller taskbar
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

    pub fn new_full(_cc: &eframe::CreationContext) -> Self {
        let state = Arc::new(Mutex::new(AppState::new()));
        let api = Arc::new(BiomeOSApi::new());

        Self {
            state: state.clone(),
            api: api.clone(),
            desktop_mode: true, // Full desktop mode
            show_launcher: false,
            launcher_search: String::new(),
            active_windows: Vec::new(),
            taskbar_height: 56.0, // Larger taskbar
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
            show_dev_panel: true, // Enable dev panel in full mode
            show_api_debug: true,
            system_notifications: Vec::new(),
        }
    }

    pub fn new_hybrid(_cc: &eframe::CreationContext) -> Self {
        let state = Arc::new(Mutex::new(AppState::new()));
        let api = Arc::new(BiomeOSApi::new());

        Self {
            state: state.clone(),
            api: api.clone(),
            desktop_mode: true, // Default to OS-like desktop mode
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
                // Desktop background with subtle pattern
                self.render_desktop_background(ui);
                
                // Handle window management
                self.render_active_windows(ctx);
            });

        // Taskbar at bottom
        egui::TopBottomPanel::bottom("taskbar")
            .exact_height(self.taskbar_height)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_taskbar(ui);
            });

        // Launcher overlay (when active)
        if self.show_launcher {
            self.render_launcher(ctx);
        }

        // System notifications
        self.render_notifications(ctx);
    }

    /// Render desktop background
    fn render_desktop_background(&self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        
        // Subtle grid pattern
        let painter = ui.painter();
        let grid_size = 64.0;
        let grid_color = egui::Color32::from_rgba_premultiplied(40, 45, 50, 32);
        
        // Vertical lines
        let mut x = 0.0;
        while x < rect.width() {
            painter.line_segment(
                [egui::pos2(rect.left() + x, rect.top()), egui::pos2(rect.left() + x, rect.bottom())],
                egui::Stroke::new(0.5, grid_color),
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = 0.0;
        while y < rect.height() {
            painter.line_segment(
                [egui::pos2(rect.left(), rect.top() + y), egui::pos2(rect.right(), rect.top() + y)],
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
                        .color(egui::Color32::from_rgba_unmultiplied(100, 120, 100, 128))
                );
            });
        });
    }

    /// Render taskbar with launcher and system info
    fn render_taskbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Launcher button
            if ui.button("🌱 Start").clicked() {
                self.show_launcher = !self.show_launcher;
            }

            ui.separator();

            // Collect window data before mutable borrow
            let window_data: Vec<(String, String, bool)> = self.active_windows.iter()
                .map(|w| (
                    w.id.clone(),
                    format!("{} {}", self.get_view_icon(&w.view), self.truncate_title(&w.title, 15)),
                    w.focused
                ))
                .collect();

            // Active windows in taskbar
            for (window_id, button_text, is_focused) in window_data {
                if ui.selectable_label(is_focused, button_text).clicked() {
                    // Focus window
                    for w in &mut self.active_windows {
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
                if !self.system_notifications.is_empty() {
                    let unread = self.system_notifications.iter()
                        .filter(|n| !n.dismissed)
                        .count();
                    if unread > 0 {
                        ui.label(format!("🔔 {}", unread));
                    }
                }
            });
        });
    }

    /// Render application launcher
    fn render_launcher(&mut self, ctx: &egui::Context) {
        let screen_rect = ctx.screen_rect();
        let launcher_size = egui::vec2(400.0, 600.0);
        let launcher_pos = egui::pos2(
            screen_rect.left() + 20.0,
            screen_rect.bottom() - self.taskbar_height - launcher_size.y - 20.0,
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
                    ui.text_edit_singleline(&mut self.launcher_search);
                });

                ui.separator();

                // Application grid
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.columns(3, |columns| {
                        let apps = self.get_launcher_apps();
                        for (i, app) in apps.iter().enumerate() {
                            let col = i % 3;
                            if columns[col].button(format!("{}\n{}", app.icon, app.name)).clicked() {
                                self.launch_app(app.view.clone());
                                self.show_launcher = false;
                            }
                        }
                    });
                });
            });
    }

    /// Get launcher application list
    fn get_launcher_apps(&self) -> Vec<LauncherApp> {
        vec![
            LauncherApp { name: "Dashboard", icon: "🏠", view: AppView::Dashboard },
            LauncherApp { name: "BYOB", icon: "🧬", view: AppView::Byob },
            LauncherApp { name: "ISO Creator", icon: "💿", view: AppView::IsoCreator },
            LauncherApp { name: "Secure Compute", icon: "🔐", view: AppView::SecureCompute },
            LauncherApp { name: "System Spinup", icon: "🚀", view: AppView::SystemSpinup },
            LauncherApp { name: "Primals", icon: "🎯", view: AppView::Primals },
            LauncherApp { name: "ToadStool", icon: "🍄", view: AppView::ToadStool },
            LauncherApp { name: "Sovereignty", icon: "🔒", view: AppView::Sovereignty },
            LauncherApp { name: "Settings", icon: "⚙️", view: AppView::Settings },
        ]
    }

    /// Launch an application in a new window
    fn launch_app(&mut self, view: AppView) {
        let window_id = format!("window_{}", self.active_windows.len());
        let window = WindowInfo {
            id: window_id,
            title: self.get_view_title(&view),
            view,
            position: egui::pos2(100.0 + (self.active_windows.len() as f32 * 30.0), 100.0),
            size: egui::vec2(800.0, 600.0),
            minimized: false,
            maximized: false,
            focused: true,
        };

        // Unfocus other windows
        for w in &mut self.active_windows {
            w.focused = false;
        }

        self.active_windows.push(window);
    }

    /// Render active windows
    fn render_active_windows(&mut self, ctx: &egui::Context) {
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

            window.show(ctx, |ui| {
                // Window content based on view type
                match window_info.view {
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
                }
            });

            // Update window info if needed
            if let Some(window) = self.active_windows.get_mut(i) {
                // Handle window interactions
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    window.focused = false;
                }
            }
        }
    }

    /// Render secure compute view - orchestrates primals (simplified)
    fn render_secure_compute_view(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("🔐 Primal Orchestration Center");
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("🐕 Beardog - Encryption Services");
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::GREEN, "●");
                        ui.label("Beardog Primal");
                        ui.label("Status: Ready");
                    });
                    
                    if ui.button("🔒 Request Encryption").clicked() {
                        self.add_notification("Beardog", "Encryption request sent to Beardog primal", NotificationType::Info);
                    }
                    
                    if ui.button("🔑 Key Management").clicked() {
                        self.add_notification("Beardog", "Opening key management interface", NotificationType::Info);
                    }
                    
                    if ui.button("🛡️ Security Policies").clicked() {
                        self.add_notification("Beardog", "Loading security policies from Beardog", NotificationType::Info);
                    }
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("🍄 ToadStool - Computation Services");
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::GREEN, "●");
                        ui.label("ToadStool Primal");
                        ui.label("Status: Computing");
                    });
                    
                    if ui.button("⚡ Submit Job").clicked() {
                        self.add_notification("ToadStool", "Job submitted to ToadStool compute cluster", NotificationType::Success);
                    }
                    
                    if ui.button("📊 View Results").clicked() {
                        self.add_notification("ToadStool", "Fetching compute results from ToadStool", NotificationType::Info);
                    }
                    
                    if ui.button("🎛️ Runtime Config").clicked() {
                        self.add_notification("ToadStool", "Opening ToadStool runtime configuration", NotificationType::Info);
                    }
                });
            });
        });

        ui.separator();

        // Orchestration controls
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("🎭 Orchestration Controls");
                
                if ui.button("🔄 Sync All Primals").clicked() {
                    self.add_notification("System", "Syncing with all ecosystem primals", NotificationType::System);
                }
                
                if ui.button("🌐 Ecosystem Health").clicked() {
                    self.add_notification("System", "Checking ecosystem-wide health status", NotificationType::System);
                }
                
                if ui.button("📋 Job Pipeline").clicked() {
                    self.add_notification("System", "Opening cross-primal job pipeline", NotificationType::Info);
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
            // Quick launch buttons
            ui.vertical(|ui| {
                ui.heading("Quick Launch");
                
                if ui.button("🖥️ Development Environment").clicked() {
                    // Launch development system
                }
                
                if ui.button("🏢 Production Cluster").clicked() {
                    // Launch production cluster
                }
                
                if ui.button("🧪 Testing Sandbox").clicked() {
                    // Launch testing environment
                }
                
                if ui.button("🔒 Secure Workspace").clicked() {
                    // Launch secure environment
                }
            });

            ui.separator();

            // System status
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

    /// Render system notifications
    fn render_notifications(&mut self, ctx: &egui::Context) {
        let screen_rect = ctx.screen_rect();
        let mut y_offset = 20.0;

        // Remove old notifications
        self.system_notifications
            .retain(|n| n.timestamp.elapsed().as_secs() < 10 || !n.dismissed);

        for notification in &mut self.system_notifications {
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
                .frame(egui::Frame::window(&ctx.style()).fill(
                    match notification.notification_type {
                        NotificationType::Error => egui::Color32::from_rgb(80, 20, 20),
                        NotificationType::Warning => egui::Color32::from_rgb(80, 60, 20),
                        NotificationType::Success => egui::Color32::from_rgb(20, 80, 20),
                        _ => egui::Color32::from_rgb(30, 35, 40),
                    }
                ))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let icon = match notification.notification_type {
                            NotificationType::Error => "❌",
                            NotificationType::Warning => "⚠️",
                            NotificationType::Success => "✅",
                            NotificationType::Info => "ℹ️",
                            NotificationType::System => "⚙️",
                        };
                        
                        ui.label(icon);
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

    /// Helper functions
    fn get_view_icon(&self, view: &AppView) -> &'static str {
        match view {
            AppView::Dashboard => "🏠",
            AppView::Byob => "🧬",
            AppView::IsoCreator => "💿",
            AppView::SecureCompute => "🔐",
            AppView::SystemSpinup => "🚀",
            AppView::Primals => "🎯",
            AppView::ToadStool => "🍄",
            AppView::Sovereignty => "🔒",
            AppView::Settings => "⚙️",
            _ => "📱",
        }
    }

    fn get_view_title(&self, view: &AppView) -> String {
        match view {
            AppView::Dashboard => "BiomeOS Dashboard",
            AppView::Byob => "Build Your Own Biome",
            AppView::IsoCreator => "ISO Creator",
            AppView::SecureCompute => "Secure Computation",
            AppView::SystemSpinup => "System Spinup",
            AppView::Primals => "Ecosystem Primals",
            AppView::ToadStool => "ToadStool Compute",
            AppView::Sovereignty => "Digital Sovereignty",
            AppView::Settings => "System Settings",
            _ => "BiomeOS Application",
        }.to_string()
    }

    fn truncate_title(&self, title: &str, max_len: usize) -> String {
        if title.len() > max_len {
            format!("{}...", &title[..max_len-3])
        } else {
            title.to_string()
        }
    }

    /// Traditional navigation for non-desktop mode
    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // biomeOS logo and title
            ui.heading("🌱 biomeOS");

            ui.separator();

            // Desktop mode toggle
            if ui.button(if self.desktop_mode { "📱 Classic" } else { "🖥️ Desktop" }).clicked() {
                self.desktop_mode = !self.desktop_mode;
            }

            if !self.desktop_mode {
                ui.separator();

                // Main navigation buttons (only in classic mode)
            if ui
                .selectable_label(self.current_view == AppView::Dashboard, "🏠 Dashboard")
                .clicked()
            {
                self.current_view = AppView::Dashboard;
            }

            if ui
                .selectable_label(self.current_view == AppView::YamlEditor, "📝 YAML Editor")
                .clicked()
            {
                self.current_view = AppView::YamlEditor;
            }

            if ui
                .selectable_label(self.current_view == AppView::Byob, "🧬 BYOB")
                .clicked()
            {
                self.current_view = AppView::Byob;
            }

            if ui
                .selectable_label(self.current_view == AppView::NicheManager, "🎭 Niches")
                .clicked()
            {
                self.current_view = AppView::NicheManager;
            }

            if ui
                .selectable_label(self.current_view == AppView::IsoCreator, "💿 ISO Creator")
                .clicked()
            {
                self.current_view = AppView::IsoCreator;
            }

            if ui
                    .selectable_label(self.current_view == AppView::SecureCompute, "🔐 Secure Compute")
                .clicked()
            {
                    self.current_view = AppView::SecureCompute;
                }

                // Additional views...
            }

            // Spacer to push settings to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .selectable_label(self.current_view == AppView::Settings, "⚙️ Settings")
                    .clicked()
                {
                    self.current_view = AppView::Settings;
                }

                ui.separator();

                // Developer tools toggle
                ui.checkbox(&mut self.show_dev_panel, "🔧 Dev");
                ui.checkbox(&mut self.show_api_debug, "🌐 API");
            });
        });
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
                self.add_notification("Test", "This is a test notification", NotificationType::Info);
            }
        });
    }

    /// Add system notification
    fn add_notification(&mut self, title: &str, message: &str, notification_type: NotificationType) {
        let notification = SystemNotification {
            id: format!("notif_{}", self.system_notifications.len()),
            title: title.to_string(),
            message: message.to_string(),
            notification_type,
            timestamp: std::time::Instant::now(),
            dismissed: false,
        };
        self.system_notifications.push(notification);
    }

    /// Render status bar
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Status: Ready");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💬 Feedback").clicked() {
                    // Handle feedback
                }
                
                ui.separator();
                ui.label("biomeOS v1.0");
            });
        });
    }
}

#[derive(Debug, Clone)]
struct LauncherApp {
    name: &'static str,
    icon: &'static str,
    view: AppView,
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
            // OS-like desktop interface
            self.render_desktop(ctx, frame);
        } else {
            // Traditional single-view interface
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

        // Developer panel (collapsible)
        if self.show_dev_panel {
            egui::SidePanel::right("dev_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.render_dev_panel(ui);
                });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_current_view(ui, ctx);
        });
        }

        // Auto-refresh the UI for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
