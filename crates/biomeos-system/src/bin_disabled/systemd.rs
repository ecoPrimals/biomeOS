//! # biomeOS Systemd Integration
//!
//! Systemd service integration for biomeOS.
//! Allows biomeOS to run as a systemd service on Linux systems.

use biomeos_system::{BiomeOSSystem, SystemConfig};
use clap::{Arg, Command};
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for systemd
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Parse command line arguments
    let matches = Command::new("biomeos-systemd")
        .version(env!("CARGO_PKG_VERSION"))
        .about("biomeOS systemd service integration")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("/etc/biomeos/system.yaml")
        )
        .arg(
            Arg::new("install")
                .long("install")
                .action(clap::ArgAction::SetTrue)
                .help("Install systemd service files")
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .action(clap::ArgAction::SetTrue)
                .help("Uninstall systemd service files")
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let install = matches.get_flag("install");
    let uninstall = matches.get_flag("uninstall");

    if install {
        install_systemd_service().await?;
        return Ok(());
    }

    if uninstall {
        uninstall_systemd_service().await?;
        return Ok(());
    }

    // Normal service operation
    tracing::info!("Starting biomeOS systemd service");

    // Load configuration
    let config = load_config_from_file(config_path).await?;

    // Create biomeOS system
    let system = Arc::new(BiomeOSSystem::new(config)?);

    // Set up signal handlers for systemd
    let system_clone = system.clone();
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM, shutting down gracefully");
            }
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT, shutting down gracefully");
            }
        }

        if let Err(e) = system_clone.shutdown().await {
            tracing::error!("Error during shutdown: {}", e);
        }

        std::process::exit(0);
    });

    // Initialize the system
    tracing::info!("Initializing biomeOS system");
    system.initialize().await?;

    // Start the system
    tracing::info!("Starting biomeOS system");
    system.start().await?;

    // Notify systemd that we're ready
    notify_systemd_ready()?;

    // Main service loop
    loop {
        // Monitor system health and report to systemd
        let state = system.get_state().await;
        
        match state.health.overall_status {
            biomeos_system::HealthStatus::Critical => {
                tracing::error!("System health is critical");
                notify_systemd_status("STATUS=System health critical")?;
            }
            biomeos_system::HealthStatus::Warning => {
                tracing::warn!("System health is degraded");
                notify_systemd_status("STATUS=System health degraded")?;
            }
            biomeos_system::HealthStatus::Healthy => {
                tracing::debug!("System health is good");
                notify_systemd_status("STATUS=System healthy")?;
            }
            _ => {
                notify_systemd_status("STATUS=System status unknown")?;
            }
        }

        // Send watchdog heartbeat to systemd
        notify_systemd_watchdog()?;

        // Sleep before next check
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}

/// Install systemd service files
async fn install_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Installing biomeOS systemd service files...");

    let service_content = include_str!("../../resources/biomeos.service");
    let service_path = "/etc/systemd/system/biomeos.service";

    // Write service file
    tokio::fs::write(service_path, service_content).await?;
    println!("✅ Installed service file: {}", service_path);

    // Reload systemd
    let output = tokio::process::Command::new("systemctl")
        .arg("daemon-reload")
        .output()
        .await?;

    if !output.status.success() {
        eprintln!("❌ Failed to reload systemd daemon");
        return Err("systemctl daemon-reload failed".into());
    }

    println!("✅ Reloaded systemd daemon");

    // Enable service
    let output = tokio::process::Command::new("systemctl")
        .arg("enable")
        .arg("biomeos")
        .output()
        .await?;

    if !output.status.success() {
        eprintln!("❌ Failed to enable biomeOS service");
        return Err("systemctl enable failed".into());
    }

    println!("✅ Enabled biomeOS service");
    println!();
    println!("🎯 biomeOS systemd service installed successfully!");
    println!("   Start with: sudo systemctl start biomeos");
    println!("   Check status: sudo systemctl status biomeos");
    println!("   View logs: sudo journalctl -u biomeos -f");

    Ok(())
}

/// Uninstall systemd service files
async fn uninstall_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗑️  Uninstalling biomeOS systemd service files...");

    // Stop service if running
    let _ = tokio::process::Command::new("systemctl")
        .arg("stop")
        .arg("biomeos")
        .output()
        .await;

    // Disable service
    let _ = tokio::process::Command::new("systemctl")
        .arg("disable")
        .arg("biomeos")
        .output()
        .await;

    // Remove service file
    let service_path = "/etc/systemd/system/biomeos.service";
    if tokio::fs::metadata(service_path).await.is_ok() {
        tokio::fs::remove_file(service_path).await?;
        println!("✅ Removed service file: {}", service_path);
    }

    // Reload systemd
    let output = tokio::process::Command::new("systemctl")
        .arg("daemon-reload")
        .output()
        .await?;

    if output.status.success() {
        println!("✅ Reloaded systemd daemon");
    }

    println!("✅ biomeOS systemd service uninstalled successfully!");

    Ok(())
}

/// Notify systemd that service is ready
fn notify_systemd_ready() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(socket) = std::env::var("NOTIFY_SOCKET") {
        notify_systemd(&socket, "READY=1")?;
        tracing::info!("Notified systemd that service is ready");
    }
    Ok(())
}

/// Notify systemd of status
fn notify_systemd_status(status: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(socket) = std::env::var("NOTIFY_SOCKET") {
        notify_systemd(&socket, status)?;
    }
    Ok(())
}

/// Send watchdog heartbeat to systemd
fn notify_systemd_watchdog() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(socket) = std::env::var("NOTIFY_SOCKET") {
        notify_systemd(&socket, "WATCHDOG=1")?;
    }
    Ok(())
}

/// Send notification to systemd
fn notify_systemd(socket_path: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::net::UnixDatagram;

    let socket = UnixDatagram::unbound()?;
    socket.send_to(message.as_bytes(), socket_path)?;
    Ok(())
}

/// Load configuration from file
async fn load_config_from_file(path: &str) -> Result<SystemConfig, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string(path).await?;
    let config: SystemConfig = serde_yaml::from_str(&content)?;
    Ok(config)
} 