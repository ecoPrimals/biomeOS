// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unit tests for storage manifest types.

use super::*;
use std::collections::HashMap;

#[test]
fn test_volume_spec_default() {
    let vol = VolumeSpec::default();
    assert_eq!(vol.metadata.name, "default-volume");
    assert!(matches!(vol.volume_type, VolumeType::EmptyDir { .. }));
    assert!(vol.mount_options.is_empty());
    assert!(!vol.access_modes.is_empty());
    assert!(matches!(vol.reclaim_policy, VolumeReclaimPolicy::Delete));
    assert!(matches!(vol.volume_mode, VolumeMode::Filesystem));
}

#[test]
fn test_volume_metadata_default() {
    let meta = VolumeMetadata::default();
    assert_eq!(meta.name, "default-volume");
    assert!(meta.description.is_none());
    assert!(meta.labels.is_empty());
    assert!(meta.annotations.is_empty());
}

#[test]
fn test_secret_spec_default() {
    let secret = SecretSpec::default();
    assert_eq!(secret.metadata.name, "default-secret");
    assert!(matches!(secret.secret_type, SecretType::Opaque));
    assert!(secret.data.is_empty());
    assert!(!secret.immutable);
}

#[test]
fn test_secret_metadata_default() {
    let meta = SecretMetadata::default();
    assert_eq!(meta.name, "default-secret");
    assert!(meta.description.is_none());
    assert!(meta.labels.is_empty());
}

#[test]
fn test_config_spec_default() {
    let config = ConfigSpec::default();
    assert!(config.data.is_empty());
    assert!(config.binary_data.is_empty());
    assert!(!config.immutable);
}

#[test]
fn test_config_metadata_default() {
    let meta = ConfigMetadata::default();
    assert_eq!(meta.name, "default-config");
    assert!(meta.description.is_none());
    assert!(meta.labels.is_empty());
}

#[test]
fn test_volume_types_serialization() {
    let empty_dir = VolumeType::EmptyDir {
        medium: Some(EmptyDirMedium::Memory),
        size_limit: Some("1Gi".to_string()),
    };
    let json = serde_json::to_string(&empty_dir).unwrap();
    assert!(json.contains("Memory"));

    let host_path = VolumeType::HostPath {
        path: "/data".to_string(),
        path_type: Some(HostPathType::Directory),
    };
    let json = serde_json::to_string(&host_path).unwrap();
    assert!(json.contains("/data"));
}

#[test]
fn test_volume_access_modes() {
    let modes = vec![
        VolumeAccessMode::ReadWriteOnce,
        VolumeAccessMode::ReadOnlyMany,
        VolumeAccessMode::ReadWriteMany,
        VolumeAccessMode::ReadWriteOncePod,
    ];
    for mode in modes {
        let json = serde_json::to_string(&mode).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_volume_reclaim_policies() {
    let policies = vec![
        VolumeReclaimPolicy::Retain,
        VolumeReclaimPolicy::Recycle,
        VolumeReclaimPolicy::Delete,
    ];
    for policy in policies {
        let json = serde_json::to_string(&policy).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_secret_types() {
    let types = vec![
        SecretType::Opaque,
        SecretType::ServiceAccountToken,
        SecretType::Tls,
        SecretType::BasicAuth,
        SecretType::Custom("my-type".to_string()),
    ];
    for t in types {
        let json = serde_json::to_string(&t).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_secret_data_variants() {
    let data_variants = vec![
        SecretData::Base64("dGVzdA==".to_string()),
        SecretData::Text("plain".to_string()),
        SecretData::File("/path/to/file".to_string()),
    ];
    for data in data_variants {
        let json = serde_json::to_string(&data).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_key_to_path() {
    let ktp = KeyToPath {
        key: "config.yaml".to_string(),
        path: "/etc/config/config.yaml".to_string(),
        mode: Some(0o644),
    };
    let json = serde_json::to_string(&ktp).unwrap();
    assert!(json.contains("config.yaml"));
}

#[test]
fn test_local_object_reference() {
    let ref_ = LocalObjectReference {
        name: "my-secret".to_string(),
    };
    let json = serde_json::to_string(&ref_).unwrap();
    assert!(json.contains("my-secret"));
}

#[test]
fn test_label_selector() {
    let selector = LabelSelector {
        match_labels: HashMap::from([("app".to_string(), "web".to_string())]),
        match_expressions: vec![LabelSelectorRequirement {
            key: "tier".to_string(),
            operator: LabelSelectorOperator::In,
            values: vec!["frontend".to_string()],
        }],
    };
    let json = serde_json::to_string(&selector).unwrap();
    assert!(json.contains("app"));
    assert!(json.contains("tier"));
}
