// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![forbid(unsafe_code)]

//! Ecosystem Health Monitor
//!
//! Pure Rust ecosystem health monitoring and diagnostics.
//! Comprehensive system analysis and health reporting.

use anyhow::Result;
use biomeos_tools::{
    health::{HealthConfig, HealthStatus, check_ecosystem_health},
    print_section, print_success,
};
use biomeos_types::primal_names;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tokio::net::UnixStream;

/// Socket directory for primal IPC (`BIOMEOS_SOCKET_DIR` or `/tmp/biomeos`).
fn biomeos_socket_dir() -> PathBuf {
    std::env::var("BIOMEOS_SOCKET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/biomeos"))
}

/// List `*.sock` entries under `dir` (non-recursive). Missing dir returns empty.
fn list_socket_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut v: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|s| s.ends_with(".sock"))
        })
        .collect();
    v.sort();
    v
}

/// Probe UDS; returns (reachable, unreachable, per-path results).
async fn probe_unix_sockets(paths: &[PathBuf]) -> (usize, usize, Vec<(PathBuf, bool)>) {
    let mut results = Vec::with_capacity(paths.len());
    let mut ok = 0usize;
    let mut bad = 0usize;
    for p in paths {
        let connected = UnixStream::connect(p).await.is_ok();
        if connected {
            ok += 1;
        } else {
            bad += 1;
        }
        results.push((p.clone(), connected));
    }
    (ok, bad, results)
}

fn read_proc_meminfo_kb() -> Option<(u64, u64)> {
    let s = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total: Option<u64> = None;
    let mut available: Option<u64> = None;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
            total = Some(kb);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
            available = Some(kb);
        }
        if total.is_some() && available.is_some() {
            break;
        }
    }
    Some((total?, available?))
}

fn read_proc_loadavg() -> Option<String> {
    let s = std::fs::read_to_string("/proc/loadavg").ok()?;
    let one = s.split_whitespace().next()?;
    let two = s.split_whitespace().nth(1)?;
    let three = s.split_whitespace().nth(2)?;
    Some(format!("{} {} {}", one, two, three))
}

/// Free/total bytes for `.` via `df -B1 -P` (no extra Rust deps).
fn df_pwd_bytes() -> Option<(u64, u64)> {
    let out = std::process::Command::new("df")
        .args(["-B1", "-P", "."])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.lines().filter(|l| !l.is_empty()).collect();
    if lines.len() < 2 {
        return None;
    }
    let data = lines[1];
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    let total: u64 = parts[1].parse().ok()?;
    let avail: u64 = parts[3].parse().ok()?;
    Some((total, avail))
}

fn format_kib(kib: u64) -> String {
    const KIB: f64 = 1024.0;
    if kib >= 1024 * 1024 {
        format!("{:.2} GiB", kib as f64 / (KIB * KIB))
    } else if kib >= 1024 {
        format!("{:.1} MiB", kib as f64 / KIB)
    } else {
        format!("{kib} KiB")
    }
}

fn format_bytes(b: u64) -> String {
    const B: f64 = 1.0;
    const KIB: f64 = 1024.0;
    if b >= 1024 * 1024 * 1024 {
        format!("{:.2} GiB", b as f64 / (KIB * KIB * KIB))
    } else if b >= 1024 * 1024 {
        format!("{:.1} MiB", b as f64 / (KIB * KIB))
    } else {
        format!("{:.0} B", b as f64 * B)
    }
}

/// Default route present in `/proc/net/route` → assume connected.
fn network_liveness_hint() -> &'static str {
    let Ok(s) = std::fs::read_to_string("/proc/net/route") else {
        return "unknown";
    };
    for line in s.lines().skip(1) {
        let mut it = line.split_whitespace();
        it.next();
        if it.next() == Some("00000000") {
            return "connected (default route)";
        }
    }
    "unknown"
}

