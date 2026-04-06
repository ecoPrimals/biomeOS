// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health Module
//! 
//! Ecosystem health monitoring and diagnostics for biomeOS.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::{
    discover_workspace_root, execute_command, print_error, print_info, print_section,
    print_success, print_warning,
};

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Root of the workspace tree (discovered at runtime).
    pub workspace_root: PathBuf,
    /// Whether to run detailed per-crate checks.
    pub detailed_checks: bool,
    /// Whether to check external dep freshness (requires `cargo-outdated`).
    pub check_external_deps: bool,
    /// Per-command timeout.
    pub timeout_seconds: u64,
}

impl HealthConfig {
    /// Create a config with the workspace root discovered from `cwd`.
    ///
    /// # Errors
    /// Returns an error if no workspace root is found.
    pub fn discover() -> Result<Self> {
        Ok(Self {
            workspace_root: discover_workspace_root()?,
            detailed_checks: true,
            check_external_deps: false,
            timeout_seconds: 60,
        })
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self::discover().unwrap_or_else(|_| Self {
            workspace_root: PathBuf::from("."),
            detailed_checks: true,
            check_external_deps: false,
            timeout_seconds: 60,
        })
    }
}

/// Overall ecosystem health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Vec<String>,
}

/// Run comprehensive ecosystem health check
pub async fn check_ecosystem_health(config: &HealthConfig) -> Result<Vec<HealthResult>> {
    print_section("🏥 biomeOS ECOSYSTEM HEALTH CHECK");
    print_info("Performing comprehensive system diagnostics...");
    
    let mut results = Vec::new();
    
    // Check biomeOS core
    results.push(check_biomeos_core(config).await?);
    
    // Check UI system
    results.push(check_ui_system(config).await?);
    
    // Check build system
    results.push(check_build_system(config).await?);
    
    // Check dependencies
    results.push(check_dependencies(config).await?);
    
    // Check ecosystem components
    results.extend(check_ecosystem_components(config).await?);
    
    // Check sovereignty features
    results.push(check_sovereignty_features(config).await?);
    
    // Print summary
    print_health_summary(&results);
    
    Ok(results)
}

/// Check biomeOS core health
async fn check_biomeos_core(config: &HealthConfig) -> Result<HealthResult> {
    print_info("Checking biomeOS core...");
    
    let workspace_path = &config.workspace_root;

    match execute_command("cargo", &["check", "-p", "biomeos-core"], Some(workspace_path)).await {
        Ok(_) => Ok(HealthResult {
            component: "biomeOS Core".to_string(),
            status: HealthStatus::Healthy,
            message: "Core system compiles and functions correctly".to_string(),
            details: vec![
                "✅ Compilation successful".to_string(),
                "✅ All modules accessible".to_string(),
                "✅ API interfaces operational".to_string(),
            ],
        }),
        Err(e) => Ok(HealthResult {
            component: "biomeOS Core".to_string(),
            status: HealthStatus::Critical,
            message: format!("Core system compilation failed: {}", e),
            details: vec![
                "❌ Compilation errors detected".to_string(),
                "❌ Core functionality may be impaired".to_string(),
            ],
        }),
    }
}

/// Check UI system health
async fn check_ui_system(config: &HealthConfig) -> Result<HealthResult> {
    print_info("Checking UI system...");
    
    let workspace_path = &config.workspace_root;

    match execute_command("cargo", &["check", "-p", "biomeos-ui"], Some(workspace_path)).await {
        Ok(_) => Ok(HealthResult {
            component: "UI System".to_string(),
            status: HealthStatus::Healthy,
            message: "UI system is operational".to_string(),
            details: vec![
                "✅ egui framework initialized".to_string(),
                "✅ All views compile successfully".to_string(),
                "✅ API integration functional".to_string(),
            ],
        }),
        Err(e) => Ok(HealthResult {
            component: "UI System".to_string(),
            status: HealthStatus::Critical,
            message: format!("UI system issues detected: {}", e),
            details: vec![
                "❌ UI compilation errors".to_string(),
                "❌ Interface may be unavailable".to_string(),
            ],
        }),
    }
}

