//! Data types and structures for the ISO Creator
//!
//! This module contains all the data structures, enums, and type definitions
//! used by the ISO Creator functionality.

/// ISO Creator tab navigation
#[derive(Debug, Clone, PartialEq)]
pub enum IsoCreatorTab {
    Configuration,
    Niches,
    Components,
    Build,
    Queue,
}

/// ISO configuration specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IsoConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub target_arch: String,
    pub boot_mode: BootMode,
    pub included_primals: Vec<String>,
    pub included_niches: Vec<String>,
    pub custom_components: Vec<String>,
    pub compression_level: u8,
    pub size_estimate: u64, // in MB
    pub created_at: String,
}

impl IsoConfig {
    /// Create a new ISO configuration with default values
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            version: "1.0.0".to_string(),
            target_arch: "x86_64".to_string(),
            boot_mode: BootMode::Hybrid,
            included_primals: vec![
                "toadstool".to_string(),
                "songbird".to_string(),
                "nestgate".to_string(),
                "squirrel".to_string(),
                "beardog".to_string(),
            ],
            included_niches: Vec::new(),
            custom_components: Vec::new(),
            compression_level: 6,
            size_estimate: 1500, // MB
            created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        }
    }

    /// Calculate estimated size based on included components
    pub fn calculate_size_estimate(&self) -> u64 {
        let mut total_size = 500u64; // Base OS size

        // Add primal sizes
        for primal in &self.included_primals {
            total_size += match primal.as_str() {
                "toadstool" => 200,
                "songbird" => 150,
                "nestgate" => 300,
                "squirrel" => 100,
                "beardog" => 250,
                _ => 100,
            };
        }

        // Add niche sizes (estimated)
        total_size += self.included_niches.len() as u64 * 400;

        // Add custom component sizes (estimated)
        total_size += self.custom_components.len() as u64 * 150;

        // Apply compression factor
        let compression_factor = match self.compression_level {
            0..=3 => 0.9,
            4..=6 => 0.7,
            7..=9 => 0.6,
            _ => 0.5,
        };

        (total_size as f64 * compression_factor) as u64
    }

    /// Validate configuration for build requirements
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("ISO name cannot be empty".to_string());
        }

        if self.included_primals.is_empty() {
            return Err("At least one primal must be included".to_string());
        }

        if self.compression_level > 9 {
            return Err("Compression level must be between 0-9".to_string());
        }

        Ok(())
    }
}

/// Boot mode for ISO
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BootMode {
    Legacy,
    UEFI,
    Hybrid,
}

impl BootMode {
    /// Get all available boot modes
    pub fn all() -> Vec<Self> {
        vec![Self::Legacy, Self::UEFI, Self::Hybrid]
    }

    /// Get display name for boot mode
    pub fn display_name(&self) -> &str {
        match self {
            Self::Legacy => "Legacy BIOS",
            Self::UEFI => "UEFI",
            Self::Hybrid => "Hybrid (BIOS + UEFI)",
        }
    }
}

/// Build status for ISO creation
#[derive(Debug, Clone, PartialEq)]
pub enum BuildStatus {
    Idle,
    Preparing,
    Building,
    Packaging,
    Completing,
    Success,
    Failed,
}

impl BuildStatus {
    /// Get color representation for UI
    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Idle => egui::Color32::GRAY,
            Self::Preparing => egui::Color32::LIGHT_BLUE,
            Self::Building => egui::Color32::YELLOW,
            Self::Packaging => egui::Color32::from_rgb(255, 165, 0), // Orange
            Self::Completing => egui::Color32::from_rgb(0, 255, 0),  // Light Green
            Self::Success => egui::Color32::GREEN,
            Self::Failed => egui::Color32::RED,
        }
    }

    /// Get icon representation for UI
    pub fn icon(&self) -> &str {
        match self {
            Self::Idle => "⏸️",
            Self::Preparing => "📋",
            Self::Building => "🔨",
            Self::Packaging => "📦",
            Self::Completing => "✅",
            Self::Success => "🎉",
            Self::Failed => "❌",
        }
    }
}

