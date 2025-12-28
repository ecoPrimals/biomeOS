# 🚀 START HERE - BiomeOS Quick Start Guide

**Welcome to BiomeOS** - The sovereignty-first ecosystem substrate!

This guide will get you from zero to running demos in **5 minutes**.

---

## 📋 Prerequisites

### Required
- **Rust**: 1.70+ (stable)
- **Git**: For cloning
- **Linux/macOS**: Primary development platforms

### Optional (for full showcase)
- **Real Primals**: NestGate, Songbird, BearDog, Toadstool, Squirrel
- **benchScale**: For multi-VM federation testing

---

## ⚡ Quick Start (5 Minutes)

### Step 1: Clone & Build (2 min)

```bash
# Clone the repository
git clone git@github.com:ecoPrimals/biomeOS.git
cd biomeOS

# Build BiomeOS (takes ~2 minutes first time)
cargo build --workspace --release
```

### Step 2: Run Tests (1 min)

```bash
# Run all tests (350+ should pass)
cargo test --workspace

# You should see:
# test result: ok. 350+ passed; 0 failed
```

### Step 3: Run First Demo (2 min)

```bash
# Start real primals (if available)
./deploy-real-primals.sh

# Run hello-biomeos demo
cd showcase/00-substrate/01-hello-biomeos
./demo.sh
```

**🎉 Congratulations! You're now running BiomeOS!**

---

## 🎭 Showcase Demos

### Available Demos (10/20 Complete)

#### 00-Substrate (Foundation) ✅
1. **01-hello-biomeos** - Runtime discovery basics
   ```bash
   cd showcase/00-substrate/01-hello-biomeos && ./demo.sh
   ```

2. **02-capability-composition** - Multi-primal workflows
   ```bash
   cd showcase/00-substrate/02-capability-composition && ./demo.sh
   ```

3. **03-niche-deployment** - One-touch deployment
   ```bash
   cd showcase/00-substrate/03-niche-deployment && ./demo.sh
   ```

4. **04-federation** - Multi-tower coordination
   ```bash
   cd showcase/00-substrate/04-federation && ./demo.sh
   ```

5. **05-custom-primals** - User-defined capabilities
   ```bash
   cd showcase/00-substrate/05-custom-primals && ./demo.sh
   ```

#### 01-NestGate (Sovereign Storage) ✅
1. **01-sovereign-storage** - JWT + Lineage auth
2. **02-zfs-snapshots** - Time-travel & ransomware protection
3. **03-lineage-collaboration** - Trust-based sharing
4. **04-federation-replication** - Geographic DR
5. **05-benchscale-validation** - Scale testing

#### 02-BirdSong P2P (Coming Next)
- P2P tunnel establishment
- BearDog encryption integration
- BTSP coordination
- Multi-hop routing
- Full ecosystem integration

---

## 🛠️ Common Tasks

### Deploy Real Primals

```bash
# Deploy all available primals
./deploy-real-primals.sh

# Check status
ps aux | grep -E "(nestgate|songbird|beardog)"

# View logs
tail -f logs/primals/nestgate.log
tail -f logs/primals/songbird.log
```

### Run Specific Tests

```bash
# Unit tests only
cargo test --workspace --lib

# Integration tests
cargo test -p biomeos-core --test discovery_integration

# Specific crate
cargo test -p biomeos-types
cargo test -p biomeos-core
```

### Discover Primals

```bash
# Use common discovery utilities
source showcase/common/discovery.sh

# Discover by capability
discover_capability "storage"     # Finds NestGate
discover_capability "encryption"  # Finds BearDog
discover_capability "orchestration"  # Finds Songbird

# Discover all
discover_all
```

### Stop Everything

```bash
# Stop all primals
./stop-primals.sh

# Or kill specific processes
pkill -f nestgate
pkill -f songbird-orchestrator
```

---

## 📚 Documentation Index

### Quick Reference
- **[README.md](README.md)** - Project overview
- **[ROOT_INDEX.md](ROOT_INDEX.md)** - Complete doc index
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Command reference

### Showcases
- **[showcase/README.md](showcase/README.md)** - Showcase guide
- **[showcase/RUNTIME_DISCOVERY.md](showcase/RUNTIME_DISCOVERY.md)** - Discovery patterns
- **[showcase/PRIMAL_ARCHITECTURE_REALITY.md](showcase/PRIMAL_ARCHITECTURE_REALITY.md)** - Architecture principles

### Session Reports
- **[SESSION_FINAL_EPIC_DEC_28_2025.md](SESSION_FINAL_EPIC_DEC_28_2025.md)** - Latest achievements
- **[MILESTONE_50_PERCENT_DEC_28_2025.md](MILESTONE_50_PERCENT_DEC_28_2025.md)** - 50% milestone
- **[TESTING_MILESTONE_DEC_28_2025.md](TESTING_MILESTONE_DEC_28_2025.md)** - Testing complete

