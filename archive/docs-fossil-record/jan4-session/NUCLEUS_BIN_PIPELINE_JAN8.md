# 🧬 PlasmidBin Pipeline - Stable Binary Management

**Date**: January 8, 2026  
**Status**: 🎯 **EVOLUTION** - From Manual Debt to Automated Pipeline  
**Purpose**: Eliminate manual binary copying deep debt

---

## 🎯 The Problem (Deep Debt)

### Manual Binary Management
```bash
# ❌ OLD WAY (Deep Debt):
# 1. Build beardog in phase1/beardog/
# 2. Manually copy to biomeOS/
# 3. Manually copy to each USB spore
# 4. Binaries get stale
# 5. MD5 mismatches
# 6. No version tracking
```

**Issues**:
- ❌ Manual copying error-prone
- ❌ Stale binaries on spores
- ❌ No single source of truth
- ❌ Hard to verify versions
- ❌ Deep debt accumulates

---

## ✅ The Solution: PlasmidBin Pipeline

### Concept: Single Source of Stable Binaries

```
biomeOS/
  plasmidBin/           ← NEW: Stable deployment binaries
    tower/
      tower             ← biomeOS orchestrator
    primals/
      beardog-server    ← BearDog v0.15.1+
      songbird          ← Songbird v3.19.0
      toadstool         ← (future)
    archive/
      YYYY-MM-DD/       ← Timestamped archives
        tower
        beardog-server
        songbird
```

### Philosophy

**PlasmidBin = Genetic Nucleus**
- Single source of truth for deployment binaries
- Stable, tested, ready-to-deploy
- Version tracked via git + timestamps
- Spores clone from nucleus (genetic siblings!)
- Archive old versions for rollback

---

## 🔧 Pipeline Stages

### Stage 1: Harvest (Pull from Primal Repos)

```bash
# scripts/harvest-primals.sh
#!/usr/bin/env bash
# Harvest stable binaries from primal repositories

# Build/pull latest stable versions
cd ../../../phase1/beardog && git pull && cargo build --release
cd ../songbird && git pull && cargo build --release

# Harvest to plasmidBin
cp phase1/beardog/target/release/beardog-server plasmidBin/primals/
cp phase1/songbird/target/release/songbird plasmidBin/primals/
cp target/release/tower plasmidBin/tower/

# Verify and stamp
./scripts/verify-nucleus.sh
```

### Stage 2: Verify (Check Integrity)

```bash
# scripts/verify-nucleus.sh
#!/usr/bin/env bash
# Verify all binaries in plasmidBin are valid

for bin in plasmidBin/tower/* plasmidBin/primals/*; do
    if [ -f "$bin" ] && [ -x "$bin" ]; then
        echo "✅ $bin"
        md5sum "$bin"
    else
        echo "❌ $bin - Invalid!"
        exit 1
    fi
done
```

### Stage 3: Archive (Timestamped Backup)

```bash
# scripts/archive-nucleus.sh
#!/usr/bin/env bash
# Archive current nucleus to timestamped folder

DATE=$(date +%Y-%m-%d_%H-%M-%S)
mkdir -p plasmidBin/archive/$DATE

cp plasmidBin/tower/* plasmidBin/archive/$DATE/
cp plasmidBin/primals/* plasmidBin/archive/$DATE/

echo "📦 Archived to: plasmidBin/archive/$DATE"
```

### Stage 4: Deploy (Spore Creation)

```bash
# Updated: crates/biomeos-spore/src/spore.rs
impl Spore {
    async fn copy_binaries(&self) -> SporeResult<()> {
        // ✅ NEW: Copy from plasmidBin
        let nucleus_path = PathBuf::from("plasmidBin");
        
        // Copy tower
        fs::copy(
            nucleus_path.join("tower/tower"),
            self.root_path.join("bin/tower")
        )?;
        
        // Copy primals
        for primal in ["beardog-server", "songbird"] {
            fs::copy(
                nucleus_path.join(format!("primals/{}", primal)),
                self.root_path.join(format!("primals/{}", primal))
            )?;
        }
        
        Ok(())
    }
}
```

---

## 📋 .gitignore Strategy

### Binary Management in Git

```gitignore
# PlasmidBin - Track directory structure, ignore binaries
plasmidBin/tower/*
plasmidBin/primals/*
plasmidBin/archive/**/*

# EXCEPT: Keep .gitkeep files
!plasmidBin/tower/.gitkeep
!plasmidBin/primals/.gitkeep
!plasmidBin/archive/.gitkeep

# Optional: Track version manifest
!plasmidBin/VERSION.txt
!plasmidBin/MANIFEST.md
```

