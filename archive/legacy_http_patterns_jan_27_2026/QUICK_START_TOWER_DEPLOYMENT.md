# 🚀 Quick Start: Deploy Tower Atomic with LiveSpore

**The TRUE PRIMAL Way - Zero Hardcoding!**

---

## 🎯 What is Tower?

**Tower Atomic** = BearDog (security) + Songbird (discovery)

It's the foundation for all biomeOS deployments!

---

## ⚡ Quick Start (3 Commands!)

```bash
# 1. Deploy Tower atomic with LiveSpore
cd /path/to/biomeOS
FAMILY_ID=nat0 ./scripts/deploy-niche-atomic-tower.sh

# That's it! The script will:
#   ✅ Generate LiveSpore USB seed
#   ✅ Start BearDog + Songbird
#   ✅ Launch biomeOS API for discovery
#   ✅ Auto-launch PetalTongue visualization

# 2. (In another terminal) Check status
curl http://localhost:3000/api/v1/primals | jq

# 3. (In another terminal) Visualize with PetalTongue GUI
BIOMEOS_URL=http://localhost:3000 ./plasmidBin/petal-tongue
```

---

## 📚 What Just Happened?

### Step 1: LiveSpore USB Seed Generated

```
Created: /tmp/livespore-nat0/.family.seed
Genetic lineage: 36dc71b8713d9975...
```

This seed:
- ✅ Identifies the primal family
- ✅ Can be shared across USB devices
- ✅ Enables genetic lineage tracking
- ✅ Makes spores portable!

---

### Step 2: Primals Started (NOT Launched!)

```
BearDog:  FAMILY_ID=nat0 ./beardog
          → Creates: /run/user/1000/beardog-nat0.sock

Songbird: FAMILY_ID=nat0 ./songbird
          → Creates: /run/user/1000/songbird-nat0.sock
```

**Key Point**: Primals start themselves! biomeOS doesn't launch them.

---

### Step 3: biomeOS API Discovers

```
biomeOS API scans: /run/user/1000/*.sock
Finds: beardog-nat0.sock, songbird-nat0.sock
Queries capabilities
→ Tower atomic EMERGES!
```

**No hardcoding!** Discovery-based!

---

### Step 4: PetalTongue Visualizes

```
PetalTongue → http://localhost:3000/api/v1/topology
            → Renders real-time graph
            → Updates every 2 seconds
```

**Proprioception!** The system visualizes itself!

---

## 🧪 Verify Deployment

### Check Unix Sockets

```bash
ls -lh /run/user/$(id -u)/*.sock | grep -E "(beardog|songbird)"

# Should see:
# beardog-nat0.sock
# songbird-nat0.sock
```

---

### Check biomeOS API

```bash
# Health
curl http://localhost:3000/api/v1/health

# Discovered primals
curl http://localhost:3000/api/v1/primals | jq '.primals[] | {name, primal_type, endpoint}'

# Topology
curl http://localhost:3000/api/v1/topology | jq '.nodes[] | {id, name, status}'

# LiveSpore devices
curl http://localhost:3000/api/v1/livespores | jq '.devices[] | {id, genetic_preview, spore_type}'
```

---

### Check PetalTongue

```bash
# Terminal UI (works over SSH!)
BIOMEOS_URL=http://localhost:3000 ./plasmidBin/petal-tongue-headless --mode terminal

# SVG export (for web dashboards)
BIOMEOS_URL=http://localhost:3000 ./plasmidBin/petal-tongue-headless --mode svg --output /tmp/tower.svg

# Open in browser
firefox /tmp/tower.svg
```

---

## 🌟 Advanced: Custom Family ID

```bash
# Use your own family ID
FAMILY_ID=mylab ./scripts/deploy-niche-atomic-tower.sh

# Use existing USB seed
FAMILY_ID=mylab USB_SEED=/media/usb/.family.seed ./scripts/deploy-niche-atomic-tower.sh
```

