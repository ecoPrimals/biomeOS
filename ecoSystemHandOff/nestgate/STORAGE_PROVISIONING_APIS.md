# NestGate Storage Provisioning & ZFS Integration APIs

**Status:** Information Gathering | **Source:** nestgate codebase analysis | **Date:** January 2025

---

## ZFS Management Architecture

NestGate provides comprehensive ZFS management through a sophisticated API structure:

```rust
pub struct ZfsManager {
    config: ZfsConfig,
    pool_manager: Arc<ZfsPoolManager>,
    dataset_manager: Arc<ZfsDatasetManager>,
    snapshot_manager: Arc<ZfsSnapshotManager>,
}
```

## Storage Tier System

### Tier Configuration
From `nestgate/code/crates/nestgate-zfs/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfigurations {
    pub hot: TierConfig,
    pub warm: TierConfig,
    pub cold: TierConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub compression: CompressionAlgorithm,
    pub record_size: u64,
    pub properties: HashMap<String, String>,
    pub performance_profile: PerformanceProfile,
}
```

### Storage Tiers Implementation
```yaml
storage_tiers:
  hot:
    path: /nestpool/hot
    compression: lz4
    recordsize: 128K
    atime: off
    
  warm:
    path: /nestpool/warm
    compression: zstd
    recordsize: 1M
    atime: off
    
  cold:
    path: /nestpool/cold
    compression: zstd-19
    recordsize: 1M
    atime: off
```

## Dataset Management APIs

### Dataset Creation
From `nestgate/code/crates/nestgate-api/src/handlers/zfs.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatasetRequest {
    /// Dataset name
    pub name: String,
    /// Parent pool or dataset
    pub parent: String,
    /// Storage tier
    pub tier: StorageTier,
    /// Dataset properties
    pub properties: Option<HashMap<String, String>>,
}
```

### HTTP API Endpoints
- `POST /api/datasets` - Create dataset
- `GET /api/datasets` - List datasets
- `GET /api/datasets/{name}` - Get dataset info
- `DELETE /api/datasets/{name}` - Destroy dataset
- `PUT /api/datasets/{name}/properties` - Set dataset properties
- `GET /api/datasets/{name}/properties` - Get dataset properties

### Dataset Information Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub name: String,
    pub used_space: u64,
    pub available_space: u64,
    pub file_count: Option<u64>,
    pub compression_ratio: Option<f64>,
    pub mount_point: String,
    pub tier: StorageTier,
    pub properties: HashMap<String, String>,
}
```

## Volume Provisioning

### Volume Management
From `nestgate/code/crates/nestgate-mcp/src/storage.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub size_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub tier: StorageTier,
    pub mount_point: String,
    pub filesystem: String,
    pub mounted: bool,
    pub health: String,
}
```

### MCP Integration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpVolumeRequest {
    pub volume_id: String,
    pub tier: StorageTier,
    pub size_gb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMountRequest {
    pub mount_id: String,
    pub mount_point: String,
    pub tier: StorageTier,
    pub size_gb: u64,
}
```

## Network Integration

### Songbird Integration
From `nestgate/code/crates/nestgate-network/src/songbird.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub name: String,
    pub service_type: String,
    pub version: String,
    pub address: String,
    pub port: u16,
    pub endpoints: Vec<String>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub health_endpoint: String,
}
```

### Connection Management
From `nestgate/code/crates/nestgate-network/src/connection_manager.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    Api,
    Nfs,
    Smb,
    Iscsi,
    S3,
    Internal(String),
    Health,
    Metrics,
}
```

## Protocol Support

### Multi-Protocol Access
NestGate supports multiple storage protocols:

- **NFS**: Network File System
- **SMB**: Server Message Block (CIFS)
- **iSCSI**: Internet Small Computer Systems Interface
- **S3**: S3-compatible object storage
- **API**: RESTful HTTP API

### Protocol Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub nfs: Option<NfsConfig>,
    pub smb: Option<SmbConfig>,
    pub iscsi: Option<IscsiConfig>,
    pub s3: Option<S3Config>,
}
```

## Snapshot Management

### Snapshot Operations
From `nestgate/code/crates/nestgate-api/src/handlers/zfs.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSnapshotRequest {
    pub name: String,
    pub dataset: String,
    pub recursive: Option<bool>,
    pub properties: Option<HashMap<String, String>>,
}
```

### Snapshot API Endpoints
- `POST /api/snapshots` - Create snapshot
- `GET /api/snapshots` - List snapshots
- `GET /api/snapshots/{name}` - Get snapshot info
- `DELETE /api/snapshots/{name}` - Destroy snapshot

## Performance & Monitoring

### Performance Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub cache_size: Option<u64>,
    pub read_ahead: Option<u64>,
    pub write_cache: Option<bool>,
    pub sync_mode: Option<SyncMode>,
}
```

