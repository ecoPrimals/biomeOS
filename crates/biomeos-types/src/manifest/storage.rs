// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Storage Specifications
//!
//! This module contains storage-related manifest types including VolumeSpec,
//! SecretSpec, ConfigSpec, and storage configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Volume specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    /// Volume metadata
    pub metadata: VolumeMetadata,

    /// Volume type
    pub volume_type: VolumeType,

    /// Mount options
    pub mount_options: Vec<String>,

    /// Access modes
    pub access_modes: Vec<VolumeAccessMode>,

    /// Capacity
    pub capacity: Option<VolumeCapacity>,

    /// Storage class
    pub storage_class: Option<String>,

    /// Reclaim policy
    pub reclaim_policy: VolumeReclaimPolicy,

    /// Volume mode
    pub volume_mode: VolumeMode,

    /// Node affinity
    pub node_affinity: Option<VolumeNodeAffinity>,
}

/// Volume metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMetadata {
    /// Volume name
    pub name: String,

    /// Volume description
    pub description: Option<String>,

    /// Volume labels
    pub labels: HashMap<String, String>,

    /// Volume annotations
    pub annotations: HashMap<String, String>,
}

/// Volume types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeType {
    /// Empty directory
    EmptyDir {
        /// Medium (memory or disk)
        medium: Option<EmptyDirMedium>,
        /// Size limit
        size_limit: Option<String>,
    },

    /// Host path
    HostPath {
        /// Path on host
        path: String,
        /// Path type
        path_type: Option<HostPathType>,
    },

    /// Network File System
    Nfs {
        /// Server
        server: String,
        /// Path
        path: String,
        /// Read only
        read_only: bool,
    },

    /// Persistent Volume Claim
    PersistentVolumeClaim {
        /// Claim name
        claim_name: String,
        /// Read only
        read_only: bool,
    },

    /// Config Map
    ConfigMap {
        /// Config map name
        name: String,
        /// Items
        items: Vec<KeyToPath>,
        /// Default mode
        default_mode: Option<u32>,
        /// Optional
        optional: bool,
    },

    /// Secret
    Secret {
        /// Secret name
        secret_name: String,
        /// Items
        items: Vec<KeyToPath>,
        /// Default mode
        default_mode: Option<u32>,
        /// Optional
        optional: bool,
    },

    /// Downward API
    DownwardAPI {
        /// Items
        items: Vec<DownwardAPIVolumeFile>,
        /// Default mode
        default_mode: Option<u32>,
    },

    /// Projected
    Projected {
        /// Sources
        sources: Vec<VolumeProjection>,
        /// Default mode
        default_mode: Option<u32>,
    },

    /// CSI (Container Storage Interface)
    Csi {
        /// Driver
        driver: String,
        /// Volume attributes
        volume_attributes: HashMap<String, String>,
        /// Node publish secret ref
        node_publish_secret_ref: Option<SecretReference>,
        /// Read only
        read_only: bool,
        /// FS type
        fs_type: Option<String>,
    },

    /// Ephemeral (boxed to reduce enum size - clippy::large_enum_variant)
    Ephemeral {
        /// Volume claim template (boxed to reduce enum size)
        volume_claim_template: Box<PersistentVolumeClaimTemplate>,
    },

    /// AWS Elastic Block Store
    AwsElasticBlockStore {
        /// Volume ID
        volume_id: String,
        /// FS type
        fs_type: String,
        /// Partition
        partition: Option<u32>,
        /// Read only
        read_only: bool,
    },

    /// Azure Disk
    AzureDisk {
        /// Disk name
        disk_name: String,
        /// Disk URI
        disk_uri: String,
        /// Caching mode
        caching_mode: Option<AzureCachingMode>,
        /// FS type
        fs_type: Option<String>,
        /// Read only
        read_only: bool,
        /// Kind
        kind: Option<AzureDiskKind>,
    },

    /// Google Compute Engine Persistent Disk
    GcePersistentDisk {
        /// PD name
        pd_name: String,
        /// FS type
        fs_type: String,
        /// Partition
        partition: Option<u32>,
        /// Read only
        read_only: bool,
    },

    /// Custom volume type
    Custom {
        /// Type name
        type_name: String,
        /// Configuration
        config: HashMap<String, serde_json::Value>,
    },
}

