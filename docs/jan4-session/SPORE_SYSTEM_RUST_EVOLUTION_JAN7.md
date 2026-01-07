# 🦀 Spore System: Bash → Modern Idiomatic Rust

**Date**: January 7, 2026  
**Status**: Deep Debt Evolution Plan  
**Philosophy**: "Bash is jelly strings - find solution fast, then evolve robustly"

---

## 🎯 Problem Statement

### Current: Bash Scripts (Jelly Strings)
```bash
# scripts/create-usb-family-seed.sh
FAMILY_SEED=$(openssl rand -base64 32)
echo "$FAMILY_SEED" > secrets/family-genesis.key
```

**Issues**:
- ❌ Error handling via `set -e` (crude)
- ❌ No type safety
- ❌ String concatenation errors
- ❌ External dependencies (openssl, sha256sum)
- ❌ Platform-specific (Linux assumptions)
- ❌ Hard to test
- ❌ Not integrated with biomeOS

### Target: Idiomatic Rust
```rust
// biomeOS native spore creation
pub async fn create_spore(config: SporeConfig) -> Result<Spore> {
    let seed = FamilySeed::generate()?;
    let spore = Spore::new(seed, config)?;
    spore.write_to_device(config.device_path).await?;
    Ok(spore)
}
```

**Benefits**:
- ✅ Compile-time type safety
- ✅ Explicit error handling
- ✅ Zero external dependencies (pure Rust crypto)
- ✅ Cross-platform (works on Linux, macOS, Windows)
- ✅ Unit testable
- ✅ Integrated with biomeOS CLI

---

## 🏗️ Architecture Design

### Module Structure
```
crates/
└── biomeos-spore/
    ├── Cargo.toml
    └── src/
        ├── lib.rs                  # Public API
        ├── seed.rs                 # Seed generation & derivation
        ├── spore.rs                # Spore structure & operations
        ├── usb.rs                  # USB device operations
        ├── crypto.rs               # Cryptographic primitives
        ├── sibling.rs              # Sibling cloning logic
        └── verify.rs               # Verification & integrity
```

### Type System
```rust
/// Family seed (parent DNA)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct FamilySeed {
    bytes: [u8; 32],           // 256-bit entropy
    family_id: String,         // "nat0"
    genesis_hash: [u8; 32],    // SHA256 for verification
}

/// Child seed (tower-specific)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ChildSeed {
    bytes: [u8; 32],
    node_id: String,           // "tower1"
    parent_hash: [u8; 32],     // Links to parent
}

/// USB Spore
pub struct Spore {
    family_seed: FamilySeed,
    config: SporeConfig,
    binaries: BinarySet,
    metadata: SporeMetadata,
}

/// Spore configuration
pub struct SporeConfig {
    device_path: PathBuf,      // /dev/sdb
    label: String,             // biomeOS1
    node_id: String,           // tower1
    family_id: String,         // nat0
}
```

---

## 🔧 Implementation Plan

### Phase 1: Core Crypto Module

**File**: `crates/biomeos-spore/src/crypto.rs`

```rust
use rand::RngCore;
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Generate cryptographically secure random bytes
pub fn generate_entropy(len: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    Ok(bytes)
}

/// Derive child key using HKDF-SHA256
pub fn derive_child_key(
    parent: &[u8],
    info: &[u8],
    salt: &[u8],
) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(Some(salt), parent);
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .map_err(|e| Error::CryptoError(format!("HKDF failed: {}", e)))?;
    Ok(okm)
}

/// SHA256 hash
pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Extract family ID (first 4 alphanumeric lowercase)
pub fn extract_family_id(seed: &[u8]) -> String {
    let hash = hash_sha256(seed);
    let hex = hex::encode(&hash[..4]);
    hex.chars()
        .filter(|c| c.is_alphanumeric())
        .take(4)
        .collect::<String>()
        .to_lowercase()
}
```

**Dependencies** (`Cargo.toml`):
```toml
[dependencies]
rand = "0.8"
sha2 = "0.10"
hkdf = "0.12"
zeroize = { version = "1.7", features = ["derive"] }
hex = "0.4"
```

---

### Phase 2: Seed Module

**File**: `crates/biomeos-spore/src/seed.rs`

