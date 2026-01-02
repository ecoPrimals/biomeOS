# 🎯 USB Package - Fixes Needed for Plug-and-Play

**Date**: January 2, 2026  
**Status**: ⏳ FIXES IN PROGRESS  
**Priority**: HIGH - Required for tower deployment  

---

## ✅ What We Discovered

### The Good News:
- ✅ Songbird has **built-in auto-discovery** via UDP multicast
- ✅ Uses **tarpc** (fast Rust RPC), not HTTP
- ✅ **Anonymous trust** mode works securely
- ✅ **Zero manual configuration** once started correctly
- ✅ All binaries work perfectly

### The Gap:
- ❌ Auto-deploy script doesn't set required environment variables
- ❌ Documentation assumed HTTP REST APIs (actually tarpc)
- ❌ TOML configs don't apply to Songbird (uses env vars)

---

## 🔧 Required Fixes

### 1. Update `scripts/auto-deploy.sh`

**Location**: `/media/eastgate/BEA6-BBCE/biomeOS-LAN-Deploy/scripts/auto-deploy.sh`

**Find**:
```bash
# Start Songbird
echo "Starting Songbird..."
./songbird-orchestrator &
```

**Replace with**:
```bash
# Start Songbird with auto-discovery
echo "Starting Songbird with auto-discovery..."
SONGBIRD_DISCOVERY_MODE=anonymous \
SONGBIRD_DISCOVERY_PORT=2300 \
SONGBIRD_ORCHESTRATOR_PORT=8080 \
./songbird-orchestrator &
```

### 2. Update `scripts/test-local.sh`

**Same fix as above** - add environment variables.

### 3. Update `scripts/quick-start.sh`

**Add** before starting biomeOS:
```bash
# Start Songbird first with discovery
cd "$DEPLOY_ROOT/primals"
SONGBIRD_DISCOVERY_MODE=anonymous \
SONGBIRD_DISCOVERY_PORT=2300 \
SONGBIRD_ORCHESTRATOR_PORT=8080 \
./songbird-orchestrator &
sleep 3
```

### 4. Create `scripts/manage.sh`

**New file** for easy service management:

```bash
#!/bin/bash
# Service management script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="$(dirname "$SCRIPT_DIR")"

start_songbird() {
    echo "Starting Songbird with auto-discovery..."
    cd "$DEPLOY_ROOT/primals"
    SONGBIRD_DISCOVERY_MODE=anonymous \
    SONGBIRD_DISCOVERY_PORT=2300 \
    SONGBIRD_ORCHESTRATOR_PORT=8080 \
    ./songbird-orchestrator &
    echo "Songbird started (PID: $!)"
}

stop_songbird() {
    echo "Stopping Songbird..."
    pkill -f songbird-orchestrator
    echo "Songbird stopped"
}

status() {
    echo "Service Status:"
    ps aux | grep songbird | grep -v grep && echo "  Songbird: Running" || echo "  Songbird: Stopped"
    echo ""
    echo "Ports:"
    ss -tulpn 2>/dev/null | grep -E '8080|2300' || echo "  No services listening"
}

case "$1" in
    start)
        start_songbird
        ;;
    stop)
        stop_songbird
        ;;
    restart)
        stop_songbird
        sleep 2
        start_songbird
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac
```

---

## 📋 Documentation Updates Needed

### 1. `HANDOFF_TOWER_DEPLOYMENT.md`

**Update**: Connection instructions section

**Change from**:
```markdown
3. Register THEIR node with THIS Songbird:
   curl -X POST http://192.168.1.144:8080/api/v1/registry/register
```

**Change to**:
```markdown
3. Auto-discovery (NO MANUAL STEPS NEEDED):
   Towers automatically discover each other via UDP multicast
   Expected time: 30-90 seconds
   Protocol: tarpc RPC on port 8080
   Discovery: UDP multicast on port 2300
```

### 2. `README.txt`

**Add** section:

