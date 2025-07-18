//! Build queue management for ISO Creator
//!
//! This module handles the build queue, job scheduling, and managing
//! multiple concurrent builds.

use crate::views::iso_creator::types::*;
use crate::views::iso_creator::build::BuildManager;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Build queue manager
pub struct BuildQueue {
    jobs: VecDeque<QueuedJob>,
    running_jobs: Vec<RunningJob>,
    completed_jobs: Vec<CompletedJob>,
    max_concurrent: usize,
    auto_start: bool,
    next_job_id: u64,
}

impl BuildQueue {
    /// Create a new build queue
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            jobs: VecDeque::new(),
            running_jobs: Vec::new(),
            completed_jobs: Vec::new(),
            max_concurrent,
            auto_start: true,
            next_job_id: 1,
        }
    }

    /// Add a job to the queue
    pub fn add_job(&mut self, config: IsoConfig, priority: JobPriority) -> String {
        let job_id = format!("job-{}", self.next_job_id);
        self.next_job_id += 1;

        let queued_job = QueuedJob {
            id: job_id.clone(),
            config,
            priority,
            queued_at: Instant::now(),
            estimated_duration: Duration::from_secs(600), // 10 minutes estimate
        };

        // Insert job based on priority
        match priority {
            JobPriority::High => self.jobs.push_front(queued_job),
            JobPriority::Normal => self.jobs.push_back(queued_job),
            JobPriority::Low => self.jobs.push_back(queued_job),
        }

        // Sort queue by priority
        self.sort_queue();

        // Auto-start if enabled and capacity available
        if self.auto_start && self.can_start_job() {
            self.start_next_job();
        }

        job_id
    }

    /// Remove a job from the queue
    pub fn remove_job(&mut self, job_id: &str) -> Result<(), String> {
        let position = self.jobs.iter().position(|job| job.id == job_id);
        
        if let Some(pos) = position {
            self.jobs.remove(pos);
            Ok(())
        } else {
            Err(format!("Job {} not found in queue", job_id))
        }
    }

    /// Start the next job in the queue
    pub fn start_next_job(&mut self) -> Option<String> {
        if !self.can_start_job() {
            return None;
        }

        if let Some(queued_job) = self.jobs.pop_front() {
            let running_job = RunningJob {
                id: queued_job.id.clone(),
                config: queued_job.config,
                priority: queued_job.priority,
                queued_at: queued_job.queued_at,
                started_at: Instant::now(),
                progress: 0.0,
                current_stage: "Preparing".to_string(),
                build_manager: Arc::new(Mutex::new(BuildManager::new(IsoCreatorConfig::default()))),
            };

            let job_id = running_job.id.clone();
            self.running_jobs.push(running_job);
            
            Some(job_id)
        } else {
            None
        }
    }

    /// Cancel a running job
    pub fn cancel_job(&mut self, job_id: &str) -> Result<(), String> {
        let position = self.running_jobs.iter().position(|job| job.id == job_id);
        
        if let Some(pos) = position {
            let running_job = self.running_jobs.remove(pos);
            
            // Cancel the actual build
            if let Ok(mut build_manager) = running_job.build_manager.lock() {
                build_manager.cancel_build().ok();
            }
            
            // Move to completed jobs as cancelled
            let completed_job = CompletedJob {
                id: running_job.id,
                config: running_job.config,
                priority: running_job.priority,
                queued_at: running_job.queued_at,
                started_at: running_job.started_at,
                completed_at: Instant::now(),
                status: BuildStatus::Failed,
                error_message: Some("Cancelled by user".to_string()),
                output_path: None,
                final_size: None,
            };
            
            self.completed_jobs.push(completed_job);
            
            // Try to start next job
            self.start_next_job();
            
            Ok(())
        } else {
            Err(format!("Running job {} not found", job_id))
        }
    }

    /// Complete a job
    pub fn complete_job(&mut self, job_id: &str, output_path: Option<String>, final_size: Option<u64>) -> Result<(), String> {
        let position = self.running_jobs.iter().position(|job| job.id == job_id);
        
        if let Some(pos) = position {
            let running_job = self.running_jobs.remove(pos);
            
            let completed_job = CompletedJob {
                id: running_job.id,
                config: running_job.config,
                priority: running_job.priority,
                queued_at: running_job.queued_at,
                started_at: running_job.started_at,
                completed_at: Instant::now(),
                status: BuildStatus::Success,
                error_message: None,
                output_path,
                final_size,
            };
            
            self.completed_jobs.push(completed_job);
            
            // Try to start next job
            self.start_next_job();
            
            Ok(())
        } else {
            Err(format!("Running job {} not found", job_id))
        }
    }

    /// Fail a job
    pub fn fail_job(&mut self, job_id: &str, error_message: String) -> Result<(), String> {
        let position = self.running_jobs.iter().position(|job| job.id == job_id);
        
        if let Some(pos) = position {
            let running_job = self.running_jobs.remove(pos);
            
            let completed_job = CompletedJob {
                id: running_job.id,
                config: running_job.config,
                priority: running_job.priority,
                queued_at: running_job.queued_at,
                started_at: running_job.started_at,
                completed_at: Instant::now(),
                status: BuildStatus::Failed,
                error_message: Some(error_message),
                output_path: None,
                final_size: None,
            };
            
            self.completed_jobs.push(completed_job);
            
            // Try to start next job
            self.start_next_job();
            
            Ok(())
        } else {
            Err(format!("Running job {} not found", job_id))
        }
    }

    /// Check if a new job can be started
    fn can_start_job(&self) -> bool {
        self.running_jobs.len() < self.max_concurrent && !self.jobs.is_empty()
    }

    /// Sort queue by priority
    fn sort_queue(&mut self) {
        let mut jobs: Vec<_> = self.jobs.drain(..).collect();
        jobs.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.queued_at.cmp(&b.queued_at))
        });
        self.jobs.extend(jobs);
    }

    /// Get queued jobs
    pub fn get_queued_jobs(&self) -> &VecDeque<QueuedJob> {
        &self.jobs
    }

    /// Get running jobs
    pub fn get_running_jobs(&self) -> &[RunningJob] {
        &self.running_jobs
    }

    /// Get completed jobs
    pub fn get_completed_jobs(&self) -> &[CompletedJob] {
        &self.completed_jobs
    }

    /// Get queue statistics
    pub fn get_statistics(&self) -> QueueStatistics {
        let total_jobs = self.jobs.len() + self.running_jobs.len() + self.completed_jobs.len();
        let successful_jobs = self.completed_jobs.iter()
            .filter(|job| job.status == BuildStatus::Success)
            .count();
        let failed_jobs = self.completed_jobs.iter()
            .filter(|job| job.status == BuildStatus::Failed)
            .count();
        
        let average_duration = if !self.completed_jobs.is_empty() {
            let total_duration: Duration = self.completed_jobs.iter()
                .map(|job| job.completed_at.duration_since(job.started_at))
                .sum();
            total_duration / self.completed_jobs.len() as u32
        } else {
            Duration::from_secs(0)
        };

        QueueStatistics {
            total_jobs,
            queued_jobs: self.jobs.len(),
            running_jobs: self.running_jobs.len(),
            completed_jobs: self.completed_jobs.len(),
            successful_jobs,
            failed_jobs,
            average_duration,
        }
    }

    /// Clear completed jobs
    pub fn clear_completed(&mut self) {
        self.completed_jobs.clear();
    }

    /// Set auto-start mode
    pub fn set_auto_start(&mut self, auto_start: bool) {
        self.auto_start = auto_start;
        
        if auto_start {
            // Try to start jobs if capacity is available
            while self.can_start_job() {
                if self.start_next_job().is_none() {
                    break;
                }
            }
        }
    }

    /// Get estimated wait time for new job
    pub fn get_estimated_wait_time(&self) -> Duration {
        if self.jobs.is_empty() {
            return Duration::from_secs(0);
        }

        let jobs_ahead = self.jobs.len();
        let available_slots = self.max_concurrent - self.running_jobs.len();
        
        if available_slots > 0 {
            // Job can start immediately
            Duration::from_secs(0)
        } else {
            // Estimate based on running jobs
            let avg_remaining_time = self.running_jobs.iter()
                .map(|job| {
                    let elapsed = job.started_at.elapsed();
                    let estimated_total = job.estimated_duration();
                    if elapsed < estimated_total {
                        estimated_total - elapsed
                    } else {
                        Duration::from_secs(60) // Minimum 1 minute
                    }
                })
                .sum::<Duration>() / self.running_jobs.len() as u32;
            
            avg_remaining_time * ((jobs_ahead / self.max_concurrent.max(1)) as u32)
        }
    }
}

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 1,
    Normal = 2,
    High = 3,
}

