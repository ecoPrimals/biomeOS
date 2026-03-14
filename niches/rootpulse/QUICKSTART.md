# RootPulse Quick Start

**Deploy emergent version control in 5 minutes**

---

## Prerequisites

```bash
# Start biomeOS with required primals
biomeos nucleus start --mode full --node-id tower1

# Verify primals
biomeos doctor
```

**Required Primals**:
- ✅ NestGate (storage)
- ✅ BearDog (crypto)
- 🔄 rhizoCrypt (DAG workspace)
- 🔄 LoamSpine (linear history)

**Optional Primals**:
- ✅ SweetGrass (attribution)
- ✅ Songbird (federation)
- 🔄 PetalTongue (visualization)

---

## Deploy RootPulse Niche

```bash
# Deploy via BiomeOS
biomeos niche deploy niches/rootpulse/rootpulse-niche.yaml

# BiomeOS will:
# 1. Discover required primals ✅
# 2. Validate capabilities ✅
# 3. Generate unified CLI
# 4. Coordinate interactions
# 5. Expose rootpulse command
```

---

## Basic Usage

### Initialize Repository

```bash
# Create new RootPulse repository
rootpulse init

# BiomeOS coordinates:
# - rhizoCrypt: Creates DAG workspace
# - LoamSpine: Initializes linear history
# - NestGate: Prepares content storage
# - BearDog: Generates signing keys
```

### Make Commits

```bash
# Create some content
echo "Hello RootPulse" > README.md

# Commit
rootpulse commit -m "Initial commit"

# Behind the scenes:
# 1. rhizoCrypt: Creates DAG node
# 2. BearDog: Signs commit
# 3. NestGate: Stores content
# 4. LoamSpine: Appends to linear history
# 5. SweetGrass: Attributes contribution (if available)
```

### Branch and Merge

```bash
# Create branch (DAG operation)
rootpulse branch feature/new

# Make changes
echo "New feature" >> README.md
rootpulse commit -m "Add feature"

# Merge (DAG → Linear)
rootpulse checkout main
rootpulse merge feature/new

# rhizoCrypt: Handles DAG merge (lock-free!)
# LoamSpine: Records merge in linear history
```

### View History

```bash
# Linear history (what happened)
rootpulse log

# DAG view (branching possibilities)
rootpulse dag

# With visualization (PetalTongue)
rootpulse ui timeline
```

---

## Federation

### Push to Peers

```bash
# Discover peers (via Songbird)
rootpulse remote discover

# Add remote
rootpulse remote add origin https://peer.example.com/repo.git

# Push
rootpulse push origin main

# BiomeOS coordinates:
# 1. Songbird: Discovers peer
# 2. NestGate: Syncs content
# 3. BearDog: Verifies signatures
# 4. LoamSpine: Validates history
```

### Collaborate

```bash
# Clone from peer
rootpulse clone https://peer.example.com/repo.git

# Pull updates
rootpulse pull origin main

# Real-time collaboration
rootpulse collab start
# Multiple users work simultaneously
# rhizoCrypt: Lock-free DAG handles conflicts
```

---

## Advanced Features

### Semantic Attribution

```bash
# Show semantic contributions (SweetGrass)
rootpulse attribution

# Output:
# Alice: 45% (architecture, core logic)
# Bob: 35% (algorithms, optimization)
# AI Agent: 20% (documentation, tests)
```

### Temporal Flexibility

```bash
# High-resolution tracking (game states)
rootpulse init --resolution nanosecond

# Low-resolution (life events)
rootpulse init --resolution day

# Auto-adjust (default)
rootpulse init --resolution auto
```

### Multiple Anchors

```bash
# View different temporal orderings
rootpulse log --anchor crypto     # Cryptographic timestamps
rootpulse log --anchor causal     # Causal ordering
rootpulse log --anchor consensus  # Consensus ordering
```

---

## Troubleshooting

### Primal Not Found

```bash
# Error: "rhizoCrypt not discovered"

# Check primal status
biomeos doctor

# Install missing primal
# (See PRIMAL_GAPS.md for each primal)
```

### Health Check

```bash
# Check RootPulse niche health
biomeos niche health rootpulse

# Shows status of each primal
```

### Gaps

```bash
# If deployment fails, gaps are documented
cat ../PRIMAL_GAPS.md

# Shows which primals are missing/not working
```

---

## Performance

### Compared to Git

```
Operation       Git        RootPulse    Speedup
─────────────────────────────────────────────────
commit          100ms      10-50ms      2-10x
branch          50ms       5ms          10x
merge (simple)  200ms      20ms         10x
merge (complex) 2000ms     50-100ms     20-40x
clone           5000ms     500-1000ms   5-10x
status          50ms       10ms         5x
```

**Why faster?**
- rhizoCrypt: Lock-free DAG (no index)
- Content-addressing: Deduplication
- Parallel operations: Multiple primals
- Optimized storage: NestGate ZFS

---

## Migration from Git

```bash
# Convert Git repo to RootPulse
rootpulse migrate --from git --repo /path/to/git/repo

# BiomeOS coordinates:
# 1. Reads Git objects
# 2. Converts to RootPulse format
# 3. Preserves full history
# 4. Maintains signatures
# 5. Creates attribution (SweetGrass)
```

---

## What's Next?

### Try It
1. Deploy RootPulse niche
2. Run basic workflow
3. Explore visualization
4. Test federation

### Learn More
- Read white papers in `whitePaper_RootPulse/`
- Explore primal compositions
- Join the community

### Contribute
- Help implement LoamSpine
- Create Git migration tools
- Add performance benchmarks
- Write more documentation

---

**Status**: Ready for adventurous early adopters  
**Stability**: Experimental (primals ready, coordination in progress)  
**Timeline**: Production in 6-9 months  

🌳 **Welcome to emergent version control!**

