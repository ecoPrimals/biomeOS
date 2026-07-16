use super::super::*;
use std::collections::HashMap;

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