### Technical Docs
- **[docs/architecture/](docs/architecture/)** - Architecture specs
- **[docs/guides/](docs/guides/)** - How-to guides
- **[specs/](specs/)** - Technical specifications

---

## 🐛 Troubleshooting

### Build Issues

**Problem**: Compilation fails
```bash
# Solution: Update Rust
rustup update stable

# Clear build cache
cargo clean
cargo build --workspace --release
```

**Problem**: Missing dependencies
```bash
# Solution: Install system dependencies (Ubuntu/Debian)
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
brew install openssl pkg-config
```

### Runtime Issues

**Problem**: Primals not found
```bash
# Solution: Check primals directory
ls -la primals/

# Deploy if missing
./deploy-real-primals.sh
```

**Problem**: Port already in use
```bash
# Solution: Stop existing processes
./stop-primals.sh

# Or find and kill specific process
lsof -i :9020  # Check NestGate port
kill -9 <PID>
```

**Problem**: Federation not working
```bash
# Solution: Start Songbird
./start-songbird.sh

# Check Songbird logs
tail -f logs/primals/songbird.log

# Verify mDNS discovery
grep "Discovered peer" logs/primals/songbird.log
```

### Demo Issues

**Problem**: Demo fails with "primal not found"
```bash
# Solution: Demo gracefully handles missing primals
# But to get full experience, deploy real primals:
./deploy-real-primals.sh

# Then re-run demo
cd showcase/00-substrate/01-hello-biomeos
./demo.sh
```

---

## 🎓 Learning Path

### 1. Understand Core Concepts (30 min)
- Read [README.md](README.md)
- Review [showcase/PRIMAL_ARCHITECTURE_REALITY.md](showcase/PRIMAL_ARCHITECTURE_REALITY.md)
- Watch demos run

### 2. Run All Substrate Demos (30 min)
```bash
cd showcase/00-substrate
for demo in 0*/; do
    cd "$demo"
    ./demo.sh
    cd ..
done
```

### 3. Explore NestGate Demos (30 min)
```bash
cd showcase/01-nestgate
for demo in 0*/; do
    cd "$demo"
    ./demo.sh
    cd ..
done
```

### 4. Read the Code (1-2 hours)
```bash
# Start with discovery system
less showcase/common/discovery.sh

# Then explore core crates
less crates/biomeos-core/src/primal_adapter/mod.rs
less crates/biomeos-types/src/lib.rs

# Check tests
less crates/biomeos-core/tests/discovery_integration.rs
```

### 5. Write Your Own Primal (2-4 hours)
- Follow [showcase/00-substrate/05-custom-primals/README.md](showcase/00-substrate/05-custom-primals/README.md)
- Use `biomeos-primal-sdk` crate
- Test with discovery system

---

## 🚀 What's Next?

### If You're a User
1. Run all 10 demos
2. Explore real primal binaries
3. Try niche deployment
4. Join the community

### If You're a Developer
1. Review the codebase
2. Run all tests
3. Pick an open TODO
4. Submit a PR

### If You're Curious
1. Read session reports
2. Understand philosophy
3. Watch demos
4. Ask questions

---

## 🤝 Getting Help

### Documentation
- Full docs: [docs/](docs/)
- Architecture: [docs/architecture/](docs/architecture/)
- Guides: [docs/guides/](docs/guides/)

### Community
- GitHub Issues: Report bugs, request features
- GitHub Discussions: Ask questions, share ideas
- Pull Requests: Contribute code

### Quick Questions
- Check [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
- Search existing docs
- Look at showcase demos

---

## 🎯 Success Criteria

You're ready to move forward when you can:

✅ Build BiomeOS from source  
✅ Run all tests (350+ passing)  
✅ Deploy real primals  
✅ Run showcase demos  
✅ Understand runtime discovery  
✅ Explain core philosophy  

**Congratulations!** You're now a BiomeOS user! 🎉

---

## 🌟 Philosophy Reminder

> **"BiomeOS discovers reality, doesn't impose it.  
>   Primals are sovereign. Discovery is runtime.  
>   No hardcoding. No vendors. No compromise."**

**You're part of the future of sovereign computing!** 🚀🌱

---

**Next Steps**:
- Explore [showcase/README.md](showcase/README.md)
- Read [SESSION_FINAL_EPIC_DEC_28_2025.md](SESSION_FINAL_EPIC_DEC_28_2025.md)
- Join the ecoPrimals ecosystem
