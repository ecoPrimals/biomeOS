# biomeOS Deployment Guide

**Version**: 1.0.0 (Neural API)  
**Date**: January 15, 2026  
**Status**: Production Ready

---

## 🚀 Quick Start (Neural API - Recommended)

### Prerequisites

```bash
# Build all binaries
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo build --release --workspace

# Harvest primal binaries from other repos (if needed)
./scripts/harvest-primals.sh

# Verify binaries exist
ls -lh plasmidBin/primals/
```

### Deploy Full Ecosystem

```bash
# 1. Start Neural API server (deployment orchestrator)
./target/release/neural-api-server --graphs-dir graphs --family-id nat0 > /tmp/primals/neural-api.log 2>&1 &

# 2. Deploy BearDog first (security foundation - required by others)
./plasmidBin/primals/beardog-server > /tmp/primals/beardog.log 2>&1 &
sleep 2  # Wait for BearDog to initialize

# 3. Deploy NUCLEUS enclave (Songbird, ToadStool, NestGate)
./plasmidBin/primals/neural-deploy 01_nucleus_enclave --family-id nat0

# 4. Deploy Security & Intelligence layer (BearDog + Squirrel)
./plasmidBin/primals/neural-deploy 02_security_intelligence --family-id nat0

# 5. Deploy BenchTop UI (PetalTongue)
./plasmidBin/primals/neural-deploy 03_benchtop_ui --family-id nat0

# 6. Check status
ls -l /tmp/*.sock
ps aux | grep -E "(beardog|songbird|toadstool|nestgate|squirrel|petal-tongue)"
```

### Or Deploy Everything at Once

```bash
# Master graph - deploys full ecosystem
./plasmidBin/primals/neural-deploy 00_full_ecosystem --family-id nat0
```

---

## 📊 Verify Deployment

### Check Running Primals

```bash
# List all primal processes
ps aux | grep -E "(beardog|songbird|toadstool|nestgate|squirrel|petal)" | grep -v grep

# Expected output:
# beardog-server
# songbird-orchestrator
# toadstool or toadstool-server
# nestgate
# squirrel
# petal-tongue-headless (if UI deployed)
# neural-api-server
```

### Check Unix Sockets

```bash
# List all primal sockets
ls -l /tmp/*.sock

# Expected sockets:
# /tmp/beardog-*.sock
# /tmp/songbird-*.sock
# /tmp/toadstool-*.sock
# /tmp/nestgate-*.sock
# /tmp/squirrel-*.sock
# /tmp/neural-api-*.sock
```

### Check Logs

```bash
# View all primal logs
ls -lh /tmp/primals/

# Tail specific primal log
tail -f /tmp/primals/songbird-nat0.log

# View Neural API orchestration log
tail -f /tmp/primals/neural-api.log
```

### Health Check

```bash
# Quick health check (check if sockets exist and respond)
for socket in /tmp/*.sock; do
  echo "Checking $socket..."
  # Add your health check command here
done
```

---

## 🛑 Stop Ecosystem

### Graceful Shutdown

```bash
# Stop all primals and clean up sockets
./scripts/stop_ecosystem.sh
```

### Manual Cleanup

```bash
# Kill specific primal
pkill beardog-server

# Remove specific socket
rm /tmp/beardog-*.sock

# Clean all sockets
rm /tmp/*.sock
```

---

## 🏗️ Deployment Architecture

### Dependency Order

```
1. BearDog (security provider)
   ↓
2. Songbird (discovery + mesh coordination)
   ↓ (depends on BearDog for security)
   ↓
3. ToadStool (compute orchestration)
   ↓ (discovers Songbird)
   ↓
4. NestGate (storage + persistence)
   ↓ (requires BearDog + Songbird)
   ↓
5. Squirrel (meta-AI routing)
   ↓ (discovers all via Songbird)
   ↓
6. PetalTongue (UI)
   (discovers all for visualization)
```

### NUCLEUS Atomics

**NUCLEUS** = Network-Unified Cryptographic Lineage & Ecosystem Universal System

- **Tower** (Songbird) - Discovery & mesh coordination
- **Node** (ToadStool) - Compute orchestration
- **Nest** (NestGate) - Storage & persistence

Deploy as one unit for complete functionality.

