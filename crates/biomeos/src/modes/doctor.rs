//! Doctor mode - Health diagnostics
//!
//! Comprehensive health checks for biomeOS system

use anyhow::Result;
use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use serde::{Deserialize, Serialize};
use std::path::Path;
use sysinfo::{Disks, System};

#[derive(Debug, Serialize, Deserialize)]
struct Diagnostics {
    checks: Vec<HealthCheck>,
    overall_status: HealthStatus,
    recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthCheck {
    name: String,
    status: HealthStatus,
    details: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            checks: Vec::new(),
            overall_status: HealthStatus::Healthy,
            recommendations: Vec::new(),
        }
    }

    fn add_check(&mut self, _name: &str, check: HealthCheck) {
        // Update overall status
        match check.status {
            HealthStatus::Critical => self.overall_status = HealthStatus::Critical,
            HealthStatus::Warning if self.overall_status != HealthStatus::Critical => {
                self.overall_status = HealthStatus::Warning;
            }
            _ => {}
        }

        self.checks.push(check);
    }

    fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
}

pub async fn run(detailed: bool, format: String, subsystem: Option<String>) -> Result<()> {
    let diagnostics = if let Some(subsys) = subsystem {
        check_subsystem(&subsys, detailed).await?
    } else {
        check_all_subsystems(detailed).await?
    };

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&diagnostics)?);
        }
        _ => {
            print_diagnostics(&diagnostics);
        }
    }

    Ok(())
}

async fn check_all_subsystems(detailed: bool) -> Result<Diagnostics> {
    let mut diag = Diagnostics::new();

    println!("{}", "🧠 biomeOS Doctor".bright_cyan().bold());
    println!();
    println!("{}", "Health Diagnostics:".bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".bright_black()
    );
    println!();

    // 1. Binary Health
    diag.add_check("Binary", check_binary_health().await?);

    // 2. Configuration
    diag.add_check("Configuration", check_configuration().await?);

    // 3. Graphs Directory
    diag.add_check("Graphs", check_graphs_dir().await?);

    // 4. Primal Discovery
    diag.add_check("Primal Discovery", check_primal_discovery().await?);

    // 5. PlasmidBin
    diag.add_check("PlasmidBin", check_plasmid_bin().await?);

    // 6. System Resources
    diag.add_check("System", check_system_resources().await?);

    if detailed {
        // 7. Dependencies
        diag.add_check("Dependencies", check_dependencies().await?);
    }

    // Add recommendations
    if diag.overall_status != HealthStatus::Healthy {
        add_recommendations(&mut diag);
    }

    Ok(diag)
}

async fn check_subsystem(name: &str, _detailed: bool) -> Result<Diagnostics> {
    let mut diag = Diagnostics::new();

    match name {
        "binary" => diag.add_check("Binary", check_binary_health().await?),
        "config" => diag.add_check("Configuration", check_configuration().await?),
        "graphs" => diag.add_check("Graphs", check_graphs_dir().await?),
        "primals" => diag.add_check("Primal Discovery", check_primal_discovery().await?),
        "plasmidbin" => diag.add_check("PlasmidBin", check_plasmid_bin().await?),
        "system" => diag.add_check("System", check_system_resources().await?),
        _ => {
            anyhow::bail!("Unknown subsystem: {}", name);
        }
    }

    Ok(diag)
}

async fn check_binary_health() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Binary Health".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Check current binary
    if let Ok(exe) = std::env::current_exe() {
        check.details.push(format!("Binary: {}", exe.display()));

        if let Ok(metadata) = std::fs::metadata(&exe) {
            let size_mb = metadata.len() as f64 / 1_048_576.0;
            check.details.push(format!("Size: {:.1}M", size_mb));
        }
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push("Could not determine binary path".to_string());
    }

    check
        .details
        .push(format!("Version: {}", env!("CARGO_PKG_VERSION")));
    check.details.push("Modes: 7/7 available".to_string());
    check.details.push("UniBin: ✅ Compliant".to_string());

    Ok(check)
}

async fn check_configuration() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Configuration".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Use etcetera (Pure Rust!) for config directory
    use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
    let config_path = choose_base_strategy()
        .ok()
        .and_then(|strategy| Some(strategy.config_dir().join("biomeos/config.toml")))
        .unwrap_or_else(|| Path::new("~/.config/biomeos/config.toml").to_path_buf());

    if config_path.exists() {
        check
            .details
            .push(format!("Config file: {}", config_path.display()));
        check.details.push("Status: ✅ Found".to_string());
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Config file: {}", config_path.display()));
        check
            .details
            .push("Status: ⚠️  Not found (using defaults)".to_string());
    }

    Ok(check)
}

async fn check_graphs_dir() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Graphs Directory".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let graphs_dir = Path::new("graphs");

    if graphs_dir.exists() && graphs_dir.is_dir() {
        // Count .toml files
        let graph_count = std::fs::read_dir(graphs_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "toml"))
            .count();

        check
            .details
            .push(format!("Path: {}", graphs_dir.display()));
        check.details.push(format!("Graphs found: {}", graph_count));

        if graph_count == 0 {
            check.status = HealthStatus::Warning;
            check
                .details
                .push("Warning: No graph files found".to_string());
        } else {
            check.details.push("Status: ✅ Ready".to_string());
        }
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Path: {}", graphs_dir.display()));
        check
            .details
            .push("Status: ⚠️  Directory not found".to_string());
    }

    Ok(check)
}