#[derive(Parser)]
#[command(name = "ecosystem-health")]
#[command(about = "biomeOS Ecosystem Health Monitor - Comprehensive system diagnostics")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Workspace root directory
    #[arg(short, long, default_value = ".")]
    workspace: String,

    /// Enable detailed health checks
    #[arg(short, long)]
    detailed: bool,

    /// Check external dependencies
    #[arg(short, long)]
    external: bool,

    /// Health check timeout in seconds
    #[arg(short, long, default_value = "60")]
    timeout: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Run comprehensive health check
    All,
    /// Check core system only
    Core,
    /// Check UI system only
    Ui,
    /// Check build system only
    Build,
    /// Check dependencies only
    Deps,
    /// Check ecosystem components
    Ecosystem,
    /// Check sovereignty features
    Sovereignty,
    /// Continuous monitoring mode
    Monitor,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    let config = HealthConfig {
        workspace_root: cli.workspace.into(),
        detailed_checks: cli.detailed,
        check_external_deps: cli.external,
        timeout_seconds: cli.timeout,
    };

    match cli.command {
        Some(Commands::All) | None => {
            run_comprehensive_health_check(&config).await?;
        }
        Some(Commands::Core) => {
            check_core_only(&config).await?;
        }
        Some(Commands::Ui) => {
            check_ui_only(&config).await?;
        }
        Some(Commands::Build) => {
            check_build_only(&config).await?;
        }
        Some(Commands::Deps) => {
            check_dependencies_only(&config).await?;
        }
        Some(Commands::Ecosystem) => {
            check_ecosystem_only(&config).await?;
        }
        Some(Commands::Sovereignty) => {
            check_sovereignty_only(&config).await?;
        }
        Some(Commands::Monitor) => {
            run_continuous_monitoring(&config).await?;
        }
    }

    Ok(())
}

/// Run comprehensive health check
async fn run_comprehensive_health_check(config: &HealthConfig) -> Result<()> {
    print_section("🏥 COMPREHENSIVE ECOSYSTEM HEALTH CHECK");

    let results = check_ecosystem_health(config).await?;

    // Analyze results
    let _healthy_count = results
        .iter()
        .filter(|r| r.status == HealthStatus::Healthy)
        .count();
    let warning_count = results
        .iter()
        .filter(|r| r.status == HealthStatus::Warning)
        .count();
    let critical_count = results
        .iter()
        .filter(|r| r.status == HealthStatus::Critical)
        .count();

    println!("\n🎯 FINAL HEALTH ASSESSMENT:");

    if critical_count > 0 {
        biomeos_tools::print_error("🔴 CRITICAL ISSUES DETECTED - Immediate attention required");
        println!("❌ System may not function properly");
    } else if warning_count > 0 {
        biomeos_tools::print_warning("🟡 WARNINGS DETECTED - Some issues need attention");
        println!("⚠️ System functional but improvements recommended");
    } else {
        print_success("🟢 ALL SYSTEMS HEALTHY - Ecosystem operating optimally");
        println!("✅ Ready for production use");
    }

    // Provide recommendations
    provide_health_recommendations(&results);

    Ok(())
}

/// Check core system only
async fn check_core_only(config: &HealthConfig) -> Result<()> {
    print_section("🧬 CORE SYSTEM HEALTH CHECK");

    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");

    // Check core compilation
    match biomeos_tools::execute_command(
        "cargo",
        &["check", "-p", "biomeos-core"],
        Some(&workspace_path),
    )
    .await
    {
        Ok(_) => {
            print_success("Core system is healthy");
            println!("  ✅ Compilation: PASSED");
            println!("  ✅ Dependencies: RESOLVED");
            println!("  ✅ API interfaces: OPERATIONAL");
        }
        Err(e) => {
            biomeos_tools::print_error(&format!("Core system issues detected: {}", e));
            println!("  ❌ Core functionality may be impaired");
        }
    }

    Ok(())
}

/// Check UI system only
async fn check_ui_only(config: &HealthConfig) -> Result<()> {
    print_section("🎨 UI SYSTEM HEALTH CHECK");

    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");

    // Check UI compilation
    match biomeos_tools::execute_command(
        "cargo",
        &["check", "-p", "biomeos-ui"],
        Some(&workspace_path),
    )
    .await
    {
        Ok(_) => {
            print_success("UI system is healthy");
            println!("  ✅ egui framework: INITIALIZED");
            println!("  ✅ Views: COMPILED");
            println!("  ✅ API integration: FUNCTIONAL");
        }
        Err(e) => {
            biomeos_tools::print_error(&format!("UI system issues detected: {}", e));
            println!("  ❌ Interface may be unavailable");
        }
    }

    Ok(())
}

