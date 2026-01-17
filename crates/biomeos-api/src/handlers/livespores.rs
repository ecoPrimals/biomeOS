// LiveSpore USB device discovery handler

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::{state::AppState, ApiError};

/// LiveSpore USB device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSporeDevice {
    /// Device ID (derived from mount point)
    pub id: String,

    /// Mount point (e.g., `/media/usb`)
    pub mount_point: String,

    /// Device label (if available)
    pub label: Option<String>,

    /// Available space in bytes
    pub available_space: u64,

    /// Total space in bytes
    pub total_space: u64,

    /// Utilization percentage (0-100)
    pub utilization_percent: f64,

    /// Has .family.seed file
    pub has_genetic_seed: bool,

    /// Genetic lineage (first 16 chars of seed, if present)
    pub genetic_preview: Option<String>,

    /// Primal binaries found (if any)
    pub primals: Vec<String>,

    /// Spore type (if detected)
    pub spore_type: Option<String>,
}

/// LiveSpore discovery response
#[derive(Debug, Serialize)]
pub struct LiveSporesResponse {
    /// Discovered USB devices
    pub devices: Vec<LiveSporeDevice>,

    /// Total count
    pub count: usize,

    /// Discovery timestamp
    pub discovered_at: String,
}

/// GET /api/v1/livespores - Discover USB devices with LiveSpore capability
pub async fn get_livespores(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<LiveSporesResponse>, ApiError> {
    info!("Discovering LiveSpore USB devices");

    // Use biomeos-spore crate for discovery
    let devices = match biomeos_spore::usb::discover_usb_devices().await {
        Ok(usb_devices) => {
            // Convert to API format
            let mut live_devices = Vec::new();

            for usb_dev in usb_devices {
                // Get mount point string early to avoid borrow issues
                let mount_point_str = usb_dev.mount_point.display().to_string();
                let mount_point = &usb_dev.mount_point;

                // Check for .family.seed
                let seed_path = mount_point.join(".family.seed");
                let has_genetic_seed = tokio::fs::metadata(&seed_path).await.is_ok();

                let genetic_preview = if has_genetic_seed {
                    match tokio::fs::read_to_string(&seed_path).await {
                        Ok(seed) => Some(seed.chars().take(16).collect()),
                        Err(_) => None,
                    }
                } else {
                    None
                };

                // Check for primal binaries
                let bin_dir = mount_point.join("plasmidBin");
                let mut primals = Vec::new();

                if tokio::fs::metadata(&bin_dir).await.is_ok() {
                    if let Ok(mut entries) = tokio::fs::read_dir(&bin_dir).await {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            if let Some(name) = entry.file_name().to_str() {
                                // Check if it's a known primal
                                if ["beardog", "songbird", "toadstool", "nestgate", "squirrel"]
                                    .contains(&name)
                                {
                                    primals.push(name.to_string());
                                }
                            }
                        }
                    }
                }

                // Detect spore type
                let config_path = mount_point.join("tower.toml");
                let spore_type = if tokio::fs::metadata(&config_path).await.is_ok() {
                    Some("live".to_string())
                } else if has_genetic_seed {
                    Some("cold".to_string())
                } else {
                    None
                };

                // Generate device ID from mount point
                let id = mount_point
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let utilization = usb_dev.utilization_percent();

                live_devices.push(LiveSporeDevice {
                    id,
                    mount_point: mount_point_str,
                    label: usb_dev.label,
                    available_space: usb_dev.available_space,
                    total_space: usb_dev.total_space,
                    utilization_percent: utilization,
                    has_genetic_seed,
                    genetic_preview,
                    primals,
                    spore_type,
                });
            }

            live_devices
        }
        Err(e) => {
            error!("Failed to discover USB devices: {}", e);
            Vec::new() // Graceful degradation
        }
    };

    let count = devices.len();
    info!("Discovered {} LiveSpore device(s)", count);

    Ok(Json(LiveSporesResponse {
        devices,
        count,
        discovered_at: chrono::Utc::now().to_rfc3339(),
    }))
}
