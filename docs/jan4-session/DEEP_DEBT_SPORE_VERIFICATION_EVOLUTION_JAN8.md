# 🔧 Deep Debt: Spore Verification & Heterogeneous Deployment Evolution

**Date**: January 8, 2026 (Late Evening)  
**Status**: 🚀 **EVOLUTION OPPORTUNITY**  
**Goal**: Evolve bash scripts to modern idiomatic Rust, enable heterogeneous deployments

---

## 🎯 Strategic Insight

**Current Issue**: Stale binaries on 3 spores (gamma, delta, epsilon)  
**Deep Debt Opportunity**: This reveals gaps in our deployment verification system  
**Evolution Goal**: Move from bash "jelly strings" to fast, safe, type-safe Rust

---

## 🔍 Current State Analysis

### What We Have (Bash Scripts):

1. **`harvest-primals.sh`** - Pulls and builds binaries
   - ✅ Works but bash "jelly string"
   - ❌ No version tracking
   - ❌ No pre-deployment validation
   - ❌ Manual process

2. **`verify-nucleus.sh`** - Checks nucleusBin integrity
   - ✅ Works but basic checks
   - ❌ Only checks existence and permissions
   - ❌ No version manifest
   - ❌ No spore comparison

3. **No spore verification** - Critical gap!
   - ❌ No pre-deployment checks
   - ❌ No binary freshness validation
   - ❌ No version tracking per spore
   - ❌ No heterogeneous deployment support

### What We Need (Modern Rust):

1. **Type-safe verification system**
   - Version tracking with semver
   - Binary checksums and validation
   - Pre-deployment checks
   - Spore health reports

2. **Heterogeneous deployment support**
   - Different versions on different spores
   - Version compatibility matrix
   - Controlled rollout capabilities
   - Gradual upgrade testing

3. **Composable CLI tools**
   - `biomeos verify nucleus` - Check nucleusBin
   - `biomeos verify spore <path>` - Check spore freshness
   - `biomeos verify all` - Check all mounted spores
   - `biomeos deploy plan` - Preview deployment changes

---

## 🏗️ Architecture: Spore Verification System

### 1. Binary Manifest (TOML)

**Location**: `nucleusBin/MANIFEST.toml`

```toml
[manifest]
version = "1.0"
created_at = "2026-01-08T15:26:05Z"
pipeline_run = "harvest-2026-01-08-1526"

[binaries.tower]
name = "tower"
version = "0.6.0"
git_commit = "22b391c"
build_date = "2026-01-08T15:26:05Z"
sha256 = "d9a15b5665695161..."
size_bytes = 7340032
source_repo = "biomeOS"

[binaries.beardog]
name = "beardog-server"
version = "0.15.0"
git_commit = "c7ad16762"
build_date = "2026-01-08T15:26:05Z"
sha256 = "b10fd19491c04e9adff5b683e6553aca"
size_bytes = 5872640
source_repo = "ecoPrimals/phase1/beardog"
features = ["btsp-api", "unix-socket"]

[binaries.songbird]
name = "songbird"
version = "3.19.0"
git_commit = "c77684cd2"
build_date = "2026-01-08T15:26:05Z"
sha256 = "c8d5cf77af4129c9..."
size_bytes = 29360128
source_repo = "ecoPrimals/phase1/songbird"
features = ["btsp-client", "port-free"]

[compatibility]
min_tower_version = "0.5.0"
min_beardog_version = "0.15.0"
min_songbird_version = "3.19.0"
```

### 2. Spore Manifest (TOML)

**Location**: `<spore_root>/.manifest.toml`

