//! Build process management for ISO Creator
//!
//! This module handles the ISO build process, including progress tracking,
//! logging, and build pipeline management.

use crate::views::iso_creator::types::*;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Build manager for ISO creation
#[derive(Debug)]
pub struct BuildManager {
    current_build: Option<BuildJob>,
    build_history: Vec<BuildJob>,
    build_config: IsoCreatorConfig,
    build_counter: usize,
}

impl BuildManager {
    /// Create a new build manager
    pub fn new(config: IsoCreatorConfig) -> Self {
        Self {
            current_build: None,
            build_history: Vec::new(),
            build_config: config,
            build_counter: 0,
        }
    }

    /// Start a new build
    pub fn start_build(&mut self, config: IsoConfig) -> Result<String, String> {
        if self.current_build.is_some() {
            return Err("A build is already in progress".to_string());
        }

        // Validate configuration
        config.validate()?;

        // Create build job
        let _build_id = format!("build-{}", self.build_counter + 1);
        self.build_counter += 1;

        let mut job = BuildJob::new(_build_id.clone(), config);
        job.start();

        self.current_build = Some(job);
        Ok(_build_id)
    }

    /// Cancel current build
    pub fn cancel_build(&mut self) -> Result<(), String> {
        if let Some(mut job) = self.current_build.take() {
            job.fail("Build cancelled by user".to_string());
            self.build_history.push(job);
            Ok(())
        } else {
            Err("No build in progress".to_string())
        }
    }

    /// Get current build status
    pub fn get_current_build(&self) -> Option<&BuildJob> {
        self.current_build.as_ref()
    }

    /// Get build history
    pub fn get_build_history(&self) -> &[BuildJob] {
        &self.build_history
    }

    /// Update build progress
    pub fn update_progress(&mut self, progress: f32, message: String) -> Result<(), String> {
        if let Some(job) = &mut self.current_build {
            job.update_progress(progress, message);
            Ok(())
        } else {
            Err("No build in progress".to_string())
        }
    }

    /// Complete current build
    pub fn complete_build(&mut self, output_path: String) -> Result<(), String> {
        if let Some(mut job) = self.current_build.take() {
            job.complete(output_path);
            self.build_history.push(job);
            Ok(())
        } else {
            Err("No build in progress".to_string())
        }
    }

    /// Fail current build
    pub fn fail_build(&mut self, error_message: String) -> Result<(), String> {
        if let Some(mut job) = self.current_build.take() {
            job.fail(error_message);
            self.build_history.push(job);
            Ok(())
        } else {
            Err("No build in progress".to_string())
        }
    }

    /// Get build statistics
    pub fn get_statistics(&self) -> BuildStats {
        BuildStats::from_jobs(&self.build_history)
    }

    /// Clear build history
    pub fn clear_history(&mut self) {
        self.build_history.clear();
    }

    /// Simulate build process (for development/testing)
    pub fn simulate_build(&mut self, config: IsoConfig) -> Result<(), String> {
        let _build_id = self.start_build(config)?;

        // Simulate build phases
        let phases = vec![
            (0.1, "📋 Preparing build environment"),
            (0.2, "📦 Collecting components"),
            (0.3, "🔧 Configuring system"),
            (0.4, "🧩 Installing primals"),
            (0.6, "🎭 Installing niches"),
            (0.7, "⚙️ Installing custom components"),
            (0.8, "🗜️ Compressing filesystem"),
            (0.9, "💿 Creating ISO image"),
            (0.95, "✅ Verifying ISO integrity"),
            (1.0, "🎉 Build completed successfully"),
        ];

        for (progress, message) in phases {
            self.update_progress(progress, message.to_string())?;

            // Simulate processing time
            std::thread::sleep(Duration::from_millis(500));
        }

        let build_name = self
            .current_build
            .as_ref()
            .map(|b| b.config.name.clone())
            .unwrap_or_else(|| "unnamed".to_string());
        let output_path = format!("{}/{}.iso", self.build_config.output_directory, build_name);

        self.complete_build(output_path)?;
        Ok(())
    }
}

