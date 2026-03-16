// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Sub-federation manager - CRUD and persistence

use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::capability::{Capability, CapabilitySet};
use crate::{FederationError, FederationResult};

use super::beardog::{request_subfederation_key, verify_member_lineage};
use super::types::{IsolationLevel, SubFederation};

/// Sub-federation manager
pub struct SubFederationManager {
    config_dir: PathBuf,
    sub_federations: std::collections::HashMap<String, SubFederation>,
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
            sub_federations: std::collections::HashMap::new(),
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

            if path.extension().and_then(|e| e.to_str()) == Some("toml")
                && let Ok(content) = fs::read_to_string(&path).await
                && let Ok(subfed) = toml::from_str::<SubFederation>(&content)
            {
                debug!("Loaded sub-federation: {}", subfed.name);
                self.sub_federations.insert(subfed.name.clone(), subfed);
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
        if self.sub_federations.contains_key(&name) {
            return Err(FederationError::Generic(format!(
                "Sub-federation '{name}' already exists"
            )));
        }

        if let Err(e) = verify_member_lineage(&parent_family, &members).await {
            warn!(
                "Lineage verification failed for sub-federation '{}': {}",
                name, e
            );
        }

        let mut subfed = SubFederation::new(
            name.clone(),
            parent_family.clone(),
            members,
            capabilities,
            isolation_level,
        );

        match request_subfederation_key(&parent_family, &name).await {
            Ok(key_ref) => {
                subfed.encryption_key_ref = Some(key_ref);
                info!("Encryption key derived for sub-federation '{}'", name);
            }
            Err(e) => {
                warn!(
                    "Could not derive encryption key for '{}': {} (sub-federation will operate without encryption)",
                    name, e
                );
            }
        }

        let subfed_to_save = subfed.clone();

        self.save(&subfed_to_save).await?;

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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use biomeos_test_utils::{remove_test_env, set_test_env};
    use tempfile::TempDir;

    use super::*;
    use crate::FederationError;
    use crate::capability::{Capability, CapabilitySet};

    fn setup_beardog_env() {
        set_test_env(
            "BEARDOG_SOCKET",
            "/tmp/nonexistent-beardog-test-manager.sock",
        );
    }

    fn teardown_beardog_env() {
        remove_test_env("BEARDOG_SOCKET");
    }

    #[test]
    fn test_manager_new() {
        let temp = TempDir::new().unwrap();
        let mgr = SubFederationManager::new(temp.path().to_path_buf());
        assert!(mgr.all().is_empty());
        assert!(mgr.get("any").is_none());
    }

    #[tokio::test]
    async fn test_load_empty_dir_creates_subfed_dir() {
        let temp = TempDir::new().unwrap();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();
        assert!(mgr.all().is_empty());
        assert!(temp.path().join("sub-federations").exists());
    }

    #[tokio::test]
    async fn test_create_subfederation() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        let subfed = mgr
            .create(
                "gaming".to_string(),
                "family-1".to_string(),
                vec!["node-a".to_string(), "node-b".to_string()],
                CapabilitySet::from_vec(vec![Capability::Gaming]),
                IsolationLevel::Low,
            )
            .await
            .unwrap();

        assert_eq!(subfed.name, "gaming");
        assert_eq!(subfed.parent_family, "family-1");
        assert_eq!(subfed.members.len(), 2);
        assert!(subfed.capabilities.has(&Capability::Gaming));
        assert_eq!(subfed.isolation_level, IsolationLevel::Low);

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_create_duplicate_returns_error() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "dup".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        )
        .await
        .unwrap();

        let err = mgr
            .create(
                "dup".to_string(),
                "family".to_string(),
                vec!["node-b".to_string()],
                CapabilitySet::new(),
                IsolationLevel::None,
            )
            .await
            .expect_err("duplicate create should fail");

        assert!(matches!(err, FederationError::Generic(_)));
        assert!(err.to_string().contains("already exists"));

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_create_empty_name_succeeds() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        let subfed = mgr
            .create(
                "".to_string(),
                "family".to_string(),
                vec!["node-a".to_string()],
                CapabilitySet::new(),
                IsolationLevel::None,
            )
            .await
            .unwrap();

        assert_eq!(subfed.name, "");
        assert!(mgr.get("").is_some());

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_list_all_subfederations() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "alpha".to_string(),
            "family".to_string(),
            vec!["node-1".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        )
        .await
        .unwrap();

        mgr.create(
            "beta".to_string(),
            "family".to_string(),
            vec!["node-2".to_string()],
            CapabilitySet::new(),
            IsolationLevel::Low,
        )
        .await
        .unwrap();

