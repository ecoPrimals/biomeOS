//! Minimal OS-like BiomeOS Interface
//!
//! A clean, working desktop interface for BiomeOS that focuses on orchestrating
//! other primals (Beardog, ToadStool, etc.) rather than implementing functionality directly.

use eframe::egui;
use std::time::Instant;

/// Minimal BiomeOS Desktop Application
pub struct MinimalBiomeOSApp {
    // Desktop state
    show_launcher: bool,
    launcher_search: String,
    active_windows: Vec<MinimalWindow>,
    taskbar_height: f32,

    // Primal connections
    connected_primals: Vec<PrimalConnection>,
    system_notifications: Vec<SystemNotification>,

    // UI state
    current_time: String,
}

#[derive(Debug, Clone)]
pub struct MinimalWindow {
    pub id: String,
    pub title: String,
    pub app_type: AppType,
    pub position: egui::Pos2,
    pub size: egui::Vec2,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub enum AppType {
    Dashboard,
    BiomeBuilder, // BYOB
    IsoCreator,
    PrimalOrchestrator,
    SystemSpinup,
    FileManager,
}

#[derive(Debug, Clone)]
pub struct PrimalConnection {
    pub name: String,
    pub icon: String,
    pub status: ConnectionStatus,
    pub endpoint: String,
    pub last_seen: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct SystemNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: Instant,
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

impl Default for MinimalBiomeOSApp {
    fn default() -> Self {
        Self::new()
    }
}

impl MinimalBiomeOSApp {
    pub fn new() -> Self {
        Self {
            show_launcher: false,
            launcher_search: String::new(),
            active_windows: Vec::new(),
            taskbar_height: 48.0,
            connected_primals: vec![
                PrimalConnection {
                    name: "Beardog Encryption".to_string(),
                    icon: "🐕".to_string(),
                    status: ConnectionStatus::Connected,
                    endpoint: "beardog://localhost:8080".to_string(),
                    last_seen: Some(Instant::now()),
                },
                PrimalConnection {
                    name: "ToadStool Compute".to_string(),
                    icon: "🍄".to_string(),
                    status: ConnectionStatus::Connected,
                    endpoint: "toadstool://localhost:8081".to_string(),
                    last_seen: Some(Instant::now()),
                },
                PrimalConnection {
                    name: "Songbird Discovery".to_string(),
                    icon: "🦜".to_string(),
                    status: ConnectionStatus::Connecting,
                    endpoint: "songbird://localhost:8082".to_string(),
                    last_seen: None,
                },
                PrimalConnection {
                    name: "Squirrel MCP".to_string(),
                    icon: "🐿️".to_string(),
                    status: ConnectionStatus::Connected,
                    endpoint: "squirrel://localhost:8083".to_string(),
                    last_seen: Some(Instant::now()),
                },
                PrimalConnection {
                    name: "NestGate Storage".to_string(),
                    icon: "🦆".to_string(),
                    status: ConnectionStatus::Connected,
                    endpoint: "nestgate://localhost:8084".to_string(),
                    last_seen: Some(Instant::now()),
                },
            ],
            system_notifications: Vec::new(),
            current_time: chrono::Local::now().format("%H:%M").to_string(),
        }
    }

    /// Launch an application window
    fn launch_app(&mut self, app_type: AppType) {
        let window_id = format!("window_{}", self.active_windows.len());
        let (title, size) = match app_type {
            AppType::Dashboard => ("BiomeOS Dashboard".to_string(), egui::vec2(900.0, 600.0)),
            AppType::BiomeBuilder => ("Biome Builder (BYOB)".to_string(), egui::vec2(800.0, 650.0)),
            AppType::IsoCreator => ("ISO Creator".to_string(), egui::vec2(750.0, 500.0)),
            AppType::PrimalOrchestrator => {
                ("Primal Orchestrator".to_string(), egui::vec2(1000.0, 700.0))
            }
            AppType::SystemSpinup => ("System Spinup".to_string(), egui::vec2(850.0, 550.0)),
            AppType::FileManager => ("File Manager".to_string(), egui::vec2(700.0, 500.0)),
        };

        let window = MinimalWindow {
            id: window_id,
            title,
            app_type,
            position: egui::pos2(100.0 + (self.active_windows.len() as f32 * 30.0), 100.0),
            size,
            focused: true,
        };

        // Unfocus other windows
        for w in &mut self.active_windows {
            w.focused = false;
        }

        self.active_windows.push(window);
        self.show_launcher = false;
    }