/// Build pipeline for ISO creation
pub struct BuildPipeline {
    stages: Vec<BuildStage>,
    current_stage: usize,
    total_stages: usize,
}

impl BuildPipeline {
    /// Create a new build pipeline
    pub fn new(config: &IsoConfig) -> Self {
        let mut stages = vec![
            BuildStage::new("prepare", "Preparing build environment", 0.1),
            BuildStage::new("collect", "Collecting components", 0.2),
            BuildStage::new("configure", "Configuring system", 0.3),
        ];

        // Add primal installation stages
        for primal in &config.included_primals {
            stages.push(BuildStage::new(
                &format!("primal-{}", primal),
                &format!("Installing primal: {}", primal),
                0.4,
            ));
        }

        // Add niche installation stages
        for niche in &config.included_niches {
            stages.push(BuildStage::new(
                &format!("niche-{}", niche),
                &format!("Installing niche: {}", niche),
                0.6,
            ));
        }

        // Add custom component stages
        for component in &config.custom_components {
            stages.push(BuildStage::new(
                &format!("component-{}", component),
                &format!("Installing component: {}", component),
                0.7,
            ));
        }

        // Add final stages
        stages.extend(vec![
            BuildStage::new("compress", "Compressing filesystem", 0.8),
            BuildStage::new("iso", "Creating ISO image", 0.9),
            BuildStage::new("verify", "Verifying ISO integrity", 0.95),
            BuildStage::new("complete", "Build completed", 1.0),
        ]);

        let total_stages = stages.len();

        Self {
            stages,
            current_stage: 0,
            total_stages,
        }
    }

    /// Get current stage
    pub fn current_stage(&self) -> Option<&BuildStage> {
        self.stages.get(self.current_stage)
    }

    /// Advance to next stage
    pub fn next_stage(&mut self) -> Option<&BuildStage> {
        if self.current_stage < self.total_stages - 1 {
            self.current_stage += 1;
            self.current_stage()
        } else {
            None
        }
    }

    /// Get overall progress
    pub fn progress(&self) -> f32 {
        if self.total_stages == 0 {
            return 0.0;
        }

        let stage_progress = self.current_stage as f32 / self.total_stages as f32;
        let current_stage_progress = self.current_stage().map(|s| s.progress).unwrap_or(0.0);

        stage_progress + (current_stage_progress / self.total_stages as f32)
    }

    /// Check if pipeline is complete
    pub fn is_complete(&self) -> bool {
        self.current_stage >= self.total_stages - 1
    }

    /// Get remaining stages
    pub fn remaining_stages(&self) -> usize {
        self.total_stages - self.current_stage
    }
}

/// Individual build stage
#[derive(Debug, Clone)]
pub struct BuildStage {
    pub id: String,
    pub name: String,
    pub progress: f32,
    pub status: BuildStatus,
    pub start_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub error: Option<String>,
}

impl BuildStage {
    /// Create a new build stage
    pub fn new(id: &str, name: &str, progress: f32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            progress,
            status: BuildStatus::Idle,
            start_time: None,
            duration: None,
            error: None,
        }
    }

    /// Start the stage
    pub fn start(&mut self) {
        self.status = BuildStatus::Building;
        self.start_time = Some(Instant::now());
    }

    /// Complete the stage
    pub fn complete(&mut self) {
        self.status = BuildStatus::Success;
        if let Some(start_time) = self.start_time {
            self.duration = Some(start_time.elapsed());
        }
    }

    /// Fail the stage
    pub fn fail(&mut self, error: String) {
        self.status = BuildStatus::Failed;
        self.error = Some(error);
        if let Some(start_time) = self.start_time {
            self.duration = Some(start_time.elapsed());
        }
    }
}

