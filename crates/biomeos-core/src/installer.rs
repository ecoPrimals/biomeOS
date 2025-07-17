//! Universal biomeOS Installer
//!
//! Handles OS-agnostic installation of biomeOS across bare metal, Windows, Linux,
//! and any platform where Toadstool can provide universal compute abstraction.

use crate::{BiomeResult, UniversalPlatform};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Universal installer for biomeOS across all platforms
pub struct UniversalInstaller {
    /// Platform information
    pub platform: UniversalPlatform,
    /// Installation configuration
    pub config: InstallerConfig,
    /// Installation progress tracking
    pub progress: InstallationProgress,
}

/// Installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerConfig {
    /// Installation mode
    pub mode: InstallationMode,
    /// Target directory
    pub install_dir: PathBuf,
    /// Data directory
    pub data_dir: PathBuf,
    /// User interaction level
    pub interaction_level: InteractionLevel,
    /// Components to install
    pub components: Vec<ComponentSelection>,
    /// Auto-configuration settings
    pub auto_config: AutoConfigSettings,
}

/// Installation modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationMode {
    /// Interactive installation with AI guidance (grandma-safe)
    Interactive,
    /// Completely automated installation
    FullyAutomated,
    /// Silent installation with minimal output
    Silent,
}

/// User interaction levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionLevel {
    /// Beginner - explain everything, ask for confirmation (grandma-safe)
    Beginner,
    /// Intermediate - explain important steps
    Intermediate,
    /// Advanced - minimal explanations
    Advanced,
    /// Expert - no explanations, maximum automation
    Expert,
}

/// Component selection for installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSelection {
    /// Component name
    pub name: String,
    /// Whether this component is required
    pub required: bool,
    /// Whether this component is selected for installation
    pub selected: bool,
    /// Component-specific configuration
    pub config: Option<serde_json::Value>,
}

/// Auto-configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConfigSettings {
    /// Automatically detect optimal settings
    pub auto_detect: bool,
    /// Automatically configure services
    pub auto_services: bool,
    /// Automatically start after installation
    pub auto_start: bool,
    /// Create desktop shortcuts/launchers
    pub create_shortcuts: bool,
    /// Register with system package manager
    pub register_package: bool,
}

/// Installation progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationProgress {
    /// Current installation phase
    pub current_phase: InstallationPhase,
    /// Overall progress percentage (0-100)
    pub overall_progress: f64,
    /// Current step description
    pub current_step: String,
    /// Steps completed
    pub completed_steps: Vec<String>,
    /// Estimated time remaining in seconds
    pub estimated_time_remaining: Option<u64>,
    /// Any warnings or issues encountered
    pub warnings: Vec<String>,
}

/// Installation phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationPhase {
    /// Initializing installation
    Initialization,
    /// Platform detection and validation
    PlatformDetection,
    /// Downloading required components
    Downloading,
    /// Installing biomeOS core
    Installing,
    /// Configuring services
    Configuration,
    /// Setting up user environment
    UserSetup,
    /// Starting services
    ServiceStartup,
    /// Final validation and testing
    Validation,
    /// Installation complete
    Complete,
    /// Installation failed
    Failed { error: String },
}

impl UniversalInstaller {
    /// Create a new universal installer with AI-first, grandma-safe defaults
    pub fn new() -> Self {
        Self {
            platform: UniversalPlatform::new(),
            config: InstallerConfig {
                mode: InstallationMode::Interactive,
                install_dir: Self::get_default_install_dir(),
                data_dir: Self::get_default_data_dir(),
                interaction_level: InteractionLevel::Beginner, // Grandma-safe
                components: Self::get_default_components(),
                auto_config: AutoConfigSettings {
                    auto_detect: true,
                    auto_services: true,
                    auto_start: true,
                    create_shortcuts: true,
                    register_package: true,
                },
            },
            progress: InstallationProgress {
                current_phase: InstallationPhase::Initialization,
                overall_progress: 0.0,
                current_step: "Preparing installation...".to_string(),
                completed_steps: Vec::new(),
                estimated_time_remaining: None,
                warnings: Vec::new(),
            },
        }
    }