### Metrics Collection
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_capacity: u64,
    pub used_capacity: u64,
    pub available_capacity: u64,
    pub compression_ratio: f64,
    pub dedup_ratio: f64,
    pub read_iops: u64,
    pub write_iops: u64,
    pub read_throughput: u64,
    pub write_throughput: u64,
}
```

## Tier Migration

### Migration Management
From `nestgate/code/crates/nestgate-api/src/handlers/zfs.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierMigrationRequest {
    pub dataset_path: String,
    pub source_tier: StorageTier,
    pub target_tier: StorageTier,
    pub priority: Option<u8>,
    pub force: Option<bool>,
}
```

### Migration API
- `POST /api/storage/migrate` - Start tier migration
- `GET /api/storage/migrations` - List active migrations
- `GET /api/storage/migrations/{id}` - Get migration status

## Security & Encryption

### Encryption Support
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: EncryptionAlgorithm,
    pub key_source: KeySource,
    pub key_format: KeyFormat,
}
```

### Access Control
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub read_only: bool,
    pub allowed_hosts: Vec<String>,
    pub allowed_networks: Vec<String>,
    pub authentication_required: bool,
}
```

## AI & Automation

### AI Optimization
From `nestgate/code/crates/nestgate-zfs/src/mcp_integration.rs`:

```rust
impl ZfsMcpStorageProvider {
    pub async fn trigger_ai_optimization(&self) -> Result<()> {
        if !self.config.enable_ai_optimization {
            return Err(nestgate_core::NestGateError::Internal(
                "AI optimization not enabled".to_string(),
            ));
        }
        // AI optimization implementation
    }
}
```

### Automation Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetAutomationConfig {
    pub auto_snapshot: bool,
    pub snapshot_schedule: String,
    pub auto_cleanup: bool,
    pub retention_policy: RetentionPolicy,
    pub auto_migration: bool,
    pub migration_thresholds: MigrationThresholds,
}
```

## Integration Points for biomeOS

### 1. Volume Provisioning for Primals ✅
- **Already Implemented**: MCP volume creation/management
- **Already Implemented**: Tier-aware storage provisioning
- **Already Implemented**: Mount point management
- 🔄 **Needs Enhancement**: biome.yaml volume definitions

### 2. Multi-Protocol Access ✅
- **Already Implemented**: NFS, SMB, iSCSI, S3 protocols
- **Already Implemented**: Protocol-specific configuration
- **Already Implemented**: Connection management
- 🔄 **Needs Enhancement**: Primal-specific protocol routing

### 3. Songbird Integration ✅
- **Already Implemented**: Service registration with Songbird
- **Already Implemented**: Health monitoring endpoints
- **Already Implemented**: Connection management through Songbird
- 🔄 **Needs Enhancement**: biomeOS service discovery patterns

### 4. ZFS Management ✅
- **Already Implemented**: Complete ZFS dataset/pool management
- **Already Implemented**: Snapshot and backup operations
- **Already Implemented**: Performance monitoring
- 🔄 **Needs Enhancement**: Automated provisioning from manifests

### 5. Security & Encryption ✅
- **Already Implemented**: ZFS encryption support
- **Already Implemented**: Access control policies
- **Already Implemented**: Authentication integration
- 🔄 **Needs Enhancement**: BearDog security policy enforcement

## Storage API Examples

### Creating a Volume for Another Primal
```bash
# Create a volume for BearDog
curl -X POST http://nestgate/api/volumes \
  -H "Content-Type: application/json" \
  -d '{
    "name": "beardog-data",
    "size_gb": 100,
    "tier": "warm",
    "access_policy": {
      "read_only": false,
      "allowed_services": ["beardog"]
    }
  }'
```

### Dataset Creation with Tier
```bash
# Create a dataset in hot tier
curl -X POST http://nestgate/api/datasets \
  -H "Content-Type: application/json" \
  -d '{
    "name": "squirrel-cache",
    "parent": "nestpool",
    "tier": "hot",
    "properties": {
      "compression": "lz4",
      "recordsize": "128K"
    }
  }'
```

## Conclusion

**NestGate provides comprehensive storage infrastructure** for biomeOS:

- **ZFS Management**: ✅ Complete pool, dataset, and snapshot management
- **Tiered Storage**: ✅ Hot/warm/cold tier automation
- **Multi-Protocol**: ✅ NFS, SMB, iSCSI, S3 support
- **Volume Provisioning**: ✅ MCP-based volume management
- **Songbird Integration**: ✅ Service discovery and connection management
- **Performance Monitoring**: ✅ Real-time metrics and optimization
- **Security**: ✅ Encryption and access control

**Ready for biomeOS Integration:**
1. Already supports volume provisioning for other services
2. Integrates with Songbird for service discovery
3. Provides comprehensive ZFS management APIs
4. Supports multiple access protocols

**Next Steps:**
1. Implement biome.yaml volume definition parsing
2. Add automated provisioning from manifest
3. Integrate BearDog security policies
4. Create Primal-specific storage templates 