### Why NOT Commit Binaries?

1. **Size**: Binaries are large (5-26 MB each)
2. **Git Bloat**: Git history grows with each binary change
3. **Build Artifacts**: Generated from source, not source itself
4. **CI/CD**: Should be built in pipeline

### Alternative: Track Versions

```
# plasmidBin/VERSION.txt
beardog-server: v0.15.1 (git: dc9b4426c)
songbird: v3.19.0 (git: a1b2c3d)
tower: v0.4.0 (git: e13b20b)
```

---

## 🎯 Evolution Path: LiveSpore & ColdSpore

### Phase 1: PlasmidBin (Current)
- ✅ Single source of stable binaries
- ✅ Manual harvest + verify
- ✅ Spores copy from nucleus

### Phase 2: Automated Pipeline
```bash
# scripts/update-nucleus.sh
# - Auto-detect new versions
# - Pull from primal repos
# - Build if needed
# - Verify integrity
# - Update VERSION.txt
# - Archive old versions
```

### Phase 3: LiveSpore Auto-Update
```rust
// LiveSpore checks for nucleus updates
impl LiveSpore {
    async fn check_for_updates(&self) -> Result<UpdateStatus> {
        let local_version = self.read_version()?;
        let nucleus_version = read_nucleus_version()?;
        
        if nucleus_version > local_version {
            Ok(UpdateStatus::Available(nucleus_version))
        } else {
            Ok(UpdateStatus::Current)
        }
    }
    
    async fn apply_update(&mut self) -> Result<()> {
        // 1. Archive current binaries
        // 2. Copy from nucleus
        // 3. Update VERSION.txt
        // 4. Restart tower (graceful)
    }
}
```

### Phase 4: ColdSpore Hibernation
```rust
// ColdSpore = archived snapshot
impl ColdSpore {
    fn freeze(&self) -> Result<()> {
        // 1. Stop all primals
        // 2. Archive to plasmidBin/archive/
        // 3. Create manifest
        // 4. Mark as hibernated
    }
    
    fn thaw(&self) -> Result<LiveSpore> {
        // 1. Restore from archive
        // 2. Convert to LiveSpore
        // 3. Check for updates
        // 4. Deploy
    }
}
```

---

## 🚀 Implementation Plan

### Today (Immediate)

1. ✅ Create `plasmidBin/` structure
2. ✅ Update `.gitignore`
3. ✅ Create `scripts/harvest-primals.sh`
4. ✅ Create `scripts/verify-nucleus.sh`
5. ✅ Update `biomeos-spore` to use `plasmidBin/`

### This Week

6. Create `scripts/update-nucleus.sh` (automated)
7. Add version tracking (`VERSION.txt`)
8. Test spore creation from nucleus
9. Deploy 5 USB spores from nucleus

### Next Sprint

10. Implement LiveSpore update checks
11. Implement ColdSpore freeze/thaw
12. Add automated testing pipeline
13. Document deployment workflow

---

## 📊 Benefits

### Before (Manual Copying)
- ❌ Error-prone
- ❌ Stale binaries
- ❌ No version tracking
- ❌ Time-consuming
- ❌ Deep debt

### After (PlasmidBin Pipeline)
- ✅ Single source of truth
- ✅ Always fresh binaries
- ✅ Version tracked
- ✅ Automated
- ✅ Debt eliminated
- ✅ Foundation for LiveSpore/ColdSpore

---

## 🎓 Key Insights

### Genetic Metaphor
- **Nucleus** = Stable genetic material (binaries)
- **Spores** = Genetic siblings cloned from nucleus
- **Archive** = Fossil record for rollback
- **LiveSpore** = Active, self-updating organism
- **ColdSpore** = Hibernated snapshot

### Pipeline Philosophy
1. **Harvest** = Pull from primal repos
2. **Verify** = Ensure integrity
3. **Archive** = Preserve history
4. **Deploy** = Clone to spores

---

**Status**: 🎯 In Progress - Eliminating Manual Debt  
**Next**: Implement harvest + verify scripts  
**Goal**: Automated LiveSpore/ColdSpore system

🧬 **PlasmidBin - The Genetic Nucleus of biomeOS!** 🌱

