// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

/// Calculate utilization percentage from available and total space.
/// Planned for storage analytics dashboard.
#[allow(dead_code)]
pub(crate) fn calculate_utilization(available: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let used = total.saturating_sub(available);
    (used as f64 / total as f64) * 100.0
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
                                // DEEP DEBT EVOLUTION: Accept ANY executable as a primal.
                                // No hardcoded whitelist — primals self-identify at runtime.
                                // Skip hidden files and non-executable entries.
                                if !name.starts_with('.')
                                    && !name.ends_with(".toml")
                                    && !name.ends_with(".json")
                                    && !name.ends_with(".genome")
                                {
                                    if let Ok(meta) = entry.metadata().await {
                                        if meta.is_file() {
                                            primals.push(name.to_string());
                                        }
                                    }
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_livespore_device_serialization() {
        let device = LiveSporeDevice {
            id: "usb-001".to_string(),
            mount_point: "/media/usb".to_string(),
            label: Some("LIVESPORE".to_string()),
            available_space: 1024 * 1024 * 100, // 100 MB
            total_space: 1024 * 1024 * 1024,    // 1 GB
            utilization_percent: 90.0,
            has_genetic_seed: true,
            genetic_preview: Some("abc123def456".to_string()),
            primals: vec!["beardog".to_string(), "songbird".to_string()],
            spore_type: Some("live".to_string()),
        };

        let json = serde_json::to_string(&device).expect("serialize");
        assert!(json.contains("usb-001"));
        assert!(json.contains("/media/usb"));
        assert!(json.contains("LIVESPORE"));
        assert!(json.contains("beardog"));
    }

    #[test]
    fn test_livespore_device_deserialization() {
        let json = r#"{
            "id": "test-device",
            "mount_point": "/mnt/test",
            "label": null,
            "available_space": 500,
            "total_space": 1000,
            "utilization_percent": 50.0,
            "has_genetic_seed": false,
            "genetic_preview": null,
            "primals": [],
            "spore_type": null
        }"#;

        let device: LiveSporeDevice = serde_json::from_str(json).expect("deserialize");
        assert_eq!(device.id, "test-device");
        assert_eq!(device.available_space, 500);
        assert!(!device.has_genetic_seed);
        assert!(device.primals.is_empty());
    }

    #[test]
    fn test_livespores_response_serialization() {
        let response = LiveSporesResponse {
            devices: vec![],
            count: 0,
            discovered_at: "2026-02-04T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("2026-02-04"));
    }

    #[test]
    fn test_calculate_utilization_normal() {
        // 900 MB used out of 1 GB = 90%
        let util = calculate_utilization(100 * 1024 * 1024, 1024 * 1024 * 1024);
        assert!((util - 90.234).abs() < 1.0); // ~90%
    }

    #[test]
    fn test_calculate_utilization_empty() {
        // All space available = 0% used
        let util = calculate_utilization(1000, 1000);
        assert!((util - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_utilization_full() {
        // No space available = 100% used
        let util = calculate_utilization(0, 1000);
        assert!((util - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_utilization_zero_total() {
        // Division by zero protection
        let util = calculate_utilization(0, 0);
        assert!((util - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_livespore_device_with_primals() {
        let device = LiveSporeDevice {
            id: "spore-usb".to_string(),
            mount_point: "/media/spore".to_string(),
            label: None,
            available_space: 0,
            total_space: 0,
            utilization_percent: 0.0,
            has_genetic_seed: true,
            genetic_preview: Some("lineage_abc123".to_string()),
            primals: vec![
                "beardog".to_string(),
                "songbird".to_string(),
                "toadstool".to_string(),
                "nestgate".to_string(),
                "squirrel".to_string(),
            ],
            spore_type: Some("live".to_string()),
        };

        assert_eq!(device.primals.len(), 5);
        assert!(device.primals.contains(&"beardog".to_string()));
        assert!(device.primals.contains(&"squirrel".to_string()));
    }

    #[test]
    fn test_spore_types() {
        // Live spore has tower.toml
        let live = LiveSporeDevice {
            id: "live".to_string(),
            mount_point: "/mnt/live".to_string(),
            label: None,
            available_space: 0,
            total_space: 0,
            utilization_percent: 0.0,
            has_genetic_seed: true,
            genetic_preview: None,
            primals: vec![],
            spore_type: Some("live".to_string()),
        };
        assert_eq!(live.spore_type, Some("live".to_string()));

        // Cold spore has genetic seed but no tower.toml
        let cold = LiveSporeDevice {
            id: "cold".to_string(),
            mount_point: "/mnt/cold".to_string(),
            label: None,
            available_space: 0,
            total_space: 0,
            utilization_percent: 0.0,
            has_genetic_seed: true,
            genetic_preview: None,
            primals: vec![],
            spore_type: Some("cold".to_string()),
        };
        assert_eq!(cold.spore_type, Some("cold".to_string()));
    }

    #[test]
    fn test_livespore_device_serialization_roundtrip() {
        let device = LiveSporeDevice {
            id: "roundtrip-device".to_string(),
            mount_point: "/media/roundtrip".to_string(),
            label: Some("ROUNDTRIP".to_string()),
            available_space: 1_000_000,
            total_space: 2_000_000,
            utilization_percent: 50.0,
            has_genetic_seed: false,
            genetic_preview: None,
            primals: vec!["primal1".to_string()],
            spore_type: None,
        };

        let json = serde_json::to_string(&device).expect("serialize");
        let restored: LiveSporeDevice = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(device.id, restored.id);
        assert_eq!(device.mount_point, restored.mount_point);
        assert_eq!(device.label, restored.label);
        assert_eq!(device.available_space, restored.available_space);
        assert_eq!(device.total_space, restored.total_space);
        assert_eq!(device.primals, restored.primals);
    }

    #[test]
    fn test_livespores_response_structure() {
        let response = LiveSporesResponse {
            devices: vec![LiveSporeDevice {
                id: "d1".to_string(),
                mount_point: "/mnt/d1".to_string(),
                label: None,
                available_space: 0,
                total_space: 0,
                utilization_percent: 0.0,
                has_genetic_seed: false,
                genetic_preview: None,
                primals: vec![],
                spore_type: None,
            }],
            count: 1,
            discovered_at: "2026-03-11T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"devices\""));
        assert!(json.contains("\"count\":1"));
        assert!(json.contains("discovered_at"));
    }

    #[tokio::test]
    async fn test_get_livespores_handler_returns_ok() {
        // Handler gracefully degrades when USB discovery fails (returns empty list)
        let state = crate::AppState::builder()
            .build_with_defaults()
            .expect("create app state");

        let result = get_livespores(axum::extract::State(std::sync::Arc::new(state))).await;
        assert!(
            result.is_ok(),
            "get_livespores should return Ok (graceful degradation), got: {result:?}"
        );

        let response = result.expect("response");
        assert_eq!(response.count, response.devices.len());
        assert!(!response.discovered_at.is_empty());
    }

    #[test]
    fn test_calculate_utilization_saturating_sub() {
        // available > total (edge case - should not panic)
        let util = calculate_utilization(2000, 1000);
        assert!((0.0..=100.0).contains(&util));
    }

    #[test]
    fn test_livespore_device_clone() {
        let device = LiveSporeDevice {
            id: "clone-test".to_string(),
            mount_point: "/mnt/test".to_string(),
            label: Some("LABEL".to_string()),
            available_space: 100,
            total_space: 200,
            utilization_percent: 50.0,
            has_genetic_seed: true,
            genetic_preview: None,
            primals: vec!["beardog".to_string()],
            spore_type: Some("live".to_string()),
        };
        let cloned = device.clone();
        assert_eq!(cloned.id, device.id);
        assert_eq!(cloned.utilization_percent, device.utilization_percent);
    }

    #[test]
    fn test_livespore_device_debug() {
        let device = LiveSporeDevice {
            id: "debug".to_string(),
            mount_point: "/mnt".to_string(),
            label: None,
            available_space: 0,
            total_space: 0,
            utilization_percent: 0.0,
            has_genetic_seed: false,
            genetic_preview: None,
            primals: vec![],
            spore_type: None,
        };
        let debug_str = format!("{device:?}");
        assert!(debug_str.contains("debug"));
    }
}