---

## 📁 Deployment Graphs

Located in `graphs/` directory:

### Primary Graphs

**`01_nucleus_enclave.toml`**
- Deploys: Songbird, ToadStool, NestGate
- Dependencies: BearDog must be running first
- Use: Core NUCLEUS deployment

**`02_security_intelligence.toml`**
- Deploys: BearDog, Squirrel
- Dependencies: None (BearDog is foundation)
- Use: Security and AI layer

**`03_benchtop_ui.toml`**
- Deploys: PetalTongue (headless)
- Dependencies: All core primals
- Use: UI visualization layer

**`00_full_ecosystem.toml`**
- Deploys: Everything in correct order
- Dependencies: None (includes all)
- Use: Complete ecosystem deployment

### Advanced Graphs

- `nucleus_ecosystem.toml` - Alternate NUCLEUS deployment
- `genetic_lineage_full_nucleus.toml` - With genetic lineage verification
- `adaptive_tower_deploy.toml` - Dynamic tower deployment
- See `graphs/` for all available graphs

---

## 🔧 Configuration

### Environment Variables

```bash
# Family ID (genetic lineage identifier)
export FAMILY_ID=nat0

# Socket directory (default: /tmp)
export SOCKET_DIR=/tmp

# Log directory
export LOG_DIR=/tmp/primals

# BearDog configuration
export BEARDOG_FAMILY_ID=nat0
export BEARDOG_NODE_ID=default

# Songbird configuration
export SONGBIRD_FAMILY_ID=nat0
export SONGBIRD_SECURITY_PROVIDER=/tmp/beardog-default-default.sock
```

### Graph Configuration

Edit TOML files in `graphs/` to customize:
- Binary paths
- Socket paths
- Dependencies
- Startup timeouts
- Health check intervals

---

## 🐛 Troubleshooting

### Primal Won't Start

**Check logs**:
```bash
tail -50 /tmp/primals/<primal-name>-<family-id>.log
```

**Common issues**:
- Missing dependency (e.g., Songbird needs BearDog)
- Socket already in use (clean up old sockets)
- Binary not found (check `plasmidBin/primals/`)
- Permissions issue (check execute permissions)

### Socket Not Created

**Symptoms**: Deployment times out waiting for socket

**Solutions**:
1. Check primal log for errors
2. Verify binary has execute permissions
3. Check socket directory exists and is writable
4. Ensure no firewall blocking Unix sockets

### Dependency Errors

**Symptom**: "No security provider configured" or similar

**Solution**: Deploy in correct order (see Dependency Order above)

---

## 🔄 Alternative Deployment Methods

### Manual Deployment (For Testing)

```bash
# Start each primal manually
./plasmidBin/primals/beardog-server &
sleep 2

./plasmidBin/primals/songbird-orchestrator &
sleep 2

./plasmidBin/primals/toadstool &
sleep 2

./plasmidBin/primals/nestgate &
```

### Via Cargo (Development)

```bash
# Run biomeos orchestrator directly
cargo run --release --bin biomeos

# Run NUCLEUS server
cargo run --release --bin nucleus -- execute-graph --graph graphs/nucleus_full.toml --family nat0
```

---

## 📚 Documentation

- **Neural API Guide**: [NEURAL_API_FINAL_STATUS_JAN_15_2026.md](NEURAL_API_FINAL_STATUS_JAN_15_2026.md)
- **Latest Updates**: [LATEST_UPDATE_JAN_15_2026.md](LATEST_UPDATE_JAN_15_2026.md)
- **Architecture**: [README.md](README.md)
- **Scripts**: [scripts/README.md](scripts/README.md)

---

## 🌟 Production Checklist

Before deploying to production:

- [ ] All binaries built with `--release` flag
- [ ] Harvest latest primal binaries (`./scripts/harvest-primals.sh`)
- [ ] Verify family seed is secure and backed up
- [ ] Configure appropriate `FAMILY_ID`
- [ ] Set up proper log rotation
- [ ] Configure systemd services (optional)
- [ ] Test rollback procedure
- [ ] Monitor socket health
- [ ] Set up alerting for primal failures

---

**Deploy with confidence using the Neural API!** 🚀

For questions or issues, see comprehensive documentation in the root directory.