/// Empty directory medium
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmptyDirMedium {
    /// Default (usually disk)
    Default,
    /// Memory
    Memory,
    /// Huge pages
    HugePages,
}

/// Host path types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostPathType {
    /// Unset
    Unset,
    /// Directory or create
    DirectoryOrCreate,
    /// Directory
    Directory,
    /// File or create
    FileOrCreate,
    /// File
    File,
    /// Socket
    Socket,
    /// Char device
    CharDevice,
    /// Block device
    BlockDevice,
}

/// Key to path mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyToPath {
    /// Key
    pub key: String,
    /// Path
    pub path: String,
    /// Mode
    pub mode: Option<u32>,
}

/// Downward API volume file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownwardAPIVolumeFile {
    /// Path
    pub path: String,
    /// Field ref
    pub field_ref: Option<ObjectFieldSelector>,
    /// Resource field ref
    pub resource_field_ref: Option<ResourceFieldSelector>,
    /// Mode
    pub mode: Option<u32>,
}

/// Object field selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectFieldSelector {
    /// API version
    pub api_version: String,
    /// Field path
    pub field_path: String,
}

/// Resource field selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFieldSelector {
    /// Container name
    pub container_name: Option<String>,
    /// Resource
    pub resource: String,
    /// Divisor
    pub divisor: Option<String>,
}

/// Volume projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeProjection {
    /// Secret
    Secret {
        /// Local object reference
        local_object_reference: LocalObjectReference,
        /// Items
        items: Vec<KeyToPath>,
        /// Optional
        optional: bool,
    },
    /// Config map
    ConfigMap {
        /// Local object reference
        local_object_reference: LocalObjectReference,
        /// Items
        items: Vec<KeyToPath>,
        /// Optional
        optional: bool,
    },
    /// Downward API
    DownwardAPI {
        /// Items
        items: Vec<DownwardAPIVolumeFile>,
    },
    /// Service account token
    ServiceAccountToken {
        /// Audience
        audience: Option<String>,
        /// Expiration seconds
        expiration_seconds: Option<i64>,
        /// Path
        path: String,
    },
}

/// Local object reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalObjectReference {
    /// Name
    pub name: String,
}

/// Secret reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretReference {
    /// Name
    pub name: String,
    /// Namespace
    pub namespace: Option<String>,
}

/// Persistent volume claim template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimTemplate {
    /// Metadata
    pub metadata: VolumeMetadata,
    /// Spec
    pub spec: PersistentVolumeClaimSpec,
}

/// Persistent volume claim spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimSpec {
    /// Access modes
    pub access_modes: Vec<VolumeAccessMode>,
    /// Resources
    pub resources: VolumeResourceRequirements,
    /// Volume name
    pub volume_name: Option<String>,
    /// Storage class
    pub storage_class: Option<String>,
    /// Volume mode
    pub volume_mode: Option<VolumeMode>,
    /// Selector
    pub selector: Option<LabelSelector>,
}

/// Volume resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeResourceRequirements {
    /// Limits
    pub limits: HashMap<String, String>,
    /// Requests
    pub requests: HashMap<String, String>,
}

/// Label selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    /// Match labels
    pub match_labels: HashMap<String, String>,
    /// Match expressions
    pub match_expressions: Vec<LabelSelectorRequirement>,
}

/// Label selector requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    /// Key
    pub key: String,
    /// Operator
    pub operator: LabelSelectorOperator,
    /// Values
    pub values: Vec<String>,
}