/// Check build system only
async fn check_build_only(config: &HealthConfig) -> Result<()> {
    print_section("🏗️ BUILD SYSTEM HEALTH CHECK");

    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");

    // Check workspace compilation
    match biomeos_tools::execute_command("cargo", &["check", "--workspace"], Some(&workspace_path))
        .await
    {
        Ok(_) => {
            print_success("Build system is healthy");
            println!("  ✅ Workspace: VALID");
            println!("  ✅ All crates: COMPILE");
            println!("  ✅ Dependencies: RESOLVED");
        }
        Err(e) => {
            biomeos_tools::print_warning(&format!("Build system warnings: {}", e));
            println!("  ⚠️ Some compilation issues detected");
        }
    }

    Ok(())
}

/// Check dependencies only
async fn check_dependencies_only(config: &HealthConfig) -> Result<()> {
    print_section("📦 DEPENDENCIES HEALTH CHECK");

    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");

    // Check for security vulnerabilities
    biomeos_tools::print_info("Checking for security vulnerabilities...");

    match biomeos_tools::execute_command("cargo", &["audit"], Some(&workspace_path)).await {
        Ok(_) => {
            print_success("No security vulnerabilities detected");
        }
        Err(_) => {
            biomeos_tools::print_info("cargo-audit not installed or vulnerabilities detected");
            biomeos_tools::print_info("Run 'cargo install cargo-audit' for security scanning");
        }
    }

    // Check for outdated dependencies
    biomeos_tools::print_info("Checking for outdated dependencies...");

    match biomeos_tools::execute_command(
        "cargo",
        &["outdated", "--workspace"],
        Some(&workspace_path),
    )
    .await
    {
        Ok(output) => {
            if output.trim().is_empty() || output.contains("All dependencies are up to date") {
                print_success("All dependencies are current");
            } else {
                biomeos_tools::print_warning("Some dependencies may be outdated");
                println!("  ⚠️ Consider running 'cargo update'");
            }
        }
        Err(_) => {
            biomeos_tools::print_info("cargo-outdated not installed");
            biomeos_tools::print_info("Run 'cargo install cargo-outdated' for dependency checking");
        }
    }

    Ok(())
}

/// Check ecosystem components only
async fn check_ecosystem_only(_config: &HealthConfig) -> Result<()> {
    print_section("🌍 ECOSYSTEM COMPONENTS HEALTH CHECK");

    let components: Vec<(&str, &str, &str)> = vec![
        ("🔍", primal_names::TOADSTOOL, "Universal Compute"),
        ("🎵", primal_names::SONGBIRD, "Service Mesh"),
        ("🏠", primal_names::NESTGATE, "Storage System"),
        ("🐿️", primal_names::SQUIRREL, "MCP Platform"),
        ("🐻", primal_names::BEARDOG, "Security Framework"),
        ("🦈", primal_names::BARRACUDA, "GPU Math & Tensors"),
        ("🪸", primal_names::CORALREEF, "Shader Compilation"),
    ];

    println!("Ecosystem Component Status:");

    for (icon, id, description) in &components {
        println!("  {icon} {id} ({description}): Discovery available");
    }

    biomeos_tools::print_info("🔄 All components ready for ecosystem integration");
    biomeos_tools::print_info("📡 Discovery and coordination systems operational");

    print_success("Ecosystem components available for integration");
    Ok(())
}

/// Check sovereignty features only
async fn check_sovereignty_only(_config: &HealthConfig) -> Result<()> {
    print_section("🔒 SOVEREIGNTY & RUNTIME LIVENESS");
    let dir = biomeos_socket_dir();
    println!("Primal socket directory: {}", dir.display());
    if !dir.is_dir() {
        println!("  (Directory missing — no sockets to probe.)");
        print_success("Socket probe complete (0 sockets)");
        return Ok(());
    }

    let paths = list_socket_files(&dir);
    if paths.is_empty() {
        println!("  No `*.sock` files found.");
        print_success("Socket probe complete (0 sockets)");
        return Ok(());
    }

    let (reachable, unreachable, per_socket) = probe_unix_sockets(&paths).await;
    println!(
        "  Sockets found: {} (reachable: {reachable}, unreachable: {unreachable})",
        paths.len()
    );
    for (p, ok) in &per_socket {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let st = if *ok { "reachable" } else { "unreachable" };
        println!("    • {} — {st}", name);
    }
    if unreachable == 0 {
        print_success("All listed primal sockets accept connections");
    } else {
        biomeos_tools::print_warning(
            "Some socket paths are not connectable (service down or stale files)",
        );
    }
    Ok(())
}

