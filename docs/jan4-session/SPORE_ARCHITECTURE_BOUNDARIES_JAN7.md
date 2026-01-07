# 🏗️ Spore System: Architectural Boundaries - Composability

**Date**: January 7, 2026  
**Status**: Architectural Correction  
**Principle**: "Complexity is a composable solution"

---

## 🎯 Core Principle

> **biomeOS orchestrates. BearDog secures.**

biomeOS should **NOT** recreate BearDog's cryptography. It should **USE** BearDog as a composable security component.

---

## ❌ What We Should NOT Do

### Anti-Pattern: Duplicate Crypto in biomeOS

```rust
// ❌ WRONG - biomeOS shouldn't implement crypto!
// crates/biomeos-spore/src/crypto.rs
use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive_child_key(parent: &[u8], info: &[u8]) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(Some(salt), parent);
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)?;
    Ok(okm)  // ❌ Duplicating BearDog's responsibility!
}

pub fn extract_family_id(seed: &[u8]) -> String {
    let hash = hash_sha256(seed);  // ❌ Crypto logic in biomeOS!
    // ... extract ID
}
```

**Problems**:
1. ❌ Duplicates BearDog's genetic lineage code
2. ❌ biomeOS now has security-critical code
3. ❌ Two places to maintain crypto logic
4. ❌ Two places to audit for security
5. ❌ Violates single responsibility
6. ❌ Defeats composability

---

## ✅ What We SHOULD Do

### Correct Pattern: Composable Architecture

```rust
// ✅ CORRECT - biomeOS manages files, BearDog handles crypto
// crates/biomeos-spore/src/seed.rs

/// biomeOS's responsibility: File management ONLY
pub struct FamilySeed {
    /// Path to seed file (biomeOS manages storage)
    file_path: PathBuf,
}

impl FamilySeed {
    /// Generate entropy and write to file
    /// (Just the raw bytes, no crypto processing)
    pub fn generate_and_write(path: PathBuf) -> Result<Self> {
        use rand::RngCore;
        
        // Generate random bytes (not crypto derivation!)
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        
        // Write to file
        fs::write(&path, &bytes)?;
        
        // Set permissions (OS-level, not crypto)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }
        
        Ok(Self { file_path: path })
    }
    
    /// Load existing seed file
    pub fn from_file(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(Error::SeedFileNotFound);
        }
        
        // Just verify it's 32 bytes, don't process it!
        let bytes = fs::read(&path)?;
        if bytes.len() != 32 {
            return Err(Error::InvalidSeedLength);
        }
        
        Ok(Self { file_path: path })
    }
    
    /// Pass seed file path to BearDog
    /// (BearDog does ALL the crypto!)
    pub fn configure_beardog(&self) -> Result<()> {
        // biomeOS just sets the env var
        // BearDog reads, derives, processes
        std::env::set_var(
            "BEARDOG_FAMILY_SEED_FILE",
            self.file_path.to_str().unwrap()
        );
        Ok(())
    }
}
```

**Benefits**:
1. ✅ biomeOS: File I/O and orchestration
2. ✅ BearDog: All cryptography and security
3. ✅ Single source of truth for crypto
4. ✅ Single place to audit
5. ✅ Composable architecture
6. ✅ Clear boundaries

---

## 🏗️ Responsibility Matrix

| Task | biomeOS | BearDog |
|------|---------|---------|
| **Generate random bytes** | ✅ Yes | - |
| **Write bytes to file** | ✅ Yes | - |
| **Set file permissions** | ✅ Yes | - |
| **Read seed file** | ✅ Yes (path) | ✅ Yes (contents) |
| **HKDF key derivation** | ❌ NO | ✅ YES |
| **Extract family ID** | ❌ NO | ✅ YES |
| **Child key generation** | ❌ NO | ✅ YES |
| **Zeroization** | ❌ NO | ✅ YES |
| **Cryptographic verification** | ❌ NO | ✅ YES |

### Clear Boundary

```
┌─────────────────────────────────────────┐
│  biomeOS (Orchestration Layer)         │
│  ✅ USB mounting                        │
│  ✅ Directory structure                 │
│  ✅ File I/O (.family.seed)            │
│  ✅ tower.toml generation              │
│  ✅ Binary copying                      │
│  ✅ Permission setting                  │
└─────────────────────────────────────────┘
                  ↓ Passes file path
┌─────────────────────────────────────────┐
│  BearDog (Security Layer)               │
│  ✅ Read seed from file                 │
│  ✅ HKDF-SHA256 derivation              │
│  ✅ Family ID extraction                │
│  ✅ Child key generation                │
│  ✅ Zeroization                         │
│  ✅ Genetic lineage verification        │
└─────────────────────────────────────────┘
```