```toml
[spore]
node_id = "node-epsilon"
family_id = "nat0"
created_at = "2026-01-08T15:31:06Z"
created_by = "biomeos-cli v0.6.0"
spore_type = "LiveSpore"
deployment_batch = "2026-01-08"

[lineage]
parent_seed_hash = "183aa0d9d68f57c4..."
child_seed_hash = "6e32319ece57c20a..."
derivation_method = "SHA256(parent || node_id || batch)"

[binaries.tower]
name = "tower"
version = "0.6.0"
sha256 = "d9a15b5665695161..."
source_manifest = "nucleusBin/MANIFEST.toml"
copied_at = "2026-01-08T15:31:09Z"

[binaries.beardog]
name = "beardog-server"
version = "0.15.0"
sha256 = "b10fd19491c04e9adff5b683e6553aca"
source_manifest = "nucleusBin/MANIFEST.toml"
copied_at = "2026-01-08T15:31:12Z"

[binaries.songbird]
name = "songbird"
version = "3.19.0"
sha256 = "c8d5cf77af4129c9..."
source_manifest = "nucleusBin/MANIFEST.toml"
copied_at = "2026-01-08T15:31:15Z"

[deployment_history]
# Future: Track when/where this spore was deployed
```

### 3. Rust Verification Module

