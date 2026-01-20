# Tower Atomic Deployment Analysis - January 20, 2026

**Date**: January 20, 2026  
**Status**: ⚠️ **IN PROGRESS - Deployment Issues Found**  
**Focus**: Proper Tower Atomic bonding + Squirrel genetic lineage

---

## 🎯 Architecture Clarification (From User)

### Correct Model

**Tower Atomic** = BearDog + Songbird (bonded unit)  
**Squirrel** = Inherits genetics from Tower, can communicate securely  
**Communication Flow**: Squirrel → Tower → Anthropic API

### Dynamic Environment Composition

**Example Scenario**:
- **Node Atomic** might have:
  - 2 Songbirds
  - 2 BearDogs
  - 1 Squirrel
- **Different lineage relatedness** based on how they were spun up
- **Friend's compute** example - different trust levels

**Key Concept**: Genetic bonding model with dynamic composition!

---

## ✅ What's Fixed

### 1. Songbird Harvested

**Location**: `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/songbird/`

```bash
songbird-x86_64-musl: 16M, statically linked ✅
```

**Reorganized Structure**:
```
plasmidBin/primals/
├── beardog/
│   └── beardog-x86_64-musl  (5.1M) ✅
├── songbird/
│   └── songbird-x86_64-musl (16M) ✅ FIXED!
└── squirrel/
    └── squirrel-x86_64-musl (4.2M) ✅
```

All binaries now follow consistent structure!

---

## ⚠️ Current Deployment Issue

### Neural API Execution

**Graph**: `tower_squirrel.toml` ✅  
**Execution**: Started ✅  
**BearDog**: Started successfully ✅  
**Songbird**: NOT starting ❌  
**Squirrel**: NOT starting ❌

### Error in Logs

```
2026-01-20T16:04:27.687266Z  INFO    ⚡ Executing node: start-songbird (type: start)
2026-01-20T16:04:27.687416Z  WARN    ❌ Songbird socket not found: /tmp/songbird-nat0.sock
2026-01-20T16:04:27.688357Z ERROR    Failed to spawn process: Permission denied (os error 13)
2026-01-20T16:04:27.688450Z ERROR    Failed to spawn process: Permission denied (os error 13)
```

**Problem**: "Permission denied" when trying to spawn Songbird and Squirrel processes.

---

## 🔍 Investigation Needed

### 1. Process Spawning Code

**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs`

**Function**: `node_primal_start` - How does it spawn processes?

**Questions**:
- Is it using correct spawn method?
- Are environment variables passed correctly?
- Are CLI arguments formatted correctly for each primal?

### 2. Primal Start Commands

**BearDog** (Working ✅):
```bash
./plasmidBin/primals/beardog/beardog-x86_64-musl server \
  --socket /tmp/beardog-nat0.sock \
  --family-id nat0
```

**Songbird** (Not Working ❌):
```bash
# What command is Neural API trying to execute?
# Does it need different arguments?
./plasmidBin/primals/songbird/songbird-x86_64-musl server \
  --??? ???
```

**Squirrel** (Not Working ❌):
```bash
# What command is Neural API trying to execute?
./plasmidBin/primals/squirrel/squirrel-x86_64-musl server \
  --??? ???
```

### 3. UniBin Modes

Each primal might expect different CLI arguments!

**Need to check**:
- What modes does Songbird support?
- What modes does Squirrel support?
- Do they use `--socket` and `--family-id` like BearDog?

---

## 📊 Current Graph Structure

**File**: `graphs/tower_squirrel.toml`

```toml
[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }
operation = { name = "start" }
operation.params = { mode = "server", family_id = "nat0" }

[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }
depends_on = ["start-beardog"]
operation = { name = "start" }
operation.params = { mode = "server", family_id = "nat0" }

[[nodes]]
id = "start-squirrel"
primal = { by_capability = "ai" }
depends_on = ["start-songbird"]
operation = { name = "start" }
operation.params = { mode = "server", family_id = "nat0" }
```

**Issue**: Graph structure looks correct, but execution is failing!

---

## 🎯 What Needs to Happen

### 1. Understand Primal CLI Requirements

**Check each primal's help**:
```bash
./plasmidBin/primals/beardog/beardog-x86_64-musl --help
./plasmidBin/primals/songbird/songbird-x86_64-musl --help
./plasmidBin/primals/squirrel/squirrel-x86_64-musl --help
```

### 2. Review Neural Executor Code

**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs`

**Check**:
- How does `node_primal_start` translate graph params to CLI args?
- Is it handling all primals the same way?
- Are there special cases needed for different primals?

### 3. Tower Atomic as Bonded Unit

**Current graph** treats them as sequential:
```
BearDog → Songbird → Squirrel
```

**Should it express bonding**?
```toml
# Tower Atomic (bonded unit)
[[nodes]]
id = "deploy-tower-atomic"
operation = { name = "bond" }  # New operation type?
operation.params = {
  primals = ["security", "discovery"],  # BearDog + Songbird
  bonding_type = "covalent",  # Shared electrons (Towers)
  family_id = "nat0"
}

# Squirrel (inherits from Tower)
[[nodes]]
id = "start-squirrel"
primal = { by_capability = "ai" }
depends_on = ["deploy-tower-atomic"]
operation = { name = "start" }
operation.params = {
  mode = "server",
  family_id = "nat0",
  inherits_from = "tower-atomic-nat0"  # Genetic lineage!
}
```

