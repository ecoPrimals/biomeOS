
use std::collections::HashMap;

use super::*;

#[test]
fn test_family_id_not_set() {
    let _ = family_id();
}

#[test]
fn test_strict_discovery_default() {
    let env: HashMap<String, String> = HashMap::new();
    assert!(!strict_discovery_with(&env));
}

#[test]
fn test_vars_constants_are_consistent() {
    assert!(vars::FAMILY_ID.starts_with("BIOMEOS_"));
    assert!(vars::SECURITY_PROVIDER.starts_with("BIOMEOS_"));
    assert!(vars::NETWORK_PROVIDER.starts_with("BIOMEOS_"));
    assert!(vars::SOCKET_DIR.starts_with("BIOMEOS_"));
}

#[test]
fn test_family_id_biomeos_precedence() {
    let mut env = HashMap::new();
    env.insert(vars::FAMILY_ID.to_string(), "biomeos-family".to_string());
    env.insert(
        vars::FAMILY_ID_LEGACY.to_string(),
        "legacy-family".to_string(),
    );
    assert_eq!(family_id_with(&env), Some("biomeos-family".to_string()));
}

#[test]
fn test_family_id_legacy_fallback() {
    let mut env = HashMap::new();
    env.insert(
        vars::FAMILY_ID_LEGACY.to_string(),
        "legacy-only".to_string(),
    );
    assert_eq!(family_id_with(&env), Some("legacy-only".to_string()));
}

#[test]
fn test_security_provider() {
    let mut env = HashMap::new();
    env.insert(
        vars::SECURITY_PROVIDER.to_string(),
        "custom-security".to_string(),
    );
    assert_eq!(
        security_provider_with(&env),
        Some("custom-security".to_string())
    );
}

#[test]
fn test_network_provider() {
    let mut env = HashMap::new();
    env.insert(
        vars::NETWORK_PROVIDER.to_string(),
        "custom-network".to_string(),
    );
    assert_eq!(
        network_provider_with(&env),
        Some("custom-network".to_string())
    );
}

#[test]
fn test_strict_discovery_enabled() {
    let mut env = HashMap::new();
    env.insert(vars::STRICT_DISCOVERY.to_string(), "1".to_string());
    assert!(strict_discovery_with(&env));
}

#[test]
fn test_socket_dir() {
    let mut env = HashMap::new();
    env.insert(vars::SOCKET_DIR.to_string(), "/run/biomeos".to_string());
    assert_eq!(socket_dir_with(&env), Some(PathBuf::from("/run/biomeos")));
}

#[test]
fn test_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert(
        vars::XDG_RUNTIME_DIR.to_string(),
        "/tmp/xdg-test".to_string(),
    );
    assert_eq!(
        xdg_runtime_dir_with(&env),
        Some(PathBuf::from("/tmp/xdg-test"))
    );
}

#[test]
fn test_plasmid_bin_dir_ecoprimals() {
    let mut env = HashMap::new();
    env.insert(vars::PLASMID_BIN.to_string(), "/eco/plasmid".to_string());
    assert_eq!(
        plasmid_bin_dir_with(&env),
        Some(PathBuf::from("/eco/plasmid"))
    );
}

#[test]
fn test_plasmid_bin_dir_biomeos_fallback() {
    let mut env = HashMap::new();
    env.insert(
        vars::PLASMID_BIN_DIR.to_string(),
        "/biomeos/bin".to_string(),
    );
    assert_eq!(
        plasmid_bin_dir_with(&env),
        Some(PathBuf::from("/biomeos/bin"))
    );
}

#[test]
fn test_vars_all_constants() {
    assert_eq!(vars::FAMILY_ID, "BIOMEOS_FAMILY_ID");
    assert_eq!(vars::FAMILY_ID_LEGACY, "FAMILY_ID");
    assert_eq!(vars::NEURAL_API_SOCKET, "NEURAL_API_SOCKET");
    assert_eq!(vars::BEARDOG_SOCKET, "BEARDOG_SOCKET");
    assert_eq!(vars::SONGBIRD_SOCKET, "SONGBIRD_SOCKET");
}