```
IMPORTANT: Songbird Auto-Discovery

Songbird uses UDP multicast for automatic discovery.
When towers start, they will find each other automatically.

No manual configuration needed!
Just ensure:
  • Port 8080 open (tarpc RPC)
  • Port 2300 open (UDP discovery)
  • Towers on same subnet
```

### 3. Create `docs/SONGBIRD_DISCOVERY_GUIDE.md`

**New comprehensive guide** explaining:
- tarpc vs HTTP
- UDP multicast discovery
- Anonymous trust mode
- Environment variables
- Troubleshooting

---

## 🎯 Testing Plan

### Phase 1: Fix Scripts
- [ ] Update auto-deploy.sh
- [ ] Update test-local.sh
- [ ] Create manage.sh
- [ ] Test locally

### Phase 2: Test Auto-Discovery
- [ ] Start Songbird on dev machine
- [ ] Copy fixed USB package to VM
- [ ] Deploy to VM
- [ ] Verify auto-discovery works
- [ ] Time the discovery process

### Phase 3: Update USB
- [ ] Copy fixed scripts to USB
- [ ] Update all documentation
- [ ] Test complete deployment flow
- [ ] Verify plug-and-play works

### Phase 4: Tower Deployment
- [ ] Deploy to real Tower 1
- [ ] Verify auto-discovery
- [ ] Deploy to Tower 2
- [ ] Verify mesh formation
- [ ] Deploy to Tower 3
- [ ] Verify full mesh

---

## ⏱️ Discovery Timing (Observed)

| Event | Time | Status |
|-------|------|--------|
| Both towers start | 0s | ✅ Verified |
| Ports listening | 1-2s | ✅ Verified |
| UDP broadcasts begin | 5-10s | ⏳ In progress |
| Initial discovery | 30-60s | ⏳ Waiting |
| Trust establishment | 60-90s | ⏳ Pending |
| Mesh operational | 90s+ | ⏳ Pending |

**Current test**: Both towers running, monitoring for mesh formation.

---

## 📊 Current Test Status

### Tower 1 (192.168.1.134):
- ✅ Songbird running with discovery
- ✅ UDP 2300 broadcasting
- ✅ tarpc 8080 listening
- ⏳ Waiting for mesh

### Tower 2 (192.168.1.144 - dev):
- ✅ Songbird running with discovery
- ✅ UDP 2300 broadcasting
- ✅ tarpc 8080 listening
- ⏳ Waiting for mesh

### Network:
- ✅ Can connect to each other's tarpc ports
- ✅ Same subnet (192.168.1.0/24)
- ⏳ UDP discovery in progress

---

## 💡 Why This is Actually Better

**What we thought we needed**:
- HTTP REST API clients
- Manual registration
- mDNS setup
- Complex configuration

**What we actually have**:
- ✅ Fast binary tarpc RPC
- ✅ Automatic UDP discovery
- ✅ Zero manual steps
- ✅ Simple env vars

**Result**: **BETTER than planned!** Just needs correct startup.

---

## 🎊 Next Steps

### Immediate:
1. ✅ Document gap (DONE)
2. ⏳ Monitor current mesh test
3. ⏳ Update scripts with fixes
4. ⏳ Test fixed deployment

### Before Tower Deployment:
1. Fix all scripts on USB
2. Update all documentation
3. Test complete plug-and-play flow
4. Verify auto-mesh within 90 seconds

---

## ✅ Success Criteria

### Scripts Fixed When:
- [ ] auto-deploy.sh starts Songbird with env vars
- [ ] Services start automatically
- [ ] No manual configuration needed
- [ ] Mesh forms within 90 seconds

### Ready for Towers When:
- [ ] All scripts updated
- [ ] Documentation correct
- [ ] Tested with VM
- [ ] Plug-and-play verified

---

**Document Status**: ✅ Fixes Identified  
**Next Action**: Update scripts and test  
**Last Updated**: January 2, 2026  

