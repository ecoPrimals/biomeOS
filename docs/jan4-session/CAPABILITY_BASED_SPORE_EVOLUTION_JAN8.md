# 🧬 Capability-Based Spore Evolution - Eliminate Hardcoding

**Date**: January 8, 2026  
**Status**: ✅ **COMPLETE** - Evolved from hardcoded to agnostic  
**Purpose**: Remove hardcoded primal names, embrace BYOB manifest system

---

## 🎯 The Problem (Hardcoding Deep Debt)

### Before (Hardcoded Primals):
```rust
// ❌ BAD: Hardcoded primal names
async fn copy_binaries(&self) -> SporeResult<()> {
    // 1. Copy beardog-server
    let beardog_src = nucleus_dir.join("beardog-server");
    async_fs::copy(&beardog_src, &beardog_dst).await?;
    
    // 2. Copy songbird
    let songbird_src = nucleus_dir.join("songbird");
    async_fs::copy(&songbird_src, &songbird_dst).await?;
    
    // What if:
    // - BearDog evolves new capabilities?
    // - Name changes (beardog-server → beardog-v2)?
    // - Chimera (songbird embeds beardog)?
    // - New primal (toadstool)?
    // ❌ Code changes required!
}
```

**Issues**:
- ❌ Spore has dev knowledge (knows about specific primals)
- ❌ Violates "primal code only has self knowledge"
- ❌ Can't handle chimeras (embedded primals)
- ❌ Can't handle name changes
- ❌ Can't handle new primals without code changes
- ❌ Doesn't leverage BYOB manifest system

---

## ✅ The Solution: Capability-Based Copying

### After (Agnostic, Discovered):
```rust
// ✅ GOOD: Capability-based, agnostic
async fn copy_binaries(&self) -> SporeResult<()> {
    // Copy tower (always required)
    async_fs::copy(nucleus_dir.join("tower/tower"), dst).await?;
    
    // Copy ALL primals from nucleusBin/primals/ (agnostic!)
    let mut entries = async_fs::read_dir(nucleus_dir.join("primals")).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.is_file() && !path.file_name().starts_with('.') {
            let file_name = path.file_name().unwrap();
            async_fs::copy(&path, primals_dst.join(file_name)).await?;
            info!("✅ Copied primal: {}", file_name);
        }
    }
    
    // ✅ Works with ANY primals in nucleusBin/
    // ✅ No code changes needed for new primals
    // ✅ Supports chimeras, renames, evolution
}
```

---

## 🔧 Key Principles

### 1. Self-Knowledge Only
**Spore knows**:
- ✅ Where to find binaries (`nucleusBin/`)
- ✅ How to copy files
- ✅ How to set permissions

**Spore does NOT know**:
- ❌ What specific primals exist
- ❌ What capabilities they provide
- ❌ How they're named

### 2. Capability-Based Discovery
```
Spore Creation (Agnostic):
  nucleusBin/primals/
    ├─ beardog-server    ← Copied
    ├─ songbird          ← Copied
    └─ toadstool         ← Copied (NEW!)
  
  ↓ Copy ALL (no hardcoding)
  
USB Spore/biomeOS/primals/
  ├─ beardog-server
  ├─ songbird
  └─ toadstool

Runtime (tower.toml decides):
  [[primals]]
  binary = "./primals/beardog-server"
  provides = ["Security"]
  
  [[primals]]
  binary = "./primals/songbird"
  provides = ["Discovery"]
  requires = ["Security"]
  
  # toadstool not used (but available!)
```

### 3. BYOB (Bring Your Own Biome) Integration
```toml
# tower.toml is the SOURCE OF TRUTH
# Spore just copies everything from nucleusBin/
# Tower decides what to run

[[primals]]
binary = "./primals/beardog-server"
provides = ["Security", "Encryption", "Trust"]
requires = []

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

# Future: Chimera example
[[primals]]
binary = "./primals/songbird-beardog-chimera"
provides = ["Discovery", "Security"]  # Both!
requires = []  # Self-contained
```

---

## 🎊 Benefits

### Evolution-Friendly
```bash
# Scenario 1: New primal (toadstool)
./scripts/harvest-primals.sh  # Adds toadstool to nucleusBin/
biomeos spore create ...       # Copies toadstool automatically
# Edit tower.toml to use it    # ✅ No code changes!

# Scenario 2: Rename (beardog-server → beardog-v2)
mv nucleusBin/primals/beardog-server nucleusBin/primals/beardog-v2
biomeos spore create ...       # Copies beardog-v2 automatically
# Update tower.toml binary path # ✅ No code changes!

# Scenario 3: Chimera (songbird embeds beardog)
# Build songbird-beardog-chimera
cp target/release/songbird-beardog-chimera nucleusBin/primals/
biomeos spore create ...       # Copies chimera automatically
# tower.toml uses chimera       # ✅ No code changes!
```

### Composability
- ✅ Primals are sovereign (self-contained)
- ✅ Spore is agnostic (no dev knowledge)
- ✅ Tower.toml is manifest (source of truth)
- ✅ Runtime discovery (capability-based)

