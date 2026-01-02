# 🔄 Scenario 04: Multi-Primal Adaptation

**Status**: 🚧 In Progress  
**Complexity**: Medium  
**Duration**: 10 minutes

---

## 🎯 What This Demonstrates

BiomeOS discovering and adapting to **multiple primals simultaneously**:

1. **Parallel Discovery**: Probe all 5 primals at once
2. **Mixed Interfaces**: Handle different CLI patterns together
3. **Adapter Registry**: Manage multiple learned interfaces
4. **Graceful Degradation**: Missing primals don't break the system

---

## 🏗️ Architecture

### Problem
- Each primal has unique CLI
- Need to manage all 5 simultaneously
- Some may be unavailable
- Different startup patterns

### Solution: Adapter Registry
```rust
// Discover all primals in parallel
let adapters = discover_all_primals(primal_paths).await?;

// Registry manages all adapters
let registry = AdapterRegistry::new();
for adapter in adapters {
    registry.register(adapter)?;
}

// Start all available primals
registry.start_all(port_assignments).await?;

// Check overall health
let health = registry.health_check().await?;
```

---

## 🚀 Running the Demo

### With Phase 1 Binaries (when available)
```bash
cd showcase/04-multi-primal-adaptation/
./demo-real.sh
```

### With Mock Primals (available now)
```bash
cd showcase/04-multi-primal-adaptation/
./demo-mock.sh
```

---

## 🎭 Mock Primals

We created mock primals to test immediately:

### Mock Squirrel (Direct Pattern)
```bash
#!/usr/bin/env bash
# ./mock-primals/squirrel-mock
# Direct execution, no subcommands
PORT=${PORT:-9010}
echo "Squirrel AI/MCP Primal starting on port $PORT"
python3 -m http.server $PORT
```

### Mock NestGate (Subcommand Pattern)
```bash
#!/usr/bin/env bash
# ./mock-primals/nestgate-mock
# Subcommand: 'service'
if [ "$1" = "service" ]; then
    PORT=${PORT:-9020}
    echo "NestGate Storage starting on port $PORT"
    python3 -m http.server $PORT
else
    echo "Usage: nestgate-mock service"
    exit 1
fi
```

### Similar patterns for all 5 primals

---

## 📊 Expected Output

```
=== Multi-Primal Adaptation Demo ===

🔍 Discovering all primals in parallel...

✅ Squirrel discovered (Direct execution)
✅ NestGate discovered (Subcommand: service)
✅ ToadStool discovered (Direct execution)
✅ BearDog discovered (Subcommand: serve)
✅ Songbird discovered (Subcommand: start)

💾 Caching all adapters...
✅ Cached 5 adapters to ~/.biomeos/primal_adapters/

🚀 Starting all primals...
✅ Squirrel started on port 9010
✅ NestGate started on port 9020
✅ ToadStool started on port 9030
✅ BearDog started on port 9040
✅ Songbird started on port 9050

🏥 Health checking...
✅ 5/5 primals healthy

🎉 Multi-Primal Adaptation Complete!

Adapter Registry:
  - Squirrel: Direct, port 9010, healthy
  - NestGate: Subcommand(service), port 9020, healthy
  - ToadStool: Direct, port 9030, healthy
  - BearDog: Subcommand(serve), port 9040, healthy
  - Songbird: Subcommand(start), port 9050, healthy
```

---

## 🔍 Verify Manually

### Check all cached adapters
```bash
ls ~/.biomeos/primal_adapters/
# squirrel.yaml  nestgate.yaml  toadstool.yaml  beardog.yaml  songbird.yaml

cat ~/.biomeos/primal_adapters/squirrel.yaml
# Shows discovered interface and capabilities
```

### Check running primals
```bash
curl http://localhost:9010/health  # Squirrel
curl http://localhost:9020/health  # NestGate
# ... etc
```

### Check logs
```bash
tail -f /tmp/squirrel.log
tail -f /tmp/nestgate.log
# ... etc
```

---

## 🧬 How It Works

### 1. Parallel Discovery
```rust
// Discover all primals concurrently
let futures: Vec<_> = primal_paths
    .iter()
    .map(|path| discover_primal_interface(path))
    .collect();

let adapters = futures::future::join_all(futures).await;
```

### 2. Adapter Registry
```rust
pub struct AdapterRegistry {
    adapters: HashMap<String, PrimalAdapter>,
}

impl AdapterRegistry {
    pub fn register(&mut self, adapter: PrimalAdapter) -> Result<()>;
    pub fn get(&self, name: &str) -> Option<&PrimalAdapter>;
    pub fn start_all(&mut self, ports: HashMap<String, u16>) -> Result<()>;
    pub fn health_check(&self) -> Result<HealthReport>;
}
```

### 3. Mixed Interface Handling
```rust
// Each adapter knows its own interface
for adapter in registry.adapters.values_mut() {
    match &adapter.interface {
        PrimalInterface::Direct { .. } => {
            // Start directly
        }
        PrimalInterface::Subcommand { start_cmd, .. } => {
            // Start with subcommand
        }
        _ => {
            // Handle other patterns
        }
    }
}
```

### 4. Graceful Degradation
```rust
// If discovery fails, just log and continue
match discover_primal_interface(path).await {
    Ok(adapter) => registry.register(adapter)?,
    Err(e) => {
        warn!("Could not discover {}: {}", path.display(), e);
        // Continue with other primals
    }
}
```

---

## 🌱 Philosophy

### Ecological Substrate
- BiomeOS manages the environment
- Each primal is autonomous
- Missing primals = degraded but functional
- No single point of failure

### Parallel Discovery
- Fast (all at once)
- Non-blocking (failures don't block others)
- Cached (reuse learned interfaces)
- Resilient (graceful degradation)

### Mixed Interfaces
- Squirrel: Direct execution
- NestGate: Subcommand pattern
- All work together seamlessly
- BiomeOS adapts to each

---

## 🎯 Success Criteria

✅ **Parallel Discovery**: All 5 primals discovered concurrently  
✅ **Mixed Interfaces**: Different CLI patterns handled  
✅ **All Cached**: Adapters saved for reuse  
✅ **All Started**: Each primal running on assigned port  
✅ **Health Checks**: All reporting healthy  
✅ **Graceful Degradation**: Missing primals don't break system

---

## 🔄 What's Next

### Scenario 05: Lifecycle Negotiation
- Request graceful stop
- Handle primal refusals
- Respect sovereignty
- Ecosystem coordination

### Scenario 06: Songbird Port Manager
- Dynamic port allocation
- Service mesh registration
- Connection routing
- Zero hardcoded endpoints

---

## 📖 References

- [Primal Adapter Pattern](../03-primal-adapter/README.md)
- [Primal Integration Architecture](../../docs/PRIMAL_INTEGRATION_ARCHITECTURE.md)
- [Adapter Registry Implementation](../../crates/biomeos-core/src/primal_adapter/registry.rs)

---

**Status**: 🚧 Building now  
**Ready**: Mock primals for immediate testing  
**Waiting**: Real primal CLIs from Phase 1 teams

*"BiomeOS speaks all dialects fluently."* 🔄🌱✨

