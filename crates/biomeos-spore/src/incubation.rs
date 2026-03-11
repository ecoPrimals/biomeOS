//! Spore incubation - Deploy spores on local computers with local entropy mixing
//!
//! This module enables USB spores to be deployed on multiple computers while maintaining
//! genetic lineage for federation. Each deployment mixes the spore's genetic seed with
//! local computer entropy to create a unique deployed node identity.
//!
//! # Architecture
//!
//! ```text
//! USB Spore (Genetic Seed)
//!   ├─> Computer A → deployed_seed = SHA256(spore_seed || entropy_A)
//!   └─> Computer B → deployed_seed = SHA256(spore_seed || entropy_B)
//!
//! Both nodes:
//!   - Share genetic lineage (can federate)
//!   - Have unique local identity
//!   - Recognize each other as siblings
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::error::SporeResult;
use crate::seed::FamilySeed;
use crate::spore_log_tracker::SporeLogTracker;

/// Local entropy gathered from the computer during incubation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEntropy {
    /// System hostname
    pub hostname: String,

    /// Machine ID (from /etc/machine-id or generated)
    pub machine_id: String,

    /// Timestamp of incubation
    pub timestamp: DateTime<Utc>,

    /// Primary network interface MAC address
    pub mac_address: Option<String>,

    /// Random nonce for additional entropy
    #[serde(with = "serde_bytes")]
    pub random_nonce: Vec<u8>,

    /// CPU info hash (optional)
    pub cpu_hash: Option<String>,

    /// Disk serial (optional, for additional uniqueness)
    pub disk_serial: Option<String>,
}

impl LocalEntropy {
    /// Generate local entropy from the current computer
    ///
    /// This gathers system-specific information to create unique entropy
    /// that will be mixed with the spore seed.
    pub fn generate(computer_name: Option<&str>) -> SporeResult<Self> {
        info!("Generating local entropy for incubation");

        // 1. Hostname
        let hostname = computer_name
            .map(|s| s.to_string())
            .or_else(|| std::env::var("HOSTNAME").ok())
            .or_else(|| std::env::var("COMPUTERNAME").ok())
            .unwrap_or_else(|| {
                warn!("Could not determine hostname, using default");
                "unknown-host".to_string()
            });

        // 2. Machine ID
        let machine_id = Self::get_machine_id()?;

        // 3. Timestamp
        let timestamp = Utc::now();

        // 4. MAC address (optional, best effort)
        let mac_address = Self::get_primary_mac_address().ok();

        // 5. Random nonce (32 bytes)
        let mut random_nonce = vec![0u8; 32];
        getrandom::getrandom(&mut random_nonce).context("Failed to generate random nonce")?;

        // 6. CPU hash (optional)
        let cpu_hash = Self::get_cpu_hash().ok();

        // 7. Disk serial (optional)
        let disk_serial = Self::get_disk_serial().ok();

        debug!("Generated entropy for hostname: {}", hostname);

        Ok(Self {
            hostname,
            machine_id,
            timestamp,
            mac_address,
            random_nonce,
            cpu_hash,
            disk_serial,
        })
    }

    /// Calculate SHA256 hash of all entropy components
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();

        hasher.update(self.hostname.as_bytes());
        hasher.update(self.machine_id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());

        if let Some(ref mac) = self.mac_address {
            hasher.update(mac.as_bytes());
        }

        hasher.update(&self.random_nonce);

        if let Some(ref cpu) = self.cpu_hash {
            hasher.update(cpu.as_bytes());
        }

        if let Some(ref disk) = self.disk_serial {
            hasher.update(disk.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Get machine ID from system
    fn get_machine_id() -> SporeResult<String> {
        // Try /etc/machine-id (Linux)
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            return Ok(id.trim().to_string());
        }

        // Try /var/lib/dbus/machine-id (Linux fallback)
        if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            return Ok(id.trim().to_string());
        }

        // Generate a stable UUID based on hostname + user
        warn!("Could not read machine-id, generating stable fallback");
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        let mut hasher = Sha256::new();
        hasher.update(hostname.as_bytes());
        hasher.update(user.as_bytes());
        hasher.update(b"biomeos-machine-id");

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get primary network interface MAC address
    ///
    /// On Linux: reads from `/sys/class/net/<interface>/address` for the first
    /// non-loopback interface. Falls back to placeholder if no interfaces found.
    fn get_primary_mac_address() -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            let net_dir = "/sys/class/net";
            if let Ok(entries) = std::fs::read_dir(net_dir) {
                let mut ifaces: Vec<std::path::PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        !name.starts_with("lo") // Skip loopback only
                    })
                    .collect();
                ifaces.sort(); // Deterministic: prefer eth0, enp0s3, etc. over wlan0

