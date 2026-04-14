// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Volume manifest types (`VolumeSpec`, access modes, selectors, projections).

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

    /// Ephemeral (boxed to reduce enum size - `clippy::large_enum_variant`)
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