```rust
use crate::crypto;
use zeroize::{Zeroize, ZeroizeOnDrop};
use serde::{Serialize, Deserialize};

/// Family seed (parent DNA)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct FamilySeed {
    #[zeroize(skip)]
    family_id: String,
    bytes: [u8; 32],
    #[zeroize(skip)]
    genesis_hash: [u8; 32],
}

impl FamilySeed {
    /// Generate new family seed
    pub fn generate() -> Result<Self> {
        let bytes = crypto::generate_entropy(32)?
            .try_into()
            .map_err(|_| Error::InvalidSeedLength)?;
        
        let family_id = crypto::extract_family_id(&bytes);
        let genesis_hash = crypto::hash_sha256(&bytes);
        
        Ok(Self {
            family_id,
            bytes,
            genesis_hash,
        })
    }
    
    /// Load from file (binary format)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = fs::read(path)?;
        if bytes.len() != 32 {
            return Err(Error::InvalidSeedLength);
        }
        
        let bytes: [u8; 32] = bytes.try_into().unwrap();
        let family_id = crypto::extract_family_id(&bytes);
        let genesis_hash = crypto::hash_sha256(&bytes);
        
        Ok(Self {
            family_id,
            bytes,
            genesis_hash,
        })
    }
    
    /// Write to file (binary format, 600 permissions)
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::write(&path, &self.bytes)?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }
        
        Ok(())
    }
    
    /// Derive child seed
    pub fn derive_child(&self, node_id: &str) -> Result<ChildSeed> {
        let hostname = gethostname::gethostname()
            .into_string()
            .unwrap_or_else(|_| "unknown".to_string());
        
        let mut info = Vec::new();
        info.extend_from_slice(node_id.as_bytes());
        info.extend_from_slice(hostname.as_bytes());
        
        let salt = b"biomeos-genetic-lineage-v1";
        let child_bytes = crypto::derive_child_key(&self.bytes, &info, salt)?;
        
        Ok(ChildSeed {
            bytes: child_bytes,
            node_id: node_id.to_string(),
            parent_hash: self.genesis_hash,
        })
    }
    
    pub fn family_id(&self) -> &str {
        &self.family_id
    }
    
    pub fn genesis_hash(&self) -> &[u8; 32] {
        &self.genesis_hash
    }
}

/// Child seed (tower-specific)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ChildSeed {
    bytes: [u8; 32],
    #[zeroize(skip)]
    node_id: String,
    #[zeroize(skip)]
    parent_hash: [u8; 32],
}

impl ChildSeed {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
    
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    
    pub fn verify_family(&self, expected_parent_hash: &[u8; 32]) -> bool {
        &self.parent_hash == expected_parent_hash
    }
}

/// Public family information (safe to share)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyInfo {
    pub family_id: String,
    pub genesis_hash: String,    // Hex encoded
    pub created_at: String,
    pub capabilities: Vec<String>,
}

impl From<&FamilySeed> for FamilyInfo {
    fn from(seed: &FamilySeed) -> Self {
        Self {
            family_id: seed.family_id.clone(),
            genesis_hash: hex::encode(&seed.genesis_hash),
            created_at: chrono::Utc::now().to_rfc3339(),
            capabilities: vec![
                "tower".to_string(),
                "orchestration".to_string(),
                "federation".to_string(),
            ],
        }
    }
}
```

---

### Phase 3: Spore Module

**File**: `crates/biomeos-spore/src/spore.rs`