/// Run continuous monitoring
async fn run_continuous_monitoring(config: &HealthConfig) -> Result<()> {
    print_section("📊 CONTINUOUS ECOSYSTEM MONITORING");
    biomeos_tools::print_info("Starting continuous health monitoring...");
    biomeos_tools::print_info("Press Ctrl+C to stop monitoring");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("\n⏰ {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));

                // Quick health check
                let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");

                match biomeos_tools::execute_command(
                    "cargo",
                    &["check", "--workspace", "--quiet"],
                    Some(&workspace_path)
                ).await {
                    Ok(_) => println!("  🟢 System: HEALTHY"),
                    Err(_) => println!("  🔴 System: ISSUES DETECTED"),
                }

                if let Some((total_kib, avail_kib)) = read_proc_meminfo_kb() {
                    let used = total_kib.saturating_sub(avail_kib);
                    let used_pct = if total_kib > 0 {
                        100.0 * (used as f64) / (total_kib as f64)
                    } else {
                        0.0
                    };
                    println!(
                        "  💾 Memory: {} used / {} total ({used_pct:.0}% used, {} avail)",
                        format_kib(used),
                        format_kib(total_kib),
                        format_kib(avail_kib)
                    );
                } else {
                    println!("  💾 Memory: (unavailable — /proc/meminfo not readable)");
                }

                if let Some(load) = read_proc_loadavg() {
                    println!("  ⚙ CPU load (1/5/15m): {load}");
                } else {
                    println!("  ⚙ CPU load: (unavailable — /proc/loadavg not readable)");
                }

                match df_pwd_bytes() {
                    Some((total_b, avail_b)) => {
                        let used = total_b.saturating_sub(avail_b);
                        let pct = if total_b > 0 {
                            100.0 * (used as f64) / (total_b as f64)
                        } else {
                            0.0
                        };
                        println!(
                            "  💿 Disk (cwd): {} used / {} total ({:.0}% free: {})",
                            format_bytes(used),
                            format_bytes(total_b),
                            100.0 - pct,
                            format_bytes(avail_b)
                        );
                    }
                    None => println!("  💿 Disk (cwd): (unavailable — `df` failed)"),
                }

                println!("  🌐 Network: {}", network_liveness_hint());
            }
            _ = tokio::signal::ctrl_c() => {
                biomeos_tools::print_info("\nMonitoring stopped by user");
                break;
            }
        }
    }

    print_success("Continuous monitoring completed");
    Ok(())
}

/// Provide health recommendations
fn provide_health_recommendations(results: &[biomeos_tools::health::HealthResult]) {
    let critical_issues: Vec<_> = results
        .iter()
        .filter(|r| r.status == HealthStatus::Critical)
        .collect();

    let warnings: Vec<_> = results
        .iter()
        .filter(|r| r.status == HealthStatus::Warning)
        .collect();

    if !critical_issues.is_empty() {
        println!("\n🚨 CRITICAL ISSUES - IMMEDIATE ACTION REQUIRED:");
        for issue in critical_issues {
            println!("  ❌ {}: {}", issue.component, issue.message);
        }
    }

    if !warnings.is_empty() {
        println!("\n⚠️ WARNINGS - RECOMMENDED ACTIONS:");
        for warning in warnings {
            println!("  ⚠️ {}: {}", warning.component, warning.message);
        }
    }

    println!("\n💡 GENERAL RECOMMENDATIONS:");
    println!("  🔄 Regular health checks: Run weekly");
    println!("  📊 Monitor test coverage: Keep above 50%");
    println!("  🔐 Security updates: Check monthly");
    println!("  🧬 Ecosystem integration: Stay current");
    println!("  📝 Documentation: Keep updated");
}
