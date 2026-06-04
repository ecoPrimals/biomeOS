// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
//! 2. Calls the security provider (discovered by capability) to derive a UNIQUE seed
//! 3. Saves the derived seed to `.lineage.seed`
//! 4. Saves metadata to `.lineage.json`
//!
//! After enrollment, the device has its own unique seed that can prove
//! ancestry to the family root, but cannot impersonate other devices.
//!
//! AGPL-3.0-or-later License

use anyhow::{Context, Result};
use biomeos_spore::beacon_genetics::{
    DirectBeardogCaller, LineageDeriver, NeuralApiCapabilityCaller, generate_device_entropy,
    load_lineage,
};
use biomeos_types::{CapabilityTaxonomy, Uuid, primal_names};
use clap::Args;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Device enrollment arguments
#[derive(Args, Debug)]
pub struct EnrollArgs {
    /// Family ID (e.g., "1894e909e454")
    #[arg(long, env = "FAMILY_ID")]
    pub family_id: String,

    /// Node ID - human-friendly name (e.g., "tower", "pixel8a")
    #[arg(long, env = biomeos_types::env_config::vars::NODE_ID_LEGACY)]
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

    /// Security provider socket path (for seed derivation)
    #[arg(long = "security-provider-socket", env = "SECURITY_PROVIDER_SOCKET")]
    pub security_socket: Option<String>,

    #[arg(skip)]
    pub security_socket_dir: Option<PathBuf>,

    /// Force re-enrollment even if lineage already exists
    #[arg(long)]
    pub force: bool,
}

/// Resolve device ID from explicit value or machine-id (pure where possible).
/// Falls back to UUID generation if no device_id provided and machine-id unavailable.
pub(crate) fn resolve_device_id(device_id: Option<&str>) -> String {
    device_id
        .map(std::string::ToString::to_string)
        .filter(|s| !s.is_empty())
        .or_else(get_machine_id)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// Validation error for enrollment paths
#[derive(Debug, thiserror::Error)]
pub(crate) enum EnrollmentValidationError {
    #[error("Device already enrolled. Use --force to re-enroll")]
    AlreadyEnrolled,
    #[error("Family seed not found")]
    FamilySeedNotFound,
}

/// Validate enrollment paths before enrollment.
/// Returns Err if family seed is missing, or if lineage exists and force is false.
pub(crate) fn validate_enrollment_paths(
    lineage_path: &Path,
    family_seed_path: &Path,
    force: bool,
) -> std::result::Result<(), EnrollmentValidationError> {
    if lineage_path.exists() && !force {
        return Err(EnrollmentValidationError::AlreadyEnrolled);
    }
    if !family_seed_path.exists() {
        return Err(EnrollmentValidationError::FamilySeedNotFound);
    }
    Ok(())
}

/// Run device enrollment
pub async fn run(args: EnrollArgs) -> Result<()> {
    info!("🧬 Device Enrollment for biomeOS");
    info!("   Family: {}", args.family_id);
    info!("   Node: {}", args.node_id);

    if matches!(
        validate_enrollment_paths(&args.lineage_seed, &args.family_seed, args.force),
        Err(EnrollmentValidationError::AlreadyEnrolled)
    ) {
        warn!(
            "⚠️  Device already enrolled (lineage exists at {})",
            args.lineage_seed.display()
        );
        warn!("   Use --force to re-enroll");
        if let Ok(lineage) = load_lineage(&args.lineage_seed) {
            info!("   Existing enrollment:");
            info!("      Device ID: {}", lineage.device_id);
            info!("      Node ID: {}", lineage.node_id);
            info!("      Family: {}", lineage.family_id);
            info!("      Generation: {}", lineage.generation);
        }
        return Ok(());
    }
    validate_enrollment_paths(&args.lineage_seed, &args.family_seed, args.force)?;

    let device_id = resolve_device_id(args.device_id.as_deref());
    if args.device_id.is_none() && get_machine_id().is_none() {
        warn!("No device ID provided, generated: {}", device_id);
    }
    info!("   Device ID: {}", device_id);

    validate_enrollment_paths(&args.lineage_seed, &args.family_seed, args.force)?;

    // Enroll the device
    info!("📝 Deriving unique device seed...");

    // Prefer Neural API routing; fall back to direct security provider for bootstrap
    let neural_socket = NeuralApiCapabilityCaller::default_socket();
    let result = if Path::new(&neural_socket).exists() {
        info!("   Neural API: {} (capability routing)", neural_socket);
        let caller = NeuralApiCapabilityCaller::new(&neural_socket);
        LineageDeriver::new(caller)
            .enroll_device(
                &args.family_seed,
                &args.lineage_seed,
                &args.family_id,
                &device_id,
                &args.node_id,
            )
            .await
            .context("Device enrollment failed (via Neural API)")?
    } else {
        let security_socket = args
            .security_socket
            .or_else(|| {
                args.security_socket_dir
                    .as_ref()
                    .map_or_else(discover_security_socket, |dir| {
                        discover_security_socket_in(Some(dir.as_path()), Some(&args.family_id))
                    })
            })
            .context(
                "Could not find security provider socket (Neural API not available). Is the security provider running?",
            )?;
        info!(
            "   Security provider (direct, bootstrap): {}",
            security_socket
        );
        let caller = DirectBeardogCaller::new(&security_socket);
        LineageDeriver::new(caller)
            .enroll_device(
                &args.family_seed,
                &args.lineage_seed,
                &args.family_id,
                &device_id,
                &args.node_id,
            )
            .await
            .context("Device enrollment failed (direct security provider)")?
    };

    info!("✅ Device enrolled successfully!");
    info!("   Lineage seed: {}", result.seed_path.display());
    info!("   Method: {}", result.lineage.derivation_method);
    info!("   Generation: {}", result.lineage.generation);

    // Generate additional device entropy and show it
    let entropy = generate_device_entropy().context("Failed to generate device entropy")?;
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

/// Bootstrap security provider discovery with an explicit socket directory.
///
/// Used when the CLI provides `--security-provider-socket-dir`, narrowing the
/// search to a single directory rather than the full 5-tier protocol.
/// Resolves the primal name from capability taxonomy instead of hardcoding.
pub(crate) fn discover_security_socket_in(
    socket_dir: Option<&Path>,
    family_id: Option<&str>,
) -> Option<String> {
    let primal =
        CapabilityTaxonomy::resolve_to_primal("encryption").unwrap_or(primal_names::BEARDOG);

    if let Some(runtime_dir) = socket_dir {
        let biomeos_dir = runtime_dir.join(biomeos_types::constants::runtime_paths::BIOMEOS_SUBDIR);
        let xdg_path = biomeos_dir.join(format!("{primal}.sock"));
        if xdg_path.exists() {
            return Some(xdg_path.to_string_lossy().to_string());
        }
        if let Some(fid) = family_id {
            let family_path = biomeos_dir.join(format!("{primal}-{fid}.sock"));
            if family_path.exists() {
                return Some(family_path.to_string_lossy().to_string());
            }
        }
    }

    None
}

/// Bootstrap security provider discovery — delegates to the 5-tier capability protocol.
fn discover_security_socket() -> Option<String> {
    use biomeos_types::capability_discovery;

    capability_discovery::discover_capability_socket("encryption", &capability_discovery::std_env)
}

#[cfg(test)]
#[path = "enroll_tests.rs"]
mod tests;