---

## 🧬 Genetic Lineage Model

### Bonding Types (From NUCLEUS docs)

**Covalent (Organo)**:
- Shared electrons (Towers)
- High trust mesh
- BearDog + Songbird = Tower Atomic

**Inheritance**:
- Squirrel inherits genetic identity from Tower
- Can communicate securely with parent Tower
- Different levels of relatedness based on spin-up method

### Dynamic Environment

**Scenario 1**: Single local deployment
- 1 Tower Atomic (BearDog + Songbird)
- 1 Squirrel (local AI, inherits from Tower)

**Scenario 2**: Distributed with friend's compute
- Local Tower Atomic (BearDog1 + Songbird1)
- Friend's Tower Atomic (BearDog2 + Songbird2)
- Squirrel (connects to both, different trust levels)

**Scenario 3**: Node Atomic
- Tower Atomic + ToadStool (compute orchestration)
- Multiple Squirrels for different workloads

---

## 🔧 Immediate Next Steps

### Step 1: Check Primal CLIs

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Check what commands each primal supports
./plasmidBin/primals/beardog/beardog-x86_64-musl --help
./plasmidBin/primals/songbird/songbird-x86_64-musl --help
./plasmidBin/primals/squirrel/squirrel-x86_64-musl --help
```

### Step 2: Test Manual Launch

```bash
# Try launching Songbird manually to see what works
./plasmidBin/primals/songbird/songbird-x86_64-musl server \
  --help

# Try launching Squirrel manually
./plasmidBin/primals/squirrel/squirrel-x86_64-musl server \
  --help
```

### Step 3: Review Executor Code

```bash
# Check how Neural API spawns processes
grep -A 30 "fn node_primal_start" crates/biomeos-atomic-deploy/src/neural_executor.rs
```

### Step 4: Fix Spawning Logic

Based on findings, update `neural_executor.rs` to:
- Pass correct CLI arguments for each primal
- Handle environment variables properly (ANTHROPIC_API_KEY, etc.)
- Set correct socket paths

### Step 5: Re-test Deployment

```bash
# Clean deployment
pkill -f "beardog.*nat0"
pkill -f "songbird.*nat0"
pkill -f "squirrel.*nat0"
pkill -f "neural-api"
rm /tmp/*-nat0.sock

# Start Neural API
export ANTHROPIC_API_KEY="sk-ant-api03-..."
./target/release/biomeos neural-api --graphs-dir graphs &

# Execute graph
echo '{"jsonrpc":"2.0","method":"neural_api.execute_graph",
  "params":{"graph_id":"tower_squirrel","family_id":"nat0"},"id":1}' \
  | nc -U /tmp/neural-api-nat0.sock

# Verify
ls -la /tmp/*-nat0.sock
ps aux | grep -E "(beardog|songbird|squirrel)" | grep nat0
```

---

## 📚 Files to Review

### Code Files
- `crates/biomeos-atomic-deploy/src/neural_executor.rs` - Process spawning
- `crates/biomeos-atomic-deploy/src/neural_graph.rs` - Graph structure
- `crates/biomeos-atomic-deploy/src/primal_launcher.rs` - Primal launching

### Graph Files
- `graphs/tower_squirrel.toml` - Current graph
- `graphs/tower_atomic.toml` - Tower-only deployment
- `graphs/BONDING_TESTS_README.md` - Bonding model docs

### Primal Source
- `/home/eastgate/Development/ecoPrimals/phase1/beardog/` - BearDog CLI
- `/home/eastgate/Development/ecoPrimals/phase1/songbird/` - Songbird CLI
- `/home/eastgate/Development/ecoPrimals/phase1/squirrel/` - Squirrel CLI

---

## 🎯 Success Criteria

### Tower Atomic Deployment
- ✅ BearDog starts and creates socket
- ✅ Songbird starts and creates socket
- ✅ BearDog and Songbird are bonded (covalent)
- ✅ Tower Atomic is operational as a unit

### Squirrel Integration
- ✅ Squirrel starts after Tower is ready
- ✅ Squirrel inherits genetic identity from Tower
- ✅ Squirrel can communicate with Tower (secure)
- ✅ Squirrel can request HTTP via Tower
- ✅ Tower handles Anthropic API calls for Squirrel

### End-to-End Test
- ✅ All three primals running
- ✅ Squirrel makes AI request
- ✅ Request routed through Tower Atomic
- ✅ Tower Atomic handles HTTPS to Anthropic
- ✅ Response flows back: Anthropic → Tower → Squirrel
- ✅ **TRUE service mesh with genetic lineage!**

---

## 💡 Key Insight

The user is describing a **genetic bonding model** where:
- **Tower Atomic** is a covalently bonded unit (BearDog + Songbird)
- **Squirrel** inherits from the Tower (like offspring)
- **Different deployments** have different levels of genetic relatedness
- **Dynamic composition** allows for complex multi-Tower, multi-Squirrel environments

This is more sophisticated than simple sequential deployment - it's about **genetic lineage and bonding chemistry**!

---

**Status**: ⚠️ Working to resolve deployment issues  
**Next**: Check primal CLIs and fix spawning logic  
**Goal**: Full Tower Atomic + Squirrel with genetic lineage!

---

**Date**: January 20, 2026  
**Version**: v0.28.0  
**Approach**: ✅ Graph-based deployment (CORRECT!)  
**Challenge**: Process spawning and primal CLI compatibility

