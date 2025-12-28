# 05 - Custom Primals: User-Defined Capabilities

**Duration**: 5 minutes  
**Prerequisites**: Understanding of BiomeOS discovery

---

## Overview

This demo shows how BiomeOS **automatically discovers and integrates custom user-defined primals** without code changes.

**What it demonstrates**:
- Zero code changes in BiomeOS
- Automatic capability discovery
- User sovereignty over primal composition
- Evolution-proof architecture

---

## The Power of User Sovereignty

### Traditional Approach (Hardcoded Services)
```rust
// ❌ Hardcoded service names
match service_name {
    "nestgate" => handle_storage(),
    "beardog" => handle_encryption(),
    "songbird" => handle_orchestration(),
    // New service? CODE CHANGE REQUIRED!
}
```

### BiomeOS Approach (Capability Discovery)
```rust
// ✅ Capability-based discovery
let storage = discover_capability("storage");
let encryption = discover_capability("encryption");
let orchestration = discover_capability("orchestration");

// New primal? DISCOVERED AUTOMATICALLY!
// No code changes, no recompilation, no deployment
```

---

## Creating a Custom Primal

### Example: MyStorage - Personal Storage Primal

```rust
// my-storage/src/main.rs
use axum::{Router, Json};
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/capability", get(capability));
    
    axum::Server::bind(&"0.0.0.0:9999".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "MyStorage",
        "version": "1.0.0"
    }))
}

async fn capability() -> Json<Value> {
    Json(json!({
        "category": "storage",
        "name": "mystorage",
        "api_type": "REST",
        "endpoints": {
            "store": "/api/store",
            "retrieve": "/api/retrieve",
            "list": "/api/list"
        }
    }))
}
```

**That's it!** BiomeOS will discover and integrate automatically.

---

## Run the Demo

```bash
cd showcase/00-substrate/05-custom-primals
./demo.sh
```

This will:
1. Create a simple custom primal
2. Start it alongside existing primals
3. Show automatic discovery
4. Demonstrate integration

---

## What You'll See

### Phase 1: Creating Custom Primal
```
🔧 Creating custom primal: MyAnalytics

Generating primal binary...
  • Capability: analytics
  • API: REST
  • Port: 9999

✅ Custom primal created successfully!
```

### Phase 2: Discovery
```
🔍 Running discovery...

Discovered Primals (6):
  ✅ NestGate (storage) - http://localhost:9020
  ✅ BearDog (encryption) - CLI
  ✅ Toadstool (compute) - CLI
  ✅ Songbird (orchestration) - mDNS
  ✅ Squirrel (configuration) - CLI
  ✅ MyAnalytics (analytics) - http://localhost:9999  ← NEW!

BiomeOS discovered custom primal automatically!
No code changes. No configuration. Just works.
```

### Phase 3: Integration
```
🔄 Testing integration with custom primal...

Query: "analytics" capability
Result: MyAnalytics (http://localhost:9999)

Calling /api/analyze endpoint...
Response: {
  "analysis_id": "a1b2c3",
  "status": "complete",
  "insights": ["trend_detected", "anomaly_found"]
}

✅ Custom primal fully integrated!
```

---

## Architecture

```
BiomeOS Discovery Engine
         │
         ├─ Scan for binaries
         │  └─ primals/*
         │
         ├─ Check HTTP endpoints
         │  └─ localhost:8000-10000
         │
         ├─ Test mDNS
         │  └─ _biome._tcp.local
         │
         └─ Query capabilities
            └─ GET /capability

            ↓

    Discovered Primals
         │
         ├─ Built-in (NestGate, BearDog, etc.)
         ├─ Community (from primal registry)
         └─ Custom (user-defined) ← YOU!
```

---

## Custom Primal Requirements

### Minimal Requirements (REST API)
```yaml
Required:
  - HTTP endpoint (any port 8000-10000)
  - /health endpoint (returns {"status": "healthy"})
  - /capability endpoint (returns capability metadata)

Optional:
  - /api/* (your custom functionality)
  - Authentication (JWT, mTLS, etc.)
  - Federation support
```

### Minimal Requirements (CLI Tool)
```yaml
Required:
  - Executable binary in primals/
  - --version flag (returns version)
  - --capability flag (returns capability metadata)

Optional:
  - Subcommands (store, retrieve, etc.)
  - Configuration files
  - Logging
```

### Minimal Requirements (mDNS Service)
```yaml
Required:
  - mDNS broadcast (_biome._tcp.local)
  - Service metadata in TXT record
  - HTTPS endpoint for API

Optional:
  - UDP broadcast
  - Peer discovery
  - Federation
```

---

## Use Cases

### Use Case 1: Personal Analytics Primal
```bash
# Create analytics primal for your data
cargo new my-analytics
# Implement capability discovery
# Deploy to primals/

# BiomeOS discovers automatically:
biomeOS discover
# Output: my-analytics (analytics) - discovered!
```

### Use Case 2: Company-Specific Primal
```bash
# Internal "acme-processor" primal
# Handles company-specific workflows
# Deploy alongside standard primals

# No BiomeOS code changes needed:
biomeOS deploy --primal acme-processor
# Works immediately with federation!
```

### Use Case 3: Research Domain Primal
```bash
# "genome-analyzer" for bioinformatics
# Specialized capability not in core primals
# Integrate with NestGate storage + Toadstool compute

# Compose capabilities:
biomeOS compose \
  --storage nestgate \
  --compute toadstool \
  --analysis genome-analyzer
```

---

## Evolution-Proof Architecture

