//! BiomeOS Configuration Builder
//!
//! Provides a flexible builder pattern for creating BiomeOS configurations
//! with customizable discovery settings and deployment-specific values.

use crate::config::*;
use std::collections::HashMap;

/// Builder for creating flexible BiomeOS configurations
#[derive(Debug, Clone)]
pub struct BiomeOSConfigBuilder {
    system: Option<SystemConfig>,
    primals: Option<PrimalConfigs>,
    security: Option<SecurityConfig>,
    licensing: Option<LicensingConfig>,
    integration: Option<IntegrationConfig>,
}

impl Default for BiomeOSConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeOSConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            system: None,
            primals: None,
            security: None,
            licensing: None,
            integration: None,
        }
    }

    /// Configure for local development (default: localhost only)
    pub fn for_local_development() -> Self {
        Self::new()
            .with_discovery_hosts(&["127.0.0.1", "localhost"])
            .with_discovery_ports(&[8080, 8081, 8082])
            .with_environment(Environment::Development)
    }

    /// Configure for production deployment (custom hosts and ports)
    pub fn for_production(hosts: &[&str], ports: &[u16]) -> Self {
        Self::new()
            .with_discovery_hosts(hosts)
            .with_discovery_ports(ports)
            .with_environment(Environment::Production)
    }

    /// Configure for testing (includes test endpoints)
    pub fn for_testing() -> Self {
        Self::new()
            .with_discovery_hosts(&["localhost", "127.0.0.1", "test.local"])
            .with_discovery_ports(&[8080, 8081, 8082, 8083])
            .with_environment(Environment::Testing)
    }

    /// Configure for registry-based discovery
    pub fn for_registry_discovery(registry_endpoint: &str) -> Self {
        let mut builder = Self::new().with_environment(Environment::Production);

        if let Some(ref mut primals) = builder.primals {
            primals.discovery.method = DiscoveryMethod::Registry {
                url: registry_endpoint.to_string(),
            };
        } else {
            let primals_config = PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::Registry {
                        url: registry_endpoint.to_string(),
                    },
                    auto_discovery: true,
                    static_endpoints: HashMap::new(),
                    scan_hosts: Vec::new(),
                    scan_ports: Vec::new(),
                },
                endpoints: HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms: 5000,
                    discovery_timeout_ms: 10000,
                    health_check_interval_ms: 30000,
                },
            };
            builder.primals = Some(primals_config);
        }

        builder
    }

    /// Configure custom discovery settings
    pub fn with_custom_discovery(
        hosts: &[&str],
        ports: &[u16],
        _registry_endpoints: &[&str],
    ) -> Self {
        Self::new()
            .with_discovery_hosts(hosts)
            .with_discovery_ports(ports)
    }

    /// Set discovery hosts from string slices
    pub fn with_discovery_hosts(mut self, hosts: &[&str]) -> Self {
        if let Some(ref mut primals) = self.primals {
            primals.discovery.scan_hosts = hosts.iter().map(|s| s.to_string()).collect();
        } else {
            let primals_config = PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::NetworkScan,
                    auto_discovery: true,
                    static_endpoints: HashMap::new(),
                    scan_hosts: hosts.iter().map(|s| s.to_string()).collect(),
                    scan_ports: Vec::new(),
                },
                endpoints: HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms: 5000,
                    discovery_timeout_ms: 10000,
                    health_check_interval_ms: 30000,
                },
            };
            self.primals = Some(primals_config);
        }
        self
    }

    /// Set discovery ports efficiently
    pub fn with_discovery_ports(mut self, ports: &[u16]) -> Self {
        if let Some(ref mut primals) = self.primals {
            primals.discovery.scan_ports = ports.to_vec();
        } else {
            let primals_config = PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::NetworkScan,
                    auto_discovery: true,
                    static_endpoints: HashMap::new(),
                    scan_hosts: Vec::new(),
                    scan_ports: ports.to_vec(),
                },
                endpoints: HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms: 5000,
                    discovery_timeout_ms: 10000,
                    health_check_interval_ms: 30000,
                },
            };
            self.primals = Some(primals_config);
        }
        self
    }

    /// Set environment efficiently
    pub fn with_environment(mut self, env: Environment) -> Self {
        if let Some(ref mut system) = self.system {
            system.environment = env;
        } else {
            let system_config = SystemConfig {
                name: "biomeOS".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: env,
                log_level: "info".to_string(),
                data_dir: "/tmp/biomeos".to_string(),
            };
            self.system = Some(system_config);
        }
        self
    }

    /// Add a static endpoint from string references
    pub fn with_static_endpoint(mut self, name: &str, endpoint: &str) -> Self {
        // Ensure we have primals config
        if self.primals.is_none() {
            self.primals = Some(PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::Static,
                    auto_discovery: true,
                    static_endpoints: HashMap::new(),
                    scan_hosts: Vec::new(),
                    scan_ports: Vec::new(),
                },
                endpoints: HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms: 5000,
                    discovery_timeout_ms: 10000,
                    health_check_interval_ms: 30000,
                },
            });
        }

        // Ensure discovery method is Static
        if let Some(ref mut primals) = self.primals {
            if !matches!(primals.discovery.method, DiscoveryMethod::Static) {
                primals.discovery.method = DiscoveryMethod::Static;
            }
            primals
                .discovery
                .static_endpoints
                .insert(name.to_string(), endpoint.to_string());
        }

        self
    }

    /// Add multiple static endpoints efficiently
    pub fn with_static_endpoints(mut self, endpoints: &[(&str, &str)]) -> Self {
        // Ensure we have primals config
        if self.primals.is_none() {
            self.primals = Some(PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::Static,
                    auto_discovery: true,
                    static_endpoints: HashMap::new(),
                    scan_hosts: Vec::new(),
                    scan_ports: Vec::new(),
                },
                endpoints: HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms: 5000,
                    discovery_timeout_ms: 10000,
                    health_check_interval_ms: 30000,
                },
            });
        }

        // Set discovery method to Static and add all endpoints
        if let Some(ref mut primals) = self.primals {
            primals.discovery.method = DiscoveryMethod::Static;
            for (name, endpoint) in endpoints {
                primals
                    .discovery
                    .static_endpoints
                    .insert(name.to_string(), endpoint.to_string());
            }
        }

        self
    }

    /// Configure timeouts efficiently
    pub fn with_timeouts(
        mut self,
        discovery_timeout_ms: u64,
        default_timeout_ms: u64,
        health_check_interval_ms: u64,
    ) -> Self {
        if let Some(ref mut primals) = self.primals {
            primals.timeouts.discovery_timeout_ms = discovery_timeout_ms;
            primals.timeouts.default_timeout_ms = default_timeout_ms;
            primals.timeouts.health_check_interval_ms = health_check_interval_ms;
        } else {
            let primals_config = PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::NetworkScan,
                    auto_discovery: true,
                    static_endpoints: HashMap::new(),
                    scan_hosts: Vec::new(),
                    scan_ports: Vec::new(),
                },
                endpoints: HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms,
                    discovery_timeout_ms,
                    health_check_interval_ms,
                },
            };
            self.primals = Some(primals_config);
        }
        self
    }

    /// Enable or disable security features
    pub fn with_security_enabled(mut self, enabled: bool) -> Self {
        if let Some(ref mut security) = self.security {
            security.enable_crypto_locks = enabled;
        } else {
            let security_config = SecurityConfig {
                enable_crypto_locks: enabled,
                genetic_key_path: None,
                ai_cat_door: AiCatDoorConfig {
                    enabled: true,
                    cost_protection_threshold: 20.0,
                    monthly_budget: 100.0,
                },
            };
            self.security = Some(security_config);
        }
        self
    }

    /// Set crypto lock path from string reference
    pub fn with_crypto_lock_path(mut self, path: &str) -> Self {
        if let Some(ref mut security) = self.security {
            security.genetic_key_path = Some(path.to_string());
        } else {
            let security_config = SecurityConfig {
                enable_crypto_locks: true,
                genetic_key_path: Some(path.to_string()),
                ai_cat_door: AiCatDoorConfig {
                    enabled: true,
                    cost_protection_threshold: 20.0,
                    monthly_budget: 100.0,
                },
            };
            self.security = Some(security_config);
        }
        self
    }

    /// Enable licensing with sovereign key path
    pub fn with_licensing(mut self, _sovereign_key_path: &str, _compliance_level: &str) -> Self {
        let licensing_config = LicensingConfig {
            license_type: LicenseType::Individual,
            organization_scale: None,
            entropy_tier: EntropyTier::HumanLived,
        };
        self.licensing = Some(licensing_config);
        self
    }

    /// Add integration endpoint efficiently
    pub fn with_integration_endpoint(mut self, _name: &str, endpoint: &str) -> Self {
        if let Some(ref mut integration) = self.integration {
            if let Some(ref mut songbird) = integration.songbird.endpoint {
                *songbird = endpoint.to_string();
            } else {
                integration.songbird.endpoint = Some(endpoint.to_string());
            }
        } else {
            let integration_config = IntegrationConfig {
                songbird: SongbirdIntegrationConfig {
                    endpoint: Some(endpoint.to_string()),
                    auto_register: true,
                    health_reporting_interval_ms: 30000,
                },
                ecosystem: EcosystemIntegrationConfig {
                    enable_cross_primal_communication: true,
                    ai_first_responses: true,
                    universal_registration: true,
                },
            };
            self.integration = Some(integration_config);
        }
        self
    }

    /// Build the final BiomeOS configuration
    pub fn build(self) -> BiomeOSConfig {
        let system = self.system.unwrap_or_else(|| SystemConfig {
            name: "biomeOS".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
            log_level: "info".to_string(),
            data_dir: "/tmp/biomeos".to_string(),
        });

        let primals = self.primals.unwrap_or_else(|| PrimalConfigs {
            discovery: DiscoveryConfig {
                method: DiscoveryMethod::NetworkScan,
                auto_discovery: true,
                static_endpoints: HashMap::new(),
                scan_hosts: vec!["localhost".to_string()],
                scan_ports: vec![8080, 8081, 8082],
            },
            endpoints: HashMap::new(),
            timeouts: TimeoutConfig {
                default_timeout_ms: 5000,
                discovery_timeout_ms: 10000,
                health_check_interval_ms: 30000,
            },
        });

        let security = self.security.unwrap_or_else(|| SecurityConfig {
            enable_crypto_locks: true,
            genetic_key_path: None,
            ai_cat_door: AiCatDoorConfig {
                enabled: true,
                cost_protection_threshold: 20.0,
                monthly_budget: 100.0,
            },
        });

        let licensing = self.licensing.unwrap_or_else(|| LicensingConfig {
            license_type: LicenseType::Individual,
            organization_scale: None,
            entropy_tier: EntropyTier::HumanLived,
        });

        let integration = self.integration.unwrap_or_else(|| IntegrationConfig {
            songbird: SongbirdIntegrationConfig {
                endpoint: None,
                auto_register: true,
                health_reporting_interval_ms: 30000,
            },
            ecosystem: EcosystemIntegrationConfig {
                enable_cross_primal_communication: true,
                ai_first_responses: true,
                universal_registration: true,
            },
        });

        BiomeOSConfig {
            system,
            primals,
            security,
            licensing,
            integration,
        }
    }
}

