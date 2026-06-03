// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

#[test]
fn test_builder_pattern() {
    let config = BiomeOSConfigBuilder::new()
        .with_environment(Environment::Testing)
        .with_port(8080)
        .with_bind_address("0.0.0.0")
        .build();

    assert_eq!(config.system.environment, Environment::Testing);
    assert_eq!(config.network.port, 8080);
    assert_eq!(config.network.bind_address, "0.0.0.0");
}

#[test]
fn test_discovery_configuration() {
    let config = BiomeOSConfigBuilder::new()
        .with_discovery_methods(vec![DiscoveryMethod::Registry, DiscoveryMethod::Dns])
        .build();

    assert_eq!(config.discovery.methods.len(), 2);
    assert!(
        config
            .discovery
            .methods
            .contains(&DiscoveryMethod::Registry)
    );
    assert!(config.discovery.methods.contains(&DiscoveryMethod::Dns));
}

#[test]
fn test_security_configuration() {
    let config = BiomeOSConfigBuilder::new()
        .with_security_enabled(true)
        .build();

    assert!(matches!(
        config.security.authentication.default_method,
        AuthMethod::ApiKey
    ));
    assert!(config.security.encryption.at_rest.enabled);
    assert!(config.security.encryption.in_transit.enabled);
}

#[test]
fn test_factory_methods() {
    let config = BiomeOSConfigBuilder::standard_development().build();

    assert_eq!(config.system.environment, Environment::Development);
    assert!(config.ui.enabled);
    assert!(matches!(
        config.ui.theme,
        biomeos_types::config::features::UITheme::Dark
    ));
    assert!(config.observability.metrics.enabled);
    assert!(config.observability.tracing.enabled);
}

#[test]
fn test_production_configuration() {
    let config = BiomeOSConfigBuilder::for_production().build();

    assert_eq!(config.system.environment, Environment::Production);
    assert_eq!(
        config.system.organization_scale,
        OrganizationScale::Enterprise
    );
    assert!(
        config
            .discovery
            .methods
            .contains(&DiscoveryMethod::Registry)
    );
}

#[test]
fn test_registry_discovery() {
    let config =
        BiomeOSConfigBuilder::for_registry_discovery("http://registry.example.com").build();

    assert!(
        config
            .discovery
            .methods
            .contains(&DiscoveryMethod::Registry)
    );
    assert!(config.discovery.registry.is_some());
    assert_eq!(
        config.discovery.registry.as_ref().unwrap().url,
        "http://registry.example.com"
    );
}

#[test]
fn test_from_config() {
    let base = BiomeOSConfigBuilder::new()
        .with_port(9999)
        .with_bind_address("192.0.2.1")
        .build();
    let config = BiomeOSConfigBuilder::from_config(base).build();
    assert_eq!(config.network.port, 9999);
    assert_eq!(config.network.bind_address, "192.0.2.1");
}

#[test]
fn test_add_discovery_method() {
    let config = BiomeOSConfigBuilder::new()
        .with_discovery_methods(vec![]) // Start empty
        .add_discovery_method(DiscoveryMethod::Dns)
        .add_discovery_method(DiscoveryMethod::Consul)
        .build();
    assert_eq!(config.discovery.methods.len(), 2);
    assert!(config.discovery.methods.contains(&DiscoveryMethod::Dns));
    assert!(config.discovery.methods.contains(&DiscoveryMethod::Consul));
}

#[test]
fn test_with_registry_discovery_and_auth() {
    let config = BiomeOSConfigBuilder::new()
        .with_registry_discovery(
            "https://registry.test",
            Some(("user".into(), "pass".into())),
        )
        .build();
    assert!(config.discovery.registry.is_some());
    let reg = config.discovery.registry.as_ref().unwrap();
    assert_eq!(reg.url, "https://registry.test");
    assert!(reg.auth.is_some());
    let auth = reg.auth.as_ref().unwrap();
    assert_eq!(auth.username, "user");
    assert_eq!(auth.password, "pass");
}

#[test]
fn test_with_dns_discovery() {
    let config = BiomeOSConfigBuilder::new()
        .with_dns_discovery(vec!["192.0.2.53".into(), "198.51.100.53".into()])
        .build();
    assert!(config.discovery.dns.is_some());
    let dns = config.discovery.dns.as_ref().unwrap();
    assert_eq!(dns.servers, vec!["192.0.2.53", "198.51.100.53"]);
}

