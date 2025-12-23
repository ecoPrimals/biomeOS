# User Management with BearDog Integration

**Status:** Production Ready | **Version:** 1.0.0 | **Date:** January 2025

---

## Overview

biomeOS user management is **fully integrated** with BearDog (the security primal) for comprehensive key and secret management. This integration provides enterprise-grade security, cryptographic key management, and unified authentication across all biomeOS components.

## Key Features

### 🔐 **BearDog Key Management**
- **Replaces traditional password hashing** with BearDog key references
- **Genetic BearDog keys** for hierarchical access control
- **Automatic key rotation** with configurable intervals
- **HSM integration** for hardware-backed key storage
- **Cross-primal key sharing** for service-to-service authentication

### 🔑 **Advanced Authentication Methods**
- **Password authentication** (validated through BearDog)
- **SSH key authentication** (keys stored in BearDog HSM)
- **API key authentication** (managed by BearDog)
- **Genetic BearDog keys** (inherited access levels)
- **Biometric authentication** (processed through BearDog)

### 🛡️ **Security Context Integration**
- **Threat assessment** for each authentication attempt
- **Compliance enforcement** (SOC2, FIPS140, GDPR, HIPAA)
- **Real-time security monitoring** with BearDog integration
- **Audit trail** for all user operations
- **Session security** with BearDog-managed contexts

## Configuration

### Basic BearDog Integration

```toml
[beardog_config]
endpoint = "https://beardog.biome.local:8443"
key_management_enabled = true
secret_storage_enabled = true
audit_logging_enabled = true
hsm_integration_enabled = true
compliance_mode = "soc2"
security_level = "confidential"

[beardog_config.auth_method]
type = "mutual_tls"
cert_path = "/etc/biomeos/certs/user-manager.crt"
key_path = "/etc/biomeos/certs/user-manager.key"
ca_path = "/etc/biomeos/certs/beardog-ca.crt"
```

### Genetic BearDog Keys

```toml
enable_genetic_keys = true

[genetic_key_config]
parent_key_fingerprint = "beardog_prime_key_abc123"
default_access_level = "PowerUser"
key_validity_days = 365
auto_renewal = true
```

## Usage Examples

### 1. Creating a User with BearDog Integration

```rust
use biomeos_system::users::*;
use biomeos_core::{BeardogAccessLevel, GeneticBeardogKey};

// Initialize user manager with BearDog
let config = UserConfig::default(); // BearDog enabled by default
let user_manager = UserManager::new(config);
await user_manager.initialize().await?;

// Create user with genetic BearDog key
let user_id = user_manager.create_user_with_beardog(
    "alice",
    UserAuthMethod::Password { 
        password: "secure_password".to_string() 
    },
    BeardogAccessLevel::SmallBusiness, // Cost-effective access level
    Some("Alice Cooper".to_string())
).await?;

println!("Created user with BearDog integration: {}", user_id);
```

---

**This integration provides enterprise-grade security while maintaining the simplicity and elegance of biomeOS user management.**