**Location**: `crates/biomeos-spore/src/verification.rs`

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryManifest {
    pub manifest: ManifestMeta,
    pub binaries: HashMap<String, BinaryInfo>,
    pub compatibility: CompatibilityInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryInfo {
    pub name: String,
    pub version: String,
    pub git_commit: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SporeManifest {
    pub spore: SporeInfo,
    pub lineage: LineageInfo,
    pub binaries: HashMap<String, SporeBinaryInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    Fresh,      // Binary matches nucleusBin
    Stale,      // Binary is older than nucleusBin
    Modified,   // Binary has different hash (manual edit?)
    Missing,    // Binary not found
    Newer,      // Binary is newer than nucleusBin (???)
}

#[derive(Debug)]
pub struct VerificationReport {
    pub spore_path: PathBuf,
    pub node_id: String,
    pub overall_status: VerificationStatus,
    pub binaries: Vec<BinaryVerification>,
    pub recommendations: Vec<String>,
}

#[derive(Debug)]
pub struct BinaryVerification {
    pub name: String,
    pub status: VerificationStatus,
    pub expected_version: String,
    pub actual_version: Option<String>,
    pub expected_sha256: String,
    pub actual_sha256: Option<String>,
}

pub struct SporeVerifier {
    nucleus_manifest: BinaryManifest,
}

impl SporeVerifier {
    /// Load nucleus manifest from nucleusBin/
    pub fn from_nucleus(nucleus_path: impl AsRef<Path>) -> Result<Self> {
        let manifest_path = nucleus_path.as_ref().join("MANIFEST.toml");
        let manifest_str = std::fs::read_to_string(manifest_path)?;
        let nucleus_manifest: BinaryManifest = toml::from_str(&manifest_str)?;
        
        Ok(Self { nucleus_manifest })
    }
    
    /// Verify a single spore against nucleusBin
    pub fn verify_spore(&self, spore_path: impl AsRef<Path>) -> Result<VerificationReport> {
        let spore_path = spore_path.as_ref();
        
        // Load spore manifest
        let spore_manifest_path = spore_path.join(".manifest.toml");
        let spore_manifest: SporeManifest = if spore_manifest_path.exists() {
            let manifest_str = std::fs::read_to_string(&spore_manifest_path)?;
            toml::from_str(&manifest_str)?
        } else {
            // Legacy spore without manifest - generate from inspection
            self.inspect_legacy_spore(spore_path)?
        };
        
        let mut binary_verifications = Vec::new();
        let mut overall_fresh = true;
        
        // Verify each binary
        for (name, expected_binary) in &self.nucleus_manifest.binaries {
            let binary_path = spore_path.join("primals").join(&expected_binary.name);
            
            let verification = if binary_path.exists() {
                let actual_sha256 = self.compute_sha256(&binary_path)?;
                
                let status = if actual_sha256 == expected_binary.sha256 {
                    VerificationStatus::Fresh
                } else {
                    overall_fresh = false;
                    VerificationStatus::Stale
                };
                
                BinaryVerification {
                    name: name.clone(),
                    status,
                    expected_version: expected_binary.version.clone(),
                    actual_version: Some(spore_manifest.binaries.get(name)
                        .map(|b| b.version.clone())
                        .unwrap_or_else(|| "unknown".to_string())),
                    expected_sha256: expected_binary.sha256.clone(),
                    actual_sha256: Some(actual_sha256),
                }
            } else {
                overall_fresh = false;
                BinaryVerification {
                    name: name.clone(),
                    status: VerificationStatus::Missing,
                    expected_version: expected_binary.version.clone(),
                    actual_version: None,
                    expected_sha256: expected_binary.sha256.clone(),
                    actual_sha256: None,
                }
            };
            
            binary_verifications.push(verification);
        }
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        if !overall_fresh {
            recommendations.push(
                "Run: biomeos spore refresh <mount_point> to update binaries".to_string()
            );
        }
        
        Ok(VerificationReport {
            spore_path: spore_path.to_path_buf(),
            node_id: spore_manifest.spore.node_id.clone(),
            overall_status: if overall_fresh {
                VerificationStatus::Fresh
            } else {
                VerificationStatus::Stale
            },
            binaries: binary_verifications,
            recommendations,
        })
    }
    
    /// Verify all mounted spores
    pub fn verify_all_spores(&self) -> Result<Vec<VerificationReport>> {
        // Auto-detect mounted USB spores
        let mount_points = self.discover_spores()?;
        
        let mut reports = Vec::new();
        for mount_point in mount_points {
            match self.verify_spore(&mount_point) {
                Ok(report) => reports.push(report),
                Err(e) => {
                    warn!("Failed to verify spore at {}: {}", mount_point.display(), e);
                }
            }
        }
        
        Ok(reports)
    }
    
    /// Compute SHA256 of a file
    fn compute_sha256(&self, path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
    
    /// Discover mounted spores (look in /media/*)
    fn discover_spores(&self) -> Result<Vec<PathBuf>> {
        // Implementation: scan /media/* for biomeOS/ directories
        todo!("Implement spore discovery")
    }
    
    /// Inspect legacy spore without manifest
    fn inspect_legacy_spore(&self, spore_path: &Path) -> Result<SporeManifest> {
        // Read tower.toml for node_id, etc.
        todo!("Implement legacy spore inspection")
    }
}
```

### 4. CLI Integration

**Location**: `crates/biomeos-cli/src/commands/verify.rs`

```rust
use clap::Args;
use biomeos_spore::verification::{SporeVerifier, VerificationStatus};

#[derive(Args)]
pub struct VerifyArgs {
    #[command(subcommand)]
    target: VerifyTarget,
}

#[derive(Subcommand)]
pub enum VerifyTarget {
    /// Verify nucleusBin integrity
    Nucleus,
    
    /// Verify a specific spore
    Spore {
        /// Path to spore mount point
        #[arg(value_name = "MOUNT_POINT")]
        mount_point: PathBuf,
    },
    
    /// Verify all mounted spores
    All,
    
    /// Compare spore versions for heterogeneous deployment
    Compare {
        /// List of spore paths to compare
        #[arg(value_name = "MOUNT_POINTS")]
        mount_points: Vec<PathBuf>,
    },
}

pub async fn run(args: VerifyArgs) -> Result<()> {
    match args.target {
        VerifyTarget::Nucleus => {
            verify_nucleus().await?;
        }
        VerifyTarget::Spore { mount_point } => {
            verify_single_spore(&mount_point).await?;
        }
        VerifyTarget::All => {
            verify_all_spores().await?;
        }
        VerifyTarget::Compare { mount_points } => {
            compare_spores(&mount_points).await?;
        }
    }
    
    Ok(())
}

async fn verify_single_spore(mount_point: &Path) -> Result<()> {
    let verifier = SporeVerifier::from_nucleus("nucleusBin")?;
    let report = verifier.verify_spore(mount_point)?;
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔍 Spore Verification Report                          ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Node: {}", report.node_id);
    println!("Path: {}", report.spore_path.display());
    println!();
    
    match report.overall_status {
        VerificationStatus::Fresh => {
            println!("✅ Status: FRESH (all binaries match nucleusBin)");
        }
        VerificationStatus::Stale => {
            println!("⚠️  Status: STALE (some binaries need update)");
        }
        _ => {
            println!("❌ Status: {:?}", report.overall_status);
        }
    }
    
    println!();
    println!("Binary Status:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    for binary in &report.binaries {
        let status_icon = match binary.status {
            VerificationStatus::Fresh => "✅",
            VerificationStatus::Stale => "⚠️ ",
            VerificationStatus::Missing => "❌",
            _ => "❓",
        };
        
        println!("{} {}: {:?}", status_icon, binary.name, binary.status);
        println!("   Expected: v{} (SHA256: {}...)", 
            binary.expected_version, 
            &binary.expected_sha256[..16]
        );
        if let Some(actual_version) = &binary.actual_version {
            println!("   Actual:   v{} (SHA256: {}...)", 
                actual_version,
                binary.actual_sha256.as_ref()
                    .map(|s| &s[..16])
                    .unwrap_or("missing")
            );
        } else {
            println!("   Actual:   MISSING");
        }
    }
    
    if !report.recommendations.is_empty() {
        println!();
        println!("💡 Recommendations:");
        for rec in &report.recommendations {
            println!("   {}", rec);
        }
    }
    
    Ok(())
}

async fn verify_all_spores() -> Result<()> {
    let verifier = SporeVerifier::from_nucleus("nucleusBin")?;
    let reports = verifier.verify_all_spores()?;
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔍 All Spores Verification Report                     ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    let mut fresh_count = 0;
    let mut stale_count = 0;
    
    for report in &reports {
        let status_icon = match report.overall_status {
            VerificationStatus::Fresh => {
                fresh_count += 1;
                "✅"
            }
            VerificationStatus::Stale => {
                stale_count += 1;
                "⚠️ "
            }
            _ => "❌",
        };
        
        println!("{} {} ({}): {:?}", 
            status_icon, 
            report.node_id, 
            report.spore_path.display(),
            report.overall_status
        );
    }
    
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Summary: {} fresh, {} stale, {} total", 
        fresh_count, stale_count, reports.len()
    );
    
    if stale_count > 0 {
        println!();
        println!("⚠️  Run 'biomeos spore refresh-all' to update stale spores");
    }
    
    Ok(())
}
```

---

## 🎯 Heterogeneous Deployment Support

### Use Case: Gradual Rollout

**Scenario**: New BearDog version with experimental feature

```bash
# Step 1: Update nucleusBin with new version
./scripts/harvest-primals.sh

# Step 2: Verify current state
biomeos verify all
# Output:
# ✅ node-alpha: v0.15.0 (Fresh)
# ✅ node-beta: v0.15.0 (Fresh)
# ⚠️  node-gamma: v0.14.0 (Stale)
# ⚠️  node-delta: v0.14.0 (Stale)
# ⚠️  node-epsilon: v0.14.0 (Stale)

# Step 3: Gradual rollout (update 1 spore first)
biomeos spore refresh /media/eastgate/BEA6-BBCE  # gamma only

# Step 4: Deploy and test gamma in isolation
# ... test experimental feature ...

# Step 5: If successful, update remaining spores
biomeos spore refresh /media/eastgate/BEA6-BBCE1  # delta
biomeos spore refresh /media/eastgate/BEA6-BBCE2  # epsilon

# Step 6: Verify heterogeneous deployment
biomeos verify compare /media/eastgate/BEA6-BBCE \
                       /media/eastgate/BEA6-BBCE1 \
                       /media/eastgate/BEA6-BBCE2
# Output:
# node-gamma:   BearDog v0.16.0 (experimental)
# node-delta:   BearDog v0.15.0 (stable)
# node-epsilon: BearDog v0.14.0 (old)
# 
# ⚠️  Version skew detected!
# Recommendation: Test inter-version compatibility
```

### Compatibility Matrix

**Location**: `nucleusBin/COMPATIBILITY.toml`

```toml
[versions]
# Define which versions can coexist in a federation

[[compatible_sets]]
# Set 1: v0.15.x series
tower = ">=0.6.0, <0.7.0"
beardog = ">=0.15.0, <0.16.0"
songbird = ">=3.19.0, <4.0.0"
note = "Stable production release"

[[compatible_sets]]
# Set 2: v0.16.x experimental
tower = ">=0.6.0, <0.7.0"
beardog = ">=0.16.0, <0.17.0"  # New experimental features
songbird = ">=3.20.0, <4.0.0"   # Must support new BearDog API
note = "Experimental - test in isolation first"

[compatibility_rules]
# Can mix v0.15 and v0.16 BearDog in same federation?
allow_version_skew = true
max_version_difference = "minor"  # major.MINOR.patch

# Warn if versions are too far apart
warn_if_skew_exceeds = "1 minor version"
```

---

## 🚀 Evolution Roadmap

### Phase 1: Foundation (Immediate)
- ✅ Create `MANIFEST.toml` generation in harvest script
- ✅ Add `.manifest.toml` creation to spore builder
- ✅ Implement `SporeVerifier` module in Rust
- ✅ Add `biomeos verify` CLI commands

### Phase 2: Automation (Short-term)
- ✅ Auto-verify before deployment
- ✅ Pre-flight checks in `deploy.sh`
- ✅ Warn on version skew
- ✅ `biomeos spore refresh` command

### Phase 3: Heterogeneous Support (Medium-term)
- ✅ Version compatibility matrix
- ✅ Controlled rollout support
- ✅ Version comparison tools
- ✅ Federation health checks

### Phase 4: Production Hardening (Long-term)
- ✅ Automatic staleness detection
- ✅ Rollback capabilities
- ✅ Binary signature verification
- ✅ Audit logging

---

## 📊 Benefits

### Type Safety
- ❌ **Before**: Bash string parsing, error-prone
- ✅ **After**: Rust types, compile-time checks

### Verification
- ❌ **Before**: No pre-deployment checks
- ✅ **After**: Automated verification, clear reports

### Heterogeneous Deployments
- ❌ **Before**: Assumed all spores identical
- ✅ **After**: Support version skew, gradual rollouts

### Composability
- ❌ **Before**: Monolithic bash scripts
- ✅ **After**: Composable Rust modules, CLI tools

### Production Readiness
- ❌ **Before**: Manual processes, easy to miss updates
- ✅ **After**: Automated, verified, auditable

---

## 🎯 Immediate Action Plan

### Step 1: Create Manifest Generator
```rust
// crates/biomeos-spore/src/manifest.rs
pub struct ManifestGenerator;
impl ManifestGenerator {
    pub fn generate_nucleus_manifest(nucleus_path: &Path) -> Result<BinaryManifest>;
    pub fn generate_spore_manifest(spore_path: &Path) -> Result<SporeManifest>;
}
```

### Step 2: Integrate into Spore Creation
```rust
// In spore.rs create_directory_structure():
let manifest_generator = ManifestGenerator::new();
let spore_manifest = manifest_generator.generate_spore_manifest(&self.root_path)?;
spore_manifest.write_to_file(&self.root_path.join(".manifest.toml"))?;
```

### Step 3: Add Verification CLI
```rust
// crates/biomeos-cli/src/bin/main.rs
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Verify spore binaries and manifests
    Verify(verify::VerifyArgs),
    
    // ... existing commands ...
}
```

### Step 4: Update Harvest Script (Temporary)
```bash
# Add manifest generation at end of harvest-primals.sh
echo "📝 Generating binary manifest..."
cargo run --release -p biomeos-cli --bin biomeos -- \
    manifest generate \
    --nucleus nucleusBin/ \
    --output nucleusBin/MANIFEST.toml
```

### Step 5: Evolve to Pure Rust (Future)
```rust
// Replace harvest-primals.sh with:
// cargo run -p biomeos-cli --bin biomeos -- harvest --all
```

---

## 🎊 Summary

**Problem**: Stale binaries on 3 spores due to lack of verification  
**Root Cause**: Manual processes, no automated checks  
**Evolution**: Bash scripts → Modern idiomatic Rust  
**Benefits**: Type safety, automation, heterogeneous deployments  

**Status**: 🚀 **READY TO EVOLVE**  
**First PR**: Implement `SporeVerifier` + `biomeos verify` command  
**Timeline**: Phase 1 completable in 1-2 hours

---

**Let's evolve the jelly strings to fast, safe, modern Rust!** 🦀🚀

