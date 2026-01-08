# 📊 biomeOS Graphs - Neural API Orchestration

**Purpose:** Graph definitions for adaptive primal coordination

**Phase:** 1.1 - Sequential Execution

---

## 🎯 Tower Niche Graphs

### `tower_deploy.toml` - Complete Tower Deployment

**Coordination:** Sequential  
**Purpose:** Deploy Songbird + BearDog + establish federation

**Phases:**
1. **Discovery** - Find available primals by capability
2. **Startup** - Launch primal daemons with retry
3. **Verification** - Verify genetic lineage
4. **Federation** - Discover peers and establish tunnels
5. **Announcement** - Broadcast capabilities

**Usage:**
```rust
let graph = GraphParser::parse_file("graphs/tower_deploy.toml")?;
let result = executor.execute(graph).await?;
```

**Deep Debt Principles:**
- ✅ Capability-based discovery (`by_capability = "discovery"`)
- ✅ No hardcoded primal names
- ✅ Runtime primal resolution
- ✅ Retry policies for robustness
- ✅ Timeout constraints

---

### `tower_health_check.toml` - Health Verification

**Coordination:** Parallel  
**Purpose:** Verify all tower components are healthy

**Checks:**
- Songbird health (parallel group 1)
- BearDog health (parallel group 1)
- Federation status (parallel group 1)

**All checks run simultaneously for speed.**

---

### `tower_shutdown.toml` - Graceful Shutdown

**Coordination:** Sequential  
**Purpose:** Gracefully shutdown tower components

**Steps:**
1. Drain active tunnels
2. Stop Songbird
3. Stop BearDog

**Ensures no data loss during shutdown.**

---

## 🧠 Neural API Principles

### Capability-Based Discovery

❌ **OLD (Hardcoded):**
```toml
primal = { by_id = "songbird-1" }  # Fragile!
```

✅ **NEW (Capability-Based):**
```toml
primal = { by_capability = "discovery" }  # Adaptive!
```

**Benefits:**
- Works with any primal that provides capability
- Enables primal evolution without breaking graphs
- Supports chimera primals (multiple capabilities)
- Enables hot-swapping and blue-green deployments

---

### Multiple Capabilities

For primals requiring multiple capabilities:

```toml
primal = { by_capabilities = ["discovery", "tunneling", "encryption"] }
```

**Discovers primal that has ALL listed capabilities.**

---

## 📈 Evolution Roadmap

### Phase 1.1: Sequential (CURRENT)
- ✅ Capability-based discovery
- ✅ Sequential execution
- ✅ Timeout constraints
- ✅ Retry policies
- ✅ Output variables

### Phase 1.2: Parallel (NEXT)
- ⏳ Parallel execution groups
- ⏳ Concurrent node startup
- ⏳ Resource pooling

### Phase 1.3: DAG (FUTURE)
- 🔮 Conditional branches
- 🔮 Data flow between nodes
- 🔮 Complex dependencies

### Phase 1.4: Pipeline (FUTURE)
- 🔮 Streaming data
- 🔮 Backpressure handling
- 🔮 Transform chains

---

## 📊 Metrics Collection

All graph executions collect:
- Node execution time
- Success/failure rates
- Primal resolution time
- Bottleneck identification

**Used by learning engine to optimize pathways.**

---

## 🧪 Testing Graphs

Test graph parsing:
```bash
cargo test --package biomeos-graph
```

Test with mock primals:
```rust
let executor = GraphExecutor::new(
    MockPrimalOperationExecutor::new()
        .with_primal("test-primal", vec!["discovery"])
);
```

---

## 🎯 Next Steps

1. **Implement `PrimalRegistry`** - Discover primals at runtime
2. **Integrate with `BiomeOS`** - Replace wave system with graph executor
3. **Deploy to liveSpore** - Test tower deployment
4. **Collect metrics** - Track performance
5. **Add parallel execution** - Speed up deployments

---

**Status:** 🎯 **Phase 1.1 Complete - Sequential Execution Ready!**

🧠 **From static waves → adaptive graphs!** 🎊