        let all = mgr.all();
        assert_eq!(all.len(), 2);
        let names: Vec<&str> = all.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_get_subfederation() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "retrieve-me".to_string(),
            "family".to_string(),
            vec!["node-x".to_string()],
            CapabilitySet::from_vec(vec![Capability::Storage]),
            IsolationLevel::Medium,
        )
        .await
        .unwrap();

        let retrieved = mgr.get("retrieve-me").unwrap();
        assert_eq!(retrieved.name, "retrieve-me");
        assert!(retrieved.is_member("node-x"));
        assert!(retrieved.capabilities.has(&Capability::Storage));

        assert!(mgr.get("nonexistent").is_none());

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_for_node() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "gaming".to_string(),
            "family".to_string(),
            vec!["node-alpha-*".to_string()],
            CapabilitySet::from_vec(vec![Capability::Gaming]),
            IsolationLevel::Low,
        )
        .await
        .unwrap();

        mgr.create(
            "storage".to_string(),
            "family".to_string(),
            vec!["node-alpha-laptop".to_string(), "node-beta".to_string()],
            CapabilitySet::from_vec(vec![Capability::Storage]),
            IsolationLevel::Low,
        )
        .await
        .unwrap();

        let for_alpha = mgr.for_node("node-alpha-laptop");
        assert_eq!(for_alpha.len(), 2);

        let for_beta = mgr.for_node("node-beta");
        assert_eq!(for_beta.len(), 1);
        assert_eq!(for_beta[0].name, "storage");

        let for_gamma = mgr.for_node("node-gamma");
        assert!(for_gamma.is_empty());

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_has_access() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "gaming".to_string(),
            "family".to_string(),
            vec!["node-alpha".to_string()],
            CapabilitySet::from_vec(vec![Capability::Gaming]),
            IsolationLevel::Low,
        )
        .await
        .unwrap();

        assert!(mgr.has_access("node-alpha", &Capability::Gaming));
        assert!(!mgr.has_access("node-alpha", &Capability::Storage));
        assert!(!mgr.has_access("node-beta", &Capability::Gaming));

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_add_member_to_nonexistent_subfed_returns_error() {
        let temp = TempDir::new().unwrap();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        let err = mgr
            .add_member("nonexistent", "node-a".to_string())
            .await
            .expect_err("add_member to nonexistent should fail");

        assert!(matches!(err, FederationError::SubFederationNotFound(_)));
        assert!(err.to_string().contains("nonexistent"));
    }

    #[tokio::test]
    async fn test_remove_member_from_nonexistent_subfed_returns_error() {
        let temp = TempDir::new().unwrap();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        let err = mgr
            .remove_member("nonexistent", "node-a")
            .await
            .expect_err("remove_member from nonexistent should fail");

        assert!(matches!(err, FederationError::SubFederationNotFound(_)));
        assert!(err.to_string().contains("nonexistent"));
    }

    #[tokio::test]
    async fn test_add_and_remove_member() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "test".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        )
        .await
        .unwrap();

        mgr.add_member("test", "node-b".to_string()).await.unwrap();
        assert!(mgr.get("test").unwrap().is_member("node-b"));

        mgr.remove_member("test", "node-b").await.unwrap();
        assert!(!mgr.get("test").unwrap().is_member("node-b"));

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_remove_nonexistent_member_is_noop() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "test".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        )
        .await
        .unwrap();

        mgr.remove_member("test", "node-nonexistent").await.unwrap();
        assert_eq!(mgr.get("test").unwrap().members.len(), 1);

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_persistence_save_and_load() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "persist".to_string(),
            "family-1".to_string(),
            vec!["node-x".to_string(), "node-y".to_string()],
            CapabilitySet::from_vec(vec![Capability::Compute, Capability::Storage]),
            IsolationLevel::Medium,
        )
        .await
        .unwrap();

        let file_path = temp.path().join("sub-federations").join("persist.toml");
        assert!(file_path.exists());

        let mut mgr2 = SubFederationManager::new(temp.path().to_path_buf());
        mgr2.load().await.unwrap();

        let loaded = mgr2.get("persist").unwrap();
        assert_eq!(loaded.name, "persist");
        assert_eq!(loaded.parent_family, "family-1");
        assert_eq!(loaded.members.len(), 2);
        assert!(loaded.capabilities.has(&Capability::Compute));
        assert!(loaded.capabilities.has(&Capability::Storage));
        assert_eq!(loaded.isolation_level, IsolationLevel::Medium);

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_load_skips_non_toml_files() {
        let temp = TempDir::new().unwrap();
        let subfed_dir = temp.path().join("sub-federations");
        std::fs::create_dir_all(&subfed_dir).unwrap();
        std::fs::write(subfed_dir.join("readme.txt"), "not toml").unwrap();
        std::fs::write(subfed_dir.join("data.json"), r#"{"name":"x"}"#).unwrap();

        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        assert!(mgr.all().is_empty());
    }

    #[tokio::test]
    async fn test_load_skips_invalid_toml() {
        let temp = TempDir::new().unwrap();
        let subfed_dir = temp.path().join("sub-federations");
        std::fs::create_dir_all(&subfed_dir).unwrap();
        std::fs::write(subfed_dir.join("bad.toml"), "not valid toml [[[[").unwrap();

        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        assert!(mgr.all().is_empty());
    }

    #[tokio::test]
    async fn test_serialization_roundtrip_via_disk() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        let subfed = mgr
            .create(
                "roundtrip".to_string(),
                "family".to_string(),
                vec!["node-a".to_string(), "node-b-*".to_string()],
                CapabilitySet::from_vec(vec![Capability::Gaming, Capability::Sync]),
                IsolationLevel::High,
            )
            .await
            .unwrap();

        let toml_str = toml::to_string_pretty(&subfed).unwrap();
        let restored: crate::SubFederation = toml::from_str(&toml_str).unwrap();

        assert_eq!(restored.name, subfed.name);
        assert_eq!(restored.parent_family, subfed.parent_family);
        assert_eq!(restored.members, subfed.members);
        assert_eq!(restored.isolation_level, subfed.isolation_level);
        assert!(restored.capabilities.has(&Capability::Gaming));
        assert!(restored.capabilities.has(&Capability::Sync));

        teardown_beardog_env();
    }

    #[tokio::test]
    async fn test_add_member_persists_to_disk() {
        let temp = TempDir::new().unwrap();
        setup_beardog_env();
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.unwrap();

        mgr.create(
            "persist-member".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        )
        .await
        .unwrap();

        mgr.add_member("persist-member", "node-b".to_string())
            .await
            .unwrap();

        let mut mgr2 = SubFederationManager::new(temp.path().to_path_buf());
        mgr2.load().await.unwrap();
        let loaded = mgr2.get("persist-member").unwrap();
        assert!(loaded.is_member("node-b"));

        teardown_beardog_env();
    }
}
