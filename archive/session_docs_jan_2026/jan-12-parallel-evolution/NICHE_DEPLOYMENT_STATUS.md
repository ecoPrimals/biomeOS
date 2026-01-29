# 🧬 Niche Deployment Status - Neural API Integration

**Date**: January 11, 2026  
**Status**: Graphs Ready, Executor Integration Needed  

---

## ✅ **What We Have**

### 1. Neural API Deployment Graphs (Complete!)
- ✅ `graphs/tower_deploy.toml` - Tower niche (security + discovery)
- ✅ `graphs/node_deploy.toml` - Node niche (compute)
- ✅ `graphs/nest_deploy.toml` - Nest niche (storage)
- ✅ `graphs/nucleus_deploy.toml` - Complete NUCLEUS (all 3!)

All graphs are **production-ready** with:
- Capability-based discovery (zero hardcoding)
- Proper sequencing with dependencies
- Health verification
- Test operations

### 2. Graph Executor Infrastructure
- ✅ `crates/biomeos-graph/src/executor.rs` - Sequential executor
- ✅ `crates/biomeos-graph/src/nucleus_executor.rs` - NUCLEUS integration
- ✅ `crates/biomeos-graph/src/context.rs` - Execution context
- ⏳ Integration with live primals (pending)

### 3. Live Primals Running
```
✅ Songbird (2 instances) - Discovery
✅ BearDog - Security/Encryption
✅ ToadStool - Compute
✅ Squirrel - AI
✅ NestGate - Storage (just started)
```

---

## ⚠️  **What's Blocking Deployment**

### Issue 1: Primal Socket Paths

**Current State:**
- BearDog: Running, but NO Unix socket created
- Squirrel: Socket at `/tmp/squirrel-squirrel.sock` (not XDG!)
- ToadStool: Socket at `/run/user/1000/toadstool-default.jsonrpc.sock` ✅
- NestGate: Needs `service start` command
- Songbird: No sockets visible

**Expected:**
- All primals: `/run/user/{uid}/{primal}-{family}.sock`
- Example: `/run/user/1000/beardog-nat0.sock`

**Root Cause:**
Primals were started WITHOUT proper command-line arguments. They're running in standalone mode without registering sockets properly.

### Issue 2: Primal Command-Line Interface

**BearDog:**
- ❌ `--help` causes core dump (SIGABRT)
- ✅ Runs without args but doesn't create socket
- 🔍 Needs: `--family-id nat0 --socket /run/user/1000/beardog-nat0.sock`

**NestGate:**
- ✅ Has proper CLI: `nestgate service start`
- ⏳ Needs to be restarted with proper args

**Squirrel:**
- ✅ Has CLI and runs successfully
- ⚠️  Uses `/tmp/` instead of XDG runtime dir
- 🔍 Needs: Socket path configuration

### Issue 3: Graph Executor Not Wired

**Current:**
- `examples/nucleus_graph_e2e.rs` exists but has compilation errors
- No `cargo run --bin deploy-niche` command
- Graphs cannot be executed yet

**Needed:**
- Fix example compilation
- Create `bin/deploy-niche.rs` binary
- Wire up `NucleusPrimalExecutor` to live primals

---

## 🎯 **Next Steps**

### Priority 1: Restart Primals Properly

```bash
# Kill current instances
pkill beardog toadstool squirrel nestgate

# Start with proper configuration
export FAMILY_ID=nat0
export SOCKET_DIR=/run/user/$(id -u)

# BearDog (security)
beardog --family-id $FAMILY_ID --socket $SOCKET_DIR/beardog-$FAMILY_ID.sock &

# ToadStool (compute) - already using Unix socket!
toadstool --family-id $FAMILY_ID &

# Squirrel (AI)
squirrel --family-id $FAMILY_ID --socket $SOCKET_DIR/squirrel-$FAMILY_ID.sock &

# NestGate (storage)
nestgate service start --family-id $FAMILY_ID &
```

**Blocker**: We need to determine the correct CLI args for each primal.

### Priority 2: Create Deployment Binary

Create `src/bin/deploy_niche.rs`:

```rust
//! Niche Deployment Binary
//!
//! Deploys niches using Neural API graphs + NUCLEUS

use biomeos_graph::{GraphExecutor, NucleusPrimalExecutor, PrimalGraph};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <graph.toml>", args[0]);
        std::process::exit(1);
    }

    let graph_path = PathBuf::from(&args[1]);
    info!("📊 Loading graph: {:?}", graph_path);

    // Load graph
    let graph = PrimalGraph::from_file(&graph_path).await?;

    // Create NUCLEUS executor
    let executor = NucleusPrimalExecutor::new().await?;
    let graph_executor = GraphExecutor::new(executor);

    // Execute!
    let result = graph_executor.execute(graph).await?;

    if result.success {
        println!("✅ Niche deployed successfully!");
    } else {
        eprintln!("❌ Deployment failed");
        std::process::exit(1);
    }

    Ok(())
}
```

### Priority 3: Test Deployments

Once primals are properly started:

```bash
# Test Tower deployment
cargo run --bin deploy_niche -- graphs/tower_deploy.toml

# Test Node deployment
cargo run --bin deploy_niche -- graphs/node_deploy.toml

# Test Nest deployment
cargo run --bin deploy_niche -- graphs/nest_deploy.toml

# Test complete NUCLEUS
cargo run --bin deploy_niche -- graphs/nucleus_deploy.toml
```

---

## 📊 **Current System State**

```
🟢 Songbird (2x) - Running, no sockets visible
🟢 BearDog      - Running, no socket
🟢 ToadStool    - Running, socket at correct location ✅
🟢 Squirrel     - Running, socket at /tmp (wrong location)
🟢 NestGate     - Just started, needs `service start`

Graphs: ✅ All ready
Executor: ⏳ Integration needed
Tests: ⏳ Blocked by primal socket issues
```

---

## 🤔 **Questions for Primal Teams**

### For BearDog:
1. Why does `--help` cause SIGABRT?
2. What CLI args create a Unix socket at `/run/user/{uid}/beardog-{family}.sock`?
3. How to register with Songbird automatically?

### For Squirrel:
1. Can socket path be configured to use XDG runtime dir?
2. What's the correct CLI for `--socket` argument?

### For NestGate:
1. Does `nestgate service start` create Unix socket automatically?
2. What's the socket path format?

### For Songbird:
1. The 2 running instances - where are their sockets?
2. Are they the `songbird_orchestrator` test binaries?

---

## 💡 **Alternative: Use Existing Connections**

ToadStool already has a proper socket! We can:
1. Test with ToadStool-only graphs first
2. Verify executor works
3. Add other primals once their sockets are fixed

**Quick Win Graph:**

```toml
# graphs/toadstool_test.toml
[graph]
name = "toadstool_test"
coordination = "Sequential"

[[nodes]]
id = "health"
primal = { by_capability = "compute" }
operation = { name = "health" }

[[nodes]]
id = "capabilities"
primal = { by_capability = "compute" }
operation = { name = "query_capabilities" }

[[edges]]
from = "health"
to = "capabilities"
```

---

## 🚀 **Immediate Action Items**

1. **Document primal CLI args** for each primal
2. **Restart all primals** with proper socket configuration
3. **Create `deploy_niche` binary**
4. **Test with ToadStool first** (already has socket)
5. **Expand to other primals** once sockets are fixed

---

**Status**: Ready to proceed once primal sockets are configured properly!

**Estimated Time**: 2-3 hours to wire everything up