                for path in ifaces {
                    let iface_str = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    let mac_path = format!("/sys/class/net/{}/address", iface_str);
                    if let Ok(mac) = std::fs::read_to_string(&mac_path) {
                        let mac = mac.trim().to_string();
                        if !mac.is_empty() {
                            return Ok(mac);
                        }
                    }
                }
            }
        }

        // Fallback: no suitable interface found (non-Linux or no interfaces)
        Ok("00:00:00:00:00:00".to_string())
    }

    /// Get CPU info hash
    fn get_cpu_hash() -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
                let mut hasher = Sha256::new();
                hasher.update(cpuinfo.as_bytes());
                return Ok(format!("{:x}", hasher.finalize()));
            }
        }

        Ok("unknown-cpu".to_string())
    }

    /// Get disk serial
    fn get_disk_serial() -> Result<String> {
        // This is platform-specific and best-effort
        // In production, might use a crate or system calls
        Ok("unknown-disk".to_string())
    }
}

/// Result of spore incubation on local computer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncubatedNode {
    /// Unique node ID (e.g., "node-spore1-laptop")
    pub node_id: String,

    /// Spore ID this node was incubated from
    pub spore_id: String,

    /// Deployed node seed (spore_seed + local_entropy)
    pub deployed_seed_hash: String,

    /// Path to local configuration
    pub local_config_path: PathBuf,

    /// When this node was incubated
    pub incubated_at: DateTime<Utc>,

    /// Local entropy hash
    pub entropy_hash: String,

    /// Original spore path (if still accessible)
    pub spore_path: Option<PathBuf>,
}

/// Configuration stored locally for an incubated node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node identity and metadata
    pub node: NodeInfo,
    /// Genetic lineage chain hashes
    pub lineage: LineageInfo,
    /// Source spore metadata
    pub spore: SporeInfo,
    /// Federation membership details
    pub federation: FederationInfo,
}

/// Information about the deployed node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Spore identifier this node was deployed from
    pub spore_id: String,
    /// Unique node identifier (derived from entropy mixing)
    pub node_id: String,
    /// Deployment timestamp
    pub deployed_at: DateTime<Utc>,
    /// Computer hostname at deployment time
    pub computer_name: String,
    /// SHA-256 hash of the local entropy used
    pub entropy_hash: String,
}

/// Lineage hash chain for verifying genetic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    /// SHA-256 hash of the parent (originating) seed
    pub parent_seed_hash: String,
    /// SHA-256 hash of the spore's seed
    pub spore_seed_hash: String,
    /// SHA-256 hash of the deployed (entropy-mixed) seed
    pub deployed_seed_hash: String,
}

/// Source spore tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeInfo {
    /// Filesystem path to the original spore (if still mounted)
    pub original_path: Option<PathBuf>,
    /// Last time the spore was accessed
    pub last_seen: DateTime<Utc>,
    /// Number of times this spore has been deployed
    pub deployment_count: usize,
}

/// Federation membership for the deployed node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationInfo {
    /// Family identifier (genetic root)
    pub family_id: String,
    /// Sub-federation memberships
    pub sub_federations: Vec<String>,
}

/// Spore incubator - Handles deployment of spores on local computers
pub struct SporeIncubator {
    spore_path: PathBuf,
    _spore_seed: FamilySeed,
}

impl SporeIncubator {
    /// Create a new incubator for a spore
    pub fn new(spore_path: impl AsRef<Path>) -> SporeResult<Self> {
        let spore_path = spore_path.as_ref().to_path_buf();

        info!("Creating incubator for spore at: {}", spore_path.display());

        // Load spore seed
        let seed_path = spore_path.join(".family.seed");
        let spore_seed = FamilySeed::from_file(&seed_path)?;

        Ok(Self {
            spore_path,
            _spore_seed: spore_seed,
        })
    }