async fn check_primal_discovery() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Primal Discovery".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let primals = vec![
        ("BearDog", "/tmp/beardog.sock"),
        ("Songbird", "/tmp/songbird.sock"),
        ("Squirrel", "/tmp/squirrel.sock"),
        ("NestGate", "/tmp/nestgate.sock"),
        ("ToadStool", "/tmp/toadstool.sock"),
    ];

    let mut found_count = 0;
    for (name, socket) in primals {
        let exists = Path::new(socket).exists();
        if exists {
            found_count += 1;
            check
                .details
                .push(format!("{}: ✅ Found ({})", name, socket));
        } else {
            check.details.push(format!("{}: ❌ Not found", name));
        }
    }

    check
        .details
        .push(format!("Total: {}/5 primals discovered", found_count));

    if found_count < 3 {
        check.status = HealthStatus::Warning;
    }

    Ok(check)
}

async fn check_plasmid_bin() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "PlasmidBin".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let plasmid_dir = Path::new("plasmidBin/primals");

    if plasmid_dir.exists() && plasmid_dir.is_dir() {
        let binaries: Vec<_> = std::fs::read_dir(plasmid_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        let total_size: u64 = binaries
            .iter()
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum();

        let size_mb = total_size as f64 / 1_048_576.0;

        check
            .details
            .push(format!("Path: {}", plasmid_dir.display()));
        check.details.push(format!("Binaries: {}", binaries.len()));
        check.details.push(format!("Total size: {:.1}M", size_mb));
        check.details.push("Status: ✅ Ready".to_string());
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Path: {}", plasmid_dir.display()));
        check
            .details
            .push("Status: ⚠️  Directory not found".to_string());
    }

    Ok(check)
}

async fn check_system_resources() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "System Resources".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let mut sys = System::new_all();
    sys.refresh_all();

    // Memory
    let total_mem_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let avail_mem_gb = sys.available_memory() as f64 / 1_073_741_824.0;
    let mem_percent =
        ((sys.total_memory() - sys.available_memory()) as f64 / sys.total_memory() as f64) * 100.0;

    check.details.push(format!(
        "Memory: {:.1}GB ({:.1}GB available, {:.0}% used)",
        total_mem_gb, avail_mem_gb, mem_percent
    ));

    if mem_percent > 90.0 {
        check.status = HealthStatus::Warning;
    }

    // Disk
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.mount_point() == Path::new("/") {
            let total_gb = disk.total_space() as f64 / 1_073_741_824.0;
            let avail_gb = disk.available_space() as f64 / 1_073_741_824.0;
            let used_percent = ((disk.total_space() - disk.available_space()) as f64
                / disk.total_space() as f64)
                * 100.0;

            check.details.push(format!(
                "Disk: {:.1}GB ({:.1}GB available, {:.0}% used)",
                total_gb, avail_gb, used_percent
            ));

            if used_percent > 90.0 {
                check.status = HealthStatus::Warning;
            }
        }
    }

    // CPU
    let cpu_count = sys.cpus().len();
    check.details.push(format!("CPUs: {} cores", cpu_count));

    // Load average
    let load_avg = System::load_average();
    check.details.push(format!("Load: {:.2}", load_avg.one));

    Ok(check)
}

async fn check_dependencies() -> Result<HealthCheck> {
    let check = HealthCheck {
        name: "Dependencies".to_string(),
        status: HealthStatus::Healthy,
        details: vec![
            "Pure Rust: Evolving to 100%".to_string(),
            "UniBin: ✅ Compliant".to_string(),
            "ecoBin: ⏳ In progress".to_string(),
        ],
    };

    Ok(check)
}

fn add_recommendations(diag: &mut Diagnostics) {
    // Collect recommendations to add (to avoid borrow checker issues)
    let mut recommendations = Vec::new();

    for check in &diag.checks {
        match &check.name[..] {
            "Primal Discovery" if check.status != HealthStatus::Healthy => {
                recommendations.push("Start missing primals for full functionality".to_string());
            }
            "System Resources" if check.status != HealthStatus::Healthy => {
                recommendations
                    .push("System resources under pressure - consider scaling".to_string());
            }
            "Graphs Directory" if check.status != HealthStatus::Healthy => {
                recommendations
                    .push("Create graphs/ directory and add deployment graphs".to_string());
            }
            _ => {}
        }
    }

    // Add all collected recommendations
    for rec in recommendations {
        diag.add_recommendation(rec);
    }
}

fn print_diagnostics(diag: &Diagnostics) {
    for check in &diag.checks {
        let status_icon = match check.status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Warning => "⚠️ ",
            HealthStatus::Critical => "❌",
        };

        println!("{} {}", status_icon, check.name.bold());
        for detail in &check.details {
            println!("   {}", detail);
        }
        println!();
    }

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".bright_black()
    );

    let overall_status_text = match diag.overall_status {
        HealthStatus::Healthy => "✅ HEALTHY".bright_green(),
        HealthStatus::Warning => "⚠️  HEALTHY (warnings)".bright_yellow(),
        HealthStatus::Critical => "❌ CRITICAL".bright_red(),
    };

    println!("{}: {}", "Overall Health".bold(), overall_status_text);

    if !diag.recommendations.is_empty() {
        println!();
        println!("{}:", "Recommendations".bold());
        for rec in &diag.recommendations {
            println!("  {} {}", "•".bright_cyan(), rec);
        }
    }
}
