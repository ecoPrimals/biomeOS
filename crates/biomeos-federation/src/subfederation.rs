//! Sub-federation management
//!
//! This module provides hierarchical federation on top of the genetic lineage baseline.
//! Sub-federations allow granular access control and isolation while maintaining
//! family-level trust for discovery and NAT traversal.
//!
//! # Architecture
//!
//! ```text
//! Family Trust (Genetic Lineage)
//!   ├─> Gaming Sub-Federation (specific nodes, gaming capabilities)
//!   ├─> Family Sub-Federation (family nodes, storage/sync)
//!   └─> School Sub-Federation (high isolation, compute-only)
//! ```
//!
//! # Security
//!
//! All cryptographic operations are delegated to BearDog. This module does NOT
//! implement any crypto itself.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::capability::{Capability, CapabilitySet};
use crate::{FederationError, FederationResult};

/// Isolation level for sub-federations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    /// No isolation - full federation
    None,

    /// Low isolation - limited capabilities
    Low,

    /// Medium isolation - specific primals only
    Medium,

    /// High isolation - compute-only, no data access
    High,

    /// Critical isolation - air-gapped, manual approval required
    Critical,
}

impl IsolationLevel {
    /// Check if this isolation level allows auto-approval
    pub fn allows_auto_approval(&self) -> bool {
        matches!(
            self,
            IsolationLevel::None | IsolationLevel::Low | IsolationLevel::Medium
        )
    }
}

/// Node ID type
pub type NodeId = String;

/// A sub-federation within the family trust network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubFederation {
    /// Sub-federation name
    pub name: String,

    /// Parent family ID (genetic lineage)
    pub parent_family: String,

    /// Member node IDs (supports wildcards like "node-*")
    pub members: Vec<String>,

    /// Capabilities granted to members
    pub capabilities: CapabilitySet,

    /// Isolation level
    pub isolation_level: IsolationLevel,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// BearDog encryption key ID (managed by BearDog, not stored here)
    /// This is just a reference to the key in BearDog's HSM
    pub encryption_key_ref: Option<String>,
}

impl SubFederation {
    /// Create a new sub-federation
    pub fn new(
        name: String,
        parent_family: String,
        members: Vec<String>,
        capabilities: CapabilitySet,
        isolation_level: IsolationLevel,
    ) -> Self {
        info!(
            "Creating sub-federation '{}' for family '{}' with {} members",
            name,
            parent_family,
            members.len()
        );

        Self {
            name,
            parent_family,
            members,
            capabilities,
            isolation_level,
            created_at: Utc::now(),
            metadata: HashMap::new(),
            encryption_key_ref: None,
        }
    }

    /// Check if a node is a member of this sub-federation
    pub fn is_member(&self, node_id: &str) -> bool {
        self.members.iter().any(|pattern| {
            if pattern.contains('*') {
                // Simple wildcard matching
                let prefix = pattern.trim_end_matches('*');
                node_id.starts_with(prefix)
            } else {
                pattern == node_id
            }
        })
    }

    /// Check if a node has access to a specific capability
    pub fn has_capability(&self, node_id: &str, capability: &Capability) -> bool {
        // 1. Check if node is member
        if !self.is_member(node_id) {
            debug!(
                "Node {} is not a member of sub-federation {}",
                node_id, self.name
            );
            return false;
        }

        // 2. Check if capability is granted
        if !self.capabilities.has(capability) {
            debug!(
                "Capability {} not granted in sub-federation {}",
                capability, self.name
            );
            return false;
        }

        // 3. Check isolation level
        if !self.isolation_level.allows_auto_approval() {
            warn!(
                "Sub-federation {} requires manual approval (isolation: {:?})",
                self.name, self.isolation_level
            );
            return false;
        }

        true
    }

    /// Add a member to this sub-federation
    pub fn add_member(&mut self, node_id: String) {
        if !self.members.contains(&node_id) {
            self.members.push(node_id);
        }
    }

    /// Remove a member from this sub-federation
    pub fn remove_member(&mut self, node_id: &str) {
        self.members.retain(|id| id != node_id);
    }