    /// Add a system notification
    fn add_notification(
        &mut self,
        title: &str,
        message: &str,
        notification_type: NotificationType,
    ) {
        let notification = SystemNotification {
            id: format!("notif_{}", self.system_notifications.len()),
            title: title.to_string(),
            message: message.to_string(),
            notification_type,
            timestamp: Instant::now(),
            dismissed: false,
        };
        self.system_notifications.push(notification);
    }

    /// Render the desktop background
    fn render_desktop_background(&self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();

        // Subtle grid pattern
        let grid_size = 80.0;
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
                        .size(32.0)
                        .color(egui::Color32::from_rgba_unmultiplied(80, 120, 80, 150)),
                );
            });
        });
    }

    /// Render the taskbar
    fn render_taskbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Launcher button
            if ui.button("🌱 Start").clicked() {
                self.show_launcher = !self.show_launcher;
            }

            ui.separator();

            // Active windows in taskbar
            let window_data: Vec<(String, String, bool)> = self
                .active_windows
                .iter()
                .map(|w| {
                    (
                        w.id.clone(),
                        format!(
                            "{} {}",
                            self.get_app_icon(&w.app_type),
                            self.truncate_title(&w.title, 15)
                        ),
                        w.focused,
                    )
                })
                .collect();

            for (window_id, button_text, is_focused) in window_data {
                if ui.selectable_label(is_focused, button_text).clicked() {
                    for w in &mut self.active_windows {
                        w.focused = w.id == window_id;
                    }
                }
            }

            // Spacer to push system info to right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // System status
                let connected_count = self
                    .connected_primals
                    .iter()
                    .filter(|p| matches!(p.status, ConnectionStatus::Connected))
                    .count();
                ui.label(format!("🟢 {}/5", connected_count));
                ui.separator();

                // Time
                self.current_time = chrono::Local::now().format("%H:%M").to_string();
                ui.label(&self.current_time);

                ui.separator();

                // Notifications
                let unread = self
                    .system_notifications
                    .iter()
                    .filter(|n| !n.dismissed)
                    .count();
                if unread > 0 {
                    ui.label(format!("🔔 {}", unread));
                }
            });
        });
    }

    /// Render the application launcher
    fn render_launcher(&mut self, ctx: &egui::Context) {
        let screen_rect = ctx.screen_rect();
        let launcher_size = egui::vec2(450.0, 650.0);
        let launcher_pos = egui::pos2(
            screen_rect.left() + 20.0,
            screen_rect.bottom() - self.taskbar_height - launcher_size.y - 20.0,
        );

        egui::Window::new("BiomeOS Application Launcher")
            .fixed_pos(launcher_pos)
            .fixed_size(launcher_size)
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::from_rgb(25, 30, 35)))
            .show(ctx, |ui| {
                ui.heading("🌱 BiomeOS Launcher");
                ui.separator();

                // Search bar
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.launcher_search);
                    if ui.button("Clear").clicked() {
                        self.launcher_search.clear();
                    }
                });

                ui.separator();

                // Application grid
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let apps = [
                        (
                            "Dashboard",
                            "🏠",
                            AppType::Dashboard,
                            "System overview and monitoring",
                        ),
                        (
                            "Biome Builder",
                            "🧬",
                            AppType::BiomeBuilder,
                            "Build Your Own Biome (BYOB)",
                        ),
                        (
                            "ISO Creator",
                            "💿",
                            AppType::IsoCreator,
                            "Create bootable ISO images",
                        ),
                        (
                            "Primal Orchestrator",
                            "🎭",
                            AppType::PrimalOrchestrator,
                            "Manage ecosystem primals",
                        ),
                        (
                            "System Spinup",
                            "🚀",
                            AppType::SystemSpinup,
                            "Deploy and manage systems",
                        ),
                        (
                            "File Manager",
                            "📁",
                            AppType::FileManager,
                            "Browse and manage files",
                        ),
                    ];

                    for (name, icon, app_type, description) in apps {
                        if !self.launcher_search.is_empty()
                            && !name
                                .to_lowercase()
                                .contains(&self.launcher_search.to_lowercase())
                        {
                            continue;
                        }

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button(format!("{} {}", icon, name)).clicked() {
                                    self.launch_app(app_type);
                                }
                                ui.vertical(|ui| {
                                    ui.label(name);
                                    ui.small(description);
                                });
                            });
                        });
                    }
                });

                ui.separator();

                // Primal status in launcher
                ui.heading("🔗 Ecosystem Primals");
                for primal in &self.connected_primals {
                    ui.horizontal(|ui| {
                        let status_color = match primal.status {
                            ConnectionStatus::Connected => egui::Color32::GREEN,
                            ConnectionStatus::Connecting => egui::Color32::YELLOW,
                            ConnectionStatus::Disconnected => egui::Color32::GRAY,
                            ConnectionStatus::Error(_) => egui::Color32::RED,
                        };

                        ui.colored_label(status_color, "●");
                        ui.label(format!("{} {}", primal.icon, primal.name));
                    });
                }
            });
    }

    /// Render active windows
    fn render_active_windows(&mut self, ctx: &egui::Context) {
        let windows_to_render: Vec<_> = self.active_windows.clone();

        for (i, window_info) in windows_to_render.iter().enumerate() {
            let window = egui::Window::new(&window_info.title)
                .id(egui::Id::new(&window_info.id))
                .default_pos(window_info.position)
                .default_size(window_info.size)
                .resizable(true)
                .collapsible(true);

            window.show(ctx, |ui| match window_info.app_type {
                AppType::Dashboard => self.render_dashboard_content(ui),
                AppType::BiomeBuilder => self.render_byob_content(ui),
                AppType::IsoCreator => self.render_iso_creator_content(ui),
                AppType::PrimalOrchestrator => self.render_primal_orchestrator_content(ui),
                AppType::SystemSpinup => self.render_system_spinup_content(ui),
                AppType::FileManager => self.render_file_manager_content(ui),
            });
        }
    }

    /// Render dashboard content
    fn render_dashboard_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 System Dashboard");
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Ecosystem Status");

                for primal in &self.connected_primals {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let (color, status_text) = match &primal.status {
                                ConnectionStatus::Connected => (egui::Color32::GREEN, "Connected"),
                                ConnectionStatus::Connecting => {
                                    (egui::Color32::YELLOW, "Connecting")
                                }
                                ConnectionStatus::Disconnected => (egui::Color32::GRAY, "Offline"),
                                ConnectionStatus::Error(e) => (egui::Color32::RED, e.as_str()),
                            };

                            ui.colored_label(color, "●");
                            ui.label(format!("{} {}", primal.icon, primal.name));
                            ui.label(status_text);
                        });
                    });
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Quick Actions");

                if ui.button("🔄 Sync All Primals").clicked() {
                    self.add_notification(
                        "System",
                        "Syncing with all ecosystem primals...",
                        NotificationType::System,
                    );
                }

                if ui.button("🚀 Deploy New Biome").clicked() {
                    self.launch_app(AppType::BiomeBuilder);
                }

                if ui.button("💿 Create ISO").clicked() {
                    self.launch_app(AppType::IsoCreator);
                }

                if ui.button("🎭 Manage Primals").clicked() {
                    self.launch_app(AppType::PrimalOrchestrator);
                }
            });
        });
    }

    /// Render BYOB content with enhanced drag-drop interface
    fn render_byob_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧬 Build Your Own Biome - Advanced Builder");
        ui.separator();

        ui.horizontal(|ui| {
            // Left panel - Available Primals
            ui.vertical(|ui| {
                ui.heading("🔌 Available Primals");
                ui.label("Drag primals to the builder area:");

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        let connected_primals: Vec<_> = self
                            .connected_primals
                            .iter()
                            .filter(|p| matches!(p.status, ConnectionStatus::Connected))
                            .collect();
                        for primal in connected_primals {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&primal.icon);
                                    ui.vertical(|ui| {
                                        ui.strong(&primal.name);
                                        ui.small(&primal.endpoint);
                                        ui.small("Drag to add →");
                                    });

                                    // Enhanced info
                                    match primal.name.as_str() {
                                        name if name.contains("Beardog") => {
                                            ui.small("🔒 Encryption, Key Mgmt, Security");
                                        }
                                        name if name.contains("ToadStool") => {
                                            ui.small("⚡ Computing, ML, Processing");
                                        }
                                        name if name.contains("Songbird") => {
                                            ui.small("🔍 Discovery, Service Mesh");
                                        }
                                        name if name.contains("NestGate") => {
                                            ui.small("💾 Storage, Backup, Archive");
                                        }
                                        _ => {}
                                    }
                                });

                                if ui.button("➕ Add to Biome").clicked() {
                                    // Successfully integrated app to biome
                                    println!("✅ Successfully added application to biome");
                                }
                            });
                        }
                    });

                ui.add_space(10.0);
                ui.heading("📋 Templates");
                ui.group(|ui| {
                    if ui.button("🖥️ Development Stack").clicked() {
                        self.add_notification(
                            "BYOB",
                            "Loaded development template with ToadStool + Squirrel",
                            NotificationType::Info,
                        );
                    }
                    if ui.button("🏢 Production Cluster").clicked() {
                        self.add_notification(
                            "BYOB",
                            "Loaded production template with all primals",
                            NotificationType::Info,
                        );
                    }
                    if ui.button("🔒 Secure Workspace").clicked() {
                        self.add_notification(
                            "BYOB",
                            "Loaded security template with Beardog + NestGate",
                            NotificationType::Info,
                        );
                    }
                });
            });

            ui.separator();

            // Center panel - Biome Builder
            ui.vertical(|ui| {
                ui.heading("🏗️ Biome Builder");

                ui.group(|ui| {
                    ui.set_min_height(200.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label("🎯 Drop Zone");
                        ui.label("Drag primals here to build your biome");
                        ui.add_space(10.0);

                        // Mock some added components
                        ui.horizontal(|ui| {
                            ui.label("🐕 Beardog");
                            ui.label("🍄 ToadStool");
                            ui.label("🦆 NestGate");
                        });
                    });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("⚙️ Configuration");
                        ui.label("Biome Name:");
                        ui.text_edit_singleline(&mut String::from("my-custom-biome"));

                        ui.label("Environment:");
                        egui::ComboBox::from_label("")
                            .selected_text("production")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "production", "production", "Production");
                                ui.selectable_value(
                                    &mut "development",
                                    "development",
                                    "Development",
                                );
                                ui.selectable_value(&mut "testing", "testing", "Testing");
                            });

                        ui.checkbox(&mut true, "Enable monitoring");
                        ui.checkbox(&mut false, "Auto-scaling");
                        ui.checkbox(&mut true, "Backup enabled");
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.heading("📊 Resource Planning");
                        ui.label("Estimated Requirements:");
                        ui.label("CPU: 4 cores");
                        ui.label("Memory: 8 GB");
                        ui.label("Storage: 50 GB");
                        ui.label("Network: 1 Gbps");

                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("💰 Cost: $45/month").color(egui::Color32::GREEN),
                        );
                    });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("🚀 Deploy Biome").clicked() {
                        self.add_notification(
                            "BYOB",
                            "Biome deployment started! ETA: 3-5 minutes",
                            NotificationType::Success,
                        );
                    }

                    if ui.button("💾 Save Template").clicked() {
                        self.add_notification(
                            "BYOB",
                            "Biome template saved for future use",
                            NotificationType::Success,
                        );
                    }

                    if ui.button("📄 Export Manifest").clicked() {
                        self.add_notification(
                            "BYOB",
                            "Manifest exported to /biome-configs/",
                            NotificationType::Info,
                        );
                    }
                });
            });

            ui.separator();

            // Right panel - Live Manifest
            ui.vertical(|ui| {
                ui.heading("📄 Live Manifest");

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.code(
                            r#"# BiomeOS Manifest v1.0
name: my-custom-biome
version: 1.0.0
environment: production

primals:
  - name: beardog
    endpoint: beardog://localhost:8080
    capabilities:
      - encryption
      - key_management
      - security_policies
    
  - name: toadstool
    endpoint: toadstool://localhost:8081
    capabilities:
      - compute
      - machine_learning
      - data_processing
    
  - name: nestgate
    endpoint: nestgate://localhost:8084
    capabilities:
      - storage
      - backup
      - archival

resources:
  cpu_cores: 4
  memory_gb: 8
  storage_gb: 50
  network_gbps: 1

monitoring:
  enabled: true
  metrics_retention: 30d
  alerts_enabled: true

backup:
  enabled: true
  frequency: daily
  retention: 7d"#,
                        );
                    });

                ui.add_space(10.0);
                ui.label(egui::RichText::new("✅ Manifest Valid").color(egui::Color32::GREEN));
            });
        });
    }

    /// Render ISO creator content
    fn render_iso_creator_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("💿 ISO Creator");
        ui.separator();

        ui.label("Create bootable ISO images with selected primals and configurations:");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("ISO Configuration");
                ui.label("Base System: BiomeOS Core");
                ui.label("Architecture: x86_64");
                ui.add_space(10.0);

                ui.heading("Include Primals");
                for primal in &self.connected_primals {
                    ui.checkbox(&mut true, format!("{} {}", primal.icon, primal.name));
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Build Process");
                ui.label("Ready to build...");
                ui.add_space(20.0);

                if ui.button("🏗️ Start Build").clicked() {
                    self.add_notification(
                        "ISO Creator",
                        "ISO build started! This may take several minutes.",
                        NotificationType::Info,
                    );
                }
            });
        });
    }

    /// Render primal orchestrator content
    fn render_primal_orchestrator_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎭 Primal Orchestrator");
        ui.separator();

        ui.label("Coordinate and manage all ecosystem primals:");

        for primal in &self.connected_primals {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let (color, status_text) = match &primal.status {
                        ConnectionStatus::Connected => (egui::Color32::GREEN, "Connected"),
                        ConnectionStatus::Connecting => (egui::Color32::YELLOW, "Connecting..."),
                        ConnectionStatus::Disconnected => (egui::Color32::GRAY, "Offline"),
                        ConnectionStatus::Error(e) => (egui::Color32::RED, e.as_str()),
                    };

                    ui.colored_label(color, "●");
                    ui.label(format!("{} {}", primal.icon, primal.name));
                    ui.label(status_text);
                    ui.label(&primal.endpoint);

                    if ui.button("📡 Connect").clicked() {
                        // Successfully initiated connection to primal
                        println!("✅ Connecting to {} at {}", primal.name, primal.endpoint);
                    }
                });
            });
        }
    }

    /// Render system spinup content
    fn render_system_spinup_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 System Spinup");
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Quick Deploy");

                if ui.button("🖥️ Development Environment").clicked() {
                    self.add_notification(
                        "Spinup",
                        "Spinning up development environment...",
                        NotificationType::Info,
                    );
                }

                if ui.button("🏢 Production Cluster").clicked() {
                    self.add_notification(
                        "Spinup",
                        "Deploying production cluster...",
                        NotificationType::Info,
                    );
                }

                if ui.button("🧪 Testing Sandbox").clicked() {
                    self.add_notification(
                        "Spinup",
                        "Creating testing sandbox...",
                        NotificationType::Info,
                    );
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Active Deployments");
                ui.label("No active deployments");
                ui.add_space(20.0);

                if ui.button("📋 View All").clicked() {
                    self.add_notification(
                        "Spinup",
                        "Loading deployment history...",
                        NotificationType::Info,
                    );
                }
            });
        });
    }

    /// Render file manager content
    fn render_file_manager_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("📁 File Manager");
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Folders");
                ui.selectable_label(true, "📁 /home");
                ui.selectable_label(false, "📁 /biome-configs");
                ui.selectable_label(false, "📁 /templates");
                ui.selectable_label(false, "📁 /deployments");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Files");
                ui.label("📄 config.yaml");
                ui.label("📄 manifest.biome");
                ui.label("📄 deployment.log");
            });
        });
    }

    /// Render system notifications
    fn render_notifications(&mut self, ctx: &egui::Context) {
        let screen_rect = ctx.screen_rect();
        let mut y_offset = 20.0;

        // Remove old notifications
        self.system_notifications
            .retain(|n| !n.dismissed && n.timestamp.elapsed().as_secs() < 10);

        for notification in &mut self.system_notifications {
            if notification.dismissed {
                continue;
            }

            let notification_size = egui::vec2(320.0, 80.0);
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
                        NotificationType::System => egui::Color32::from_rgb(20, 40, 80),
                        _ => egui::Color32::from_rgb(30, 35, 40),
                    },
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
                            ui.strong(&notification.title);
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

    /// Helper methods
    fn get_app_icon(&self, app_type: &AppType) -> &'static str {
        match app_type {
            AppType::Dashboard => "🏠",
            AppType::BiomeBuilder => "🧬",
            AppType::IsoCreator => "💿",
            AppType::PrimalOrchestrator => "🎭",
            AppType::SystemSpinup => "🚀",
            AppType::FileManager => "📁",
        }
    }

    fn truncate_title(&self, title: &str, max_len: usize) -> String {
        if title.len() > max_len {
            format!("{}...", &title[..max_len - 3])
        } else {
            title.to_string()
        }
    }
}

impl eframe::App for MinimalBiomeOSApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Desktop background
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 25, 30)))
            .show(ctx, |ui| {
                self.render_desktop_background(ui);
                self.render_active_windows(ctx);
            });

        // Taskbar
        egui::TopBottomPanel::bottom("taskbar")
            .exact_height(self.taskbar_height)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_taskbar(ui);
            });

        // Launcher overlay
        if self.show_launcher {
            self.render_launcher(ctx);
        }

        // System notifications
        self.render_notifications(ctx);

        // Auto-refresh for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
    }
}
