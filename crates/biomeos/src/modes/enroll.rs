// SPDX-License-Identifier: AGPL-3.0-only
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
    DirectBeardogCaller, LineageDeriver, generate_device_entropy,
};
use biomeos_types::Uuid;
use biomeos_types::primal_names::BEARDOG;
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

    #[arg(skip)]
    pub beardog_socket_dir: Option<PathBuf>,

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
#[derive(Debug)]
pub(crate) enum EnrollmentValidationError {
    AlreadyEnrolled,
    FamilySeedNotFound,
}

impl std::fmt::Display for EnrollmentValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyEnrolled => write!(f, "Device already enrolled. Use --force to re-enroll"),
            Self::FamilySeedNotFound => write!(f, "Family seed not found"),
        }
    }
}

impl std::error::Error for EnrollmentValidationError {}

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
    validate_enrollment_paths(&args.lineage_seed, &args.family_seed, args.force)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let device_id = resolve_device_id(args.device_id.as_deref());
    if args.device_id.is_none() && get_machine_id().is_none() {
        warn!("No device ID provided, generated: {}", device_id);
    }
    info!("   Device ID: {}", device_id);

    validate_enrollment_paths(&args.lineage_seed, &args.family_seed, args.force)?;

    let beardog_socket = args
        .beardog_socket
        .or_else(|| {
            args.beardog_socket_dir
                .as_ref()
                .map_or_else(discover_beardog_socket, |dir| {
                    discover_beardog_socket_in(Some(dir.as_path()), Some(&args.family_id))
                })
        })
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

pub(crate) fn discover_beardog_socket_in(
    socket_dir: Option<&Path>,
    family_id: Option<&str>,
) -> Option<String> {
    if let Some(runtime_dir) = socket_dir {
        let biomeos_dir = runtime_dir.join("biomeos");
        let xdg_path = biomeos_dir.join(format!("{BEARDOG}.sock"));
        if xdg_path.exists() {
            return Some(xdg_path.to_string_lossy().to_string());
        }
        if let Some(fid) = family_id {
            let family_path = biomeos_dir.join(format!("{BEARDOG}-{fid}.sock"));
            if family_path.exists() {
                return Some(family_path.to_string_lossy().to_string());
            }
        }
    }

    None
}

