# BiomeOS Enhanced Showcases: Capability-Based Discovery

**Status**: ✅ 2/5 Single-Primal Showcases Complete  
**Philosophy**: Zero Hardcoding, Runtime Discovery, Evolution Resilience

---

## 🎯 What Makes These Showcases Different

### Old Approach (Problems)
```bash
# Hardcoded knowledge
SONGBIRD_ENDPOINT="http://localhost:3000"  # Breaks if port changes
curl $SONGBIRD_ENDPOINT/api/v1/info       # Breaks if API changes

# Named dependencies
if command -v songbird; then              # Breaks if renamed
    songbird tower start
fi
```

**Problems:**
- Recompilation needed when primals change
- Fails if primal renamed or replaced
- No graceful degradation
- Tight coupling between BiomeOS and primals

### New Approach (Solutions)
```bash
# Capability-based discovery
endpoint=$(discover_primal_by_capability "service_registry")

# Runtime interface probing
probe_primal_interface "$endpoint"  # Discovers current API

# Graceful degradation
if [ -z "$endpoint" ]; then
    graceful_degradation "service_registry" "Start Songbird or alternative"
fi
```

**Benefits:**
- Works when primals evolve
- Works with alternate implementations
- Clear error messages
- Loose coupling via capabilities

---

## 🔧 Core Infrastructure

### `common/capability-discovery.sh`

**Zero-knowledge discovery library** used by all showcases.

**Key Functions:**

1. **`discover_primal_by_capability(capability, [type])`**
   - Discovers primals by what they DO, not what they're CALLED
   - Tries multiple methods: env vars → discovery service → network scan
   - Returns endpoint or gracefully fails

2. **`probe_primal_interface(endpoint)`**
   - Discovers HOW to communicate with a primal
   - Probes common endpoint patterns
   - Sets: `HEALTH_ENDPOINT`, `INFO_ENDPOINT`, `CAPABILITIES_ENDPOINT`

3. **`verify_primal_capability(endpoint, capability)`**
   - Verifies primal actually provides required capability
   - Returns 0 if verified, 1 if not

4. **`start_primal_smart(name, port, [args])`**
   - Intelligently finds and starts primal binary
   - Searches multiple locations
   - Handles logging and PID management

5. **`graceful_degradation(capability, suggestion)`**
   - Handles missing primals gracefully
   - Clear messaging for operators
   - Suggests resolution paths

**Philosophy:**
> "Each primal knows only itself. BiomeOS discovers everything at runtime."

---

## ✅ Completed Showcases

### 1. Songbird Discovery (Enhanced)
**File**: `songbird-discovery-enhanced.sh`

**Demonstrates:**
- ✅ Zero hardcoded knowledge of "Songbird"
- ✅ Discovery via `service_registry` capability
- ✅ Multiple discovery methods (env → mDNS → scan)
- ✅ Runtime interface adaptation
- ✅ Universal Port Authority pattern
- ✅ Evolution resilience testing

**Key Scenarios Tested:**
1. What if Songbird's API changes? → BiomeOS discovers new interface
2. What if Songbird's port changes? → Discovery finds current port
3. What if alternate primal provides service_registry? → Works with either
4. What if Songbird unavailable? → Graceful degradation

**Run:**
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/showcase/01-single-primal
./songbird-discovery-enhanced.sh
```

### 2. Toadstool Compute (Enhanced)
**File**: `toadstool-compute-enhanced.sh`

**Demonstrates:**
- ✅ Discovery via `compute` capability (not "Toadstool" name)
- ✅ Runtime interface adaptation
- ✅ Task submission and monitoring
- ✅ GPU capability detection
- ✅ Distributed compute support check
- ✅ Resilience to compute primal evolution

**Key Scenarios Tested:**
1. What if Toadstool updates API? → BiomeOS re-probes endpoints
2. What if alternate compute primal appears? → Can use both
3. What if compute unavailable? → Graceful degradation
4. What if GPU capability added/removed? → Dynamic capability query

**Run:**
```bash
./toadstool-compute-enhanced.sh
```

---

## 🚧 In Progress

### 3. BearDog Security (Pending)
**File**: `beardog-security-enhanced.sh`

**Will Demonstrate:**
- Discovery via `encryption` capability
- Entropy hierarchy pattern
- Multi-layer encryption
- Key rotation handling

### 4. Nestgate Storage (Pending)
**File**: `nestgate-storage-enhanced.sh`

**Will Demonstrate:**
- Discovery via `storage` capability
- Lineage-gated access control
- Data sovereignty patterns
- Volume lifecycle management

### 5. Squirrel AI (Pending)
**File**: `squirrel-ai-enhanced.sh`

**Will Demonstrate:**
- Discovery via `ai` capability
- MCP protocol adaptation
- Multi-agent orchestration
- Tool execution pipeline

---

## 📐 Architecture Principles

### 1. Capability-Based Discovery

**Never hardcode:**
- ❌ Service names ("Songbird", "Toadstool")
- ❌ Endpoints ("localhost:3000")
- ❌ API paths ("/api/v1/...")
- ❌ Port numbers (3000, 8080)

**Always discover:**
- ✅ By capability ("service_registry", "compute")
- ✅ Via multiple methods (env → mDNS → scan)
- ✅ At runtime (probe interfaces)
- ✅ With verification (check capabilities)

### 2. Runtime Interface Adaptation

**Instead of assuming:**
```rust
// BAD: Hardcoded interface
let endpoint = "http://localhost:3000/api/v1/info";
```

**Discover interface:**
```bash
# GOOD: Discovered interface
probe_primal_interface "$discovered_endpoint"
# Sets: HEALTH_ENDPOINT, INFO_ENDPOINT, etc.
```

### 3. Graceful Degradation

**Instead of failing hard:**
```bash
# BAD: Hard failure
if [ ! -f /usr/bin/songbird ]; then
    echo "Error: Songbird not found"
    exit 1
