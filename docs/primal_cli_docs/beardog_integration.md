# 🐻 BearDog Integration Reference

**Source**: BearDog Team Response  
**Date**: December 25, 2025  
**Status**: ✅ Production Ready  
**Quality**: Grade A (91/100)

---

## ⚠️ IMPORTANT: BearDog is NOT a Server!

**Type**: Library + CLI Tool  
**Integration**: Cargo dependency (not service management)

**This means**:
- ❌ No start/stop commands
- ❌ No ports to manage
- ❌ No lifecycle management needed
- ✅ Other primals **import** BearDog as a library

---

## Quick Reference

```yaml
beardog:
  type: "library_and_cli"
  is_server: false
  
  # How to integrate
  integration_method: "cargo_dependency"
  cargo_toml: |
    [dependencies]
    beardog-core = "0.9.4"
    beardog-tunnel = "0.9.0"
  
  # CLI for operations
  cli_binary: "./beardog"
  cli_commands:
    - "status"      # Health check
    - "key"         # Key management
    - "encrypt"     # Encryption
    - "decrypt"     # Decryption
    - "hsm"         # HSM operations
  
  # Health check
  health_check_cli: "./beardog status"
  health_format: "JSON"
  
  # Configuration
  config_method: "environment_variables"
  key_env_vars:
    - "BEARDOG_LOG_LEVEL"
    - "BEARDOG_HSM_TYPE"
    - "BEARDOG_STORAGE_PATH"
```

---

## Integration Methods (Sovereignty-Respecting)

### ⚠️ CRITICAL: Respect Primal Sovereignty

**BiomeOS does NOT force BearDog on other primals!**

### Method 1: BiomeOS CLI Adapter (PRIMARY)

**BiomeOS uses BearDog CLI for ecosystem operations**:

```rust
// BiomeOS adapter (doesn't violate sovereignty)
impl BearDogCliAdapter {
    pub async fn check_availability() -> Result<bool> {
        Command::new("./beardog")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
    }
    
    pub async fn check_health() -> Result<HealthStatus> {
        let output = Command::new("./beardog")
            .arg("status")
            .output()?;
        
        let health: HealthStatus = 
            serde_json::from_slice(&output.stdout)?;
        Ok(health)
    }
}
```

### Method 2: BiomeOS Chimera System (COMPOSITION)

**BiomeOS composes primals via chimeras**:

```rust
// BiomeOS chimera includes BearDog
// crates/biomeos-chimera/Cargo.toml
[dependencies]
beardog-core = "0.9.4"

// Chimera can provide security to fused primals
impl Chimera {
    fn with_security(&mut self) -> &mut Self {
        // Chimera uses beardog-core
        self.security = Some(CryptoService::new()?);
        self
    }
}
```

### Method 3: Primal Self-Integration (THEIR CHOICE)

**Each primal decides whether to add BearDog**:

```toml
# In Songbird's Cargo.toml (THEIR choice, not BiomeOS forcing it)
[dependencies]
beardog-core = "0.9.4"  # ← Songbird decides to add this

# In NestGate's Cargo.toml (THEIR choice)
[dependencies]
beardog-core = "0.9.4"  # ← NestGate decides to add this
```

**BiomeOS role**: Provide guidance, not enforcement

### ❌ What BiomeOS Must NOT Do

```rust
// ❌ WRONG: BiomeOS adding BearDog to other primals' Cargo.toml
fn inject_beardog_into_primal(&self, primal_path: &Path) -> Result<()> {
    // This violates primal sovereignty!
    self.add_dependency(&cargo_toml, "beardog-core", "0.9.4")?;
}

// ✅ RIGHT: BiomeOS provides guidance
fn provide_integration_guide(&self) -> Guide {
    Guide::new("If your primal needs security, consider adding beardog-core")
}
```

---

## CLI Commands

### Version & Help
```bash
./beardog --version
./beardog --help
```

### Status Check (Health)
```bash
./beardog status
```

**Output** (JSON):
```json
{
  "status": "healthy",
  "version": "0.9.4",
  "timestamp": "2025-12-25T03:00:00Z",
  "components": {
    "hsm": {
      "status": "healthy",
      "available": ["yubikey", "tpm", "software"],
      "active": "yubikey"
    },
    "crypto": {
      "status": "healthy",
      "providers": ["rustcrypto", "ring"]
    },
    "storage": {
      "status": "healthy",
      "path": "/var/lib/beardog"
    }
  },
  "capabilities": ["crypto", "hsm", "genetics", "btsp", "cross_primal"]
}
```

### Key Management
```bash
./beardog key generate --name my-key
./beardog key list
./beardog key export --name my-key
./beardog key import --file key.json
./beardog key derive --parent parent-key --name child-key
```

### Encryption/Decryption
```bash
./beardog encrypt --input file.txt --output file.enc
./beardog decrypt --input file.enc --output file.txt
./beardog stream_encrypt --input large-file.bin  # For 100GB+ files
./beardog stream_decrypt --input large-file.enc
```

### HSM Operations
```bash
./beardog hsm discover
./beardog hsm list
```

