// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Manifest Management
//!
//! Handles finding, validating, and deploying federation manifests

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};

use super::config::FederationConfig;

/// Find a manifest by name in configured directories
pub fn find_manifest(config: &FederationConfig, manifest: &str) -> Result<PathBuf> {
    // First, check if it's a direct file path
    let direct_path = PathBuf::from(manifest);
    if direct_path.exists() {
        return Ok(direct_path);
    }

    // Search in template directories
    for template_dir in &config.manifests.template_dirs {
        let manifest_path = template_dir.join(format!("{}.yaml", manifest));
        if manifest_path.exists() {
            return Ok(manifest_path);
        }
    }

    // Search in custom directory
    if let Some(custom_dir) = &config.manifests.custom_dir {
        let manifest_path = custom_dir.join(format!("{}.yaml", manifest));
        if manifest_path.exists() {
            return Ok(manifest_path);
        }
    }

    Err(anyhow::anyhow!("Manifest '{}' not found in any configured directory", manifest))
}

/// Validate a manifest file
pub fn validate_manifest(manifest_path: &PathBuf) -> Result<()> {
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!("Manifest file does not exist: {}", manifest_path.display()));
    }

    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;

    // Basic YAML validation
    let _: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("Invalid YAML in manifest: {}", manifest_path.display()))?;

    info!("✓ Manifest validation passed: {}", manifest_path.display());
    Ok(())
}

/// Deploy a manifest using kubectl or similar tool
pub fn deploy_manifest(
    config: &FederationConfig,
    manifest: &str,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let manifest_path = find_manifest(config, manifest)?;
    
    info!("Deploying manifest: {}", manifest_path.display());
    
    // Validate first
    validate_manifest(&manifest_path)?;
    
    if dry_run {
        info!("✓ Dry run completed successfully for {}", manifest_path.display());
        return Ok(());
    }

    // Determine deployment method based on manifest content
    let content = fs::read_to_string(&manifest_path)?;
    
    if content.contains("apiVersion:") {
        // Kubernetes manifest
        deploy_kubernetes_manifest(&manifest_path, force)
    } else {
        // Custom federation manifest
        deploy_federation_manifest(config, &manifest_path, force)
    }
}

fn deploy_kubernetes_manifest(manifest_path: &PathBuf, force: bool) -> Result<()> {
    let mut cmd = Command::new("kubectl");
    cmd.arg("apply").arg("-f").arg(manifest_path);
    
    if force {
        cmd.arg("--force");
    }

    let output = cmd.output()
        .with_context(|| "Failed to execute kubectl - ensure it's installed and configured")?;

    if output.status.success() {
        info!("✓ Kubernetes manifest deployed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("kubectl failed: {}", stderr))
    }
}

fn deploy_federation_manifest(config: &FederationConfig, manifest_path: &PathBuf, _force: bool) -> Result<()> {
    // Custom federation deployment logic would go here
    warn!("Custom federation manifest deployment not yet implemented");
    info!("Would deploy {} using federation deployment", manifest_path.display());
    Ok(())
}

/// List available manifests in configured directories
pub fn list_manifests(config: &FederationConfig, detailed: bool) -> Result<()> {
    info!("Available Federation Manifests:");
    info!("================================");

    for template_dir in &config.manifests.template_dirs {
        if template_dir.exists() {
            list_manifests_in_dir(template_dir, "Template", detailed)?;
        }
    }

    if let Some(custom_dir) = &config.manifests.custom_dir {
        if custom_dir.exists() {
            list_manifests_in_dir(custom_dir, "Custom", detailed)?;
        }
    }

    Ok(())
}

fn list_manifests_in_dir(dir: &PathBuf, category: &str, detailed: bool) -> Result<()> {
    info!("\n{} Manifests ({}/):", category, dir.display());
    
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    let mut count = 0;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            count += 1;
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("<invalid>");
            
            if detailed {
                let metadata = fs::metadata(&path)?;
                let size = metadata.len();
                let modified = metadata.modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();
                
                info!("  {} ({} bytes, modified: {})", name, size, modified);
            } else {
                info!("  {}", name);
            }
        }
    }
    
    if count == 0 {
        info!("  (no manifests found)");
    }
    
    Ok(())
} 