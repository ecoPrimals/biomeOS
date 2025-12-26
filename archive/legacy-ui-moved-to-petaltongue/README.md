# Legacy UI Code - Evolved into petalTongue

**Date:** December 23, 2025  
**Context:** BiomeOS was in stasis while Gen 1 primals matured  
**Evolution:** UI code evolved into standalone primal **petalTongue**

---

## What Happened

While biomeOS was paused to let Gen 1 primals (beardog, songbird, toadstool, nestgate, squirrel) mature, the UI requirements became clear enough to warrant their own primal.

**petalTongue** emerged as the universal UI primal:
- **Location:** `../../petalTongue/`
- **Status:** Production-ready with 26+ tests
- **Features:** 
  - Visual modality (2D graph visualization)
  - Audio modality (sonification for blind users)
  - Accessibility-first design
  - Real-time BiomeOS integration
- **Philosophy:** "Any topology, any modality, any human"

---

## What's Archived Here

This directory contains the legacy UI code from biomeOS before the split:

- **`ui/`** - Desktop UI application (~989 LOC in minimal_app.rs)
  - Had compilation errors (API mismatches with core)
  - Was becoming too complex for a single module
  - Evolved into petalTongue's richer architecture

- **`biomeos-desktop/`** - Desktop packaging (if present)
  - Superseded by petalTongue's binary

---

## Why Archive (Not Delete)

**Fossil Record:** Understanding how the UI evolved from biomeOS monolith to dedicated primal.

**Historical Context:** Shows the transition from:
- Single monolithic system → Specialized primals
- One modality (visual only) → Multiple modalities (visual + audio + future)
- Accommodation approach → Celebration of diversity

---

## Migration Path

If you need UI functionality:

### Old Way (Legacy, Don't Use)
```bash
cd biomeOS/
cargo run --bin biomeos-ui  # DOESN'T WORK (archived)
```

### New Way (Current)
```bash
# petalTongue as standalone
cd ../petalTongue/
cargo run --release -p petal-tongue-ui

# petalTongue with live BiomeOS integration
BIOMEOS_URL=http://localhost:3000 cargo run --release -p petal-tongue-ui
```

### BiomeOS CLI (For Orchestration)
```bash
cd biomeOS/
cargo run -p biomeos-cli -- health
cargo run -p biomeos-cli -- discover --capability compute
cargo run -p biomeos-cli -- chimera list
```

---

## See Also

- `../../petalTongue/` - Current UI primal
- `../../petalTongue/README.md` - petalTongue overview
- `../BIOMEOS_RESCOPE_PLAN.md` - Cleanup rationale
- `../COMPREHENSIVE_AUDIT_REPORT_DEC_23_2025.md` - Full audit

---

*Archived December 23, 2025 during biomeOS modernization after stasis period.*