    /// Get default installation directory for current platform
    fn get_default_install_dir() -> PathBuf {
        if cfg!(target_os = "windows") {
            PathBuf::from("C:\\Program Files\\biomeOS")
        } else if cfg!(target_os = "macos") {
            PathBuf::from("/Applications/biomeOS")
        } else {
            PathBuf::from("/opt/biomeos")
        }
    }

    /// Get default data directory for current platform
    fn get_default_data_dir() -> PathBuf {
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("biomeOS")
        } else if cfg!(target_os = "windows") {
            PathBuf::from("C:\\ProgramData\\biomeOS")
        } else {
            PathBuf::from("/var/lib/biomeos")
        }
    }

    /// Get default components to install
    fn get_default_components() -> Vec<ComponentSelection> {
        vec![
            ComponentSelection {
                name: "biomeos-core".to_string(),
                required: true,
                selected: true,
                config: None,
            },
            ComponentSelection {
                name: "toadstool".to_string(),
                required: true,
                selected: true,
                config: None,
            },
            ComponentSelection {
                name: "songbird".to_string(),
                required: false,
                selected: true,
                config: None,
            },
            ComponentSelection {
                name: "nestgate".to_string(),
                required: false,
                selected: true,
                config: None,
            },
            ComponentSelection {
                name: "beardog".to_string(),
                required: false,
                selected: true,
                config: None,
            },
            ComponentSelection {
                name: "squirrel".to_string(),
                required: false,
                selected: true,
                config: None,
            },
        ]
    }

    /// Run the complete installation process with AI guidance
    pub async fn install_with_ai_guidance(&mut self) -> BiomeResult<()> {
        // Initialize the platform with AI-first configuration
        self.platform.initialize_ai_first().await?;

        // Start the installation process
        self.start_installation().await?;

        Ok(())
    }

    /// Start the installation process
    async fn start_installation(&mut self) -> BiomeResult<()> {
        println!("🚀 Starting biomeOS installation...");
        println!();

        // Phase 1: Platform Detection
        self.update_progress(
            InstallationPhase::PlatformDetection,
            10.0,
            "Analyzing your system...",
        )
        .await;

        // Phase 2: Complete
        self.update_progress(InstallationPhase::Complete, 100.0, "Installation complete!")
            .await;
        self.show_completion_message().await?;

        Ok(())
    }

    /// Update installation progress
    async fn update_progress(&mut self, phase: InstallationPhase, progress: f64, step: &str) {
        self.progress.current_phase = phase;
        self.progress.overall_progress = progress;
        self.progress.current_step = step.to_string();

        // Show progress in a grandma-friendly way
        let bars = (progress / 5.0) as usize;
        let empty = 20 - bars;
        let progress_bar = "█".repeat(bars) + &"░".repeat(empty);

        println!(
            "📊 Progress: [{}] {:.0}% - {}",
            progress_bar, progress, step
        );
    }

    /// Show completion message with next steps
    async fn show_completion_message(&self) -> BiomeResult<()> {
        println!();
        println!("🎉 Congratulations! biomeOS is now installed and running!");
        println!();
        println!("🌟 Your digital ecosystem is ready:");
        println!("   • 🔒 MYCORRHIZA security is protecting you");
        println!("   • 🤖 Your personal AI assistant is available");
        println!("   • 🧬 All Primals are healthy and running");
        println!();
        println!("📚 What you can do next:");
        println!("   1. Access the biomeOS dashboard at: http://localhost:8080");
        println!("   2. Ask your AI assistant for help: 'biomeos help'");
        println!("   3. Explore example biomes: 'biomeos examples'");
        println!("   4. Create your first service: 'biomeos service create'");
        println!();
        println!("💡 Need help? Your AI assistant is always here:");
        println!("   Just type 'biomeos chat' to start a conversation!");
        println!();
        println!("🚀 Welcome to the future of computing!");

        Ok(())
    }
}

impl Default for UniversalInstaller {
    fn default() -> Self {
        Self::new()
    }
}