/// Niche package specification
#[derive(Debug, Clone)]
pub struct NichePackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: String,
    pub size_mb: u64,
    pub features: Vec<String>,
    pub dependencies: Vec<String>,
    pub manifest_path: String,
    pub icon_path: Option<String>,
}

impl NichePackage {
    /// Create a new niche package
    pub fn new(id: String, name: String, description: String) -> Self {
        let manifest_path = format!("/niches/{}/manifest.yaml", id);
        Self {
            id,
            name,
            description,
            author: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            category: "General".to_string(),
            size_mb: 100,
            features: Vec::new(),
            dependencies: Vec::new(),
            manifest_path,
            icon_path: None,
        }
    }

    /// Check if package has dependency on specific primal
    pub fn has_primal_dependency(&self, primal: &str) -> bool {
        self.dependencies.contains(&primal.to_string())
    }

    /// Get category icon
    pub fn category_icon(&self) -> &str {
        match self.category.as_str() {
            "Gaming" => "🎮",
            "Research" => "🔬",
            "Development" => "💻",
            "Security" => "🔒",
            "Network" => "🌐",
            "Database" => "🗄️",
            "AI" => "🤖",
            _ => "📦",
        }
    }
}

/// Custom component specification
#[derive(Debug, Clone)]
pub struct CustomComponent {
    pub name: String,
    pub description: String,
    pub component_type: ComponentType,
    pub source_path: String,
    pub destination_path: String,
    pub size_mb: u64,
    pub required: bool,
}

impl CustomComponent {
    /// Create a new custom component
    pub fn new(name: String, description: String, component_type: ComponentType) -> Self {
        Self {
            name,
            description,
            component_type,
            source_path: "/custom/components".to_string(),
            destination_path: "/usr/local/bin".to_string(),
            size_mb: 10,
            required: false,
        }
    }

    /// Get type icon
    pub fn type_icon(&self) -> &str {
        match self.component_type {
            ComponentType::Binary => "⚙️",
            ComponentType::Library => "📚",
            ComponentType::Configuration => "🔧",
            ComponentType::Documentation => "📄",
            ComponentType::Template => "📋",
            ComponentType::Script => "📝",
        }
    }
}

/// Component type classification
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Binary,
    Library,
    Configuration,
    Documentation,
    Template,
    Script,
}

impl ComponentType {
    /// Get all available component types
    pub fn all() -> Vec<Self> {
        vec![
            Self::Binary,
            Self::Library,
            Self::Configuration,
            Self::Documentation,
            Self::Template,
            Self::Script,
        ]
    }

    /// Get display name for component type
    pub fn display_name(&self) -> &str {
        match self {
            Self::Binary => "Binary Executable",
            Self::Library => "Library",
            Self::Configuration => "Configuration File",
            Self::Documentation => "Documentation",
            Self::Template => "Template",
            Self::Script => "Script",
        }
    }
}

/// ISO template specification
#[derive(Debug, Clone)]
pub struct IsoTemplate {
    pub name: String,
    pub description: String,
    pub use_case: String,
    pub included_components: Vec<String>,
    pub size_estimate: u64,
    pub difficulty: TemplateDifficulty,
    pub tags: Vec<String>,
    pub author: String,
    pub version: String,
}

impl IsoTemplate {
    /// Create a new ISO template
    pub fn new(name: String, description: String, use_case: String) -> Self {
        Self {
            name,
            description,
            use_case,
            included_components: Vec::new(),
            size_estimate: 1000,
            difficulty: TemplateDifficulty::Beginner,
            tags: Vec::new(),
            author: "biomeOS Team".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    /// Get difficulty icon
    pub fn difficulty_icon(&self) -> &str {
        match self.difficulty {
            TemplateDifficulty::Beginner => "🟢",
            TemplateDifficulty::Intermediate => "🟡",
            TemplateDifficulty::Advanced => "🟠",
            TemplateDifficulty::Expert => "🔴",
        }
    }

    /// Get use case icon
    pub fn use_case_icon(&self) -> &str {
        match self.use_case.to_lowercase().as_str() {
            s if s.contains("gaming") => "🎮",
            s if s.contains("research") => "🔬",
            s if s.contains("development") => "💻",
            s if s.contains("server") => "🖥️",
            s if s.contains("desktop") => "🖥️",
            s if s.contains("minimal") => "📦",
            _ => "🏗️",
        }
    }
}

/// Template difficulty level
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl TemplateDifficulty {
    /// Get all difficulty levels
    pub fn all() -> Vec<Self> {
        vec![
            Self::Beginner,
            Self::Intermediate,
            Self::Advanced,
            Self::Expert,
        ]
    }

    /// Get display name for difficulty
    pub fn display_name(&self) -> &str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
            Self::Expert => "Expert",
        }
    }
}

