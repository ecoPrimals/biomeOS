# 🔌 Scenario 03: Primal Adapter Pattern

**Status**: ✅ Implemented  
**Complexity**: Medium  
**Duration**: 5 minutes

---

## 🎯 What This Demonstrates

BiomeOS's **CLI-agnostic primal integration** using the Primal Adapter Pattern:

1. **Interface Discovery**: Automatically learn how to talk to any primal
2. **Adapter Caching**: Persist learned interfaces for fast reuse
3. **Future-Proof**: Handle primal evolution without code changes
4. **Sovereignty-First**: Adapt to primals, don't dictate their interfaces

---

## 🏗️ Architecture

### Problem
- Each primal has unique CLI interface
- Squirrel: direct execution
- NestGate: `service` subcommand
- ToadStool: different pattern
- Forcing standardization violates sovereignty

### Solution: Primal Adapter Pattern
```rust
// BiomeOS learns each primal's interface
let adapter = discover_primal_interface(Path::new("./squirrel-bin")).await?;

// Interface discovered automatically
match adapter.interface {
    PrimalInterface::Direct { args } => { /* learned! */ }
    PrimalInterface::Subcommand { start_cmd, .. } => { /* learned! */ }
    // ... other patterns
}

// Start using discovered interface
adapter.start(9010).await?;
```

---

## 🚀 Running the Demo

### Quick Run
```bash
cd showcase/03-primal-adapter/
./demo.sh
```

### What Happens
1. **Discovers** Squirrel's CLI interface (direct execution)
2. **Identifies** capabilities (can start, graceful shutdown, etc.)
3. **Caches** learned interface to `~/.biomeos/primal_adapters/squirrel.yaml`
4. **Verifies** cache by loading it back

---

## 📊 Expected Output

```
=== Primal Adapter Demo with Squirrel ===

✅ Found Squirrel binary

🔍 Discovering Squirrel interface...
✅ Discovered interface:
   Name: squirrel
   Interface: Direct { args: [] }
   Can start: true
   Can stop: false
   Graceful shutdown: true

💾 Saving to cache...
✅ Cached at ~/.biomeos/primal_adapters/squirrel.yaml

📂 Loading from cache...
✅ Loaded from cache: squirrel

🎉 Primal Adapter Pattern working!

Key Benefits:
  ✅ CLI-agnostic (learned Squirrel's interface)
  ✅ Cached for fast reuse
  ✅ No hardcoded assumptions
  ✅ Respects primal sovereignty
```

---

## 🔍 Verify Manually

### Check Cached Adapter
```bash
cat ~/.biomeos/primal_adapters/squirrel.yaml
```

Example output:
```yaml
name: squirrel
binary: ../../phase1bins/squirrel-bin
interface:
  Direct:
    args: []
capabilities:
  lifecycle:
    can_start: true
    can_stop: false
    can_restart: false
    graceful_shutdown: true
    can_refuse: true
  health_check:
    url_pattern: http://localhost:PORT/health
    expected_status: 200
    timeout:
      secs: 2
      nanos: 0
  port_config:
    EnvVar: PORT
  has_version_cmd: true
  has_fast_help: false
discovered_at: '2025-12-24T...'
version: null
```

---

## 🧬 How It Works

### 1. Interface Discovery
BiomeOS probes common patterns:
- **Direct execution** (no subcommands)
- **Subcommand patterns** (serve, start, service, run)
- **Service managers** (systemd, docker)
- **API-based** (HTTP lifecycle endpoints)

### 2. Capability Detection
For each discovered interface:
- Can we start it?
- Can we stop it?
- How to configure port?
- Health check endpoint?
- Version command?

### 3. Caching
Learned interfaces cached to:
- Avoid re-discovery overhead
- Enable offline operation
- Share knowledge across BiomeOS instances

### 4. Cache Invalidation
Cache invalidated when:
- Primal version changes
- Manual force re-discovery
- Cache older than 7 days (optional)

---

## 🌱 Philosophy

### Cell Senescence, Not Overwatch
- BiomeOS **learns** how to talk to primals
- BiomeOS **adapts** to primal interfaces
- Primals **control** their own CLI
- BiomeOS **respects** sovereignty

### Future-Proof Evolution
- New primals? Discover automatically
- Primal CLI changes? Re-discover and adapt
- New interface patterns? Add to probe list
- No code changes needed

---

## 📚 Key Concepts

### 1. PrimalAdapter
Encapsulates knowledge about a primal:
- Name and binary path
- Discovered interface pattern
- Capabilities (what it can do)
- Current lifecycle state

### 2. PrimalInterface
How to communicate:
- `Direct`: Like Squirrel (no subcommands)
- `Subcommand`: Like NestGate (service subcommand)
- `Service`: Systemd/Docker managed
- `Api`: HTTP-based lifecycle
- `Unknown`: Still learning

### 3. PrimalCapabilities
What primal supports:
- Lifecycle (start/stop/restart)
- Health checks
- Port configuration method
- Version queries

### 4. AdapterCache
Persistent storage:
- Save discovered adapters
- Load for fast reuse
- Invalidate when needed
- Share across instances

---

## 🎯 Success Criteria

✅ **Interface Discovered**: Automatically learned Squirrel's CLI  
✅ **Capabilities Identified**: Knows what Squirrel can do  
✅ **Cached Successfully**: Saved to ~/.biomeos/primal_adapters/  
✅ **Cache Reused**: Loaded from disk works  
✅ **No Hardcoding**: Zero assumptions about Squirrel's interface

---

## 🔄 What's Next

### Scenario 04: Multi-Primal Adaptation
- Discover multiple primals (all 5)
- Handle different interfaces simultaneously
- Cache and reuse all adapters

### Scenario 05: Lifecycle Negotiation
- Request primal to start
- Handle refusals gracefully
- Respect sovereignty

### Scenario 06: Songbird Port Manager
- Delegate port allocation to Songbird
- Dynamic service mesh registration
- Zero hardcoded endpoints

---

## 📖 References

- [Primal Integration Architecture](../../docs/PRIMAL_INTEGRATION_ARCHITECTURE.md)
- [Phase 1 Integration Gaps](../../docs/PHASE1_INTEGRATION_GAPS.md)
- [Adapter Pattern Implementation](../../crates/biomeos-core/src/primal_adapter/)

---

**Status**: ✅ Working  
**Architecture**: Primal Adapter Pattern  
**Philosophy**: Adapt, don't dictate

*"BiomeOS learns to speak each primal's language."* 🔌🌱✨

