//! Device Enrollment Mode
//!
//! Enrolls a new device into the family by deriving a unique lineage seed.
//!
//! ## Usage
//!
//! ```bash
//! biomeos enroll --family-id 1894e909e454 --node-id tower --device-id $(cat /etc/machine-id)
//! ```
//!
//! ## What It Does
//!
//! 1. Reads the shared `.family.seed` file
//! 2. Calls BearDog to derive a UNIQUE seed for this device
//! 3. Saves the derived seed to `.lineage.seed`
//! 4. Saves metadata to `.lineage.json`
//!
//! After enrollment, the device has its own unique seed that can prove
//! ancestry to the family root, but cannot impersonate other devices.
//!
//! AGPL-3.0-only License

use anyhow::{Context, Result};
use biomeos_spore::beacon_genetics::{
    generate_device_entropy, DirectBeardogCaller, LineageDeriver,
};
use biomeos_types::Uuid;
use clap::Args;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Device enrollment arguments
#[derive(Args, Debug)]
pub struct EnrollArgs {
    /// Family ID (e.g., "1894e909e454")
    #[arg(long, env = "FAMILY_ID")]
    pub family_id: String,

    /// Node ID - human-friendly name (e.g., "tower", "pixel8a")
    #[arg(long, env = "NODE_ID")]
    pub node_id: String,

    /// Device ID - unique hardware identifier
    /// Defaults to /etc/machine-id content or generated UUID
    #[arg(long, env = "DEVICE_ID")]
    pub device_id: Option<String>,

    /// Path to family seed file
    #[arg(long, default_value = ".family.seed")]
    pub family_seed: PathBuf,

    /// Path to output lineage seed file
    #[arg(long, default_value = ".lineage.seed")]
    pub lineage_seed: PathBuf,

    /// BearDog socket path (for derivation)
    #[arg(long, env = "BEARDOG_SOCKET")]
    pub beardog_socket: Option<String>,

    /// Force re-enrollment even if lineage already exists
    #[arg(long)]
    pub force: bool,
}

/// Run device enrollment
pub async fn run(args: EnrollArgs) -> Result<()> {
    info!("🧬 Device Enrollment for biomeOS");
    info!("   Family: {}", args.family_id);
    info!("   Node: {}", args.node_id);

    // Check if already enrolled
    if args.lineage_seed.exists() && !args.force {
        warn!(
            "⚠️  Device already enrolled (lineage exists at {})",
            args.lineage_seed.display()
        );
        warn!("   Use --force to re-enroll");

        // Load and display existing lineage
        if let Ok(lineage) = LineageDeriver::<DirectBeardogCaller>::load_lineage(&args.lineage_seed)
        {
            info!("   Existing enrollment:");
            info!("      Device ID: {}", lineage.device_id);
            info!("      Node ID: {}", lineage.node_id);
            info!("      Family: {}", lineage.family_id);
            info!("      Generation: {}", lineage.generation);
        }

        return Ok(());
    }

    // Get device ID
    let device_id = args
        .device_id
        .clone()
        .or_else(get_machine_id)
        .unwrap_or_else(|| {
            let id = Uuid::new_v4().to_string();
            warn!("No device ID provided, generated: {}", id);
            id
        });

    info!("   Device ID: {}", device_id);

    // Check family seed exists
    if !args.family_seed.exists() {
        error!("❌ Family seed not found at {}", args.family_seed.display());
        error!("   Please ensure the family seed file exists before enrollment.");
        return Err(anyhow::anyhow!(
            "Family seed not found: {}",
            args.family_seed.display()
        ));
    }

    // Discover BearDog socket
    let beardog_socket = args
        .beardog_socket
        .or_else(discover_beardog_socket)
        .context("Could not find BearDog socket. Is BearDog running?")?;

    info!("   BearDog: {}", beardog_socket);

    // Create capability caller (direct to BearDog, not through Neural API)
    let caller = DirectBeardogCaller::new(&beardog_socket);
    let deriver = LineageDeriver::new(caller);

    // Enroll the device
    info!("📝 Deriving unique device seed...");

    let result = deriver
        .enroll_device(
            &args.family_seed,
            &args.lineage_seed,
            &args.family_id,
            &device_id,
            &args.node_id,
        )
        .await
        .context("Device enrollment failed")?;

    info!("✅ Device enrolled successfully!");
    info!("   Lineage seed: {}", result.seed_path.display());
    info!("   Method: {}", result.lineage.derivation_method);
    info!("   Generation: {}", result.lineage.generation);

    // Generate additional device entropy and show it
    let entropy = generate_device_entropy();
    info!("   Device entropy: {} bytes generated", entropy.len());

    Ok(())
}

/// Get machine ID from /etc/machine-id (Linux)
fn get_machine_id() -> Option<String> {
    std::fs::read_to_string("/etc/machine-id")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Discover BearDog socket path
fn discover_beardog_socket() -> Option<String> {
    // Try XDG runtime dir first
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let xdg_path = format!("{}/biomeos/beardog.sock", runtime_dir);
        if std::path::Path::new(&xdg_path).exists() {
            return Some(xdg_path);
        }

        // Try with family ID
        if let Ok(family_id) = std::env::var("FAMILY_ID") {
            let family_path = format!("{}/biomeos/beardog-{}.sock", runtime_dir, family_id);
            if std::path::Path::new(&family_path).exists() {
                return Some(family_path);
            }
        }
    }

    // Try XDG-compliant paths (no hardcoded UID)
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let xdg_socket = paths.primal_socket("beardog");
    if xdg_socket.exists() {
        return Some(xdg_socket.to_string_lossy().to_string());
    }

    // Also check family-suffixed variant
    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "family".to_string());
    let family_socket = paths.primal_socket(&format!("beardog-{}", family_id));
    if family_socket.exists() {
        return Some(family_socket.to_string_lossy().to_string());
    }

    // Legacy /tmp fallback
    for name in &["beardog.sock", &format!("beardog-{}.sock", family_id)] {
        let path = format!("/tmp/{}", name);
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_id() {
        // This test may or may not find a machine-id depending on platform
        let _ = get_machine_id();
    }

    #[test]
    fn test_discover_beardog_socket_handles_missing() {
        // Should return None when no socket exists
        std::env::remove_var("XDG_RUNTIME_DIR");
        // Note: This might still find a socket if one exists on the system
        let _ = discover_beardog_socket();
    }
}
