// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Sub-federation tests

use std::env;
use tempfile::TempDir;

use super::manager::SubFederationManager;
use super::types::{IsolationLevel, SubFederation};
use crate::capability::{Capability, CapabilitySet};

#[cfg(test)]
mod run {
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
        assert!(IsolationLevel::Medium.allows_auto_approval());
        assert!(!IsolationLevel::High.allows_auto_approval());
        assert!(!IsolationLevel::Critical.allows_auto_approval());
    }

    #[test]
    fn test_subfederation_serialization_roundtrip() {
        let subfed = SubFederation::new(
            "serial-test".to_string(),
            "family-1".to_string(),
            vec!["node-a".to_string(), "node-b-*".to_string()],
            CapabilitySet::from_vec(vec![Capability::Storage, Capability::Compute]),
            IsolationLevel::Medium,
        );

        let toml_str = toml::to_string(&subfed).expect("serialize SubFederation to TOML");
        let restored: SubFederation =
            toml::from_str(&toml_str).expect("deserialize SubFederation from TOML");
        assert_eq!(restored.name, subfed.name);
        assert_eq!(restored.parent_family, subfed.parent_family);
        assert_eq!(restored.members, subfed.members);
        assert_eq!(restored.isolation_level, subfed.isolation_level);
    }

    #[test]
    fn test_isolation_level_serialization() {
        let levels = [
            IsolationLevel::None,
            IsolationLevel::Low,
            IsolationLevel::Medium,
            IsolationLevel::High,
            IsolationLevel::Critical,
        ];
        for level in &levels {
            let json = serde_json::to_string(level).expect("serialize IsolationLevel");
            let restored: IsolationLevel =
                serde_json::from_str(&json).expect("deserialize IsolationLevel");
            assert_eq!(*level, restored);
        }
    }

    #[test]
    fn test_add_member_no_duplicates() {
        let mut subfed = SubFederation::new(
            "test".to_string(),
            "family".to_string(),
            vec!["node-alpha".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        );
        subfed.add_member("node-alpha".to_string());
        assert_eq!(subfed.members.len(), 1);
        subfed.add_member("node-beta".to_string());
        assert_eq!(subfed.members.len(), 2);
    }

    #[test]
    fn test_has_capability_high_isolation_denies() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::Storage);
        let subfed = SubFederation::new(
            "high".to_string(),
            "family".to_string(),
            vec!["node-alpha".to_string()],
            caps,
            IsolationLevel::High,
        );
        assert!(
            !subfed.has_capability("node-alpha", &Capability::Storage),
            "High isolation should deny auto-approval"
        );
    }

    #[tokio::test]
    async fn test_subfederation_manager_load_empty_dir() {
        let temp = TempDir::new().expect("create temp dir");
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.expect("load should succeed on empty dir");
        assert!(mgr.all().is_empty());
    }

    #[tokio::test]
    async fn test_subfederation_manager_create_and_get() {
        let temp = TempDir::new().expect("create temp dir");
        env::set_var("BEARDOG_SOCKET", "/tmp/nonexistent-beardog-test-12345.sock");
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());

        let subfed = mgr
            .create(
                "gaming".to_string(),
                "family-1".to_string(),
                vec!["node-a".to_string()],
                CapabilitySet::from_vec(vec![Capability::Gaming]),
                IsolationLevel::Low,
            )
            .await
            .expect("create sub-federation");

        assert_eq!(subfed.name, "gaming");
        let retrieved = mgr.get("gaming").expect("get should return created subfed");
        assert_eq!(retrieved.name, "gaming");
        assert_eq!(mgr.all().len(), 1);

        env::remove_var("BEARDOG_SOCKET");
    }

    #[tokio::test]
    async fn test_subfederation_manager_create_duplicate_error() {
        let temp = TempDir::new().expect("create temp dir");
        env::set_var("BEARDOG_SOCKET", "/tmp/nonexistent-beardog-test-12345.sock");
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());

        mgr.create(
            "dup".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        )
        .await
        .expect("first create succeeds");

        let err = mgr
            .create(
                "dup".to_string(),
                "family".to_string(),
                vec!["node-b".to_string()],
                CapabilitySet::new(),
                IsolationLevel::None,
            )
            .await
            .expect_err("second create with same name should fail");
        assert!(matches!(err, crate::FederationError::Generic(_)));
        assert!(err.to_string().contains("already exists"));

        env::remove_var("BEARDOG_SOCKET");
    }

    #[tokio::test]
    async fn test_subfederation_manager_for_node_and_has_access() {
        let temp = TempDir::new().expect("create temp dir");
        env::set_var("BEARDOG_SOCKET", "/tmp/nonexistent-beardog-test-12345.sock");
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());

        mgr.create(
            "gaming".to_string(),
            "family".to_string(),
            vec!["node-alpha-*".to_string()],
            CapabilitySet::from_vec(vec![Capability::Gaming]),
            IsolationLevel::Low,
        )
        .await
        .expect("create sub-federation");

        let for_node = mgr.for_node("node-alpha-laptop");
        assert_eq!(for_node.len(), 1);
        assert!(mgr.has_access("node-alpha-laptop", &Capability::Gaming));
        assert!(!mgr.has_access("node-beta-laptop", &Capability::Gaming));

        env::remove_var("BEARDOG_SOCKET");
    }

    #[tokio::test]
    async fn test_subfederation_manager_add_member_not_found_error() {
        let temp = TempDir::new().expect("create temp dir");
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());

        let err = mgr
            .add_member("nonexistent", "node-a".to_string())
            .await
            .expect_err("add_member to nonexistent subfed should fail");
        assert!(matches!(
            err,
            crate::FederationError::SubFederationNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_subfederation_manager_remove_member_not_found_error() {
        let temp = TempDir::new().expect("create temp dir");
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());

        let err = mgr
            .remove_member("nonexistent", "node-a")
            .await
            .expect_err("remove_member from nonexistent subfed should fail");
        assert!(matches!(
            err,
            crate::FederationError::SubFederationNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_subfederation_manager_add_remove_member_persists() {
        let temp = TempDir::new().expect("create temp dir");
        env::set_var("BEARDOG_SOCKET", "/tmp/nonexistent-beardog-test-12345.sock");
        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());

        mgr.create(
            "test".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        )
        .await
        .expect("create");

        mgr.add_member("test", "node-b".to_string())
            .await
            .expect("add member");
        assert!(mgr.get("test").expect("get").is_member("node-b"));

        mgr.remove_member("test", "node-b")
            .await
            .expect("remove member");
        assert!(!mgr.get("test").expect("get").is_member("node-b"));

        env::remove_var("BEARDOG_SOCKET");
    }

    #[test]
    fn test_subfederation_set_encryption_key_ref() {
        let mut subfed = SubFederation::new(
            "test".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        );
        assert!(subfed.encryption_key_ref.is_none());
        subfed.set_encryption_key_ref("key-ref-123".to_string());
        assert_eq!(subfed.encryption_key_ref.as_deref(), Some("key-ref-123"));
    }

    #[test]
    fn test_subfederation_remove_member() {
        let mut subfed = SubFederation::new(
            "test".to_string(),
            "family".to_string(),
            vec!["node-a".to_string(), "node-b".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        );
        subfed.remove_member("node-b");
        assert_eq!(subfed.members.len(), 1);
        assert!(subfed.is_member("node-a"));
        assert!(!subfed.is_member("node-b"));
    }

    #[test]
    fn test_subfederation_has_capability_critical_isolation_denies() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::Storage);
        let subfed = SubFederation::new(
            "critical".to_string(),
            "family".to_string(),
            vec!["node-alpha".to_string()],
            caps,
            IsolationLevel::Critical,
        );
        assert!(
            !subfed.has_capability("node-alpha", &Capability::Storage),
            "Critical isolation should deny auto-approval"
        );
    }

    #[test]
    fn test_subfederation_has_capability_medium_isolation_allows() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::Compute);
        let subfed = SubFederation::new(
            "medium".to_string(),
            "family".to_string(),
            vec!["node-alpha".to_string()],
            caps,
            IsolationLevel::Medium,
        );
        assert!(
            subfed.has_capability("node-alpha", &Capability::Compute),
            "Medium isolation should allow auto-approval"
        );
    }

    #[test]
    fn test_subfederation_wildcard_prefix_empty() {
        let subfed = SubFederation::new(
            "test".to_string(),
            "family".to_string(),
            vec!["*".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        );
        assert!(subfed.is_member("any-node"));
        assert!(subfed.is_member(""));
    }

    #[test]
    fn test_subfederation_exact_match_no_wildcard() {
        let subfed = SubFederation::new(
            "test".to_string(),
            "family".to_string(),
            vec!["node-exact".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        );
        assert!(subfed.is_member("node-exact"));
        assert!(!subfed.is_member("node-exact-extra"));
        assert!(!subfed.is_member("node-exac"));
    }

    #[test]
    fn test_subfederation_manager_new() {
        let temp = TempDir::new().expect("create temp dir");
        let mgr = SubFederationManager::new(temp.path().to_path_buf());
        assert!(mgr.all().is_empty());
        assert!(mgr.get("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_subfederation_manager_load_existing_toml() {
        let temp = TempDir::new().expect("create temp dir");
        let subfed_dir = temp.path().join("sub-federations");
        std::fs::create_dir_all(&subfed_dir).expect("create subfed dir");

        let subfed = SubFederation::new(
            "loaded".to_string(),
            "family-1".to_string(),
            vec!["node-x".to_string()],
            CapabilitySet::from_vec(vec![Capability::Gaming]),
            IsolationLevel::Low,
        );
        let toml_content = toml::to_string_pretty(&subfed).expect("serialize");
        std::fs::write(subfed_dir.join("loaded.toml"), toml_content).expect("write");

        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.expect("load");
        let loaded = mgr.get("loaded").expect("get loaded subfed");
        assert_eq!(loaded.name, "loaded");
        assert_eq!(loaded.parent_family, "family-1");
        assert!(loaded.is_member("node-x"));
    }

    #[tokio::test]
    async fn test_subfederation_manager_load_skips_non_toml() {
        let temp = TempDir::new().expect("create temp dir");
        let subfed_dir = temp.path().join("sub-federations");
        std::fs::create_dir_all(&subfed_dir).expect("create subfed dir");
        std::fs::write(subfed_dir.join("readme.txt"), "not toml").expect("write");

        let mut mgr = SubFederationManager::new(temp.path().to_path_buf());
        mgr.load().await.expect("load");
        assert!(mgr.all().is_empty());
    }

    #[test]
    fn test_subfederation_metadata_serialization() {
        let mut subfed = SubFederation::new(
            "meta".to_string(),
            "family".to_string(),
            vec!["node-a".to_string()],
            CapabilitySet::new(),
            IsolationLevel::None,
        );
        subfed
            .metadata
            .insert("key1".to_string(), "value1".to_string());
        let toml_str = toml::to_string(&subfed).expect("serialize");
        let restored: SubFederation = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(restored.metadata.get("key1"), Some(&"value1".to_string()));
    }
}