    /// Incubate this spore on the local computer
    ///
    /// This creates a deployed node by mixing the spore seed with local entropy.
    /// The result is stored in ~/.config/biomeos/deployed-nodes/{spore-id}/
    ///
    /// # Arguments
    ///
    /// * `computer_name` - Optional name for this computer (uses hostname if None)
    /// * `deploy_local` - If true, also create local deployment in /tmp
    pub async fn incubate(
        &self,
        computer_name: Option<&str>,
        _deploy_local: bool, // Future: local vs remote deployment choice
    ) -> SporeResult<IncubatedNode> {
        info!("Incubating spore on local computer");

        // 1. Generate local entropy
        let local_entropy = LocalEntropy::generate(computer_name)?;
        let entropy_hash = local_entropy.hash();

        debug!("Generated local entropy hash: {}", entropy_hash);

        // 2. Derive deployed node seed
        let deployed_seed = self.derive_deployed_seed(&local_entropy)?;
        let deployed_seed_hash = Self::hash_seed(&deployed_seed);

        // 3. Determine spore ID and node ID
        let spore_id = self.extract_spore_id()?;
        let node_id = format!("node-{}-{}", spore_id, local_entropy.hostname);

        info!("Creating incubated node: {}", node_id);

        // 4. Create local configuration
        let local_config_path = self.get_local_config_path(&spore_id)?;
        self.create_local_config(
            &local_config_path,
            &spore_id,
            &node_id,
            &deployed_seed_hash,
            &entropy_hash,
            &local_entropy,
        )
        .await?;

        // 5. Store deployed seed securely
        self.store_deployed_seed(&local_config_path, &deployed_seed)
            .await?;

        // 6. Log incubation to spore
        let log_tracker = SporeLogTracker::new(&self.spore_path)?;
        log_tracker.initialize().await?;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("computer_name".to_string(), local_entropy.hostname.clone());
        metadata.insert("entropy_hash".to_string(), entropy_hash.clone());
        metadata.insert("node_id".to_string(), node_id.clone());

        log_tracker
            .record_event(crate::spore_log_tracker::SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: crate::spore_log_tracker::SporeEventType::Custom(
                    "incubation".to_string(),
                ),
                node_id: Some(node_id.clone()),
                deployed_to: Some(local_entropy.hostname.clone()),
                metadata,
            })
            .await?;

        info!("✅ Spore incubated successfully as node: {}", node_id);