#[test]
fn test_with_timeouts() {
    let config = BiomeOSConfigBuilder::new()
        .with_timeouts(
            Duration::from_secs(60),
            Duration::from_secs(15),
            Duration::from_secs(20),
        )
        .build();
    assert_eq!(
        config.system.timeouts.default_request_timeout,
        Duration::from_secs(60)
    );
    assert_eq!(
        config.system.timeouts.connection_timeout,
        Duration::from_secs(15)
    );
    assert_eq!(
        config.system.timeouts.discovery_timeout,
        Duration::from_secs(20)
    );
}

#[test]
fn test_with_data_dir_and_config_dir() {
    let config = BiomeOSConfigBuilder::new()
        .with_data_dir("/var/lib/biomeos")
        .with_config_dir("/etc/biomeos")
        .build();
    assert_eq!(
        config.system.data_dir,
        std::path::PathBuf::from("/var/lib/biomeos")
    );
    assert_eq!(
        config.system.config_dir,
        std::path::PathBuf::from("/etc/biomeos")
    );
}

#[test]
fn test_with_security_disabled() {
    let config = BiomeOSConfigBuilder::new()
        .with_security_enabled(false)
        .build();
    assert!(matches!(
        config.security.authentication.default_method,
        AuthMethod::None
    ));
    assert!(!config.security.encryption.at_rest.enabled);
    assert!(!config.security.encryption.in_transit.enabled);
}

#[test]
fn test_with_tls() {
    let config = BiomeOSConfigBuilder::new()
        .with_tls("/etc/certs/server.pem", "/etc/certs/key.pem")
        .build();
    assert!(config.network.tls.is_some());
    let tls = config.network.tls.as_ref().unwrap();
    assert!(tls.enabled);
    assert_eq!(
        tls.cert_file.as_ref().unwrap(),
        &std::path::PathBuf::from("/etc/certs/server.pem")
    );
    assert_eq!(
        tls.key_file.as_ref().unwrap(),
        &std::path::PathBuf::from("/etc/certs/key.pem")
    );
}

#[test]
fn test_with_observability() {
    let config = BiomeOSConfigBuilder::new()
        .with_observability(true, false)
        .build();
    assert!(config.observability.metrics.enabled);
    assert!(!config.observability.tracing.enabled);
}

#[test]
fn test_with_ui_settings() {
    let config = BiomeOSConfigBuilder::new()
        .with_ui_enabled(true)
        .with_ui_theme(UITheme::Light)
        .with_ui_language("fr")
        .build();
    assert!(config.ui.enabled);
    assert!(matches!(config.ui.theme, UITheme::Light));
    assert_eq!(config.ui.language, "fr");
}

#[test]
fn test_config_accessors() {
    let mut builder = BiomeOSConfigBuilder::new().with_port(1234);
    assert_eq!(builder.config().network.port, 1234);
    builder.config_mut().network.port = 5678;
    assert_eq!(builder.config().network.port, 5678);
}

#[test]
fn test_distributed_deployment() {
    let config = BiomeOSConfigBuilder::distributed_deployment().build();
    assert_eq!(config.system.environment, Environment::Production);
    assert_eq!(
        config.system.organization_scale,
        OrganizationScale::Enterprise
    );
    assert!(config.discovery.methods.contains(&DiscoveryMethod::Consul));
    assert!(
        config
            .discovery
            .methods
            .contains(&DiscoveryMethod::Kubernetes)
    );
    assert!(config.security.encryption.at_rest.enabled);
}

#[test]
fn test_development_full() {
    let config = BiomeOSConfigBuilder::development_full().build();
    assert_eq!(config.system.environment, Environment::Development);
    assert!(config.ui.enabled);
    assert!(
        config
            .security
            .authentication
            .methods
            .contains(&AuthMethod::ApiKey)
    );
    assert_eq!(config.ui.language, "en");
}

#[test]
fn test_default_builder() {
    let builder = BiomeOSConfigBuilder::default();
    let config = builder.build();
    // Default config is built successfully
    assert!(config.network.port > 0);
}