impl JobPriority {
    /// Get all priority levels
    pub fn all() -> Vec<Self> {
        vec![Self::Low, Self::Normal, Self::High]
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
        }
    }

    /// Get color for UI
    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Low => egui::Color32::GRAY,
            Self::Normal => egui::Color32::WHITE,
            Self::High => egui::Color32::from_rgb(255, 165, 0), // Orange
        }
    }
}

/// Queued job
#[derive(Debug, Clone)]
pub struct QueuedJob {
    pub id: String,
    pub config: IsoConfig,
    pub priority: JobPriority,
    pub queued_at: Instant,
    pub estimated_duration: Duration,
}

impl QueuedJob {
    /// Get wait time since queued
    pub fn wait_time(&self) -> Duration {
        self.queued_at.elapsed()
    }
}

/// Running job
#[derive(Debug)]
pub struct RunningJob {
    pub id: String,
    pub config: IsoConfig,
    pub priority: JobPriority,
    pub queued_at: Instant,
    pub started_at: Instant,
    pub progress: f32,
    pub current_stage: String,
    pub build_manager: Arc<Mutex<BuildManager>>,
}

impl RunningJob {
    /// Get estimated total duration
    pub fn estimated_duration(&self) -> Duration {
        // Base duration estimate
        let mut duration = Duration::from_secs(300); // 5 minutes base
        
        // Add time for each primal
        duration += Duration::from_secs(60 * self.config.included_primals.len() as u64);
        
        // Add time for each niche
        duration += Duration::from_secs(120 * self.config.included_niches.len() as u64);
        
        // Add time for each custom component
        duration += Duration::from_secs(30 * self.config.custom_components.len() as u64);
        
        // Adjust for compression level
        match self.config.compression_level {
            0..=3 => duration += Duration::from_secs(60),
            4..=6 => duration += Duration::from_secs(120),
            7..=9 => duration += Duration::from_secs(180),
            _ => duration += Duration::from_secs(240),
        }
        
        duration
    }