```rust
use crate::seed::{FamilySeed, FamilyInfo};
use std::path::{Path, PathBuf};

/// USB Spore structure
pub struct Spore {
    family_seed: FamilySeed,
    config: SporeConfig,
    root_path: PathBuf,
}

/// Spore configuration
#[derive(Debug, Clone)]
pub struct SporeConfig {
    pub label: String,        // biomeOS1
    pub node_id: String,      // tower1
    pub mount_point: PathBuf, // /media/usb
}

impl Spore {
    /// Create new spore
    pub fn new(family_seed: FamilySeed, config: SporeConfig) -> Self {
        Self {
            family_seed,
            root_path: config.mount_point.join("biomeOS"),
            config,
        }
    }
    
    /// Initialize spore structure on USB
    pub async fn initialize(&self) -> Result<()> {
        // Create directory structure
        self.create_directory_structure()?;
        
        // Write family seed
        let seed_path = self.root_path.join(".family.seed");
        self.family_seed.write_to_file(&seed_path)?;
        
        // Write public info
        let info: FamilyInfo = (&self.family_seed).into();
        let secrets_dir = self.root_path.join("secrets");
        fs::create_dir_all(&secrets_dir)?;
        
        let pub_path = secrets_dir.join("family-genesis.pub");
        let json = serde_json::to_string_pretty(&info)?;
        fs::write(&pub_path, json)?;
        
        // Create tower.toml
        self.create_tower_config()?;
        
        // Copy binaries
        self.copy_binaries().await?;
        
        Ok(())
    }
    
    fn create_directory_structure(&self) -> Result<()> {
        let dirs = [
            "bin",
            "primals",
            "primals/certs",
            "secrets",
            "logs",
            "config",
        ];
        
        for dir in &dirs {
            fs::create_dir_all(self.root_path.join(dir))?;
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let secrets = self.root_path.join("secrets");
            let mut perms = fs::metadata(&secrets)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&secrets, perms)?;
        }
        
        Ok(())
    }
    
    fn create_tower_config(&self) -> Result<()> {
        let config = format!(
            r#"# BiomeOS Tower Configuration
# Port-Free Architecture - Unix Sockets + UDP Multicast

[tower]
family = "{family_id}"
concurrent_startup = true

# BearDog - Security Primal (Port-Free!)
[[primals]]
binary = "./primals/beardog"
provides = ["Security", "Encryption", "Trust"]
requires = []

[primals.env]
BEARDOG_FAMILY_SEED_FILE = "${{USB_ROOT}}/.family.seed"
BEARDOG_NODE_ID = "{node_id}"
RUST_LOG = "info"

# Songbird - Discovery Orchestrator (UDP Multicast)
[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[primals.env]
SONGBIRD_FAMILY_ID = "{family_id}"
SONGBIRD_NODE_ID = "{node_id}"
SECURITY_ENDPOINT = "unix:///tmp/beardog-{family_id}-{node_id}.sock"
RUST_LOG = "info"
"#,
            family_id = self.family_seed.family_id(),
            node_id = self.config.node_id,
        );
        
        let config_path = self.root_path.join("tower.toml");
        fs::write(config_path, config)?;
        
        Ok(())
    }
    
    async fn copy_binaries(&self) -> Result<()> {
        // Copy from primalBins/
        let source_dir = PathBuf::from("primalBins");
        let target_dir = self.root_path.join("primals");
        
        for binary in &["beardog", "songbird"] {
            let src = source_dir.join(binary);
            let dst = target_dir.join(binary);
            
            if src.exists() {
                tokio::fs::copy(&src, &dst).await?;
                
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = tokio::fs::metadata(&dst).await?.permissions();
                    perms.set_mode(0o755);
                    tokio::fs::set_permissions(&dst, perms).await?;
                }
            }
        }
        
        // Copy tower binary
        let tower_src = PathBuf::from("target/release/tower");
        let tower_dst = self.root_path.join("bin/tower");
        if tower_src.exists() {
            tokio::fs::copy(&tower_src, &tower_dst).await?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = tokio::fs::metadata(&tower_dst).await?.permissions();
                perms.set_mode(0o755);
                tokio::fs::set_permissions(&tower_dst, perms).await?;
            }
        }
        
        Ok(())
    }
    
    /// Clone this spore to create a sibling
    pub async fn clone_sibling(
        &self,
        new_config: SporeConfig,
    ) -> Result<Spore> {
        // Reuse same family seed (siblings!)
        let sibling_seed = FamilySeed::from_file(
            self.root_path.join(".family.seed")
        )?;
        
        let sibling = Spore::new(sibling_seed, new_config);
        sibling.initialize().await?;
        
        Ok(sibling)
    }
}
```

---

### Phase 4: CLI Integration

**File**: `crates/biomeos-cli/src/commands/spore.rs`

