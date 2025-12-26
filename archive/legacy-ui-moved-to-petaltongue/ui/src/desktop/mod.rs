//! Desktop Mode UI Components
//!
//! This module contains the OS-like desktop interface components:
//! - Desktop background and grid rendering
//! - Taskbar with launcher and system tray
//! - Window management
//! - Application launcher
//! - System notifications

mod background;
mod launcher;
mod notifications;
mod taskbar;
pub mod windows;

pub use background::render_desktop_background;
pub use launcher::render_launcher;
pub use notifications::render_notifications;
pub use taskbar::render_taskbar;
pub use windows::render_active_windows;