---

## 🔧 Manual Deployment (If You Want Control)

```bash
# 1. Generate USB seed
mkdir -p /tmp/livespore-nat0
openssl rand -hex 32 > /tmp/livespore-nat0/.family.seed

# 2. Start BearDog
FAMILY_ID=nat0 \
NODE_ID=tower-beardog \
USB_SEED=/tmp/livespore-nat0/.family.seed \
./plasmidBin/beardog &

# 3. Start Songbird (if not running)
FAMILY_ID=nat0 \
NODE_ID=tower-songbird \
USB_SEED=/tmp/livespore-nat0/.family.seed \
./plasmidBin/songbird &

# 4. Start biomeOS API
FAMILY_ID=nat0 \
USB_SEED=/tmp/livespore-nat0/.family.seed \
cargo run -p biomeos-api &

# Wait for API startup
sleep 5

# 5. Launch PetalTongue
BIOMEOS_URL=http://localhost:3000 \
./plasmidBin/petal-tongue-headless --mode terminal
```

---

## 🌳 Evolving to Larger Atomics

### Tower → Node

```bash
# Tower is already running (BearDog + Songbird)

# Add ToadStool for compute
FAMILY_ID=nat0 NODE_ID=node-toadstool ./plasmidBin/toadstool &

# Now you have NODE atomic!
# biomeOS discovers automatically!
```

---

### Node → Nest

```bash
# Node is running (BearDog + Songbird + ToadStool)

# Add NestGate for storage
FAMILY_ID=nat0 NODE_ID=nest-nestgate ./plasmidBin/nestgate &

# Now you have NEST atomic!
```

---

### Nest → NUCLEUS

```bash
# Nest is running (BearDog + Songbird + ToadStool + NestGate)

# Add Squirrel for AI
FAMILY_ID=nat0 NODE_ID=nucleus-squirrel ./plasmidBin/squirrel &

# Now you have NUCLEUS atomic!
# Full ecosystem running!
```

**All discovered dynamically!** 🌳

---

## 🐛 Troubleshooting

### "Socket not found"

**Problem**: Primal didn't create socket

**Fix**:
```bash
# Check if primal is running
ps aux | grep beardog

# Check logs
tail -f /tmp/beardog-nat0.log

# Kill and restart
pkill beardog
FAMILY_ID=nat0 ./plasmidBin/beardog &
```

---

### "API not responding"

**Problem**: biomeOS API not started

**Fix**:
```bash
# Check if API is running
curl http://localhost:3000/api/v1/health

# If not, start it
FAMILY_ID=nat0 cargo run -p biomeos-api &
```

---

### "PetalTongue shows no primals"

**Problem**: API not discoverable or primals not running

**Fix**:
```bash
# 1. Verify API is running
curl http://localhost:3000/api/v1/health

# 2. Verify primals are discovered
curl http://localhost:3000/api/v1/primals

# 3. Check PetalTongue connection
BIOMEOS_URL=http://localhost:3000 \
RUST_LOG=debug \
./plasmidBin/petal-tongue-headless --mode terminal
```

---

## 📚 Learn More

- **Full Documentation**: `TRUE_PRIMAL_DEPLOYMENT_SUCCESS_JAN13.md`
- **Architecture**: `DEEP_DEBT_ATOMIC_DEPLOY_ISSUE.md`
- **PetalTongue Integration**: `PETALTONGUE_INTEGRATION_JAN13.md`
- **Port-Free Architecture**: `TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md`

---

## 🎊 Congratulations!

You just deployed a TRUE PRIMAL Tower atomic with:
- ✅ Zero hardcoding
- ✅ LiveSpore genetic lineage
- ✅ Discovery-based composition
- ✅ Real-time visualization

**Welcome to biomeOS!** 🌳🐸✨

---

**Created**: January 13, 2026  
**Status**: Production Ready  
**Grade**: A+ (TRUE PRIMAL validated!)