```rust
use biomeos_spore::{FamilySeed, Spore, SporeConfig};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct SporeArgs {
    #[command(subcommand)]
    command: SporeCommands,
}

#[derive(Subcommand)]
enum SporeCommands {
    /// Create new USB spore with genetic lineage
    Create {
        /// Mount point (e.g., /media/usb)
        #[arg(short, long)]
        mount: PathBuf,
        
        /// Label (e.g., biomeOS1)
        #[arg(short, long)]
        label: String,
        
        /// Node ID (e.g., tower1)
        #[arg(short, long)]
        node: String,
    },
    
    /// Clone existing spore to create sibling
    Clone {
        /// Source spore mount point
        #[arg(short, long)]
        from: PathBuf,
        
        /// Target spore mount point
        #[arg(short, long)]
        to: PathBuf,
        
        /// New node ID for sibling
        #[arg(short, long)]
        node: String,
    },
    
    /// Verify spore integrity
    Verify {
        /// Spore mount point
        mount: PathBuf,
    },
    
    /// Show spore information
    Info {
        /// Spore mount point
        mount: PathBuf,
    },
}

pub async fn handle_spore(args: SporeArgs) -> Result<()> {
    match args.command {
        SporeCommands::Create { mount, label, node } => {
            create_spore(mount, label, node).await
        }
        SporeCommands::Clone { from, to, node } => {
            clone_spore(from, to, node).await
        }
        SporeCommands::Verify { mount } => {
            verify_spore(mount).await
        }
        SporeCommands::Info { mount } => {
            info_spore(mount).await
        }
    }
}

async fn create_spore(
    mount: PathBuf,
    label: String,
    node: String,
) -> Result<()> {
    println!("🔐 Creating new USB spore...");
    
    // Generate family seed
    let family_seed = FamilySeed::generate()?;
    println!("   Family ID: {}", family_seed.family_id());
    println!("   Genesis Hash: {}", hex::encode(family_seed.genesis_hash()));
    
    // Create spore
    let config = SporeConfig {
        label,
        node_id: node,
        mount_point: mount,
    };
    
    let spore = Spore::new(family_seed, config);
    spore.initialize().await?;
    
    println!("✅ Spore created successfully!");
    println!("   Location: {}/biomeOS", spore.root_path().display());
    
    Ok(())
}

async fn clone_spore(
    from: PathBuf,
    to: PathBuf,
    node: String,
) -> Result<()> {
    println!("🔄 Cloning spore to create sibling...");
    
    // Load source spore
    let source_config = SporeConfig {
        label: "source".to_string(),
        node_id: "temp".to_string(),
        mount_point: from,
    };
    
    let source_seed = FamilySeed::from_file(
        from.join("biomeOS/.family.seed")
    )?;
    let source = Spore::new(source_seed, source_config);
    
    // Clone to new location
    let target_config = SporeConfig {
        label: format!("biomeOS-{}", node),
        node_id: node,
        mount_point: to,
    };
    
    let sibling = source.clone_sibling(target_config).await?;
    
    println!("✅ Sibling spore created!");
    println!("   Shares family: {}", sibling.family_id());
    
    Ok(())
}
```

**Usage**:
```bash
# Create new spore
biomeos spore create \
  --mount /media/usb \
  --label biomeOS1 \
  --node tower1

# Clone sibling
biomeos spore clone \
  --from /media/usb1 \
  --to /media/usb2 \
  --node tower2

# Verify spore
biomeos spore verify /media/usb1

# Show info
biomeos spore info /media/usb1
```

---

## 📊 Migration Strategy

### Phase 1: Create Rust Module (Parallel)
- Create `crates/biomeos-spore/`
- Implement core functionality
- Add comprehensive tests
- **Keep bash scripts working** (don't break current system)

### Phase 2: CLI Integration
- Add `biomeos spore` commands
- Wire up to Rust implementation
- Test side-by-side with bash scripts

### Phase 3: Deprecate Bash
- Mark bash scripts as deprecated
- Update documentation to use CLI
- Move scripts to `scripts/legacy/`

### Phase 4: Remove Bash (Optional)
- After 1-2 releases of Rust version
- Keep as reference if needed
- Full Rust spore system

---

## 🎯 Benefits

### Bash → Rust Evolution

| Aspect | Bash (Jelly) | Rust (Robust) |
|--------|--------------|---------------|
| **Type Safety** | ❌ Strings everywhere | ✅ Compile-time types |
| **Error Handling** | ⚠️ `set -e` | ✅ `Result<T, E>` |
| **Testing** | ❌ Hard to test | ✅ Unit testable |
| **Dependencies** | ⚠️ External (openssl) | ✅ Pure Rust crypto |
| **Cross-Platform** | ❌ Linux-only | ✅ Works everywhere |
| **Integration** | ❌ Shell scripts | ✅ Native biomeOS |
| **Memory Safety** | ⚠️ No zeroization | ✅ Zeroize on drop |
| **Performance** | ⚠️ Process spawning | ✅ Native binary |

---

## 🚀 Next Steps

### Immediate
1. Create `crates/biomeos-spore/` module
2. Implement seed generation & derivation
3. Add tests

### Short-Term
1. Implement full Spore struct
2. Add CLI commands
3. Test parallel to bash scripts

### Medium-Term
1. Migrate all spore operations to Rust
2. Deprecate bash scripts
3. Update documentation

---

**Date**: January 7, 2026, 22:15 UTC  
**Status**: Evolution plan ready  
**Philosophy**: "Bash found the solution, Rust makes it robust"  
**Next**: Implement `biomeos-spore` crate