---

## 📝 Updated Implementation

### biomeOS Spore Module (Simplified)

**File**: `crates/biomeos-spore/src/lib.rs`

```rust
//! biomeOS Spore System
//! 
//! Responsibilities:
//! - USB device management
//! - File system operations
//! - Configuration generation
//! - Binary deployment
//! 
//! Security/Crypto: Delegated to BearDog!

pub mod seed;
pub mod spore;
pub mod usb;

pub use seed::FamilySeed;
pub use spore::{Spore, SporeConfig};
```

**File**: `crates/biomeos-spore/src/seed.rs`

```rust
use std::path::{Path, PathBuf};
use std::fs;

/// Family seed file manager
/// 
/// biomeOS manages the FILE, BearDog handles the CRYPTO
pub struct FamilySeed {
    file_path: PathBuf,
}

impl FamilySeed {
    /// Generate random bytes and write to file
    /// 
    /// NOTE: This generates entropy only!
    /// BearDog handles all cryptographic processing.
    pub fn generate_and_write<P: AsRef<Path>>(path: P) -> Result<Self> {
        use rand::RngCore;
        
        let path = path.as_ref().to_path_buf();
        
        // Generate 256 bits of entropy
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        
        // Write to file
        fs::write(&path, &bytes)?;
        
        // Secure permissions (Unix only)
        #[cfg(unix)]
        Self::set_secure_permissions(&path)?;
        
        Ok(Self { file_path: path })
    }
    
    /// Load existing seed file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            return Err(Error::SeedFileNotFound(path));
        }
        
        // Verify it's 32 bytes (but don't process the contents!)
        let metadata = fs::metadata(&path)?;
        if metadata.len() != 32 {
            return Err(Error::InvalidSeedLength {
                expected: 32,
                found: metadata.len(),
            });
        }
        
        Ok(Self { file_path: path })
    }
    
    /// Get file path (to pass to BearDog)
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
    
    /// Configure environment for BearDog to use this seed
    pub fn configure_beardog_env(&self) -> Result<()> {
        std::env::set_var(
            "BEARDOG_FAMILY_SEED_FILE",
            self.file_path.to_str()
                .ok_or(Error::InvalidPath)?
        );
        Ok(())
    }
    
    #[cfg(unix)]
    fn set_secure_permissions(path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);  // Owner read/write only
        fs::set_permissions(path, perms)?;
        Ok(())
    }
}

/// NO crypto functions here!
/// All crypto is in BearDog.
```

**File**: `crates/biomeos-spore/src/spore.rs`

```rust
use crate::seed::FamilySeed;
use std::path::{Path, PathBuf};

/// USB Spore structure
pub struct Spore {
    root_path: PathBuf,
    config: SporeConfig,
}

pub struct SporeConfig {
    pub label: String,
    pub node_id: String,
}

impl Spore {
    pub async fn create(mount_point: &Path, config: SporeConfig) -> Result<Self> {
        let root_path = mount_point.join("biomeOS");
        
        let mut spore = Self { root_path, config };
        
        // biomeOS responsibilities: orchestration only
        spore.create_directory_structure()?;
        spore.generate_seed_file()?;
        spore.create_tower_config()?;
        spore.copy_binaries().await?;
        
        Ok(spore)
    }
    
    fn create_directory_structure(&self) -> Result<()> {
        let dirs = ["bin", "primals", "primals/certs", "secrets", "logs"];
        for dir in &dirs {
            fs::create_dir_all(self.root_path.join(dir))?;
        }
        Ok(())
    }
    
    fn generate_seed_file(&self) -> Result<()> {
        let seed_path = self.root_path.join(".family.seed");
        FamilySeed::generate_and_write(&seed_path)?;
        Ok(())
    }
    
    fn create_tower_config(&self) -> Result<()> {
        // Generate tower.toml with seed FILE reference
        let config = format!(
            r#"[tower]
family = "auto"  # BearDog extracts from seed
concurrent_startup = true

[[primals]]
binary = "./primals/beardog"
provides = ["Security", "Encryption", "Trust"]

[primals.env]
# biomeOS provides FILE PATH
# BearDog handles crypto
BEARDOG_FAMILY_SEED_FILE = "${{USB_ROOT}}/.family.seed"
BEARDOG_NODE_ID = "{node_id}"
RUST_LOG = "info"

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[primals.env]
# BearDog will populate these after reading seed:
# SONGBIRD_FAMILY_ID = (auto from BearDog)
# SONGBIRD_NODE_ID = (from BEARDOG_NODE_ID)
SONGBIRD_NODE_ID = "{node_id}"
SECURITY_ENDPOINT = "unix:///tmp/beardog-${{FAMILY}}-{node_id}.sock"
RUST_LOG = "info"
"#,
            node_id = self.config.node_id,
        );
        
        fs::write(self.root_path.join("tower.toml"), config)?;
        Ok(())
    }
    
    async fn copy_binaries(&self) -> Result<()> {
        // Copy beardog, songbird, tower binaries
        // (File operations, no crypto)
        // ... implementation
        Ok(())
    }
}
```