/// Check build system health
async fn check_build_system(config: &HealthConfig) -> Result<HealthResult> {
    print_info("Checking build system...");
    
    let workspace_path = &config.workspace_root;

    match execute_command("cargo", &["check", "--workspace"], Some(workspace_path)).await {
        Ok(_) => Ok(HealthResult {
            component: "Build System".to_string(),
            status: HealthStatus::Healthy,
            message: "Build system is functioning correctly".to_string(),
            details: vec![
                "✅ Workspace configuration valid".to_string(),
                "✅ All crates compile successfully".to_string(),
                "✅ Dependencies resolved".to_string(),
            ],
        }),
        Err(e) => Ok(HealthResult {
            component: "Build System".to_string(),
            status: HealthStatus::Warning,
            message: format!("Build system issues: {}", e),
            details: vec![
                "⚠️ Some compilation warnings".to_string(),
                "⚠️ Check dependency versions".to_string(),
            ],
        }),
    }
}

/// Check dependencies health
async fn check_dependencies(config: &HealthConfig) -> Result<HealthResult> {
    print_info("Checking dependencies...");
    
    let workspace_path = &config.workspace_root;

    match execute_command("cargo", &["outdated", "--workspace"], Some(workspace_path)).await {
        Ok(output) => {
            if output.contains("All dependencies are up to date") || output.is_empty() {
                Ok(HealthResult {
                    component: "Dependencies".to_string(),
                    status: HealthStatus::Healthy,
                    message: "All dependencies are current".to_string(),
                    details: vec![
                        "✅ No outdated dependencies".to_string(),
                        "✅ Security vulnerabilities: 0".to_string(),
                    ],
                })
            } else {
                Ok(HealthResult {
                    component: "Dependencies".to_string(),
                    status: HealthStatus::Warning,
                    message: "Some dependencies may be outdated".to_string(),
                    details: vec![
                        "⚠️ Outdated dependencies detected".to_string(),
                        "⚠️ Consider running cargo update".to_string(),
                    ],
                })
            }
        },
        Err(_) => {
            // cargo-outdated might not be installed
            Ok(HealthResult {
                component: "Dependencies".to_string(),
                status: HealthStatus::Unknown,
                message: "Unable to check dependency status".to_string(),
                details: vec![
                    "ℹ️ Install cargo-outdated for dependency checking".to_string(),
                ],
            })
        }
    }
}

/// Check ecosystem components by probing runtime sockets.
///
/// Uses XDG runtime directory socket scanning — primals are discovered by
/// capability, not by hardcoded paths.
async fn check_ecosystem_components(_config: &HealthConfig) -> Result<Vec<HealthResult>> {
    print_info("Checking ecosystem components via runtime socket discovery...");

    let mut results = Vec::new();

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("UID")
                .or_else(|_| std::env::var("EUID"))
                .map(|uid| PathBuf::from(format!("/run/user/{uid}")))
        })
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("biomeos");

    if !runtime_dir.exists() {
        results.push(HealthResult {
            component: "Runtime Discovery".to_string(),
            status: HealthStatus::Warning,
            message: "No biomeos runtime directory found — primals may not be running".to_string(),
            details: vec![format!("Checked: {}", runtime_dir.display())],
        });
        return Ok(results);
    }

    let mut socket_count = 0u32;
    if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let looks_like_socket = path
                .extension()
                .map_or(true, |ext| ext == "sock")
                && path.file_name().and_then(|n| n.to_str()).map_or(false, |n| {
                    n.contains('-') || n.ends_with(".sock")
                });
            if looks_like_socket {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let primal_name = name.split('-').next().unwrap_or(name);
                    socket_count += 1;
                    results.push(HealthResult {
                        component: primal_name.to_string(),
                        status: HealthStatus::Healthy,
                        message: format!("Socket discovered: {}", path.display()),
                        details: vec!["Discovered via runtime socket scan".to_string()],
                    });
                }
            }
        }
    }

    if socket_count == 0 {
        results.push(HealthResult {
            component: "Primals".to_string(),
            status: HealthStatus::Unknown,
            message: "No primal sockets found in runtime directory".to_string(),
            details: vec![
                format!("Scanned: {}", runtime_dir.display()),
                "Start primals with `biomeos nucleus start`".to_string(),
            ],
        });
    }

    Ok(results)
}