/// Label selector operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelSelectorOperator {
    /// Value is in the set
    In,
    /// Value is not in the set
    NotIn,
    /// Key exists
    Exists,
    /// Key does not exist
    DoesNotExist,
}

/// Azure caching modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AzureCachingMode {
    /// No caching
    None,
    /// Read-only caching
    ReadOnly,
    /// Read-write caching
    ReadWrite,
}

/// Azure disk kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AzureDiskKind {
    /// Shared disk
    Shared,
    /// Dedicated disk
    Dedicated,
    /// Managed disk
    Managed,
}

/// Volume access modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeAccessMode {
    /// Read-write by a single node
    ReadWriteOnce,
    /// Read-only by many nodes
    ReadOnlyMany,
    /// Read-write by many nodes
    ReadWriteMany,
    /// Read-write by a single pod
    ReadWriteOncePod,
}

/// Volume capacity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeCapacity {
    /// Storage
    pub storage: String,
}

/// Volume reclaim policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeReclaimPolicy {
    /// Retain the volume after release
    Retain,
    /// Recycle the volume (scrub data)
    Recycle,
    /// Delete the volume after release
    Delete,
}

/// Volume modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeMode {
    /// Filesystem-backed volume
    Filesystem,
    /// Raw block device volume
    Block,
}

/// Volume node affinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeNodeAffinity {
    /// Required
    pub required: Option<NodeSelector>,
}

/// Node selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelector {
    /// Node selector terms
    pub node_selector_terms: Vec<NodeSelectorTerm>,
}

/// Node selector term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorTerm {
    /// Match expressions
    pub match_expressions: Vec<NodeSelectorRequirement>,
    /// Match fields
    pub match_fields: Vec<NodeSelectorRequirement>,
}

/// Node selector requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorRequirement {
    /// Key
    pub key: String,
    /// Operator
    pub operator: NodeSelectorOperator,
    /// Values
    pub values: Vec<String>,
}

/// Node selector operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeSelectorOperator {
    /// Value is in the set
    In,
    /// Value is not in the set
    NotIn,
    /// Key exists
    Exists,
    /// Key does not exist
    DoesNotExist,
    /// Value is greater than
    Gt,
    /// Value is less than
    Lt,
}

/// Secret specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSpec {
    /// Secret metadata
    pub metadata: SecretMetadata,

    /// Secret type
    pub secret_type: SecretType,

    /// Secret data
    pub data: HashMap<String, SecretData>,

    /// Immutable
    pub immutable: bool,
}

/// Secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// Secret name
    pub name: String,

    /// Secret description
    pub description: Option<String>,

    /// Secret labels
    pub labels: HashMap<String, String>,

    /// Secret annotations
    pub annotations: HashMap<String, String>,
}

/// Secret types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretType {
    /// Opaque secret
    Opaque,
    /// Service account token
    ServiceAccountToken,
    /// Docker config
    DockerConfig,
    /// Docker config JSON
    DockerConfigJson,
    /// Basic auth
    BasicAuth,
    /// SSH auth
    SshAuth,
    /// TLS secret
    Tls,
    /// Bootstrap token
    BootstrapToken,
    /// Custom type
    Custom(String),
}

/// Secret data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretData {
    /// Base64 encoded data
    Base64(String),
    /// Plain text data
    Text(String),
    /// File reference
    File(String),
    /// External reference
    External(ExternalSecretRef),
}

/// External secret reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSecretRef {
    /// Provider
    pub provider: SecretProvider,
    /// Key
    pub key: String,
    /// Version
    pub version: Option<String>,
}