        Ok(IncubatedNode {
            node_id,
            spore_id,
            deployed_seed_hash,
            local_config_path,
            incubated_at: Utc::now(),
            entropy_hash,
            spore_path: Some(self.spore_path.clone()),
        })
    }

    /// Derive deployed node seed from spore seed + local entropy
    ///
    /// Formula: deployed_seed = SHA256(spore_seed || local_entropy_hash)
    ///
    /// This ensures:
    /// - Each deployment is unique
    /// - Same spore on different computers = different seeds
    /// - Deterministic (same computer + same spore = same seed)
    fn derive_deployed_seed(&self, local_entropy: &LocalEntropy) -> SporeResult<Vec<u8>> {
        let spore_seed_bytes = std::fs::read(self.spore_path.join(".family.seed"))?;
        let entropy_hash = local_entropy.hash();

        let mut hasher = Sha256::new();
        hasher.update(&spore_seed_bytes);
        hasher.update(entropy_hash.as_bytes());

        Ok(hasher.finalize().to_vec())
    }

    /// Hash a seed for display/storage
    fn hash_seed(seed: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        format!("{:x}", hasher.finalize())
    }

    /// Extract spore ID from spore path or config
    fn extract_spore_id(&self) -> SporeResult<String> {
        // Try to read from .spore.json or tower.toml
        let tower_toml_path = self.spore_path.join("tower.toml");

        if tower_toml_path.exists() {
            let content = std::fs::read_to_string(&tower_toml_path)?;

            // Parse TOML and extract node_id from meta section
            if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                if let Some(meta) = config.get("meta") {
                    if let Some(node_id) = meta.get("node_id") {
                        if let Some(id) = node_id.as_str() {
                            return Ok(id.to_string());
                        }
                    }
                }
            }
        }

        // Fallback: use directory name or UUID
        Ok(self
            .spore_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string())
    }

    /// Get local configuration path for this spore
    fn get_local_config_path(&self, spore_id: &str) -> SporeResult<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;

        let config_dir = PathBuf::from(home)
            .join(".config")
            .join("biomeos")
            .join("deployed-nodes")
            .join(spore_id);

        Ok(config_dir)
    }

    /// Create local configuration for incubated node
    async fn create_local_config(
        &self,
        config_path: &Path,
        spore_id: &str,
        node_id: &str,
        deployed_seed_hash: &str,
        entropy_hash: &str,
        local_entropy: &LocalEntropy,
    ) -> SporeResult<()> {
        // Create directory
        fs::create_dir_all(config_path).await?;

        // Read parent and spore seed hashes
        let spore_seed_bytes = std::fs::read(self.spore_path.join(".family.seed"))?;
        let spore_seed_hash = Self::hash_seed(&spore_seed_bytes);

        // For parent seed, we'd need to track this in the spore manifest
        // For now, use spore seed as parent (or read from manifest)
        let parent_seed_hash = spore_seed_hash.clone(); // Future: Read from manifest for lineage chain

        // Extract family_id from tower.toml
        let family_id = self
            .extract_family_id()
            .unwrap_or_else(|_| "unknown".to_string());

        // Create node config
        let config = NodeConfig {
            node: NodeInfo {
                spore_id: spore_id.to_string(),
                node_id: node_id.to_string(),
                deployed_at: Utc::now(),
                computer_name: local_entropy.hostname.clone(),
                entropy_hash: entropy_hash.to_string(),
            },
            lineage: LineageInfo {
                parent_seed_hash,
                spore_seed_hash,
                deployed_seed_hash: deployed_seed_hash.to_string(),
            },
            spore: SporeInfo {
                original_path: Some(self.spore_path.clone()),
                last_seen: Utc::now(),
                deployment_count: 1,
            },
            federation: FederationInfo {
                family_id,
                sub_federations: vec![],
            },
        };

        // Write node.toml
        let config_toml =
            toml::to_string_pretty(&config).context("Failed to serialize node config")?;
        fs::write(config_path.join("node.toml"), config_toml).await?;

        // Write entropy.json for reference
        let entropy_json =
            serde_json::to_string_pretty(&local_entropy).context("Failed to serialize entropy")?;
        fs::write(config_path.join("entropy.json"), entropy_json).await?;

        info!("Created local config at: {}", config_path.display());

        Ok(())
    }

    /// Store deployed seed securely
    async fn store_deployed_seed(
        &self,
        config_path: &Path,
        deployed_seed: &[u8],
    ) -> SporeResult<()> {
        let seed_path = config_path.join(".deployed.seed");
        fs::write(&seed_path, deployed_seed).await?;

        // Set secure permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&seed_path).await?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&seed_path, perms).await?;
        }

        debug!("Stored deployed seed securely");
        Ok(())
    }

    /// Extract family_id from tower.toml
    fn extract_family_id(&self) -> Result<String> {
        let tower_toml_path = self.spore_path.join("tower.toml");
        let content = std::fs::read_to_string(&tower_toml_path)?;

        if let Ok(config) = toml::from_str::<toml::Value>(&content) {
            if let Some(tower) = config.get("tower") {
                if let Some(family) = tower.get("family") {
                    if let Some(family_str) = family.as_str() {
                        return Ok(family_str.to_string());
                    }
                }
            }

            // Fallback: check meta.family_id
            if let Some(meta) = config.get("meta") {
                if let Some(family_id) = meta.get("family_id") {
                    if let Some(id) = family_id.as_str() {
                        return Ok(id.to_string());
                    }
                }
            }
        }

        Ok("unknown".to_string())
    }
}