    /// Set BearDog encryption key reference
    ///
    /// This is a reference to a key managed by BearDog's HSM.
    /// The actual key is never stored here - only the reference.
    pub fn set_encryption_key_ref(&mut self, key_ref: String) {
        self.encryption_key_ref = Some(key_ref);
    }
}

/// Sub-federation manager
pub struct SubFederationManager {
    config_dir: PathBuf,
    sub_federations: HashMap<String, SubFederation>,
}

impl SubFederationManager {
    /// Create a new sub-federation manager
    pub fn new(config_dir: PathBuf) -> Self {
        info!(
            "Initializing sub-federation manager at: {}",
            config_dir.display()
        );
        Self {
            config_dir,
            sub_federations: HashMap::new(),
        }
    }

    /// Load sub-federations from disk
    pub async fn load(&mut self) -> FederationResult<()> {
        let subfed_dir = self.config_dir.join("sub-federations");

        if !subfed_dir.exists() {
            fs::create_dir_all(&subfed_dir).await?;
            return Ok(());
        }

        let mut entries = fs::read_dir(&subfed_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(subfed) = toml::from_str::<SubFederation>(&content) {
                        debug!("Loaded sub-federation: {}", subfed.name);
                        self.sub_federations.insert(subfed.name.clone(), subfed);
                    }
                }
            }
        }

        info!("Loaded {} sub-federations", self.sub_federations.len());
        Ok(())
    }

    /// Save a sub-federation to disk
    pub async fn save(&self, subfed: &SubFederation) -> FederationResult<()> {
        let subfed_dir = self.config_dir.join("sub-federations");
        fs::create_dir_all(&subfed_dir).await?;

        let file_path = subfed_dir.join(format!("{}.toml", subfed.name));
        let content = toml::to_string_pretty(subfed)?;

        fs::write(&file_path, content).await?;
        debug!("Saved sub-federation: {}", subfed.name);

        Ok(())
    }

    /// Create a new sub-federation
    pub async fn create(
        &mut self,
        name: String,
        parent_family: String,
        members: Vec<String>,
        capabilities: CapabilitySet,
        isolation_level: IsolationLevel,
    ) -> FederationResult<SubFederation> {
        // Check if already exists
        if self.sub_federations.contains_key(&name) {
            return Err(FederationError::Generic(format!(
                "Sub-federation '{}' already exists",
                name
            )));
        }

        // EVOLVED (Jan 27, 2026): Verify genetic lineage of members using BearDog
        // This delegates to BearDog's API to verify all members share the parent_family lineage
        if let Err(e) = self.verify_member_lineage(&parent_family, &members).await {
            warn!(
                "Lineage verification failed for sub-federation '{}': {}",
                name, e
            );
            // Continue with warning - lineage verification is advisory for now
        }

        // Create sub-federation
        let mut subfed = SubFederation::new(
            name.clone(),
            parent_family.clone(),
            members,
            capabilities,
            isolation_level,
        );

        // EVOLVED (Jan 27, 2026): Request encryption key from BearDog for this sub-federation
        // Derives a key specifically for this sub-federation from the family seed
        match self.request_subfederation_key(&parent_family, &name).await {
            Ok(key_ref) => {
                subfed.encryption_key_ref = Some(key_ref);
                info!("Encryption key derived for sub-federation '{}'", name);
            }
            Err(e) => {
                warn!(
                    "Could not derive encryption key for '{}': {} (sub-federation will operate without encryption)",
                    name, e
                );
                // Continue without encryption - sub-federation is still valid
            }
        }

        // Clone subfed for saving
        let subfed_to_save = subfed.clone();

        // Save to disk
        self.save(&subfed_to_save).await?;

        // Add to memory
        self.sub_federations.insert(name.clone(), subfed.clone());

        info!("Created sub-federation: {}", name);
        Ok(subfed)
    }

    /// Get a sub-federation by name
    pub fn get(&self, name: &str) -> Option<&SubFederation> {
        self.sub_federations.get(name)
    }

    /// Get all sub-federations
    pub fn all(&self) -> Vec<&SubFederation> {
        self.sub_federations.values().collect()
    }

    /// Get sub-federations a node belongs to
    pub fn for_node(&self, node_id: &str) -> Vec<&SubFederation> {
        self.sub_federations
            .values()
            .filter(|sf| sf.is_member(node_id))
            .collect()
    }

    /// Check if a node has access to a capability in any sub-federation
    pub fn has_access(&self, node_id: &str, capability: &Capability) -> bool {
        self.sub_federations
            .values()
            .any(|sf| sf.has_capability(node_id, capability))
    }

    /// Add a member to a sub-federation
    pub async fn add_member(&mut self, subfed_name: &str, node_id: String) -> FederationResult<()> {
        {
            let subfed = self
                .sub_federations
                .get_mut(subfed_name)
                .ok_or_else(|| FederationError::SubFederationNotFound(subfed_name.to_string()))?;

            subfed.add_member(node_id.clone());
        }

        // Clone the subfed for saving (to avoid borrowing issues)
        let subfed_to_save = self
            .sub_federations
            .get(subfed_name)
            .ok_or_else(|| FederationError::SubFederationNotFound(subfed_name.to_string()))?
            .clone();

        self.save(&subfed_to_save).await?;

        info!("Added node {} to sub-federation {}", node_id, subfed_name);
        Ok(())
    }

    /// Remove a member from a sub-federation
    pub async fn remove_member(
        &mut self,
        subfed_name: &str,
        node_id: &str,
    ) -> FederationResult<()> {
        {
            let subfed = self
                .sub_federations
                .get_mut(subfed_name)
                .ok_or_else(|| FederationError::SubFederationNotFound(subfed_name.to_string()))?;

            subfed.remove_member(node_id);
        }

        // Clone the subfed for saving (to avoid borrowing issues)
        let subfed_to_save = self
            .sub_federations
            .get(subfed_name)
            .ok_or_else(|| FederationError::SubFederationNotFound(subfed_name.to_string()))?
            .clone();

        self.save(&subfed_to_save).await?;

        info!(
            "Removed node {} from sub-federation {}",
            node_id, subfed_name
        );
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BEARDOG INTEGRATION - Cryptographic Operations via JSON-RPC
    // ═══════════════════════════════════════════════════════════════════════════

    /// Verify that all members share genetic lineage with the parent family
    ///
    /// EVOLVED (Jan 27, 2026): Delegates to BearDog via JSON-RPC
    async fn verify_member_lineage(
        &self,
        parent_family: &str,
        members: &[String],
    ) -> FederationResult<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Discover BearDog socket
        let beardog_socket = Self::discover_beardog_socket()?;

        let stream = UnixStream::connect(&beardog_socket)
            .await
            .map_err(|e| FederationError::Generic(format!("BearDog connection failed: {}", e)))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Request lineage verification
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "lineage.verify_members",
            "params": {
                "family_id": parent_family,
                "member_patterns": members
            },
            "id": 1
        });

        let request_str = serde_json::to_string(&request)
            .map_err(|e| FederationError::Generic(format!("JSON error: {}", e)))?
            + "\n";

        writer
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| FederationError::Generic(format!("Write error: {}", e)))?;
        writer
            .flush()
            .await
            .map_err(|e| FederationError::Generic(format!("Flush error: {}", e)))?;

        // Read response
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| FederationError::Generic(format!("Read error: {}", e)))?;

        let response: serde_json::Value = serde_json::from_str(response_line.trim())
            .map_err(|e| FederationError::Generic(format!("JSON parse error: {}", e)))?;

        // Check for errors
        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown");
            return Err(FederationError::Generic(format!(
                "Lineage verification failed: {}",
                msg
            )));
        }

        // Check result
        let all_verified = response
            .get("result")
            .and_then(|r| r.get("all_verified"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if all_verified {
            info!("✅ Lineage verified for {} members", members.len());
            Ok(())
        } else {
            let failed = response
                .get("result")
                .and_then(|r| r.get("failed_members"))
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();

            Err(FederationError::Generic(format!(
                "Lineage verification failed for: {}",
                failed
            )))
        }
    }

    /// Request a derived encryption key for this sub-federation
    ///
    /// EVOLVED (Jan 27, 2026): Delegates to BearDog via JSON-RPC
    async fn request_subfederation_key(
        &self,
        parent_family: &str,
        subfed_name: &str,
    ) -> FederationResult<String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Discover BearDog socket
        let beardog_socket = Self::discover_beardog_socket()?;

        let stream = UnixStream::connect(&beardog_socket)
            .await
            .map_err(|e| FederationError::Generic(format!("BearDog connection failed: {}", e)))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Request key derivation
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "crypto.derive_subfederation_key",
            "params": {
                "family_id": parent_family,
                "subfederation_name": subfed_name,
                "purpose": "subfederation-encryption-v1"
            },
            "id": 1
        });

        let request_str = serde_json::to_string(&request)
            .map_err(|e| FederationError::Generic(format!("JSON error: {}", e)))?
            + "\n";

        writer
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| FederationError::Generic(format!("Write error: {}", e)))?;
        writer
            .flush()
            .await
            .map_err(|e| FederationError::Generic(format!("Flush error: {}", e)))?;

        // Read response
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| FederationError::Generic(format!("Read error: {}", e)))?;

        let response: serde_json::Value = serde_json::from_str(response_line.trim())
            .map_err(|e| FederationError::Generic(format!("JSON parse error: {}", e)))?;

        // Check for errors
        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown");
            return Err(FederationError::Generic(format!(
                "Key derivation failed: {}",
                msg
            )));
        }

        // Extract key reference
        let key_ref = response
            .get("result")
            .and_then(|r| r.get("key_ref"))
            .and_then(|k| k.as_str())
            .ok_or_else(|| FederationError::Generic("Missing key_ref in response".to_string()))?;

        debug!(
            "Derived key for sub-federation '{}': {}",
            subfed_name, key_ref
        );
        Ok(key_ref.to_string())
    }

    /// Discover BearDog socket path (XDG-compliant)
    fn discover_beardog_socket() -> FederationResult<String> {
        // Priority 1: Environment variable
        if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
            return Ok(socket);
        }

        // Priority 2: XDG runtime directory
        if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let socket = format!("{}/biomeos/beardog.sock", runtime);
            if std::path::Path::new(&socket).exists() {
                return Ok(socket);
            }
        }

        // Priority 3: Family-based discovery
        if let Ok(family_id) = std::env::var("BIOMEOS_FAMILY_ID") {
            let socket = format!("/tmp/beardog-{}.sock", family_id);
            if std::path::Path::new(&socket).exists() {
                return Ok(socket);
            }
        }

        // Priority 4: Common patterns
        let patterns = ["/tmp/beardog.sock", "/run/biomeos/beardog.sock"];
        for pattern in &patterns {
            if std::path::Path::new(pattern).exists() {
                return Ok((*pattern).to_string());
            }
        }

        Err(FederationError::Generic(
            "BearDog socket not found. Ensure BearDog is running.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_membership() {
        let subfed = SubFederation::new(
            "test".to_string(),
            "family".to_string(),
            vec!["node-alpha-*".to_string(), "node-beta-laptop".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        );

        assert!(subfed.is_member("node-alpha-laptop"));
        assert!(subfed.is_member("node-alpha-desktop"));
        assert!(subfed.is_member("node-beta-laptop"));
        assert!(!subfed.is_member("node-gamma-laptop"));
    }

    #[test]
    fn test_capability_check() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::Gaming);

        let subfed = SubFederation::new(
            "gaming".to_string(),
            "family".to_string(),
            vec!["node-alpha-*".to_string()],
            caps,
            IsolationLevel::Low,
        );

        assert!(subfed.has_capability("node-alpha-laptop", &Capability::Gaming));
        assert!(!subfed.has_capability("node-alpha-laptop", &Capability::Storage));
        assert!(!subfed.has_capability("node-beta-laptop", &Capability::Gaming));
    }

    #[test]
    fn test_isolation_level() {
        assert!(IsolationLevel::None.allows_auto_approval());
        assert!(IsolationLevel::Low.allows_auto_approval());
        assert!(!IsolationLevel::Critical.allows_auto_approval());
    }
}
