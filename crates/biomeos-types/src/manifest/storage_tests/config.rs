use super::super::*;
use std::collections::HashMap;

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
fn config_data_yaml_json_toml_roundtrip() {
    let y = ConfigData::Yaml(serde_json::Value::String("x".into()));
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
