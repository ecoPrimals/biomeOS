# 03 - One-Touch Niche Deployment

**Duration**: 2 minutes  
**Audience**: Human users AND Agentic AI  
**Complexity**: Zero-touch deployment

---

## Overview

This demo shows how biomeOS makes deploying **established niches** trivially easy.

**What is a Niche?**
A niche is a pre-configured composition of primals that solves a specific use case.

**Examples**:
- `secure-storage` = NestGate + BearDog + Songbird
- `sovereign-compute` = Toadstool + Songbird + NestGate
- `research-federation` = Songbird + NestGate + BearDog + Toadstool

---

## The Power of Niches

### Traditional Approach (Complex)
```bash
# ❌ Manual deployment (dozens of steps)
1. Install NestGate
2. Configure JWT secrets
3. Install Songbird
4. Configure TLS certificates
5. Install BearDog
6. Generate lineage keys
7. Configure all primals to talk to each other
8. Set up storage backends
9. Configure federation
10. Test everything
... (30+ more steps) ...

Time: 2-4 hours
Errors: Common
Success rate: ~60%
```

### BiomeOS Approach (One-Touch)
```bash
# ✅ One-touch deployment
biomeOS deploy --niche secure-storage

Time: 30 seconds
Errors: Rare (auto-recovery)
Success rate: 99%+
```

---

## Run the Demo

### For Humans
```bash
cd showcase/00-substrate/03-niche-deployment
./demo.sh
```

### For AI Agents
```bash
# JSON API
curl -X POST http://localhost:PORT/biomeOS/deploy \
  -H "Content-Type: application/json" \
  -d '{"niche": "secure-storage", "mode": "auto"}'

# Or programmatic
biomeOS deploy --niche secure-storage --json
```

---

## What You'll See

### Phase 1: Niche Selection
```
🌱 BiomeOS Niche Deployment

Available Niches:
  1. secure-storage      ← Encrypted storage with federation
  2. sovereign-compute   ← Privacy-preserving computation
  3. research-lab        ← Multi-user research environment
  4. development-tower   ← Full development stack

Select niche (or 'auto' for recommendation): 1
```

### Phase 2: Auto-Discovery
```
🔍 Discovering environment...
✅ Detected: Linux x86_64, 16GB RAM, 500GB disk
✅ Network: Connected, no NAT restrictions
✅ Already running: NestGate (port 9020)

📋 Niche Requirements:
  Required: storage, encryption
  Optional: orchestration
  
✅ All requirements satisfied!
```

### Phase 3: One-Touch Deploy
```
🚀 Deploying niche: secure-storage

[1/3] Validating primals...
  ✓ NestGate: Already running (9020)
  ✓ BearDog: Available (CLI)
  ✓ Songbird: Available (will start)

[2/3] Configuring niche...
  ✓ JWT secrets: Generated
  ✓ Lineage keys: Generated
  ✓ Storage backend: Configured (ZFS)
  ✓ Federation: Enabled

[3/3] Starting services...
  ✓ Songbird: Started (mDNS discovery active)
  ✓ Federation: Connected to 1 peer

✅ Niche deployed successfully!

Access your niche:
  Storage: http://localhost:9020
  Federation: https://localhost:8080
  Status: biomeOS status
```

### Phase 4: Verification
```
🔍 Verifying deployment...

Primal Status:
  ✅ NestGate: Healthy (storage ready)
  ✅ Songbird: Healthy (federation active)
  ✅ BearDog: Ready (encryption available)

Niche Capabilities:
  ✅ Store data with encryption
  ✅ Federated replication
  ✅ Lineage-based access control
  ✅ Automatic snapshots
  ✅ Privacy-preserving sharing

📊 Resource Usage:
  CPU: 2% (idle)
  Memory: 156MB / 16GB
  Disk: 2GB / 500GB
  Network: 12KB/s (discovery)

🎉 Your niche is ready to use!
```

---

## Niche Manifests

### Example: secure-storage.niche.yaml
```yaml
name: secure-storage
version: 1.0.0
description: Encrypted storage with federation

# Capabilities required
requires:
  - capability: storage
    provider: nestgate
    config:
      jwt_auth: true
      backend: zfs
      
  - capability: encryption
    provider: beardog
    config:
      mode: birdsong
      lineage: auto-generate
      
  - capability: orchestration
    provider: songbird
    optional: true
    config:
      federation: true
      discovery: mdns

# Auto-configuration
auto_config:
  jwt_secret: generate
  tls_certs: self-signed
  storage_pool: auto-detect
  lineage_keys: generate
  
# Resources
resources:
  min_memory: 512MB
  min_disk: 10GB
  recommended_memory: 2GB
  recommended_disk: 100GB

# Health checks
health:
  - endpoint: storage
    method: GET
    path: /health
    
  - process: songbird-orchestrator
    required: false
    
# Usage examples
examples:
  - name: Store encrypted file
    command: |
      biomeOS store --file mydata.txt --encrypt
      
  - name: List stored files
    command: |
      biomeOS list --storage
      
  - name: Replicate to federation
    command: |
      biomeOS replicate --file mydata.txt --towers 3
```

---

## AI Agent Integration

