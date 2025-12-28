# 01 - Hello BiomeOS: Runtime Discovery Demo

**Duration**: 2 minutes  
**Prerequisites**: NestGate running on port 9020

---

## Overview

This demo demonstrates BiomeOS's core capability: **agnostic runtime discovery**.

**What it shows**:
- Zero hardcoding - discover what's actually available
- Adapt to primal reality (server, CLI, library)
- Compose capabilities dynamically

---

## The Problem We Solve

**❌ Traditional approach** (hardcoding):
```bash
# Assumes everything is standardized
curl http://localhost:9020/health  # NestGate
curl http://localhost:9000/health  # Songbird
curl http://localhost:9040/health  # BearDog
# Breaks when primals evolve!
```

**✅ BiomeOS approach** (discovery):
```bash
# Discover what's actually available
./discover-and-adapt.sh
# Works regardless of primal evolution!
```

---

## Run the Demo

```bash
cd showcase/00-substrate/01-hello-biomeos
./demo.sh
```

---

## What You'll See

1. **Discovery Phase**: BiomeOS discovers available primals
2. **Adaptation Phase**: BiomeOS adapts to each primal's architecture
3. **Composition Phase**: BiomeOS composes capabilities

Example output:
```
╔════════════════════════════════════════╗
║  BiomeOS Runtime Discovery             ║
╚════════════════════════════════════════╝

🔍 Discovering available primals...
✅ Discovered 7 primals:
  - beardog (encryption, CLI tool)
  - nestgate (storage, REST API)
  - songbird (orchestration, investigating...)
  - toadstool (compute, CLI tool)
  - squirrel (configuration, CLI tool)
  - loamspine (analysis)
  - petaltongue (UI)

🔍 Checking capabilities...
✅ Storage: http://localhost:9020 (NestGate REST API)
✅ Encryption: ../primals/beardog (CLI tool)
✅ Compute: ../primals/toadstool (CLI tool)
⚠  Orchestration: Not available

✅ Discovery complete
```

---

## Key Concepts

### 1. Zero Hardcoding
BiomeOS doesn't know primal names or endpoints. It discovers them.

### 2. Agnostic Adaptation
Each primal has a different architecture:
- **NestGate**: REST API server (requires JWT)
- **BearDog**: CLI tool (in-house crypto)
- **Toadstool**: Runtime launcher
- **Squirrel**: Interactive CLI

BiomeOS adapts to each.

### 3. Capability-Based Discovery
Instead of "Is Nest Gate running?", ask "Is storage capability available?"

```bash
# Capability-based query
storage=$(discover_capability "storage")
# Returns: http://localhost:9020 (or error if unavailable)

# Now use it
curl $storage/api/v1/zfs/datasets
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│          BiomeOS Discovery              │
│                                         │
│  1. Scan for available primals          │
│  2. Determine type (API/CLI/lib)        │
│  3. Map to capabilities                 │
│  4. Create adapters                     │
└─────────────────────────────────────────┘
         │           │          │
         ▼           ▼          ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │NestGate│  │BearDog │  │Toadstool│
    │REST API│  │CLI Tool│  │ Launcher│
    │Port    │  │Binary  │  │ Binary  │
    │ 9020   │  │Execute │  │ Execute │
    └────────┘  └────────┘  └────────┘
```

---

## Technical Details

### Discovery Process
1. **Scan**: Check `primals/` directory for binaries
2. **Probe**: Run `--help` to determine type
3. **Test**: Try health endpoints for servers
4. **Map**: Associate with capabilities

### Adaptation Strategy
```bash
case "$primal_type" in
    rest_api)
        # Use curl/HTTP client
        curl "$endpoint/health"
        ;;
    cli_tool)
        # Execute binary
        ./primals/$primal --command
        ;;
    library)
        # Link and call
        # (future: WASM, FFI, etc.)
        ;;
esac
```

---

## Success Criteria

✅ **Discovery works**: Finds all available primals  
✅ **Adaptation works**: Correctly identifies primal types  
✅ **No hardcoding**: No primal names in biomeOS core  
✅ **Resilient**: Gracefully handles missing primals  

---

## Next Steps

After this demo:
- **02-capability-composition**: Compose multiple capabilities
- **03-primal-evolution**: Show how primals can evolve
- **04-custom-primals**: User-defined primals

---

## Files in This Demo

- `demo.sh` - Main demo script
- `discover-and-adapt.sh` - Discovery implementation
- `README.md` - This file
- `validate.sh` - Validation script (for benchScale)

---

**Philosophy**: *"BiomeOS discovers reality, doesn't impose it."*

