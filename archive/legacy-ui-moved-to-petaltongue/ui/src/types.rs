//! UI Type Definitions for BiomeOS Desktop
//!
//! This module contains the core types used across the UI layer.

use eframe::egui;

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

impl WindowInfo {
    /// Create a new window with default size and position
    #[must_use]
    pub fn new(id: String, title: String, view: AppView, offset: usize) -> Self {
        Self {
            id,
            title,
            view,
            position: egui::pos2(100.0 + (offset as f32 * 30.0), 100.0),
            size: egui::vec2(800.0, 600.0),
            minimized: false,
            maximized: false,
            focused: true,
        }
    }
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

impl SystemNotification {
    /// Create a new notification
    #[must_use]
    pub fn new(
        id: String,
        title: String,
        message: String,
        notification_type: NotificationType,
    ) -> Self {
        Self {
            id,
            title,
            message,
            notification_type,
            timestamp: std::time::Instant::now(),
            dismissed: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    System,
}

impl NotificationType {
    /// Get icon for notification type
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Error => "❌",
            Self::Warning => "⚠️",
            Self::Success => "✅",
            Self::Info => "ℹ️",
            Self::System => "⚙️",
        }
    }

    /// Get background color for notification type
    #[must_use]
    pub const fn background_color(self) -> egui::Color32 {
        match self {
            Self::Error => egui::Color32::from_rgb(80, 20, 20),
            Self::Warning => egui::Color32::from_rgb(80, 60, 20),
            Self::Success => egui::Color32::from_rgb(20, 80, 20),
            Self::Info | Self::System => egui::Color32::from_rgb(30, 35, 40),
        }
    }
}

/// Secure Compute View for orchestrating primals
#[derive(Debug, Clone)]
pub struct SecureComputeView {
    pub connected_primals: Vec<PrimalConnection>,
    pub last_sync: Option<std::time::Instant>,
}

impl Default for SecureComputeView {
    fn default() -> Self {
        // Note: In production, this should use Songbird discovery
        // rather than hardcoded localhost endpoints
        Self {
            connected_primals: Vec::new(),
            last_sync: None,
        }
    }
}

impl SecureComputeView {
    /// Update primal connections from discovery service
    pub fn update_from_discovery(&mut self, discovered: Vec<PrimalConnection>) {
        self.connected_primals = discovered;
        self.last_sync = Some(std::time::Instant::now());
    }
}

#[derive(Debug, Clone)]
pub struct PrimalConnection {
    pub name: String,
    pub primal_type: String, // "beardog", "toadstool", "songbird", etc.
    pub status: ConnectionStatus,
    pub endpoint: String,
    pub last_heartbeat: Option<std::time::Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

impl ConnectionStatus {
    /// Get status indicator color
    #[must_use]
    pub const fn color(&self) -> egui::Color32 {
        match self {
            Self::Connected => egui::Color32::GREEN,
            Self::Connecting => egui::Color32::YELLOW,
            Self::Disconnected => egui::Color32::GRAY,
            Self::Error(_) => egui::Color32::RED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl AppView {
    /// Get icon for view
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "🏠",
            Self::Byob => "🧬",
            Self::IsoCreator => "💿",
            Self::SecureCompute => "🔐",
            Self::SystemSpinup => "🚀",
            Self::Primals => "🎯",
            Self::ToadStool => "🍄",
            Self::Sovereignty => "🔒",
            Self::Settings => "⚙️",
            Self::YamlEditor => "📝",
            Self::NicheManager => "🎭",
            Self::Installation => "📦",
            Self::DesktopLauncher => "📱",
        }
    }

    /// Get title for view
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "BiomeOS Dashboard",
            Self::Byob => "Build Your Own Biome",
            Self::IsoCreator => "ISO Creator",
            Self::SecureCompute => "Secure Computation",
            Self::SystemSpinup => "System Spinup",
            Self::Primals => "Ecosystem Primals",
            Self::ToadStool => "ToadStool Compute",
            Self::Sovereignty => "Digital Sovereignty",
            Self::Settings => "System Settings",
            Self::YamlEditor => "YAML Editor",
            Self::NicheManager => "Niche Manager",
            Self::Installation => "Installation",
            Self::DesktopLauncher => "Desktop Launcher",
        }
    }
}

/// Launcher application entry
#[derive(Debug, Clone)]
pub struct LauncherApp {
    pub name: &'static str,
    pub icon: &'static str,
    pub view: AppView,
}

impl LauncherApp {
    /// Get default launcher apps
    #[must_use]
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                name: "Dashboard",
                icon: "🏠",
                view: AppView::Dashboard,
            },
            Self {
                name: "BYOB",
                icon: "🧬",
                view: AppView::Byob,
            },
            Self {
                name: "ISO Creator",
                icon: "💿",
                view: AppView::IsoCreator,
            },
            Self {
                name: "Secure Compute",
                icon: "🔐",
                view: AppView::SecureCompute,
            },
            Self {
                name: "System Spinup",
                icon: "🚀",
                view: AppView::SystemSpinup,
            },
            Self {
                name: "Primals",
                icon: "🎯",
                view: AppView::Primals,
            },
            Self {
                name: "ToadStool",
                icon: "🍄",
                view: AppView::ToadStool,
            },
            Self {
                name: "Sovereignty",
                icon: "🔒",
                view: AppView::Sovereignty,
            },
            Self {
                name: "Settings",
                icon: "⚙️",
                view: AppView::Settings,
            },
        ]
    }
}