### Simple API
```python
# Python example for agentic AI
from biomeos import BiomeOS

# Initialize
bio = BiomeOS()

# One-line deployment
niche = bio.deploy_niche("secure-storage")

# Use the niche
niche.store(data="sensitive research", encrypt=True)
niche.replicate(towers=3)

# That's it!
```

### JSON API
```json
POST /biomeOS/api/v1/niche/deploy
{
  "niche": "secure-storage",
  "auto_config": true,
  "options": {
    "federation": true,
    "encryption": "birdsong"
  }
}

Response:
{
  "status": "deployed",
  "niche_id": "secure-storage-abc123",
  "endpoints": {
    "storage": "http://localhost:9020",
    "federation": "https://localhost:8080"
  },
  "capabilities": [
    "encrypted_storage",
    "federated_replication",
    "lineage_access_control"
  ]
}
```

### CLI for Humans
```bash
# Interactive
biomeOS deploy

# One-liner
biomeOS deploy --niche secure-storage

# With options
biomeOS deploy \
  --niche secure-storage \
  --federation \
  --auto-config

# Check status
biomeOS status

# Stop niche
biomeOS stop secure-storage
```

---

## Built-in Niches

### 1. secure-storage
**Use case**: Encrypted file storage with federation  
**Primals**: NestGate + BearDog + Songbird  
**Setup time**: 30 seconds

### 2. sovereign-compute
**Use case**: Privacy-preserving computation  
**Primals**: Toadstool + BearDog + NestGate + Songbird  
**Setup time**: 1 minute

### 3. research-lab
**Use case**: Multi-user research environment  
**Primals**: All primals + federation  
**Setup time**: 2 minutes

### 4. development-tower
**Use case**: Full development stack  
**Primals**: All primals + monitoring  
**Setup time**: 2 minutes

### 5. minimal-storage
**Use case**: Just storage, no encryption  
**Primals**: NestGate only  
**Setup time**: 10 seconds

---

## Key Features

### For Humans
- 🎯 **One command** to deploy complex systems
- 🤖 **Auto-configuration** (no manual setup)
- 📊 **Status dashboard** (see everything at a glance)
- 🔧 **Zero maintenance** (auto-updates, auto-recovery)

### For AI Agents
- 🔌 **Simple API** (REST + JSON)
- 📝 **Declarative** (describe what you want)
- 🎯 **Idempotent** (safe to retry)
- 📊 **Structured output** (easy to parse)

---

## Architecture

```
┌─────────────────────────────────────────┐
│       User or AI Agent                   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│      biomeOS Niche Deployer             │
│                                         │
│  1. Parse niche manifest                │
│  2. Discover available primals          │
│  3. Auto-configure requirements         │
│  4. Deploy & validate                   │
└─────────────────────────────────────────┘
         │           │          │
         ▼           ▼          ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │NestGate│  │Songbird│  │BearDog │
    │ Config │  │ Config │  │ Config │
    │  Auto  │  │  Auto  │  │  Auto  │
    └────────┘  └────────┘  └────────┘
```

---

## Success Criteria

✅ **Zero-touch**: No manual configuration required  
✅ **Fast**: < 60 seconds for any niche  
✅ **Reliable**: 99%+ success rate  
✅ **Universal**: Works for humans AND AI agents  
✅ **Declarative**: Describe what, not how  

---

## Real-World Scenarios

### Scenario 1: Researcher Needs Storage
```bash
# Researcher (human)
$ biomeOS deploy --niche secure-storage
✅ Deployed in 30 seconds
$ biomeOS store --file experiment_data.csv --encrypt
✅ Stored with lineage-based encryption

# Takes 30 seconds vs 2 hours manual setup
```

### Scenario 2: AI Agent Provisions Compute
```python
# AI agent (autonomous)
bio = BiomeOS()
niche = bio.deploy_niche("sovereign-compute")
result = niche.compute(
    code="train_model.py",
    data="dataset.npz",
    privacy="lineage-protected"
)
# AI agent provisions, runs, cleans up - zero human intervention
```

### Scenario 3: Lab Sets Up Research Environment
```bash
# Lab admin (human)
$ biomeOS deploy --niche research-lab --users 10
✅ Multi-user environment ready
✅ All 10 users can access
✅ Federated with 2 other towers
✅ Encrypted storage for all

# From zero to production in 2 minutes
```

---

## Technical Details

### Niche Deployment Flow
```
1. Parse manifest
   ↓
2. Discover environment (OS, resources, network)
   ↓
3. Check primal availability
   ↓
4. Auto-configure (secrets, certs, backends)
   ↓
5. Start required primals
   ↓
6. Validate health
   ↓
7. Return endpoints & status
```

### Auto-Configuration Magic
```yaml
# Manifest says: jwt_auth: true
# BiomeOS does:
1. Generate secure JWT secret (openssl rand -base64 48)
2. Set NESTGATE_JWT_SECRET environment variable
3. Start NestGate with JWT enabled
4. Store secret for future use

# User does: NOTHING
```

---

## Next Steps

After this demo:
- **04-federation**: Multi-tower niche deployment
- **05-custom-niches**: Create your own niche definitions
- **06-ai-agent-examples**: AI agents using biomeOS

---

**Philosophy**: *"Deploy once, use everywhere. Works for humans, works for AI. This is zero-touch sovereignty."*

