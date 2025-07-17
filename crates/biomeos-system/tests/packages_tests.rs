//! Package Management Tests for biomeOS

use biomeos_system::packages::*;
use std::collections::HashMap;

#[test]
fn test_package_config_default() {
    let config = PackageConfig::default();

    assert!(config.install_dir.ends_with("packages"));
    assert!(config.cache_dir.ends_with("packages"));
    assert!(config.db_path.ends_with("packages.db"));
    assert_eq!(config.repositories.len(), 2);
    assert!(!config.auto_update);
    assert_eq!(config.update_interval_seconds, 86400); // 24 hours
    assert!(config.verify_packages);
    assert!(!config.allow_untrusted);
}

#[test]
fn test_repository_type_variants() {
    let types = vec![
        RepositoryType::Official,
        RepositoryType::Community,
        RepositoryType::Private,
        RepositoryType::Local,
    ];

    for repo_type in types {
        let json = serde_json::to_string(&repo_type).unwrap();
        let _from_json: RepositoryType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_trust_level_variants() {
    let levels = vec![
        TrustLevel::Trusted,
        TrustLevel::Partial,
        TrustLevel::Untrusted,
    ];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let _from_json: TrustLevel = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_package_category_variants() {
    let categories = vec![
        PackageCategory::System,
        PackageCategory::Primal,
        PackageCategory::Application,
        PackageCategory::Library,
        PackageCategory::Development,
        PackageCategory::Games,
        PackageCategory::Multimedia,
        PackageCategory::Office,
        PackageCategory::Science,
        PackageCategory::Other,
    ];

    for category in categories {
        let json = serde_json::to_string(&category).unwrap();
        let _from_json: PackageCategory = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_package_type_variants() {
    let types = vec![
        PackageType::Binary,
        PackageType::Source,
        PackageType::Container,
        PackageType::Wasm,
        PackageType::BiomeTemplate,
        PackageType::PrimalExtension,
    ];

    for package_type in types {
        let json = serde_json::to_string(&package_type).unwrap();
        let _from_json: PackageType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_package_state_variants() {
    let states = vec![
        PackageState::Installed,
        PackageState::NotInstalled,
        PackageState::Installing,
        PackageState::Updating,
        PackageState::Removing,
        PackageState::Failed,
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let _from_json: PackageState = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_package_health_variants() {
    let healths = vec![
        PackageHealth::Healthy,
        PackageHealth::Degraded,
        PackageHealth::Broken,
        PackageHealth::Unknown,
    ];

    for health in healths {
        let json = serde_json::to_string(&health).unwrap();
        let _from_json: PackageHealth = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_dependency_type_variants() {
    let types = vec![DependencyType::Runtime, DependencyType::Build];

    for dep_type in types {
        let json = serde_json::to_string(&dep_type).unwrap();
        let _from_json: DependencyType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_version_constraint_variants() {
    let constraints = vec![
        VersionConstraint::Exact("1.0.0".to_string()),
        VersionConstraint::Min("1.0.0".to_string()),
        VersionConstraint::Max("2.0.0".to_string()),
        VersionConstraint::Range("1.0.0".to_string(), "2.0.0".to_string()),
        VersionConstraint::Any,
    ];

    for constraint in constraints {
        let json = serde_json::to_string(&constraint).unwrap();
        let _from_json: VersionConstraint = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_file_type_variants() {
    let types = vec![
        FileType::Regular,
        FileType::Directory,
        FileType::Symlink,
        FileType::Executable,
        FileType::Config,
        FileType::Data,
    ];

    for file_type in types {
        let json = serde_json::to_string(&file_type).unwrap();
        let _from_json: FileType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_repository_status_variants() {
    let statuses = vec![
        RepositoryStatus::Available,
        RepositoryStatus::Unavailable,
        RepositoryStatus::Updating,
        RepositoryStatus::Error("test error".to_string()),
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let _from_json: RepositoryStatus = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_repository_config_creation() {
    let repo_config = RepositoryConfig {
        name: "test_repo".to_string(),
        url: "https://packages.test.com".to_string(),
        repo_type: RepositoryType::Community,
        priority: 10,
        enabled: true,
        trust_level: TrustLevel::Partial,
    };

    assert_eq!(repo_config.name, "test_repo");
    assert_eq!(repo_config.url, "https://packages.test.com");
    assert!(matches!(repo_config.repo_type, RepositoryType::Community));
    assert_eq!(repo_config.priority, 10);
    assert!(repo_config.enabled);
    assert!(matches!(repo_config.trust_level, TrustLevel::Partial));
}

#[test]
fn test_dependency_creation() {
    let dependency = Dependency {
        name: "test_dep".to_string(),
        version: VersionConstraint::Min("1.0.0".to_string()),
        dep_type: DependencyType::Runtime,
        optional: false,
    };

    assert_eq!(dependency.name, "test_dep");
    assert!(!dependency.optional);
    assert!(matches!(dependency.dep_type, DependencyType::Runtime));

    match dependency.version {
        VersionConstraint::Min(v) => assert_eq!(v, "1.0.0"),
        _ => panic!("Expected Min version constraint"),
    }
}

#[test]
fn test_package_file_creation() {
    let package_file = PackageFile {
        path: "/usr/bin/test_app".into(),
        size: 1024,
        checksum: "sha256:abc123".to_string(),
        permissions: 0o755,
        file_type: FileType::Executable,
    };

    assert_eq!(package_file.path.to_string_lossy(), "/usr/bin/test_app");
    assert_eq!(package_file.size, 1024);
    assert_eq!(package_file.checksum, "sha256:abc123");
    assert_eq!(package_file.permissions, 0o755);
    assert!(matches!(package_file.file_type, FileType::Executable));
}

#[tokio::test]
async fn test_package_manager_creation() {
    let config = PackageConfig::default();
    let manager = PackageManager::new(config.clone());

    assert_eq!(manager.config.install_dir, config.install_dir);
    assert_eq!(manager.config.cache_dir, config.cache_dir);
    assert_eq!(manager.config.db_path, config.db_path);
    assert_eq!(manager.config.repositories.len(), config.repositories.len());
    assert_eq!(manager.config.auto_update, config.auto_update);
    assert_eq!(
        manager.config.update_interval_seconds,
        config.update_interval_seconds
    );
    assert_eq!(manager.config.verify_packages, config.verify_packages);
    assert_eq!(manager.config.allow_untrusted, config.allow_untrusted);
}

#[tokio::test]
async fn test_package_manager_initialization() {
    let config = PackageConfig::default();
    let manager = PackageManager::new(config);

    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[test]
fn test_package_config_serialization() {
    let config = PackageConfig::default();

    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let from_json: PackageConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.install_dir, from_json.install_dir);
    assert_eq!(config.cache_dir, from_json.cache_dir);
    assert_eq!(config.db_path, from_json.db_path);
    assert_eq!(config.repositories.len(), from_json.repositories.len());
    assert_eq!(config.auto_update, from_json.auto_update);
    assert_eq!(
        config.update_interval_seconds,
        from_json.update_interval_seconds
    );
    assert_eq!(config.verify_packages, from_json.verify_packages);
    assert_eq!(config.allow_untrusted, from_json.allow_untrusted);
}

#[test]
fn test_package_status_creation() {
    let status = PackageStatus {
        state: PackageState::Installed,
        health: PackageHealth::Healthy,
        last_update_check: None,
        available_updates: vec![],
        errors: vec![],
    };

    assert!(matches!(status.state, PackageState::Installed));
    assert!(matches!(status.health, PackageHealth::Healthy));
    assert!(status.last_update_check.is_none());
    assert!(status.available_updates.is_empty());
    assert!(status.errors.is_empty());
}

#[test]
fn test_repository_package_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("author".to_string(), "Test Author".to_string());

    let repo_package = RepositoryPackage {
        name: "test_package".to_string(),
        version: "1.0.0".to_string(),
        description: "Test package".to_string(),
        download_url: "https://example.com/package.tar.gz".to_string(),
        size: 2048,
        checksum: "sha256:def456".to_string(),
        metadata,
    };

    assert_eq!(repo_package.name, "test_package");
    assert_eq!(repo_package.version, "1.0.0");
    assert_eq!(repo_package.description, "Test package");
    assert!(repo_package.download_url.starts_with("https://"));
    assert_eq!(repo_package.size, 2048);
    assert!(repo_package.checksum.starts_with("sha256:"));
    assert_eq!(repo_package.metadata.len(), 1);
}

#[test]
fn test_cached_package_creation() {
    let repo_package = RepositoryPackage {
        name: "cached_test".to_string(),
        version: "1.0.0".to_string(),
        description: "Cached test package".to_string(),
        download_url: "https://example.com/cached.tar.gz".to_string(),
        size: 1024,
        checksum: "sha256:cached123".to_string(),
        metadata: HashMap::new(),
    };

    let now = chrono::Utc::now();
    let cached_package = CachedPackage {
        package: repo_package,
        cache_path: "/tmp/cache/cached_test-1.0.0.tar.gz".into(),
        cached_at: now,
        expires_at: now + chrono::Duration::hours(24),
    };

    assert_eq!(cached_package.package.name, "cached_test");
    assert!(cached_package
        .cache_path
        .ends_with("cached_test-1.0.0.tar.gz"));
    assert!(cached_package.expires_at > cached_package.cached_at);
}

#[test]
fn test_repository_creation() {
    let repo_config = RepositoryConfig {
        name: "main".to_string(),
        url: "https://packages.biomeos.org".to_string(),
        repo_type: RepositoryType::Official,
        priority: 1,
        enabled: true,
        trust_level: TrustLevel::Trusted,
    };

    let repo = Repository {
        config: repo_config,
        status: RepositoryStatus::Available,
        packages: HashMap::new(),
        last_update: None,
    };

    assert_eq!(repo.config.name, "main");
    assert!(matches!(repo.status, RepositoryStatus::Available));
    assert!(repo.packages.is_empty());
    assert!(repo.last_update.is_none());
}

#[test]
fn test_dependency_optional() {
    let optional_dep = Dependency {
        name: "optional_feature".to_string(),
        version: VersionConstraint::Any,
        dep_type: DependencyType::Runtime,
        optional: true,
    };

    let required_dep = Dependency {
        name: "required_lib".to_string(),
        version: VersionConstraint::Min("2.0.0".to_string()),
        dep_type: DependencyType::Runtime,
        optional: false,
    };

    assert!(optional_dep.optional);
    assert!(!required_dep.optional);
    assert!(matches!(optional_dep.version, VersionConstraint::Any));

    match required_dep.version {
        VersionConstraint::Min(v) => assert_eq!(v, "2.0.0"),
        _ => panic!("Expected Min version constraint"),
    }
}

#[test]
fn test_repository_priority() {
    let high_priority = RepositoryConfig {
        name: "high".to_string(),
        url: "https://high.repo.com".to_string(),
        repo_type: RepositoryType::Official,
        priority: 1,
        enabled: true,
        trust_level: TrustLevel::Trusted,
    };

    let low_priority = RepositoryConfig {
        name: "low".to_string(),
        url: "https://low.repo.com".to_string(),
        repo_type: RepositoryType::Community,
        priority: 10,
        enabled: true,
        trust_level: TrustLevel::Partial,
    };

    // Lower numbers should have higher priority
    assert!(high_priority.priority < low_priority.priority);
}