### Deep Debt Elimination
- ❌ Hardcoding → ✅ Agnostic discovery
- ❌ Dev knowledge → ✅ Self-knowledge only
- ❌ Brittle → ✅ Evolution-friendly
- ❌ Coupled → ✅ Composable

---

## 📋 Implementation

### Changes to `biomeos-spore/src/spore.rs`

```rust
/// Copy ALL primals from nucleusBin/ (agnostic)
async fn copy_binaries(&self) -> SporeResult<()> {
    // 1. Copy tower (always required)
    async_fs::copy(
        nucleus_dir.join("tower/tower"),
        self.root_path.join("bin/tower")
    ).await?;
    
    // 2. Copy ALL primals (capability-based discovery)
    let primals_src_dir = nucleus_dir.join("primals");
    let mut entries = async_fs::read_dir(&primals_src_dir).await?;
    
    let mut primal_count = 0;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        // Skip dotfiles (.gitkeep, etc.)
        if path.file_name().starts_with('.') {
            continue;
        }
        
        // Only copy files (not directories)
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            async_fs::copy(&path, primals_dst.join(file_name)).await?;
            primal_count += 1;
            info!("✅ Copied primal: {}", file_name);
        }
    }
    
    info!("✅ Copied tower + {} primals (capability-based)", primal_count);
    Ok(())
}
```

### Benefits of This Approach

1. **Zero Hardcoding** - No primal names in code
2. **Self-Knowledge Only** - Spore knows structure, not contents
3. **BYOB Compatible** - Works with manifest system
4. **Evolution-Friendly** - New primals work automatically
5. **Chimera-Ready** - Embedded primals supported
6. **Rename-Safe** - Name changes don't break deployment

---

## 🔮 Future: Enhanced BYOB Integration

### Phase 1: Agnostic Copying (✅ COMPLETE)
- Copy ALL binaries from nucleusBin/
- No hardcoded primal names
- tower.toml decides what runs

### Phase 2: Manifest Validation (NEXT)
```rust
// Validate that tower.toml references exist in primals/
async fn validate_manifest(&self) -> SporeResult<()> {
    let tower_toml = self.read_tower_config()?;
    
    for primal in tower_toml.primals {
        let binary_path = self.root_path.join(&primal.binary);
        if !binary_path.exists() {
            return Err(SporeError::MissingPrimal(primal.binary));
        }
    }
    
    Ok(())
}
```

### Phase 3: Capability Checking
```rust
// Verify that required capabilities are available
async fn check_capabilities(&self) -> SporeResult<()> {
    let tower_toml = self.read_tower_config()?;
    
    // Build capability map
    let mut provided = HashSet::new();
    for primal in &tower_toml.primals {
        provided.extend(primal.provides.clone());
    }
    
    // Check all requirements are met
    for primal in &tower_toml.primals {
        for required in &primal.requires {
            if !provided.contains(required) {
                return Err(SporeError::MissingCapability(required.clone()));
            }
        }
    }
    
    Ok(())
}
```

### Phase 4: Dynamic Harvesting
```bash
# harvest-primals.sh could use tower.toml to know what to fetch
./scripts/harvest-primals.sh --from tower.toml

# Only fetches primals mentioned in tower.toml
# Reduces spore size, optimizes for specific deployments
```

---

## 🎓 Key Insights

### Biological Metaphor
- **Nucleus** = Contains all genetic material (nucleusBin/)
- **Spore** = Carries complete genome (ALL primals)
- **Expression** = Runtime decides which genes activate (tower.toml)
- **Evolution** = New genes can be added without changing spore mechanism

### Architectural Principles
1. **Separation of Concerns**:
   - Spore: Transport mechanism (agnostic)
   - NucleusBin: Genetic library (complete)
   - Tower.toml: Expression manifest (specific)

2. **Primal Sovereignty**:
   - Each primal is self-contained
   - No primal has knowledge of others
   - Discovery happens at runtime

3. **Capability-Based**:
   - Primals provide capabilities
   - Requirements expressed as capabilities
   - Not tied to specific implementations

---

## 📊 Comparison

### Before (Hardcoded):
```
Spore Creation:
  ❌ Copy beardog-server
  ❌ Copy songbird
  ❌ Copy (hardcoded list)

Add New Primal:
  1. Update harvest script
  2. Update spore.rs (hardcode name)
  3. Rebuild biomeOS
  4. Create new spores
```

### After (Capability-Based):
```
Spore Creation:
  ✅ Copy tower/
  ✅ Copy primals/* (ALL)
  ✅ Agnostic, discovered

Add New Primal:
  1. Add to nucleusBin/primals/
  2. Update tower.toml (if needed)
  ✅ Spore creation unchanged!
```

---

## 🎯 Status

**Evolution**: ✅ COMPLETE  
**Deep Debt**: ELIMINATED  
**Hardcoding**: REMOVED  
**Capability-Based**: IMPLEMENTED  
**BYOB Integration**: FOUNDATION LAID

**Next**: Manifest validation + capability checking

🧬 **Spores are now truly agnostic and evolution-friendly!** 🌱

