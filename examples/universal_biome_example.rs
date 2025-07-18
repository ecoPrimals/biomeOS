//! # Universal Biome Example
//!
//! This example demonstrates how to use the universal biome system with
//! capability-based discovery instead of hard-coded Primal names.
//!
//! The example shows:
//! - Creating a universal biome manifest
//! - Using capability-based requirements
//! - Discovering primals by capability
//! - Bootstrapping an ecosystem

mod universal_biome_example;

use biomeos_core::UniversalBiomeCoordinator;
use std::collections::HashMap;
use tokio;
use tracing::{info, Level};
use universal_biome_example::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Universal Biome Example");

    // Example 1: Create a universal biome manifest for a web application
    let web_app_manifest = create_web_app_manifest()?;
    info!("Created web application manifest");

    // Example 2: Create a universal biome manifest for AI/ML workload
    let ai_ml_manifest = create_ai_ml_manifest()?;
    info!("Created AI/ML workload manifest");

    // Example 3: Create a universal biome manifest for a distributed database
    let database_manifest = create_database_manifest()?;
    info!("Created database manifest");

    // Example 4: Bootstrap ecosystems using universal coordinator
    let _coordinator = UniversalBiomeCoordinator::new();

    // Note: In a real implementation, you would have actual primals running
    // that implement the UniversalPrimalProvider trait. This example shows
    // the structure without actual deployment.

    info!(
        "Web App Biome requires capabilities: {:?}",
        web_app_manifest.get_all_required_capabilities()
    );
    info!(
        "AI/ML Biome requires capabilities: {:?}",
        ai_ml_manifest.get_all_required_capabilities()
    );
    info!(
        "Database Biome requires capabilities: {:?}",
        database_manifest.get_all_required_capabilities()
    );

    // Example 5: Show how this is agnostic to primal implementations
    demonstrate_agnostic_approach(&web_app_manifest)?;

    // Example 6: Show YAML output example
    example_yaml_output()?;

    info!("Universal Biome Example completed successfully");
    Ok(())
}
