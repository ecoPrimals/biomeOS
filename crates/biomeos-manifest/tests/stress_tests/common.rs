//! Common structures and imports for stress tests

use biomeos_manifest::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Tournament configuration for stress testing
#[derive(Clone)]
pub struct TournamentConfig {
    pub max_players: usize,
    pub regions: Vec<String>,
    pub physics_quality: String,
    pub anti_cheat: bool,
    pub match_duration: String,
}

/// Primal type enumeration for testing
#[derive(Debug, Clone)]
pub enum PrimalType {
    Songbird,
    NestGate,
    Toadstool,
    Beardog,
    Squirrel,
}

/// Create a massive tournament configuration
pub fn create_massive_tournament_config(player_count: usize) -> TournamentConfig {
    TournamentConfig {
        max_players: player_count,
        regions: vec![
            "us-east".to_string(),
            "us-west".to_string(),
            "eu-west".to_string(),
            "eu-central".to_string(),
            "ap-southeast".to_string(),
            "ap-northeast".to_string(),
        ],
        physics_quality: "ultra".to_string(),
        anti_cheat: true,
        match_duration: "20m".to_string(),
    }
}

/// Get current memory usage (mock implementation)
pub fn get_memory_usage() -> usize {
    // Mock implementation - in real tests this would query actual memory usage
    std::process::id() as usize * 1024 // Placeholder
}

/// Add cross-region replication to a manifest
pub fn add_cross_region_replication(manifest: &BiomeManifest) -> Result<(), String> {
    // Mock implementation for cross-region replication
    if manifest.metadata.name.is_empty() {
        return Err("Invalid manifest for replication".to_string());
    }
    Ok(())
}

/// Validate primal dependencies in a manifest
pub fn validate_primal_dependencies(manifest: &BiomeManifest) -> Result<(), String> {
    // Mock implementation for primal dependency validation
    if manifest.primals.is_empty() {
        return Err("No primals found in manifest".to_string());
    }
    Ok(())
} 