// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for [`crate::deployment_mode`].

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use crate::deployment_mode::{DeploymentFromEnvParams, DeploymentMode, HostOS, IsolationLevel};
use std::path::{Path, PathBuf};

#[test]
fn test_deployment_mode_from_env_cold() {
    let mode = DeploymentMode::from_env_string_with_params(
        "cold",
        DeploymentFromEnvParams {
            media_path: Some("/media/usb0".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    match mode {
        DeploymentMode::ColdSpore { media_path, .. } => {
            assert_eq!(media_path, PathBuf::from("/media/usb0"));
        }
        _ => panic!("Expected ColdSpore"),
    }
}

#[test]
fn test_deployment_mode_from_env_live() {
    let mode =
        DeploymentMode::from_env_string_with_params("live", DeploymentFromEnvParams::default())
            .unwrap();

    match mode {
        DeploymentMode::LiveSpore { root_partition, .. } => {
            assert_eq!(root_partition, PathBuf::from("/"));
        }
        _ => panic!("Expected LiveSpore"),
    }
}

#[test]
fn test_deployment_mode_from_env_sibling() {
    let mode =
        DeploymentMode::from_env_string_with_params("sibling", DeploymentFromEnvParams::default())
            .unwrap();

    match mode {
        DeploymentMode::SiblingSpore { .. } => {
            // Success
        }
        _ => panic!("Expected SiblingSpore"),
    }
}

#[test]
fn test_socket_prefix_cold() {
    let mode = DeploymentMode::ColdSpore {
        media_path: PathBuf::from("/media/usb0"),
        persistence: false,
        host_os: HostOS::Unknown,
    };

    assert_eq!(mode.socket_prefix(), PathBuf::from("/media/usb0/runtime"));
}

#[test]
fn test_socket_prefix_sibling() {
    let mode = DeploymentMode::SiblingSpore {
        host_os: HostOS::Unknown,
        install_dir: PathBuf::from("/home/user/.local/share/biomeos"),
        isolation: IsolationLevel::Shared,
    };

    assert_eq!(
        mode.socket_prefix(),
        PathBuf::from("/home/user/.local/share/biomeos/runtime")
    );
}

#[test]
fn test_description() {
    let mode = DeploymentMode::ColdSpore {
        media_path: PathBuf::from("/media/usb0"),
        persistence: true,
        host_os: HostOS::Linux {
            distro: "Ubuntu".to_string(),
        },
    };

    let desc = mode.description();
    assert!(desc.contains("Cold Spore"));
    assert!(desc.contains("persistent"));
}

#[test]
fn test_host_os_name() {
    let os = HostOS::Linux {
        distro: "Ubuntu 22.04".to_string(),
    };

    assert_eq!(os.name(), "Linux (Ubuntu 22.04)");
}

#[test]
fn test_from_env_string_invalid() {
    let result = DeploymentMode::from_env_string_with_params(
        "invalid_mode",
        DeploymentFromEnvParams::default(),
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid deployment mode"));
}

#[test]
fn test_from_env_string_variants() {
    // cold_spore, livespore, sibling_spore (underscore variants)
    let cold = DeploymentMode::from_env_string_with_params(
        "cold_spore",
        DeploymentFromEnvParams::default(),
    )
    .unwrap();
    assert!(matches!(cold, DeploymentMode::ColdSpore { .. }));

    let live = DeploymentMode::from_env_string_with_params(
        "live_spore",
        DeploymentFromEnvParams::default(),
    )
    .unwrap();
    assert!(matches!(live, DeploymentMode::LiveSpore { .. }));

    let sibling = DeploymentMode::from_env_string_with_params(
        "sibling_spore",
        DeploymentFromEnvParams::default(),
    )
    .unwrap();
    assert!(matches!(sibling, DeploymentMode::SiblingSpore { .. }));
}

#[test]
fn test_from_env_string_cold_persistence() {
    let mode = DeploymentMode::from_env_string_with_params(
        "cold",
        DeploymentFromEnvParams {
            media_path: Some("/media/usb1".to_string()),
            persistence: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    match mode {
        DeploymentMode::ColdSpore { persistence, .. } => assert!(persistence),
        _ => panic!("Expected ColdSpore"),
    }
}

#[test]
fn test_socket_prefix_livespore_with_xdg() {
    let mode = DeploymentMode::LiveSpore {
        root_partition: PathBuf::from("/"),
        boot_partition: PathBuf::from("/boot"),
        installed_version: "1.0.0".to_string(),
    };
    let prefix = mode.socket_prefix_with_runtime(Some("/run/user/1000"), None);
    assert_eq!(prefix, PathBuf::from("/run/user/1000/biomeos"));
}

#[test]
fn test_description_cold_ephemeral() {
    let mode = DeploymentMode::ColdSpore {
        media_path: PathBuf::from("/media/usb0"),
        persistence: false,
        host_os: HostOS::Unknown,
    };
    let desc = mode.description();
    assert!(desc.contains("Cold Spore"));
    assert!(desc.contains("ephemeral"));
}

#[test]
fn test_description_livespore() {
    let mode = DeploymentMode::LiveSpore {
        root_partition: PathBuf::from("/"),
        boot_partition: PathBuf::from("/boot"),
        installed_version: "2.0.0".to_string(),
    };
    let desc = mode.description();
    assert!(desc.contains("Live Spore"));
    assert!(desc.contains("2.0.0"));
}

#[test]
fn test_description_siblingspore() {
    let mode = DeploymentMode::SiblingSpore {
        host_os: HostOS::Windows {
            version: "11".to_string(),
        },
        install_dir: PathBuf::from("/opt/biomeos"),
        isolation: IsolationLevel::Full,
    };
    let desc = mode.description();
    assert!(desc.contains("Sibling Spore"));
    assert!(desc.contains("Windows"));
}

#[test]
fn test_host_os_name_all_variants() {
    assert_eq!(
        HostOS::MacOS {
            version: "14.0".to_string()
        }
        .name(),
        "macOS 14.0"
    );
    assert_eq!(
        HostOS::Windows {
            version: "11".to_string()
        }
        .name(),
        "Windows 11"
    );
    assert_eq!(
        HostOS::BSD {
            variant: "FreeBSD".to_string()
        }
        .name(),
        "FreeBSD"
    );
    assert_eq!(HostOS::Unknown.name(), "Unknown OS");
}

#[test]
fn test_deployment_mode_serialization() {
    let mode = DeploymentMode::SiblingSpore {
        host_os: HostOS::Unknown,
        install_dir: PathBuf::from("/home/test/.local/share/biomeos"),
        isolation: IsolationLevel::Sandboxed,
    };
    let json = serde_json::to_string(&mode).expect("serialize");
    let restored: DeploymentMode = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(mode, restored);
}

#[test]
fn test_isolation_level_serialization() {
    let levels = [
        IsolationLevel::Sandboxed,
        IsolationLevel::Shared,
        IsolationLevel::Full,
    ];
    for level in levels {
        let json = serde_json::to_string(&level).expect("serialize");
        let restored: IsolationLevel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(level, restored);
    }
}

#[test]
fn test_socket_prefix_livespore_without_xdg() {
    let mode = DeploymentMode::LiveSpore {
        root_partition: PathBuf::from("/"),
        boot_partition: PathBuf::from("/boot"),
        installed_version: "1.0".to_string(),
    };
    let prefix = mode.socket_prefix_with_runtime(None, None);
    assert!(prefix.to_string_lossy().contains("biomeos"));
    assert!(
        prefix.to_string_lossy().contains("/run/user/")
            || prefix.to_string_lossy().contains("biomeos")
    );
}

#[test]
fn test_from_env_string_coldspore_livespore_variants() {
    let c1 = DeploymentMode::from_env_string_with_params(
        "coldspore",
        DeploymentFromEnvParams::default(),
    )
    .unwrap();
    assert!(matches!(c1, DeploymentMode::ColdSpore { .. }));
    let c2 = DeploymentMode::from_env_string_with_params(
        "livespore",
        DeploymentFromEnvParams::default(),
    )
    .unwrap();
    assert!(matches!(c2, DeploymentMode::LiveSpore { .. }));
}

#[test]
fn test_from_env_string_biomeos_version() {
    let mode = DeploymentMode::from_env_string_with_params(
        "live",
        DeploymentFromEnvParams {
            installed_version: Some("9.9.9".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    match mode {
        DeploymentMode::LiveSpore {
            installed_version, ..
        } => {
            assert_eq!(installed_version, "9.9.9");
        }
        _ => panic!("expected LiveSpore"),
    }
}

#[test]
fn test_deployment_mode_serde_coldspore() {
    let mode = DeploymentMode::ColdSpore {
        media_path: PathBuf::from("/media/usb"),
        persistence: false,
        host_os: HostOS::Unknown,
    };
    let json = serde_json::to_string(&mode).expect("serialize");
    let _: DeploymentMode = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn test_is_removable_mount_detects_standard_paths() {
    assert!(DeploymentMode::is_removable_mount(
        "/dev/loop0",
        Path::new("/media/usb0")
    ));
    assert!(DeploymentMode::is_removable_mount(
        "/dev/loop0",
        Path::new("/mnt/data")
    ));
    assert!(DeploymentMode::is_removable_mount(
        "/dev/loop0",
        Path::new("/run/media/user/volume")
    ));
}

#[test]
fn test_is_removable_mount_sd_without_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(!DeploymentMode::is_removable_mount("/dev/sda1", tmp.path()));
}

#[test]
fn test_is_removable_mount_sd_with_biomeos_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join(".biomeos-spore"), b"1").expect("marker");
    assert!(DeploymentMode::is_removable_mount("/dev/sda1", tmp.path()));
}

#[test]
fn test_is_removable_mount_mmcblk_with_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join(".biomeos-spore"), b"1").expect("marker");
    assert!(DeploymentMode::is_removable_mount(
        "/dev/mmcblk0p1",
        tmp.path()
    ));
}

#[test]
fn test_from_env_string_cold_default_media_when_unset() {
    let mode =
        DeploymentMode::from_env_string_with_params("cold", DeploymentFromEnvParams::default())
            .expect("cold");
    match mode {
        DeploymentMode::ColdSpore { media_path, .. } => {
            assert_eq!(media_path, PathBuf::from("/media/biomeos"));
        }
        _ => panic!("expected ColdSpore"),
    }
}

#[test]
fn test_detect_isolation_level_sandboxed_alias() {
    assert!(matches!(
        DeploymentMode::isolation_level_from_env(Some("sandbox")),
        IsolationLevel::Sandboxed
    ));
}

#[test]
fn test_detect_isolation_level_shared_and_full() {
    assert!(matches!(
        DeploymentMode::isolation_level_from_env(Some("shared")),
        IsolationLevel::Shared
    ));

    assert!(matches!(
        DeploymentMode::isolation_level_from_env(Some("full")),
        IsolationLevel::Full
    ));
}

#[test]
fn test_detect_isolation_level_unknown_falls_back_to_shared() {
    assert!(matches!(
        DeploymentMode::isolation_level_from_env(Some("not-a-real-level")),
        IsolationLevel::Shared
    ));
}
