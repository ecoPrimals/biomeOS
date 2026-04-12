// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]

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

#[test]
fn volume_type_nfs_roundtrip() {
    let v = VolumeType::Nfs {
        server: "nfs.example".into(),
        path: "/export".into(),
        read_only: true,
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: VolumeType = serde_json::from_str(&json).unwrap();
    assert!(matches!(
        back,
        VolumeType::Nfs {
            read_only: true,
            ..
        }
    ));
}

#[test]
fn volume_type_csi_roundtrip() {
    let v = VolumeType::Csi {
        driver: "csi.example".into(),
        volume_attributes: HashMap::from([("k".into(), "v".into())]),
        node_publish_secret_ref: Some(SecretReference {
            name: "sec".into(),
            namespace: Some("ns".into()),
        }),
        read_only: false,
        fs_type: Some("ext4".into()),
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: VolumeType = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, VolumeType::Csi { .. }));
}

#[test]
fn volume_type_persistent_volume_claim_roundtrip() {
    let v = VolumeType::PersistentVolumeClaim {
        claim_name: "pvc-1".into(),
        read_only: false,
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: VolumeType = serde_json::from_str(&json).unwrap();
    assert!(matches!(
        back,
        VolumeType::PersistentVolumeClaim {
            claim_name: ref n,
            ..
        } if n == "pvc-1"
    ));
}

#[test]
fn volume_type_ephemeral_roundtrip() {
    let tpl = PersistentVolumeClaimTemplate {
        metadata: VolumeMetadata {
            name: "eph-vol".into(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        },
        spec: PersistentVolumeClaimSpec {
            access_modes: vec![VolumeAccessMode::ReadWriteOnce],
            resources: VolumeResourceRequirements {
                limits: HashMap::new(),
                requests: HashMap::from([("storage".into(), "1Gi".into())]),
            },
            volume_name: None,
            storage_class: Some("fast".into()),
            volume_mode: Some(VolumeMode::Filesystem),
            selector: None,
        },
    };
    let v = VolumeType::Ephemeral {
        volume_claim_template: Box::new(tpl),
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: VolumeType = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, VolumeType::Ephemeral { .. }));
}

#[test]
fn config_data_yaml_json_toml_roundtrip() {
    let y = ConfigData::Yaml(serde_yaml::Value::String("x".into()));
    let j = ConfigData::Json(serde_json::json!({"a": 1}));
    let t = ConfigData::Toml("k = 1".into());
    for data in [y, j, t] {
        let json = serde_json::to_string(&data).unwrap();
        let back: ConfigData = serde_json::from_str(&json).unwrap();
        let _ = format!("{back:?}");
    }
}

#[test]
fn storage_class_spec_roundtrip() {
    let spec = StorageClassSpec {
        metadata: StorageClassMetadata {
            name: "gold".into(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        },
        provisioner: "example.com/provisioner".into(),
        parameters: HashMap::from([("type".into(), "ssd".into())]),
        reclaim_policy: VolumeReclaimPolicy::Delete,
        allow_volume_expansion: true,
        volume_binding_mode: VolumeBindingMode::WaitForFirstConsumer,
        allowed_topologies: vec![TopologySelectorTerm {
            match_label_expressions: vec![TopologySelectorLabelRequirement {
                key: "zone".into(),
                values: vec!["a".into()],
            }],
        }],
    };
    let json = serde_json::to_string(&spec).unwrap();
    let back: StorageClassSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(back.provisioner, spec.provisioner);
}

#[test]
fn secret_provider_cloud_variants_roundtrip() {
    let cases = vec![
        SecretProvider::AwsSecretsManager {
            region: "us-east-1".into(),
        },
        SecretProvider::AzureKeyVault {
            vault_url: "https://vault.vault.azure.net".into(),
        },
        SecretProvider::GoogleSecretManager {
            project_id: "p".into(),
        },
        SecretProvider::Vault {
            address: "https://vault:8200".into(),
            path: "secret/data/app".into(),
        },
        SecretProvider::Kubernetes {
            namespace: "default".into(),
            name: "cred".into(),
        },
        SecretProvider::Custom {
            provider_name: "custom".into(),
            config: HashMap::from([("k".into(), "v".into())]),
        },
    ];
    for p in cases {
        let json = serde_json::to_string(&p).unwrap();
        let back: SecretProvider = serde_json::from_str(&json).unwrap();
        let _ = format!("{back:?}");
    }
}

#[test]
fn node_selector_operator_gt_lt_roundtrip() {
    let req = NodeSelectorRequirement {
        key: "cpu".into(),
        operator: NodeSelectorOperator::Gt,
        values: vec!["4".into()],
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: NodeSelectorRequirement = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.operator, NodeSelectorOperator::Gt));
}

#[test]
fn volume_projection_secret_roundtrip() {
    let vp = VolumeProjection::Secret {
        local_object_reference: LocalObjectReference { name: "s".into() },
        items: vec![KeyToPath {
            key: "k".into(),
            path: "p".into(),
            mode: None,
        }],
        optional: true,
    };
    let json = serde_json::to_string(&vp).unwrap();
    let back: VolumeProjection = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, VolumeProjection::Secret { .. }));
}

