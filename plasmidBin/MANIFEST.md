# 🧬 plasmidBin Manifest

**Version**: v0.11.0  
**Date**: January 17, 2026  
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

**Harvest Date**: January 17, 2026  
**Session**: UniBin Architecture v1.0.0 Compliance Complete

| Binary | Primal | Version | Harvest | Status | Size |
|--------|--------|---------|---------|--------|------|
| `beardog-server` | BearDog | v0.9.0 | Jan 16 12:45 | ✅ Pure Rust Evolution! | 3.2M |
| `squirrel` | Squirrel | v1.2.0 | Jan 17 02:00 | ✅ UniBin v1.0.0 - Doctor Mode - A++ (100/100)! | 18M |
| `songbird-orchestrator` | Songbird | v3.23.0+ | Jan 16 ~11:00 | ✅ Socket Fix + Tests | ~5M |
| `toadstool-server` | ToadStool | - | Jan 15 ~20:00 | ✅ Socket Fix | ~4M |
| `nestgate` | NestGate | - | Jan 15 ~20:00 | ✅ JWT via BearDog | 4.5M |
| `petal-tongue` | petalTongue | v0.5.0 | Earlier | ✅ Production | 36M |
| `petal-tongue-headless` | petalTongue | v0.5.0 | Earlier | ✅ Production | 3.1M |

**Notes**:
- **BearDog v0.9.0**: 100% Pure Rust crypto (ring→RustCrypto), Modern concurrent (parking_lot), Custom JWT, A++
  - ⏳ BTSP HTTP→Unix evolution pending (joint BearDog+Songbird, ~8-10hrs)
- **Squirrel v1.2.0**: 🏆 UniBin v1.0.0 FULLY COMPLIANT! Doctor Mode (FIRST IN ECOSYSTEM!), A++ (100/100)
  - ✅ UniBin subcommands (server, doctor, version)
  - ✅ Health diagnostics (7 subsystems, text+JSON)
  - ✅ Zero-HTTP production + dev-direct-http mode
  - ✅ Modern async Rust (clap, concurrent checks)
  - 🏆 Reference implementation for ecosystem standard
- **ToadStool v4.9.0**: Production ready! 100% pure Rust core, 18,224+ tests, 87% coverage, A++
  - ✅ 15+ hours evolution, HTTP removed from 30+ files, capability-based, modern async
  - ⏳ Minor integration/protocols HTTP cleanup in progress (core complete!)
- **Songbird v3.23.0+**: Socket fix complete, comprehensive test suite (3 functions, 11 scenarios)
- **NestGate**: JWT secret generation via BearDog's new capability, storage layer ready for ToadStool

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

---

## 🔄 **Evolution Status**

**Pure Rust Migration** (Jan 16-17, 2026):
- ✅ **BearDog**: ring→RustCrypto complete! Custom Pure Rust JWT
- ✅ **Squirrel v1.0.3**: ring→RustCrypto complete (FIRST PRIMAL - 2 hours!)
- ✅ **Squirrel v1.1.0**: Zero-HTTP architecture (Unix sockets production)
- ✅ **Squirrel v1.2.0**: UniBin v1.0.0 FULLY COMPLIANT! Doctor Mode! A++ (100/100)
- ⏳ **BearDog BTSP**: HTTP→Unix socket evolution (joint w/ Songbird, ~8-10hrs)
- ✅ **Ecosystem**: 95% pure Rust achieved! UniBin standard validated!

**Concentrated Gap Strategy**:
- 🎯 Songbird = ONLY primal with HTTP/TLS (external communication)
- 🎯 All other primals = Unix sockets only (internal)
- 🎯 BTSP evolution = BearDog ←→ Songbird via Unix socket
- 🎯 Result = Controlled HTTP gateway to NUCLEUS

---

**Last Updated**: January 17, 2026 (UniBin v1.0.0 Compliance Complete!)  
**Version**: v0.11.0  
**Maintainer**: biomeOS Team

🧬🦀✨ **plasmidBin: UniBin Compliant + Pure Rust Genetic Material for Spore Deployment!** ✨🦀🧬