    /// Get elapsed time since started
    pub fn elapsed_time(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Get estimated remaining time
    pub fn estimated_remaining_time(&self) -> Duration {
        let elapsed = self.elapsed_time();
        let estimated_total = self.estimated_duration();
        
        if elapsed < estimated_total {
            estimated_total - elapsed
        } else {
            Duration::from_secs(0)
        }
    }
}

/// Completed job
#[derive(Debug, Clone)]
pub struct CompletedJob {
    pub id: String,
    pub config: IsoConfig,
    pub priority: JobPriority,
    pub queued_at: Instant,
    pub started_at: Instant,
    pub completed_at: Instant,
    pub status: BuildStatus,
    pub error_message: Option<String>,
    pub output_path: Option<String>,
    pub final_size: Option<u64>,
}

impl CompletedJob {
    /// Get total duration (queue + build time)
    pub fn total_duration(&self) -> Duration {
        self.completed_at.duration_since(self.queued_at)
    }

    /// Get build duration
    pub fn build_duration(&self) -> Duration {
        self.completed_at.duration_since(self.started_at)
    }

    /// Get wait time in queue
    pub fn queue_duration(&self) -> Duration {
        self.started_at.duration_since(self.queued_at)
    }

    /// Check if job was successful
    pub fn is_successful(&self) -> bool {
        self.status == BuildStatus::Success
    }
}

/// Queue statistics
#[derive(Debug)]
pub struct QueueStatistics {
    pub total_jobs: usize,
    pub queued_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
    pub successful_jobs: usize,
    pub failed_jobs: usize,
    pub average_duration: Duration,
}

impl QueueStatistics {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.completed_jobs > 0 {
            (self.successful_jobs as f32 / self.completed_jobs as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f32 {
        if self.completed_jobs > 0 {
            (self.failed_jobs as f32 / self.completed_jobs as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Get utilization rate (running jobs / max capacity)
    pub fn utilization_rate(&self, max_concurrent: usize) -> f32 {
        if max_concurrent > 0 {
            (self.running_jobs as f32 / max_concurrent as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Queue event for notifications
#[derive(Debug, Clone)]
pub enum QueueEvent {
    JobQueued(String),
    JobStarted(String),
    JobCompleted(String),
    JobFailed(String, String),
    JobCancelled(String),
    QueueEmpty,
    QueueFull,
}

/// Queue event handler
pub trait QueueEventHandler {
    fn handle_event(&self, event: QueueEvent);
}

/// Simple event handler implementation
pub struct SimpleEventHandler;

impl QueueEventHandler for SimpleEventHandler {
    fn handle_event(&self, event: QueueEvent) {
        match event {
            QueueEvent::JobQueued(id) => println!("Job {} queued", id),
            QueueEvent::JobStarted(id) => println!("Job {} started", id),
            QueueEvent::JobCompleted(id) => println!("Job {} completed", id),
            QueueEvent::JobFailed(id, error) => println!("Job {} failed: {}", id, error),
            QueueEvent::JobCancelled(id) => println!("Job {} cancelled", id),
            QueueEvent::QueueEmpty => println!("Queue is empty"),
            QueueEvent::QueueFull => println!("Queue is full"),
        }
    }
} 