/// List all locally incubated nodes
pub async fn list_local_nodes() -> SporeResult<Vec<NodeConfig>> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory")?;

    let nodes_dir = PathBuf::from(home)
        .join(".config")
        .join("biomeos")
        .join("deployed-nodes");

    if !nodes_dir.exists() {
        return Ok(vec![]);
    }

    let mut nodes = Vec::new();
    let mut entries = fs::read_dir(&nodes_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let node_config_path = entry.path().join("node.toml");

        if node_config_path.exists() {
            if let Ok(content) = fs::read_to_string(&node_config_path).await {
                if let Ok(config) = toml::from_str::<NodeConfig>(&content) {
                    nodes.push(config);
                }
            }
        }
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== LocalEntropy Tests ==========

    #[test]
    fn test_local_entropy_generation() {
        let entropy = LocalEntropy::generate(Some("test-computer")).expect("generate entropy");

        assert_eq!(entropy.hostname, "test-computer");
        assert!(!entropy.machine_id.is_empty());
        assert_eq!(entropy.random_nonce.len(), 32);

        let hash = entropy.hash();
        assert_eq!(hash.len(), 64); // SHA256 hex
    }

    #[test]
    fn test_local_entropy_generation_no_name() {
        let entropy = LocalEntropy::generate(None).expect("generate entropy");

        // Should use system hostname or fallback
        assert!(!entropy.hostname.is_empty());
        assert_eq!(entropy.random_nonce.len(), 32);
    }

    #[test]
    fn test_entropy_hash_deterministic() {
        let entropy = LocalEntropy {
            hostname: "test".to_string(),
            machine_id: "12345".to_string(),
            timestamp: Utc::now(),
            mac_address: Some("00:11:22:33:44:55".to_string()),
            random_nonce: vec![1, 2, 3],
            cpu_hash: None,
            disk_serial: None,
        };

        let hash1 = entropy.hash();
        let hash2 = entropy.hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_entropy_hash_varies_with_hostname() {
        let now = Utc::now();

        let entropy1 = LocalEntropy {
            hostname: "host-a".to_string(),
            machine_id: "same-id".to_string(),
            timestamp: now,
            mac_address: None,
            random_nonce: vec![1, 2, 3],
            cpu_hash: None,
            disk_serial: None,
        };

        let entropy2 = LocalEntropy {
            hostname: "host-b".to_string(),
            machine_id: "same-id".to_string(),
            timestamp: now,
            mac_address: None,
            random_nonce: vec![1, 2, 3],
            cpu_hash: None,
            disk_serial: None,
        };

        assert_ne!(entropy1.hash(), entropy2.hash());
    }

    #[test]
    fn test_entropy_hash_includes_optional_fields() {
        let now = Utc::now();

        let entropy_without = LocalEntropy {
            hostname: "test".to_string(),
            machine_id: "id".to_string(),
            timestamp: now,
            mac_address: None,
            random_nonce: vec![1],
            cpu_hash: None,
            disk_serial: None,
        };

        let entropy_with_mac = LocalEntropy {
            hostname: "test".to_string(),
            machine_id: "id".to_string(),
            timestamp: now,
            mac_address: Some("00:11:22:33:44:55".to_string()),
            random_nonce: vec![1],
            cpu_hash: None,
            disk_serial: None,
        };

        let entropy_with_cpu = LocalEntropy {
            hostname: "test".to_string(),
            machine_id: "id".to_string(),
            timestamp: now,
            mac_address: None,
            random_nonce: vec![1],
            cpu_hash: Some("cpu_hash_val".to_string()),
            disk_serial: None,
        };

        let entropy_with_disk = LocalEntropy {
            hostname: "test".to_string(),
            machine_id: "id".to_string(),
            timestamp: now,
            mac_address: None,
            random_nonce: vec![1],
            cpu_hash: None,
            disk_serial: Some("disk_serial_val".to_string()),
        };

        // All should produce different hashes
        let h0 = entropy_without.hash();
        let h1 = entropy_with_mac.hash();
        let h2 = entropy_with_cpu.hash();
        let h3 = entropy_with_disk.hash();

        assert_ne!(h0, h1);
        assert_ne!(h0, h2);
        assert_ne!(h0, h3);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_entropy_clone() {
        let entropy = LocalEntropy {
            hostname: "clone-test".to_string(),
            machine_id: "id".to_string(),
            timestamp: Utc::now(),
            mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
            random_nonce: vec![10, 20, 30],
            cpu_hash: Some("cpu".to_string()),
            disk_serial: Some("disk".to_string()),
        };

        let cloned = entropy.clone();
        assert_eq!(cloned.hostname, entropy.hostname);
        assert_eq!(cloned.machine_id, entropy.machine_id);
        assert_eq!(cloned.random_nonce, entropy.random_nonce);
        assert_eq!(cloned.hash(), entropy.hash());
    }

    #[test]
    fn test_entropy_serialization_json() {
        let entropy = LocalEntropy {
            hostname: "test-host".to_string(),
            machine_id: "abc123".to_string(),
            timestamp: Utc::now(),
            mac_address: Some("00:11:22:33:44:55".to_string()),
            random_nonce: vec![1, 2, 3, 4],
            cpu_hash: None,
            disk_serial: None,
        };

        let json = serde_json::to_string(&entropy).expect("serialize");
        let deserialized: LocalEntropy = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.hostname, "test-host");
        assert_eq!(deserialized.machine_id, "abc123");
        assert_eq!(
            deserialized.mac_address,
            Some("00:11:22:33:44:55".to_string())
        );
    }

    // ========== IncubatedNode Tests ==========

    #[test]
    fn test_incubated_node_creation() {
        let node = IncubatedNode {
            node_id: "node-spore1-laptop".to_string(),
            spore_id: "spore1".to_string(),
            deployed_seed_hash: "hash123".to_string(),
            local_config_path: PathBuf::from("/home/user/.config/biomeos/deployed-nodes/spore1"),
            incubated_at: Utc::now(),
            entropy_hash: "entropy_hash_456".to_string(),
            spore_path: Some(PathBuf::from("/media/usb/biomeOS")),
        };

        assert_eq!(node.node_id, "node-spore1-laptop");
        assert_eq!(node.spore_id, "spore1");
        assert!(node.spore_path.is_some());
    }

    #[test]
    fn test_incubated_node_serialization() {
        let node = IncubatedNode {
            node_id: "node-test".to_string(),
            spore_id: "test-spore".to_string(),
            deployed_seed_hash: "deadbeef".to_string(),
            local_config_path: PathBuf::from("/tmp/test"),
            incubated_at: Utc::now(),
            entropy_hash: "abc123".to_string(),
            spore_path: None,
        };

        let json = serde_json::to_string(&node).expect("serialize");
        let deserialized: IncubatedNode = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.node_id, "node-test");
        assert!(deserialized.spore_path.is_none());
    }

    // ========== NodeConfig / NodeInfo Tests ==========

    #[test]
    fn test_node_info_creation() {
        let info = NodeInfo {
            spore_id: "spore-abc".to_string(),
            node_id: "node-abc-laptop".to_string(),
            deployed_at: Utc::now(),
            computer_name: "my-laptop".to_string(),
            entropy_hash: "hash".to_string(),
        };

        assert_eq!(info.spore_id, "spore-abc");
        assert_eq!(info.computer_name, "my-laptop");
    }

    #[test]
    fn test_lineage_info_creation() {
        let lineage = LineageInfo {
            parent_seed_hash: "parent".to_string(),
            spore_seed_hash: "spore".to_string(),
            deployed_seed_hash: "deployed".to_string(),
        };

        assert_eq!(lineage.parent_seed_hash, "parent");
        assert_eq!(lineage.spore_seed_hash, "spore");
        assert_eq!(lineage.deployed_seed_hash, "deployed");
    }

    #[test]
    fn test_federation_info_creation() {
        let fed = FederationInfo {
            family_id: "fam-123".to_string(),
            sub_federations: vec!["sub-1".to_string(), "sub-2".to_string()],
        };

        assert_eq!(fed.family_id, "fam-123");
        assert_eq!(fed.sub_federations.len(), 2);
    }

    #[test]
    fn test_spore_info_creation() {
        let info = SporeInfo {
            original_path: Some(PathBuf::from("/media/usb/biomeOS")),
            last_seen: Utc::now(),
            deployment_count: 3,
        };

        assert_eq!(info.deployment_count, 3);
        assert!(info.original_path.is_some());
    }

    #[test]
    fn test_node_config_serialization() {
        let config = NodeConfig {
            node: NodeInfo {
                spore_id: "sp1".to_string(),
                node_id: "n1".to_string(),
                deployed_at: Utc::now(),
                computer_name: "host".to_string(),
                entropy_hash: "h".to_string(),
            },
            lineage: LineageInfo {
                parent_seed_hash: "p".to_string(),
                spore_seed_hash: "s".to_string(),
                deployed_seed_hash: "d".to_string(),
            },
            spore: SporeInfo {
                original_path: None,
                last_seen: Utc::now(),
                deployment_count: 0,
            },
            federation: FederationInfo {
                family_id: "fam".to_string(),
                sub_federations: vec![],
            },
        };

        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: NodeConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.node.node_id, "n1");
        assert_eq!(deserialized.federation.family_id, "fam");
    }
}