/// Secret providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretProvider {
    /// AWS Secrets Manager
    AwsSecretsManager {
        /// AWS region
        region: String,
    },
    /// Azure Key Vault
    AzureKeyVault {
        /// Vault URL
        vault_url: String,
    },
    /// Google Secret Manager
    GoogleSecretManager {
        /// GCP project ID
        project_id: String,
    },
    /// HashiCorp Vault
    Vault {
        /// Vault server address
        address: String,
        /// Secret path within vault
        path: String,
    },
    /// Kubernetes secret
    Kubernetes {
        /// Kubernetes namespace
        namespace: String,
        /// Secret name
        name: String,
    },
    /// Custom provider
    Custom {
        /// Provider name
        provider_name: String,
        /// Provider configuration
        config: HashMap<String, String>,
    },
}

/// Config specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSpec {
    /// Config metadata
    #[serde(default)]
    pub metadata: ConfigMetadata,

    /// Config data
    #[serde(default)]
    pub data: HashMap<String, ConfigData>,

    /// Binary data
    #[serde(default)]
    pub binary_data: HashMap<String, Vec<u8>>,

    /// Immutable
    #[serde(default)]
    pub immutable: bool,
}

/// Config metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Config name
    pub name: String,

    /// Config description
    pub description: Option<String>,

    /// Config labels
    pub labels: HashMap<String, String>,

    /// Config annotations
    pub annotations: HashMap<String, String>,
}

/// Config data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigData {
    /// String value
    String(String),
    /// YAML value
    Yaml(serde_yaml::Value),
    /// JSON value
    Json(serde_json::Value),
    /// TOML value
    Toml(String),
    /// Properties value
    Properties(HashMap<String, String>),
    /// File reference
    File(String),
    /// Template
    Template {
        /// Template string
        template: String,
        /// Template variables
        variables: HashMap<String, String>,
    },
}

/// Storage class specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClassSpec {
    /// Storage class metadata
    pub metadata: StorageClassMetadata,

    /// Provisioner
    pub provisioner: String,

    /// Parameters
    pub parameters: HashMap<String, String>,

    /// Reclaim policy
    pub reclaim_policy: VolumeReclaimPolicy,

    /// Allow volume expansion
    pub allow_volume_expansion: bool,

    /// Volume binding mode
    pub volume_binding_mode: VolumeBindingMode,

    /// Allowed topologies
    pub allowed_topologies: Vec<TopologySelectorTerm>,
}

/// Storage class metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClassMetadata {
    /// Storage class name
    pub name: String,

    /// Storage class description
    pub description: Option<String>,

    /// Storage class labels
    pub labels: HashMap<String, String>,

    /// Storage class annotations
    pub annotations: HashMap<String, String>,
}

/// Volume binding modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeBindingMode {
    /// Bind immediately when claim is created
    Immediate,
    /// Wait for first consumer pod to bind
    WaitForFirstConsumer,
}

/// Topology selector term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySelectorTerm {
    /// Match label expressions
    pub match_label_expressions: Vec<TopologySelectorLabelRequirement>,
}

/// Topology selector label requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySelectorLabelRequirement {
    /// Key
    pub key: String,
    /// Values
    pub values: Vec<String>,
}

/// Default implementations
impl Default for VolumeSpec {
    fn default() -> Self {
        Self {
            metadata: VolumeMetadata::default(),
            volume_type: VolumeType::EmptyDir {
                medium: None,
                size_limit: None,
            },
            mount_options: Vec::new(),
            access_modes: vec![VolumeAccessMode::ReadWriteOnce],
            capacity: None,
            storage_class: None,
            reclaim_policy: VolumeReclaimPolicy::Delete,
            volume_mode: VolumeMode::Filesystem,
            node_affinity: None,
        }
    }
}

impl Default for VolumeMetadata {
    fn default() -> Self {
        Self {
            name: "default-volume".to_string(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}

impl Default for SecretSpec {
    fn default() -> Self {
        Self {
            metadata: SecretMetadata::default(),
            secret_type: SecretType::Opaque,
            data: HashMap::new(),
            immutable: false,
        }
    }
}

impl Default for SecretMetadata {
    fn default() -> Self {
        Self {
            name: "default-secret".to_string(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        Self {
            name: "default-config".to_string(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