/// Build job specification
#[derive(Debug, Clone)]
pub struct BuildJob {
    pub id: String,
    pub config: IsoConfig,
    pub status: BuildStatus,
    pub progress: f32,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
    pub build_log: Vec<String>,
}

impl BuildJob {
    /// Create a new build job
    pub fn new(id: String, config: IsoConfig) -> Self {
        Self {
            id,
            config,
            status: BuildStatus::Idle,
            progress: 0.0,
            started_at: None,
            completed_at: None,
            output_path: None,
            error_message: None,
            build_log: Vec::new(),
        }
    }

    /// Start the build job
    pub fn start(&mut self) {
        self.status = BuildStatus::Preparing;
        self.started_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        self.build_log.push("🚀 Build job started".to_string());
    }

    /// Complete the build job successfully
    pub fn complete(&mut self, output_path: String) {
        self.status = BuildStatus::Success;
        self.progress = 1.0;
        self.completed_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        self.output_path = Some(output_path);
        self.build_log
            .push("✅ Build completed successfully".to_string());
    }

    /// Fail the build job
    pub fn fail(&mut self, error_message: String) {
        self.status = BuildStatus::Failed;
        self.completed_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        self.error_message = Some(error_message.clone());
        self.build_log
            .push(format!("❌ Build failed: {}", error_message));
    }

    /// Update build progress
    pub fn update_progress(&mut self, progress: f32, message: String) {
        self.progress = progress.clamp(0.0, 1.0);
        self.build_log.push(message);
    }

    /// Get duration if build is completed
    pub fn duration(&self) -> Option<String> {
        if let (Some(started), Some(completed)) = (&self.started_at, &self.completed_at) {
            // Simple duration calculation (in a real implementation, use proper datetime parsing)
            Some(format!("Duration: {} - {}", started, completed))
        } else {
            None
        }
    }
}

/// Build statistics
#[derive(Debug, Clone)]
pub struct BuildStats {
    pub total_builds: usize,
    pub successful_builds: usize,
    pub failed_builds: usize,
    pub average_size: u64,
    pub total_size: u64,
}

impl BuildStats {
    /// Calculate statistics from build jobs
    pub fn from_jobs(jobs: &[BuildJob]) -> Self {
        let total_builds = jobs.len();
        let successful_builds = jobs
            .iter()
            .filter(|job| job.status == BuildStatus::Success)
            .count();
        let failed_builds = jobs
            .iter()
            .filter(|job| job.status == BuildStatus::Failed)
            .count();

        let total_size: u64 = jobs.iter().map(|job| job.config.size_estimate).sum();
        let average_size = if total_builds > 0 {
            total_size / total_builds as u64
        } else {
            0
        };

        Self {
            total_builds,
            successful_builds,
            failed_builds,
            average_size,
            total_size,
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.total_builds > 0 {
            (self.successful_builds as f32 / self.total_builds as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// ISO Creator configuration
#[derive(Debug, Clone)]
pub struct IsoCreatorConfig {
    pub output_directory: String,
    pub default_compression_level: u8,
    pub default_target_architecture: String,
    pub default_boot_mode: BootMode,
    pub include_all_primals: bool,
    pub include_demos: bool,
    pub include_documentation: bool,
    pub max_concurrent_builds: usize,
    pub build_timeout_minutes: u64,
}

impl Default for IsoCreatorConfig {
    fn default() -> Self {
        Self {
            output_directory: "/tmp/biomeos-isos".to_string(),
            default_compression_level: 6,
            default_target_architecture: "x86_64".to_string(),
            default_boot_mode: BootMode::Hybrid,
            include_all_primals: true,
            include_demos: true,
            include_documentation: true,
            max_concurrent_builds: 2,
            build_timeout_minutes: 60,
        }
    }
}
