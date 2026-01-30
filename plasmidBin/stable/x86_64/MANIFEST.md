# ecoBin Stable Release Manifest

**Architecture:** x86_64-unknown-linux-gnu  
**Build Date:** January 30, 2026  
**Release:** NUCLEUS Legendary Session  
**Status:** Stable (A++ avg 101.2/100)

## Primals Harvested

| Primal | Version | Grade | Commit | Features |
|--------|---------|-------|--------|----------|
| **BearDog** | 0.9.0 | A++ (100/100) | eaedf55a0 | Socket std, Zero panics |
| **Songbird** | Latest | A+ | - | Socket std, Discovery |
| **Toadstool** | Latest | A++ | 279e1a3d | Socket std, barraCUDA 50 ops |
| **NestGate** | Latest | A+++ (110/100) | 5bc0b0ea | Socket-only mode, Legendary |
| **Squirrel** | Latest | A+ (98/100) | b59500ef | Discovery helpers, Speed |

## Socket Standard

All primals implement: `/run/user/$UID/biomeos/{primal}.sock`

**Features:**
- XDG Base Directory compliant
- 5-tier discovery pattern
- Automatic creation (0700 permissions)
- JSON-RPC 2.0 protocol

## Usage

```bash
# Deploy from plasmidBin
export BIOMEOS_PLASMID_PATH=./plasmidBin/stable/x86_64/primals

# Start primals
$BIOMEOS_PLASMID_PATH/beardog server &
$BIOMEOS_PLASMID_PATH/songbird server &
$BIOMEOS_PLASMID_PATH/toadstool &
$BIOMEOS_PLASMID_PATH/nestgate service start --socket-only &
$BIOMEOS_PLASMID_PATH/squirrel &
```

## Validation Status

- Tower Atomic: ⏳ In Progress
- Node Atomic: ⏳ Pending
- Nest Atomic: ⏳ Pending
- Full NUCLEUS: ⏳ Pending

---

**Harvested:** January 30, 2026  
**Source:** phase1/{beardog,songbird,toadstool,nestgate,squirrel}  
**Architecture:** ecoBin + plasmidBin