### Cross-Primal Messaging
```bash
./beardog cross_primal --to nestgate --message "Hello"
```

---

## Configuration

### Environment Variables (57 total)

**Key Variables**:
```bash
BEARDOG_LOG_LEVEL=info              # Logging level
BEARDOG_HSM_TYPE=auto_detect        # yubikey, tpm, software, auto_detect
BEARDOG_STORAGE_PATH=/var/lib/beardog
BEARDOG_CONFIG_FILE=/etc/beardog/config.toml
```

### Config File (Optional)
Location: `/etc/beardog/config.toml` or via `BEARDOG_CONFIG_FILE`

```toml
[general]
log_level = "info"

[hsm]
type = "auto_detect"  # or yubikey, tpm, software
prefer_hardware = true

[storage]
path = "/var/lib/beardog"

[crypto]
default_algorithm = "aes-256-gcm"
```

---

## Available Libraries

```toml
[dependencies]
beardog-core = "0.9.4"        # Core crypto services
beardog-tunnel = "0.9.0"      # BTSP secure tunneling
beardog-security = "0.1.0"    # Security primitives
beardog-genetics = "0.1.0"    # Genetic cryptography
beardog-types = "3.0.0"       # Common types
```

---

## Capabilities

BearDog provides:
- **crypto**: Cryptographic operations (encrypt, decrypt, sign, verify)
- **hsm**: HSM integration (YubiKey, TPM, Android, iOS, Software)
- **genetics**: Genetic cryptography (lineage-based encryption)
- **btsp**: Secure tunneling protocol
- **cross_primal**: Cross-primal secure messaging

---

## Quality Metrics

- **Grade**: A (91/100)
- **Tests**: 3,785+ (100% pass rate)
- **Coverage**: 85%
- **Memory Safety**: TOP 0.1%
- **Unsafe Blocks**: 15 (Android JNI only)
- **Zero Hardcoding**: ✅
- **Production Ready**: ✅

---

## NOT APPLICABLE (BearDog is not a server)

These concepts don't apply to BearDog:
- ❌ Start/stop commands
- ❌ Port configuration
- ❌ Service discovery
- ❌ Graceful shutdown signals
- ❌ Load balancing
- ❌ Process management

---

## BiomeOS Integration Guide

### For Primal Adapter Pattern

**BearDog is different** - it's a library, not a service to manage.

```rust
// BiomeOS doesn't manage BearDog lifecycle
// Instead, guide primals to add BearDog as dependency

pub struct BearDogAdapter {
    // CLI adapter for status checks only
}

impl BearDogAdapter {
    pub fn check_availability() -> Result<bool> {
        // Check if beardog CLI is available
        Command::new("./beardog")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
    }
    
    pub async fn check_health() -> Result<HealthStatus> {
        // Run CLI health check
        let output = Command::new("./beardog")
            .arg("status")
            .output()?;
        
        let health: HealthStatus = serde_json::from_slice(&output.stdout)?;
        Ok(health)
    }
}

// Other primals import beardog-core directly (no adapter needed)
```

### Recommendation for Other Primals

**When integrating Songbird, NestGate, ToadStool, etc.**:

Add to their documentation:
```markdown
## Security with BearDog

For enhanced security features, add BearDog to your Cargo.toml:

```toml
[dependencies]
beardog-core = "0.9.4"
```

Then use in your code:
```rust
use beardog_core::CryptoService;
let crypto = CryptoService::new()?;
```
```

---

## Architecture Diagram

```
┌──────────────────────────────────┐
│         BiomeOS                  │
│  (manages primal lifecycle)      │
└────────────┬─────────────────────┘
             │
     ┌───────┴────────┬────────────┐
     ▼                ▼            ▼
┌─────────┐      ┌─────────┐  ┌─────────┐
│Songbird │      │NestGate │  │ToadStool│
│(server) │      │(server) │  │(server) │
└────┬────┘      └────┬────┘  └────┬────┘
     │                │            │
     │  uses          │  uses      │  uses
     │  beardog-core  │  beardog   │  beardog
     │                │            │
     └────────┬───────┴────────────┘
              ▼
       ┌──────────────┐
       │   BearDog    │  ◄── Library (not a server)
       │  (library)   │      Imported by others
       └──────────────┘
```

---

## Next Steps

### For BiomeOS
1. **Document** library-based integration pattern
2. **Guide** other primals on adding BearDog
3. **Optional**: Use BearDog internally for security
4. **Create** CLI adapter for health checks

### For Other Primals (Future)
When integrating Songbird, NestGate, etc.:
1. Add BearDog to their Cargo.toml
2. Use for security features
3. Document their BearDog usage

---

## Contact

- **Team**: @beardog-team
- **Documentation**: See BearDog repo
- **Questions**: Respond to integration document

---

**Documented By**: BearDog Team  
**For**: BiomeOS Integration  
**Date**: December 25, 2025  
**Status**: ✅ Ready (library integration)  
**Type**: Library + CLI, NOT a server

---

*"Security as a library, not a service."* 🐻✨

