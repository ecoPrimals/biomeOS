// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::super::capability_domains::*;

#[test]
fn test_capability_registry_from_toml() {
    let toml_content = r#"
[metadata]
version = "1.0.0"

[domains.security]
provider = "beardog"
capabilities = ["crypto", "encryption", "security"]

[domains.network]
provider = "songbird"
capabilities = ["discovery", "http"]
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_registry.toml");
    std::fs::write(&path, toml_content).unwrap();

    let registry = CapabilityRegistry::from_toml(&path).unwrap();
    assert_eq!(registry.config_entry_count(), 5);
    assert_eq!(registry.resolve("crypto"), Some("beardog".into()));
    assert_eq!(registry.resolve("discovery"), Some("songbird".into()));
    assert_eq!(registry.resolve("crypto.encrypt"), Some("beardog".into()));
}

#[test]
fn test_capability_registry_falls_back_to_const() {
    let registry = CapabilityRegistry::default();
    assert_eq!(registry.config_entry_count(), 0);
    assert_eq!(registry.resolve("security"), Some("beardog".into()));
    assert_eq!(registry.resolve("storage"), Some("nestgate".into()));
    assert_eq!(registry.resolve("unknown"), None);
}

#[test]
fn test_capability_registry_skips_wildcard_provider() {
    let toml_content = r#"
[domains.health]
provider = "*"
capabilities = ["health.liveness", "health.readiness"]

[domains.storage]
provider = "nestgate"
capabilities = ["storage"]
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wildcard.toml");
    std::fs::write(&path, toml_content).unwrap();

    let registry = CapabilityRegistry::from_toml(&path).unwrap();
    assert_eq!(registry.config_entry_count(), 1);
    assert_eq!(registry.resolve("storage"), Some("nestgate".into()));
}

#[test]
fn test_capability_registry_config_overrides_const() {
    let toml_content = r#"
[domains.security]
provider = "custom-sec-primal"
capabilities = ["security"]
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("override.toml");
    std::fs::write(&path, toml_content).unwrap();

    let registry = CapabilityRegistry::from_toml(&path).unwrap();
    assert_eq!(
        registry.resolve("security"),
        Some("custom-sec-primal".into()),
    );
}

#[test]
fn test_capability_registry_from_real_config() {
    let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/capability_registry.toml");
    if !config_path.exists() {
        eprintln!("Skipping: config/capability_registry.toml not found");
        return;
    }

    let registry = CapabilityRegistry::from_toml(&config_path).unwrap();
    assert!(
        registry.config_entry_count() > 40,
        "Real config should have 40+ capability entries, got {}",
        registry.config_entry_count()
    );
    assert_eq!(registry.resolve("crypto"), Some("beardog".into()));
    assert_eq!(registry.resolve("ecology"), Some("airspring".into()));
    assert_eq!(registry.resolve("game"), Some("ludospring".into()));
}