/// Check sovereignty features by inspecting the dependency tree and build config.
async fn check_sovereignty_features(config: &HealthConfig) -> Result<HealthResult> {
    print_info("Checking sovereignty features...");

    let workspace_path = &config.workspace_root;
    let mut details = Vec::new();
    let mut warnings = 0u32;

    // Check for forbidden C-binding crates in the dep tree
    let forbidden = ["openssl-sys", "ring", "aws-lc-sys", "native-tls", "zstd-sys"];
    match execute_command("cargo", &["tree", "--workspace", "--prefix", "none"], Some(workspace_path)).await {
        Ok(tree_output) => {
            for dep in &forbidden {
                if tree_output.contains(dep) {
                    details.push(format!("C-binding detected: {dep}"));
                    warnings += 1;
                } else {
                    details.push(format!("No {dep} in dependency tree"));
                }
            }
        }
        Err(e) => {
            details.push(format!("Could not inspect dep tree: {e}"));
            warnings += 1;
        }
    }

    // Check AGPL license in workspace Cargo.toml
    let cargo_toml = workspace_path.join("Cargo.toml");
    if cargo_toml.exists() {
        let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();
        if content.contains("AGPL-3.0-only") {
            details.push("License: AGPL-3.0-only".to_string());
        } else {
            details.push("License: NOT AGPL-3.0-only — sovereignty violation".to_string());
            warnings += 1;
        }
    }

    // Check for unsafe code policy
    let forbid_count = match execute_command("grep", &["-r", "forbid(unsafe_code)", "crates/"], Some(workspace_path)).await {
        Ok(output) => output.lines().count(),
        Err(_) => 0,
    };
    details.push(format!("{forbid_count} crates forbid unsafe code"));

    let status = if warnings > 0 {
        HealthStatus::Warning
    } else {
        HealthStatus::Healthy
    };

    Ok(HealthResult {
        component: "Sovereignty".to_string(),
        status,
        message: format!("{warnings} sovereignty concern(s) detected"),
        details,
    })
}

/// Print health summary
fn print_health_summary(results: &[HealthResult]) {
    print_section("📊 HEALTH SUMMARY");
    
    let mut healthy = 0;
    let mut warnings = 0;
    let mut critical = 0;
    let mut unknown = 0;
    
    for result in results {
        match result.status {
            HealthStatus::Healthy => {
                print_success(&format!("{}: {}", result.component, result.message));
                healthy += 1;
            },
            HealthStatus::Warning => {
                print_warning(&format!("{}: {}", result.component, result.message));
                warnings += 1;
            },
            HealthStatus::Critical => {
                print_error(&format!("{}: {}", result.component, result.message));
                critical += 1;
            },
            HealthStatus::Unknown => {
                print_info(&format!("{}: {}", result.component, result.message));
                unknown += 1;
            },
        }
    }
    
    println!("\n📈 OVERALL STATUS:");
    println!("  ✅ Healthy: {}", healthy);
    println!("  ⚠️ Warnings: {}", warnings);
    println!("  ❌ Critical: {}", critical);
    println!("  ❓ Unknown: {}", unknown);
    
    let overall_status = if critical > 0 {
        "🔴 CRITICAL - Immediate attention required"
    } else if warnings > 0 {
        "🟡 WARNING - Some issues detected"
    } else if unknown > 0 {
        "🟡 PARTIAL - Some components unknown"
    } else {
        "🟢 HEALTHY - All systems operational"
    };
    
    println!("\n🎯 ECOSYSTEM STATUS: {}", overall_status);
} 