fi
```

**Fail gracefully:**
```bash
# GOOD: Graceful degradation
if ! endpoint=$(discover_primal_by_capability "service_registry"); then
    graceful_degradation "service_registry" \
        "Start Songbird or set DISCOVERY_ENDPOINT"
    # Continue with reduced functionality
fi
```

### 4. Zero Compile-Time Dependencies

**Philosophy:**
> "BiomeOS doesn't import primal code. Ever."

All communication via:
- HTTP/JSON
- Discovered endpoints
- Probed interfaces
- Runtime capabilities

---

## 🎓 Evolution Scenarios

### Scenario 1: Primal API Changes

**Old System:**
```
Songbird v2.0 changes /api/v1/info → /api/v2/info
❌ BiomeOS breaks
❌ Need to update code
❌ Need to recompile
❌ Need to redeploy
```

**New System:**
```
Songbird v2.0 changes /api/v1/info → /api/v2/info
✅ probe_primal_interface() discovers new path
✅ No code changes
✅ No recompilation
✅ Works immediately
```

### Scenario 2: Alternate Primal Appears

**Old System:**
```
New primal "ServiceMesh" provides service_registry
❌ BiomeOS still hardcoded to Songbird
❌ Cannot use new primal
❌ Need code changes to support
```

**New System:**
```
New primal "ServiceMesh" provides service_registry
✅ discover_primal_by_capability() finds both
✅ Can use either/both
✅ Automatic load balancing
✅ No code changes needed
```

### Scenario 3: Port Changes

**Old System:**
```
Songbird moves from port 3000 → 8080
❌ Hardcoded endpoint breaks
❌ Need configuration update
❌ Manual intervention required
```

**New System:**
```
Songbird moves from port 3000 → 8080
✅ Network scan discovers new port
✅ Or set DISCOVERY_ENDPOINT env var
✅ Clear error message if not found
✅ Self-healing possible
```

### Scenario 4: New Capabilities

**Old System:**
```
Toadstool adds GPU compute capability
❌ BiomeOS doesn't know about it
❌ Need to update code to use
❌ Manual capability tracking
```

**New System:**
```
Toadstool adds GPU compute capability
✅ verify_primal_capability() discovers it
✅ Can immediately use new feature
✅ Automatic capability query
✅ Works without updates
```

---

## 🔬 Testing Strategy

### For Each Showcase

1. **Functional Testing**
   - Does discovery work?
   - Does interface adaptation work?
   - Does the feature work end-to-end?

2. **Evolution Testing**
   - What if primal API changes?
   - What if primal moves ports?
   - What if alternate primal used?

3. **Degradation Testing**
   - What if primal unavailable?
   - Are error messages clear?
   - Does system continue gracefully?

4. **Gap Documentation**
   - What doesn't work?
   - Why doesn't it work?
   - How to fix it?

### Gap Reports

Each showcase generates a gap report:
```
gaps/
├── songbird-capability-discovery-gaps.md
├── compute-capability-gaps.md
├── encryption-capability-gaps.md
└── ...
```

These document:
- What works
- What doesn't work
- Why it doesn't work
- How to fix it
- Evolution scenarios tested

---

## 🚀 Running the Showcases

### Prerequisites

1. **Primal binaries** in one of:
   - `../../../../primalBins/`
   - `../../../phase1bins/`
   - System PATH

2. **Required tools:**
   - `curl`
   - `jq`
   - `bash 4.0+`

3. **Available ports:**
   - Check with common library functions

### Run Individual Showcase

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/showcase/01-single-primal

# Run enhanced Songbird demo
./songbird-discovery-enhanced.sh

# Run enhanced Toadstool demo
./toadstool-compute-enhanced.sh

# (More as completed)
```

### Run All Single-Primal Showcases

```bash
./run-all-enhanced.sh  # Coming soon
```

---

## 📊 Success Metrics

### For BiomeOS

- ✅ Works with primals it's never seen
- ✅ Adapts to primal API changes
- ✅ Continues when primals unavailable
- ✅ No recompilation for primal updates
- ✅ Clear error messages for operators

### For Primal Ecosystem

- ✅ Primals can evolve independently
- ✅ New primals work automatically
- ✅ No coordination needed for changes
- ✅ Each primal controls its own interface
- ✅ Sovereignty preserved

---

## 🎯 Next Steps

1. **Complete remaining single-primal showcases** (BearDog, Nestgate, Squirrel)
2. **Create cross-primal showcases** (Songbird + Toadstool, etc.)
3. **Test with real primal evolution** (different versions)
4. **Document all gaps** found during testing
5. **Update BiomeOS core** based on findings

---

## 💡 Key Insights

1. **Capability > Identity**
   - What a primal DOES matters, not what it's CALLED

2. **Runtime > Compile-time**
   - Discover at runtime, never hardcode at compile-time

3. **Adaptation > Assumption**
   - Probe and adapt, never assume interface

4. **Degradation > Failure**
   - Continue gracefully, never fail hard

5. **Sovereignty > Dependency**
   - Primals control themselves, BiomeOS adapts to them

---

**Status**: Deep debt solutions applied ✅  
**Philosophy**: Capability-based, evolution-resilient, sovereign ✅  
**Ready for**: Cross-primal workflows and ecosystem demos ✅