### Scenario: New Primal Released
```
Year 2026: "QuantumCrypt" primal released
  - Capability: post-quantum encryption
  - API: REST + gRPC

BiomeOS response:
  1. Discovery scans find QuantumCrypt
  2. Capability query returns metadata
  3. Integration happens automatically
  4. Users can now use "encryption" from QuantumCrypt OR BearDog

NO CODE CHANGES IN BIOMEOS!
```

### Scenario: Primal Evolution
```
BearDog v0.9 → v2.0
  - API changed from CLI to REST
  - New capabilities added

BiomeOS response:
  1. Discovery detects new API type
  2. Adapts interaction pattern
  3. Exposes new capabilities
  4. Old capabilities still work

NO CODE CHANGES IN BIOMEOS!
```

### Scenario: User Composition
```
User creates "SecureAnalytics" niche:
  - Storage: NestGate
  - Encryption: BearDog
  - Analytics: MyAnalytics (custom)
  - Orchestration: Songbird

BiomeOS response:
  1. Discovers all four primals
  2. Validates capabilities
  3. Composes into niche
  4. Deploys as single unit

NO CODE CHANGES IN BIOMEOS!
```

---

## Developer Experience

### Creating Your First Custom Primal

#### Step 1: Scaffold
```bash
# Use BiomeOS primal SDK
cargo new --bin my-primal
cd my-primal
cargo add biomeos-primal-sdk
```

#### Step 2: Implement
```rust
use biomeos_primal_sdk::prelude::*;

#[tokio::main]
async fn main() {
    Primal::new("my-primal")
        .capability("analytics")
        .version("1.0.0")
        .endpoint("/analyze", analyze_handler)
        .run()
        .await;
}

async fn analyze_handler(req: Request) -> Response {
    // Your custom logic here
    json!({ "result": "analysis complete" })
}
```

#### Step 3: Deploy
```bash
cargo build --release
cp target/release/my-primal ../biomeOS/primals/
```

#### Step 4: Discover
```bash
cd ../biomeOS
./showcase/common/discovery.sh
# Output: ✅ my-primal (analytics) discovered!
```

**Total time: 15 minutes**

---

## Community Primal Registry

### Publishing Your Primal
```bash
# Sign with your lineage
beardog sign --primal my-primal

# Publish to registry
biomeOS publish --primal my-primal \
  --description "Custom analytics for X" \
  --license MIT

# Now discoverable globally!
```

### Discovering Community Primals
```bash
# Search registry
biomeOS search --capability analytics

Output:
  1. my-analytics (you) - Custom analytics
  2. data-insights (alice) - ML analytics
  3. trend-detector (bob) - Time series

# Install and use
biomeOS install data-insights
# Automatically integrated!
```

---

## Security Considerations

### Trust Levels
```yaml
# Your custom primal
trust: full
reason: "You built it"
lineage: "Your genesis device"

# Community primal (unsigned)
trust: capability
reason: "Functional but unverified"
lineage: null

# Community primal (lineage-signed)
trust: identity
reason: "Author verified via lineage"
lineage: "alice@genesis-device-123"
```

### Sandboxing
```yaml
# BiomeOS can sandbox untrusted primals
sandbox:
  enabled: true
  allow_network: false  # Block internet
  allow_disk: limited   # Limited disk access
  allow_federation: no  # No federation
```

---

## Testing Your Custom Primal

### Test Discovery
```bash
# Ensure primal is discoverable
curl http://localhost:9999/health
# Expected: {"status": "healthy"}

curl http://localhost:9999/capability
# Expected: {"category": "analytics", ...}
```

### Test Integration
```bash
# Use BiomeOS discovery
./showcase/common/discovery.sh
# Should list your primal

# Use in composition
biomeOS compose \
  --storage nestgate \
  --analytics my-primal
# Should work seamlessly
```

---

## Success Criteria

✅ **Zero Code Changes**: BiomeOS discovers without modification  
✅ **Automatic Integration**: Works with existing primals  
✅ **Evolution Proof**: New versions discovered automatically  
✅ **User Sovereignty**: You control your primal stack  
✅ **Community Ready**: Can publish to registry  

---

## Real-World Examples

### Example 1: Academic Research
```
Professor creates "genome-analyzer":
  - Custom analysis for genomics
  - Uses NestGate for storage
  - Uses Toadstool for compute
  - Published to university primal registry

Students can:
  - Install with one command
  - Use immediately in research
  - Contribute improvements
```

### Example 2: Enterprise
```
Company creates "acme-workflow":
  - Internal business logic
  - Integrates with existing primals
  - Deployed on private federation

Benefits:
  - Sovereign infrastructure
  - No vendor lock-in
  - Custom functionality
```

### Example 3: Community Project
```
Open source "climate-model":
  - Community-maintained
  - Lineage-signed by maintainers
  - Published to public registry

Users can:
  - Trust via lineage verification
  - Contribute to codebase
  - Deploy on their towers
```

---

## Next Steps

After this demo:
- **../01-nestgate**: Deep dive into NestGate storage
- **../02-birdsong-p2p**: BirdSong P2P tunnels
- **../03-p2p-coordination**: Full P2P ecosystem

---

## Philosophy

> "BiomeOS doesn't decide what primals exist.  
>  Users decide. Developers decide.  
>  BiomeOS discovers reality, it doesn't impose it.  
>  As new primals evolve, no code changes required.  
>  This is sovereignty. This is the future."

---

**The Primal SDK makes it easy. The Discovery Engine makes it automatic. You make it yours.**