#[test]
fn external_secret_ref_roundtrip() {
    let r = SecretData::External(ExternalSecretRef {
        provider: SecretProvider::Kubernetes {
            namespace: "ns".into(),
            name: "x".into(),
        },
        key: "password".into(),
        version: Some("2".into()),
    });
    let json = serde_json::to_string(&r).unwrap();
    let back: SecretData = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, SecretData::External(_)));
}

#[test]
fn volume_type_config_map_and_secret_roundtrip() {
    let cm = VolumeType::ConfigMap {
        name: "cfg".into(),
        items: vec![KeyToPath {
            key: "app.toml".into(),
            path: "app.toml".into(),
            mode: Some(0o644),
        }],
        default_mode: Some(0o644),
        optional: false,
    };
    let sec = VolumeType::Secret {
        secret_name: "sec".into(),
        items: vec![],
        default_mode: None,
        optional: true,
    };
    for v in [cm, sec] {
        let json = serde_json::to_string(&v).unwrap();
        let back: VolumeType = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            back,
            VolumeType::ConfigMap { .. } | VolumeType::Secret { .. }
        ));
    }
}

#[test]
fn volume_type_downward_api_and_projected_roundtrip() {
    let down = VolumeType::DownwardAPI {
        items: vec![DownwardAPIVolumeFile {
            path: "labels".into(),
            field_ref: Some(ObjectFieldSelector {
                api_version: "v1".into(),
                field_path: "metadata.labels".into(),
            }),
            resource_field_ref: None,
            mode: Some(0o644),
        }],
        default_mode: Some(0o644),
    };
    let proj = VolumeType::Projected {
        sources: vec![
            VolumeProjection::ConfigMap {
                local_object_reference: LocalObjectReference { name: "c".into() },
                items: vec![],
                optional: false,
            },
            VolumeProjection::DownwardAPI { items: vec![] },
        ],
        default_mode: None,
    };
    for v in [down, proj] {
        let json = serde_json::to_string(&v).unwrap();
        let back: VolumeType = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            back,
            VolumeType::DownwardAPI { .. } | VolumeType::Projected { .. }
        ));
    }
}

#[test]
fn volume_type_cloud_disks_roundtrip() {
    let aws = VolumeType::AwsElasticBlockStore {
        volume_id: "vol-abc".into(),
        fs_type: "ext4".into(),
        partition: Some(1),
        read_only: false,
    };
    let azure = VolumeType::AzureDisk {
        disk_name: "d1".into(),
        disk_uri: "https://disk".into(),
        caching_mode: Some(AzureCachingMode::ReadOnly),
        fs_type: Some("ext4".into()),
        read_only: true,
        kind: Some(AzureDiskKind::Dedicated),
    };
    let gce = VolumeType::GcePersistentDisk {
        pd_name: "pd1".into(),
        fs_type: "ext4".into(),
        partition: None,
        read_only: true,
    };
    for v in [aws, azure, gce] {
        let json = serde_json::to_string(&v).unwrap();
        let back: VolumeType = serde_json::from_str(&json).unwrap();
        let _ = format!("{back:?}");
    }
}

#[test]
fn volume_projection_downward_api_and_service_account_roundtrip() {
    let d = VolumeProjection::DownwardAPI {
        items: vec![DownwardAPIVolumeFile {
            path: "cpu".into(),
            field_ref: None,
            resource_field_ref: Some(ResourceFieldSelector {
                container_name: Some("app".into()),
                resource: "limits.cpu".into(),
                divisor: Some("1m".into()),
            }),
            mode: None,
        }],
    };
    let sat = VolumeProjection::ServiceAccountToken {
        audience: Some("api".into()),
        expiration_seconds: Some(3600),
        path: "token".into(),
    };
    for p in [d, sat] {
        let json = serde_json::to_string(&p).unwrap();
        let back: VolumeProjection = serde_json::from_str(&json).unwrap();
        let _ = format!("{back:?}");
    }
}

#[test]
fn volume_spec_full_roundtrip() {
    let spec = VolumeSpec {
        metadata: VolumeMetadata {
            name: "vol-a".into(),
            description: Some("d".into()),
            labels: HashMap::from([("a".into(), "b".into())]),
            annotations: HashMap::new(),
        },
        volume_type: VolumeType::EmptyDir {
            medium: None,
            size_limit: None,
        },
        mount_options: vec!["rw".into()],
        access_modes: vec![VolumeAccessMode::ReadWriteOnce],
        capacity: Some(VolumeCapacity {
            storage: "10Gi".into(),
        }),
        storage_class: Some("fast".into()),
        reclaim_policy: VolumeReclaimPolicy::Retain,
        volume_mode: VolumeMode::Block,
        node_affinity: None,
    };
    let json = serde_json::to_string(&spec).unwrap();
    let back: VolumeSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(back.metadata.name, spec.metadata.name);
    assert!(matches!(back.volume_mode, VolumeMode::Block));
}
