# 🧬 plasmidBin Manifest

**Version**: v0.8.1  
**Date**: January 9, 2026  
**Purpose**: Stable deployment binaries for spore creation

---

## 📦 **What is plasmidBin?**

`plasmidBin/` is the **single source of truth** for stable, production-ready primal binaries used in spore deployment. The name "plasmid" reflects its role as a carrier of genetic material (binaries) that can be transferred between systems (spores).

**Semantic Compression**: `plasmidBin` > `nucleusBins` or `primalBins`
- More precise biological metaphor (plasmids carry genetic information)
- Cleaner, more concise naming
- Better semantic alignment with spore/genetic lineage concepts

---

## 🏗️ **Structure**

```
plasmidBin/
├── primals/              # All primal binaries (capability-based)
│   ├── beardog-server    # Security, encryption, identity
│   ├── songbird          # P2P, discovery, BTSP
│   ├── toadstool         # Compute orchestration
│   ├── nestgate          # Storage, provenance, compression
│   ├── petal-tongue      # Universal UI (GUI)
│   └── petal-tongue-headless  # Universal UI (CLI)
├── tower/                # Tower orchestrator (future)
│   └── tower             # biomeOS orchestrator binary
├── archive/              # Old versions (for rollback)
│   └── ...
├── VERSION.txt           # Current version
└── MANIFEST.md           # This file
```

---

## 🔄 **Workflow**

### **1. Harvest Binaries** (from Phase 1 primals)

```bash
# Run harvest script to copy latest binaries
./scripts/harvest-primals.sh

# Or manually copy
cp /path/to/ecoPrimals/phase1/beardog/target/release/beardog-server plasmidBin/primals/
cp /path/to/ecoPrimals/phase1/songbird/target/release/songbird plasmidBin/primals/
cp /path/to/ecoPrimals/phase1/toadstool/target/release/toadstool plasmidBin/primals/
cp /path/to/ecoPrimals/phase1/nestgate/target/release/nestgate plasmidBin/primals/
cp /path/to/ecoPrimals/phase2/petalTongue/target/release/petal-tongue plasmidBin/primals/
cp /path/to/ecoPrimals/phase2/petalTongue/target/release/petal-tongue-headless plasmidBin/primals/
```

### **2. Create Spores** (from plasmidBin)

```bash
# Create spore (automatically uses plasmidBin/)
cargo run --bin biomeos -- spore create /media/user/USB/biomeOS

# Spore creation copies ALL binaries from plasmidBin/primals/
# This is capability-based and agnostic (no hardcoded primal names)
```

### **3. Deploy Spores** (on target systems)

```bash
# Spore is self-contained and ready to deploy
# All binaries are in the spore's primals/ directory
```

---

## 🎯 **Design Principles**

### **1. Capability-Based** (Not Name-Based)
- No hardcoded primal names in spore creation
- Copy ALL binaries from `plasmidBin/primals/`
- Niche manifests (`niches/*.toml`) define which primals are used
- Enables BYOB (Bring Your Own Biome) flexibility

### **2. Single Source of Truth**
- `plasmidBin/` is the ONLY source for spore binaries
- No copying from `target/release/` or other locations
- Ensures consistency across all spores

### **3. Agnostic Evolution**
- New primals: Just add to `plasmidBin/primals/`
- Renamed primals: Update binary name, no code changes
- Chimeras: Embed primals, no deployment changes

### **4. Versioning**
- `VERSION.txt` tracks current version
- `archive/` stores old versions for rollback
- Each spore records which version it was created from

---

## 📊 **Current Binaries**

| Binary | Primal | Version | Status | Size |
|--------|--------|---------|--------|------|
| `petal-tongue` | petalTongue | v0.4.0 | ✅ Production | 21MB |
| `petal-tongue-headless` | petalTongue | v0.4.0 | ✅ Production | 3.1MB |
| `beardog-server` | BearDog | v0.15.2 | ⏳ Pending harvest | - |
| `songbird` | Songbird | v3.19.0 | ⏳ Pending harvest | - |
| `toadstool` | Toadstool | - | ⏳ Pending harvest | - |
| `nestgate` | NestGate | - | ⏳ Pending harvest | - |

---

## 🚀 **Next Steps**

1. **Harvest all Phase 1 binaries**
   - Run `./scripts/harvest-primals.sh`
   - Or manually copy from Phase 1 projects

2. **Test spore creation**
   - Create test spore: `cargo run --bin biomeos -- spore create /tmp/test-spore`
   - Verify all binaries copied

3. **Deploy to USB**
   - Create spore on USB: `cargo run --bin biomeos -- spore create /media/user/USB/biomeOS`
   - Test on target system

---

## 📚 **References**

- **Spore System**: `crates/biomeos-spore/`
- **Harvest Script**: `scripts/harvest-primals.sh`
- **Niche Manifests**: `niches/*.toml`
- **BYOB Spec**: `specs/BYOB_BUILD_YOUR_OWN_BIOME_SPECIFICATION.md`

---

**Last Updated**: January 9, 2026  
**Maintainer**: biomeOS Team

🧬 **plasmidBin: Genetic Material for Spore Deployment** 🌱