---

## 🔄 Integration Flow

### 1. biomeOS Creates Spore
```bash
$ biomeos spore create --mount /media/usb --node tower1
```

**biomeOS does**:
1. Create directory structure ✅
2. Generate 32 random bytes ✅
3. Write to `.family.seed` ✅
4. Set file permissions (600) ✅
5. Create `tower.toml` with `BEARDOG_FAMILY_SEED_FILE` ✅
6. Copy binaries ✅

**biomeOS does NOT**:
- ❌ Process the seed bytes
- ❌ Derive keys
- ❌ Extract family ID
- ❌ Any cryptography

### 2. Tower Starts
```bash
$ cd /media/usb/biomeOS
$ ./bin/tower run --config tower.toml
```

**biomeOS orchestrator**:
1. Reads `tower.toml` ✅
2. Sets env: `BEARDOG_FAMILY_SEED_FILE=.family.seed` ✅
3. Starts BearDog primal ✅

### 3. BearDog Initializes
```rust
// Inside BearDog's initialization
fn initialize() -> Result<(String, String)> {
    // BearDog reads the file path from env
    let seed_path = env::var("BEARDOG_FAMILY_SEED_FILE")?;
    
    // BearDog reads and processes the seed
    let seed_bytes = fs::read(seed_path)?;
    
    // BearDog does ALL the crypto:
    let family_id = extract_family_id(&seed_bytes);  // ✅ BearDog
    let child_key = derive_child_key(&seed_bytes);   // ✅ BearDog
    let node_id = compute_node_id(&child_key);       // ✅ BearDog
    
    // Zeroize parent seed (security!)
    seed_bytes.zeroize();  // ✅ BearDog
    
    Ok((family_id, node_id))
}
```

**BearDog does**:
- ✅ Read seed file contents
- ✅ HKDF-SHA256 key derivation
- ✅ Extract family ID
- ✅ Generate child keys
- ✅ Zeroize sensitive data
- ✅ All cryptography!

---

## 📊 Dependencies

### biomeOS Spore Module
```toml
[dependencies]
rand = "0.8"        # Just for entropy generation
tokio = "1.0"       # Async I/O
serde = "1.0"       # Config serialization
# NO crypto libraries! That's BearDog's job.
```

### BearDog (Has Crypto)
```toml
[dependencies]
hkdf = "0.12"       # ✅ BearDog has this
sha2 = "0.10"       # ✅ BearDog has this
zeroize = "1.7"     # ✅ BearDog has this
# All crypto is here!
```

---

## 🎯 Principle: Composability

```
biomeOS = Orchestration Layer
    ↓ Composes with
BearDog = Security Layer
    ↓ Composes with
Songbird = Discovery Layer
    ↓ Composes with
ToadStool = Workload Layer
```

**Each layer has clear responsibilities.**
**No duplication of complex logic.**
**Composable, not monolithic.**

---

## ✅ Summary

### What biomeOS Does (File Management)
- ✅ Generate random entropy
- ✅ Write `.family.seed` file
- ✅ Set file permissions
- ✅ Create directory structure
- ✅ Generate `tower.toml`
- ✅ Copy binaries
- ✅ Set environment variables

### What BearDog Does (Security/Crypto)
- ✅ Read seed file
- ✅ HKDF key derivation
- ✅ Family ID extraction
- ✅ Child key generation
- ✅ Genetic lineage verification
- ✅ Zeroization
- ✅ All cryptography

### Architectural Benefit
> **"Complexity is a composable solution"**

- Single source of truth for crypto (BearDog)
- Single place to audit security
- Clear architectural boundaries
- Composable, not duplicated
- Modern, idiomatic Rust

---

**Date**: January 7, 2026, 22:30 UTC  
**Status**: Architectural boundaries clarified  
**Principle**: Compose, don't duplicate  
**Result**: biomeOS orchestrates, BearDog secures ✅