/// Build environment setup
pub struct BuildEnvironment {
    pub temp_dir: String,
    pub output_dir: String,
    pub work_dir: String,
    pub tools_available: std::collections::HashMap<String, bool>,
}

impl BuildEnvironment {
    /// Create a new build environment
    pub fn new(output_dir: String) -> Self {
        let temp_dir = format!("/tmp/biomeos-build-{}", uuid::Uuid::new_v4());
        let work_dir = format!("{}/work", temp_dir);

        Self {
            temp_dir,
            output_dir,
            work_dir,
            tools_available: std::collections::HashMap::new(),
        }
    }

    /// Setup the build environment
    pub fn setup(&mut self) -> Result<(), String> {
        // Create directories
        std::fs::create_dir_all(&self.temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        std::fs::create_dir_all(&self.work_dir)
            .map_err(|e| format!("Failed to create work directory: {}", e))?;

        std::fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Check available tools
        self.check_tools()?;

        Ok(())
    }

    /// Check required tools availability
    fn check_tools(&mut self) -> Result<(), String> {
        let required_tools = vec![
            "genisoimage",
            "mksquashfs",
            "syslinux",
            "grub-mkrescue",
            "xorriso",
        ];

        for tool in required_tools {
            let available = Command::new("which")
                .arg(tool)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false);

            self.tools_available.insert(tool.to_string(), available);
        }

        // Check if essential tools are available
        let essential_tools = vec!["genisoimage", "mksquashfs"];
        for tool in essential_tools {
            if !self.tools_available.get(tool).unwrap_or(&false) {
                return Err(format!("Required tool '{}' is not available", tool));
            }
        }

        Ok(())
    }

    /// Cleanup the build environment
    pub fn cleanup(&self) -> Result<(), String> {
        std::fs::remove_dir_all(&self.temp_dir)
            .map_err(|e| format!("Failed to cleanup temp directory: {}", e))?;
        Ok(())
    }

    /// Get disk space information
    pub fn get_disk_space(&self) -> Result<DiskSpace, String> {
        // In a real implementation, this would check actual disk space
        Ok(DiskSpace {
            total: 100 * 1024 * 1024 * 1024,    // 100GB
            available: 50 * 1024 * 1024 * 1024, // 50GB
            used: 50 * 1024 * 1024 * 1024,      // 50GB
        })
    }
}

/// Disk space information
#[derive(Debug)]
pub struct DiskSpace {
    pub total: u64,
    pub available: u64,
    pub used: u64,
}

impl DiskSpace {
    /// Check if there's enough space for build
    pub fn has_enough_space(&self, required: u64) -> bool {
        self.available >= required
    }

    /// Get available space in MB
    pub fn available_mb(&self) -> u64 {
        self.available / (1024 * 1024)
    }

    /// Get usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.used as f32 / self.total as f32) * 100.0
    }
}

/// Build logger for tracking build progress
pub struct BuildLogger {
    logs: Vec<LogEntry>,
    max_entries: usize,
}

impl BuildLogger {
    /// Create a new build logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            logs: Vec::new(),
            max_entries,
        }
    }

    /// Add a log entry
    pub fn log(&mut self, level: LogLevel, message: String) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message,
        };

        self.logs.push(entry);

        // Keep only the most recent entries
        if self.logs.len() > self.max_entries {
            self.logs.remove(0);
        }
    }

    /// Get all log entries
    pub fn get_logs(&self) -> &[LogEntry] {
        &self.logs
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.logs.clear();
    }

    /// Get logs by level
    pub fn get_logs_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
}

/// Log level
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl LogLevel {
    /// Get color for UI display
    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Info => egui::Color32::WHITE,
            Self::Warning => egui::Color32::YELLOW,
            Self::Error => egui::Color32::RED,
            Self::Debug => egui::Color32::GRAY,
        }
    }

    /// Get icon for UI display
    pub fn icon(&self) -> &str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Error => "❌",
            Self::Debug => "🔍",
        }
    }
}