fn discover_beardog_socket() -> Option<String> {
    let socket_dir = std::env::var("XDG_RUNTIME_DIR").ok().map(PathBuf::from);
    let family_id = std::env::var("FAMILY_ID").ok();
    if let Some(s) = discover_beardog_socket_in(socket_dir.as_deref(), family_id.as_deref()) {
        return Some(s);
    }

    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let xdg_socket = paths.primal_socket(BEARDOG);
    if xdg_socket.exists() {
        return Some(xdg_socket.to_string_lossy().to_string());
    }

    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "family".to_string());
    let family_socket = paths.primal_socket(&format!("{BEARDOG}-{family_id}"));
    if family_socket.exists() {
        return Some(family_socket.to_string_lossy().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_resolve_device_id_explicit() {
        let id = resolve_device_id(Some("custom-device-123"));
        assert_eq!(id, "custom-device-123");
    }

    #[test]
    fn test_resolve_device_id_empty_string_filters() {
        let id = resolve_device_id(Some(""));
        assert!(!id.is_empty());
    }

    #[test]
    fn test_resolve_device_id_whitespace_only_passes() {
        let id = resolve_device_id(Some("   "));
        assert_eq!(id, "   ");
    }

    #[test]
    fn test_resolve_device_id_none_generates() {
        let id = resolve_device_id(None);
        assert!(!id.is_empty());
        // Should be UUID format when machine-id unavailable
        assert!(id.len() >= 32);
    }

    #[test]
    fn test_validate_enrollment_paths_family_seed_missing() {
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage = temp.path().join(".lineage.seed");
        let family = temp.path().join("nonexistent.family.seed");
        let result = validate_enrollment_paths(&lineage, &family, false);
        assert!(matches!(
            result,
            Err(EnrollmentValidationError::FamilySeedNotFound)
        ));
    }

    #[test]
    fn test_validate_enrollment_paths_already_enrolled() {
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage = temp.path().join(".lineage.seed");
        std::fs::write(&lineage, "existing").expect("write lineage");
        let family = temp.path().join(".family.seed");
        std::fs::write(&family, "seed").expect("write family");
        let result = validate_enrollment_paths(&lineage, &family, false);
        assert!(matches!(
            result,
            Err(EnrollmentValidationError::AlreadyEnrolled)
        ));
    }

    #[test]
    fn test_validate_enrollment_paths_force_ok() {
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage = temp.path().join(".lineage.seed");
        std::fs::write(&lineage, "existing").expect("write lineage");
        let family = temp.path().join(".family.seed");
        std::fs::write(&family, "seed").expect("write family");
        let result = validate_enrollment_paths(&lineage, &family, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_enrollment_paths_fresh_enrollment() {
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage = temp.path().join(".lineage.seed");
        let family = temp.path().join(".family.seed");
        std::fs::write(&family, "seed").expect("write family");
        let result = validate_enrollment_paths(&lineage, &family, false);
        assert!(
            result.is_ok(),
            "fresh enrollment (no lineage) should succeed"
        );
    }

    #[test]
    fn test_enrollment_validation_error_display() {
        let already = EnrollmentValidationError::AlreadyEnrolled;
        assert!(
            already.to_string().contains("already enrolled"),
            "AlreadyEnrolled display: {}",
            already
        );
        assert!(already.to_string().contains("force"));

        let not_found = EnrollmentValidationError::FamilySeedNotFound;
        assert!(
            not_found.to_string().contains("not found"),
            "FamilySeedNotFound display: {}",
            not_found
        );
    }

    #[test]
    fn test_get_machine_id() {
        // This test may or may not find a machine-id depending on platform
        let _ = get_machine_id();
    }

    #[test]
    fn test_discover_beardog_socket_handles_missing() {
        assert!(discover_beardog_socket_in(None, None).is_none());
    }

    #[tokio::test]
    async fn test_run_fails_when_family_seed_missing() {
        let temp = tempfile::tempdir().expect("temp dir");
        let args = EnrollArgs {
            family_id: "test-family".to_string(),
            node_id: "test-node".to_string(),
            device_id: Some("test-device-123".to_string()),
            family_seed: temp.path().join("nonexistent.family.seed"),
            lineage_seed: temp.path().join(".lineage.seed"),
            beardog_socket: None,
            beardog_socket_dir: None,
            force: false,
        };
        let result = run(args).await;
        assert!(result.is_err(), "run should fail when family seed missing");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Family seed not found"),
            "Expected family seed error: {err}"
        );
    }

    #[tokio::test]
    async fn test_run_fails_when_beardog_socket_missing() {
        let temp = tempfile::tempdir().expect("temp dir");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "test-seed-content").expect("write family seed");

        let args = EnrollArgs {
            family_id: "test-family".to_string(),
            node_id: "test-node".to_string(),
            device_id: Some("test-device-123".to_string()),
            family_seed,
            lineage_seed: temp.path().join(".lineage.seed"),
            beardog_socket: None,
            beardog_socket_dir: Some(temp.path().to_path_buf()),
            force: false,
        };
        let result = run(args).await;
        assert!(
            result.is_err(),
            "run should fail when BearDog socket not found"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("BearDog") || err.to_string().contains("socket"),
            "Expected BearDog/socket error: {err}"
        );
    }

    #[tokio::test]
    async fn test_run_without_device_id_uses_resolve_fallback() {
        let temp = tempfile::tempdir().expect("temp dir");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "test-seed").expect("write family seed");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: None,
            family_seed,
            lineage_seed: temp.path().join(".lineage.seed"),
            beardog_socket: None,
            beardog_socket_dir: Some(temp.path().to_path_buf()),
            force: false,
        };
        let result = run(args).await;
        assert!(
            result.is_err(),
            "run without device_id should fail at BearDog (or family seed) when socket missing"
        );
    }

    #[tokio::test]
    async fn test_run_uses_device_id_when_provided() {
        let temp = tempfile::tempdir().expect("temp dir");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "test-seed").expect("write family seed");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: Some("custom-device-id-xyz".to_string()),
            family_seed,
            lineage_seed: temp.path().join(".lineage.seed"),
            beardog_socket: None,
            beardog_socket_dir: None,
            force: false,
        };
        let result = run(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_returns_ok_when_already_enrolled_no_force() {
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage_seed = temp.path().join(".lineage.seed");
        std::fs::write(&lineage_seed, "existing-lineage-seed").expect("write lineage");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "test-seed").expect("write family seed");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: Some("device-1".to_string()),
            family_seed,
            lineage_seed,
            beardog_socket: None,
            beardog_socket_dir: None,
            force: false,
        };
        let result = run(args).await;
        assert!(
            result.is_ok(),
            "already enrolled should return Ok (early exit): {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_returns_ok_when_already_enrolled_but_load_lineage_fails() {
        // Lineage exists but .lineage.json is invalid - load_lineage fails, we still return Ok early
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage_seed = temp.path().join(".lineage.seed");
        std::fs::write(&lineage_seed, "x").expect("write lineage");
        let lineage_json = lineage_seed.with_extension("json");
        std::fs::write(&lineage_json, "{invalid json").expect("write invalid json");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "test-seed").expect("write family seed");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: Some("device-1".to_string()),
            family_seed,
            lineage_seed,
            beardog_socket: None,
            beardog_socket_dir: None,
            force: false,
        };
        let result = run(args).await;
        assert!(
            result.is_ok(),
            "already enrolled with unloadable lineage should still return Ok: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_force_attempts_enrollment_when_lineage_exists() {
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage_seed = temp.path().join(".lineage.seed");
        std::fs::write(&lineage_seed, "existing-lineage-seed").expect("write lineage");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "test-seed").expect("write family seed");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: Some("device-1".to_string()),
            family_seed,
            lineage_seed,
            beardog_socket: None,
            beardog_socket_dir: None,
            force: true,
        };
        let result = run(args).await;
        assert!(
            result.is_err(),
            "force re-enroll without BearDog should fail: {result:?}"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("BearDog") || err.to_string().contains("socket"),
            "Expected BearDog/socket error: {err}"
        );
    }

    #[tokio::test]
    async fn test_run_fails_when_lineage_seed_is_directory() {
        let temp = tempfile::tempdir().expect("temp dir");
        let lineage_seed = temp.path().join(".lineage.seed");
        std::fs::create_dir_all(&lineage_seed).expect("create lineage dir");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "test-seed").expect("write family seed");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: Some("device-1".to_string()),
            family_seed,
            lineage_seed,
            beardog_socket: None,
            beardog_socket_dir: None,
            force: false,
        };
        let result = run(args).await;
        assert!(
            result.is_ok(),
            "lineage_seed as dir: exists() is true, early exit"
        );
    }

    #[test]
    fn test_enroll_args_construction() {
        let args = EnrollArgs {
            family_id: "fam123".to_string(),
            node_id: "tower".to_string(),
            device_id: Some("dev456".to_string()),
            family_seed: std::path::PathBuf::from(".family.seed"),
            lineage_seed: std::path::PathBuf::from(".lineage.seed"),
            beardog_socket: None,
            beardog_socket_dir: None,
            force: false,
        };
        assert_eq!(args.family_id, "fam123");
        assert_eq!(args.node_id, "tower");
        assert_eq!(args.device_id, Some("dev456".to_string()));
        assert!(!args.force);
        assert_eq!(args.family_seed, std::path::PathBuf::from(".family.seed"));
        assert_eq!(args.lineage_seed, std::path::PathBuf::from(".lineage.seed"));
    }

    #[test]
    fn test_enroll_args_with_custom_paths() {
        let custom_family = PathBuf::from("/custom/.family.seed");
        let custom_lineage = PathBuf::from("/custom/.lineage.seed");
        let args = EnrollArgs {
            family_id: "f".to_string(),
            node_id: "n".to_string(),
            device_id: None,
            family_seed: custom_family.clone(),
            lineage_seed: custom_lineage.clone(),
            beardog_socket: Some("/tmp/beardog.sock".to_string()),
            beardog_socket_dir: None,
            force: true,
        };
        assert_eq!(args.family_seed, custom_family);
        assert_eq!(args.lineage_seed, custom_lineage);
        assert!(args.force);
        assert_eq!(args.beardog_socket, Some("/tmp/beardog.sock".to_string()));
    }

    #[test]
    fn test_discover_beardog_socket_finds_default_socket() {
        let temp = tempfile::tempdir().expect("temp dir");
        let biomeos_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).expect("create biomeos dir");
        let socket_path = biomeos_dir.join("beardog.sock");
        std::fs::write(&socket_path, "").expect("create socket file");

        let result = discover_beardog_socket_in(Some(temp.path()), None);
        assert!(
            result.is_some(),
            "Should find socket when socket_dir/biomeos/beardog.sock exists"
        );
        assert!(result.unwrap().contains("beardog.sock"));
    }

    #[test]
    fn test_discover_beardog_socket_finds_family_suffixed_socket() {
        let temp = tempfile::tempdir().expect("temp dir");
        let biomeos_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).expect("create biomeos dir");
        let socket_path = biomeos_dir.join("beardog-testfamily123.sock");
        std::fs::write(&socket_path, "").expect("create socket file");

        let result = discover_beardog_socket_in(Some(temp.path()), Some("testfamily123"));
        assert!(
            result.is_some(),
            "Should find beardog-{{family_id}}.sock when socket_dir and family_id provided"
        );
        assert!(result.unwrap().contains("beardog-testfamily123.sock"));
    }

    #[tokio::test]
    async fn test_run_fails_when_family_seed_empty() {
        let temp = tempfile::tempdir().expect("temp dir");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "").expect("write empty family seed");
        let lineage_seed = temp.path().join(".lineage.seed");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: Some("device-xyz".to_string()),
            family_seed,
            lineage_seed,
            beardog_socket: None,
            beardog_socket_dir: None,
            force: false,
        };
        let result = run(args).await;
        assert!(
            result.is_err(),
            "run with empty family seed should fail at BearDog or derivation"
        );
    }

    #[tokio::test]
    async fn test_run_fails_when_beardog_socket_connection_refused() {
        let temp = tempfile::tempdir().expect("temp dir");
        let family_seed = temp.path().join(".family.seed");
        std::fs::write(&family_seed, "valid-seed-content").expect("write family seed");
        let lineage_seed = temp.path().join(".lineage.seed");
        let nonexistent_socket = temp.path().join("nonexistent.sock");

        let args = EnrollArgs {
            family_id: "test".to_string(),
            node_id: "node".to_string(),
            device_id: Some("device-xyz".to_string()),
            family_seed,
            lineage_seed,
            beardog_socket: Some(nonexistent_socket.to_string_lossy().to_string()),
            beardog_socket_dir: None,
            force: false,
        };
        let result = run(args).await;
        assert!(
            result.is_err(),
            "run with nonexistent BearDog socket should fail"
        );
    }
}