/// Quick configuration factory functions for common use cases
impl BiomeOSConfigBuilder {
    /// Create configuration for standard Primal discovery
    pub fn standard_primals() -> Self {
        Self::new()
            .with_static_endpoints(&[
                ("toadstool", "http://localhost:8084"),
                ("songbird", "http://localhost:8081"),
                ("nestgate", "http://localhost:8082"),
                ("beardog", "http://localhost:8083"),
            ])
            .with_discovery_hosts(&["localhost", "127.0.0.1"])
            .with_discovery_ports(&[8080, 8081, 8082, 8083, 8084])
    }

    /// Create configuration for distributed deployment
    pub fn distributed_deployment(cluster_hosts: &[&str]) -> Self {
        Self::new()
            .with_discovery_hosts(cluster_hosts)
            .with_discovery_ports(&[8080, 8081, 8082, 8083, 8084])
            .with_environment(Environment::Production)
            .with_security_enabled(true)
    }

    /// Create configuration for development with all features
    pub fn development_full() -> Self {
        Self::new()
            .with_static_endpoints(&[
                ("toadstool", "http://localhost:8084"),
                ("songbird", "http://localhost:8081"),
                ("nestgate", "http://localhost:8082"),
                ("beardog", "http://localhost:8083"),
            ])
            .with_discovery_hosts(&["localhost", "127.0.0.1"])
            .with_discovery_ports(&[8080, 8081, 8082, 8083, 8084])
            .with_environment(Environment::Development)
            .with_timeouts(5000, 2000, 10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let config = BiomeOSConfigBuilder::new()
            .with_discovery_hosts(&["host1", "host2"])
            .with_discovery_ports(&[8080, 8081])
            .with_environment(Environment::Testing)
            .build();

        assert_eq!(config.system.environment, Environment::Testing);
        assert_eq!(config.primals.discovery.scan_hosts, vec!["host1", "host2"]);
        assert_eq!(config.primals.discovery.scan_ports, vec![8080, 8081]);
    }

    #[test]
    fn test_static_endpoints() {
        let config = BiomeOSConfigBuilder::new()
            .with_static_endpoint("test", "http://localhost:8080")
            .build();

        assert!(config
            .primals
            .discovery
            .static_endpoints
            .contains_key("test"));
        assert_eq!(
            config
                .primals
                .discovery
                .static_endpoints
                .get("test")
                .unwrap(),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_factory_methods() {
        let config = BiomeOSConfigBuilder::standard_primals().build();

        assert!(config
            .primals
            .discovery
            .static_endpoints
            .contains_key("toadstool"));
        assert!(config
            .primals
            .discovery
            .static_endpoints
            .contains_key("songbird"));
        assert!(config
            .primals
            .discovery
            .static_endpoints
            .contains_key("nestgate"));
        assert!(config
            .primals
            .discovery
            .static_endpoints
            .contains_key("beardog"));
    }
}
