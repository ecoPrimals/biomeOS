# 🚀 NUCLEUS Validation Plan - January 30, 2026

**Date:** January 30, 2026  
**Status:** Ready for Validation  
**Primals:** All 5 socket-standardized (100%)

---

## 🎯 **Mission Objective**

Validate all 3 NUCLEUS atomic patterns with the complete, socket-standardized ecosystem:

1. **Tower Atomic** (BearDog + Songbird) - Security + Network ✅ PREVIOUSLY VALIDATED
2. **Node Atomic** (Tower + Toadstool) - Add GPU Compute 🆕 READY FOR TEST
3. **Nest Atomic** (Tower + NestGate + Squirrel) - Add Storage + AI 🆕 READY FOR TEST
4. **Full NUCLEUS** - All 5 primals integrated 🆕 READY FOR TEST

---

## 📊 **Current Ecosystem State**

### **Socket Standardization Status: 100%**

```
✅ BearDog   - A++ (100/100) - /run/user/$UID/biomeos/beardog.sock
✅ Songbird  - A+            - /run/user/$UID/biomeos/songbird.sock
✅ Toadstool - A++           - /run/user/$UID/biomeos/toadstool.sock
✅ NestGate  - A+++ (110/100) - /run/user/$UID/biomeos/nestgate.sock
✅ Squirrel  - A+ (98/100)   - /run/user/$UID/biomeos/squirrel.sock
```

### **Recent Updates (January 30, 2026)**

**Toadstool** (Commit `279e1a3d` - 09:07 AM):
- ✅ Socket standardization complete (1.25h)
- ✅ 5-tier discovery pattern
- ✅ All primal paths updated
- ✅ barraCUDA 50 operations (+178% growth)
- ✅ Production panics: 84 → 0

**NestGate** (Commit `5bc0b0ea` - 10:09 AM):
- ✅ Socket-only mode ALREADY implemented (proactive!)
- ✅ `--socket-only` daemon flag ready
- ✅ No HTTP conflicts (port 8080 freed)
- ✅ No external dependencies required

**Squirrel** (Commit `b59500ef` - 10:10 AM):
- ✅ Socket standardization complete (3h - FASTEST!)
- ✅ Discovery helpers (FIRST in ecosystem!)
- ✅ 5-tier discovery pattern
- ✅ 505/505 tests passing

**BearDog** (Commit `eaedf55a0` - 09:19 AM):
- ✅ Deep debt complete (A++ 100/100)
- ✅ 5,010/5,010 tests passing
- ✅ Zero production panics
- ✅ Socket standardization included

---

## 🧪 **Validation Phases**

### **Phase 1: Tower Atomic (REVALIDATION)** ✅

**Primals**: BearDog + Songbird  
**Purpose**: Security + Network foundation  
**Previous Status**: Validated successfully Jan 29, 2026  
**Current Action**: Quick revalidation with latest versions

**Steps**:
1. Start BearDog with primal identity
2. Verify BearDog socket creation
3. Start Songbird with BearDog as security provider
4. Verify Songbird socket creation
5. Health check both primals
6. Test JSON-RPC communication
7. Verify security handshake

**Expected Sockets**:
- `/run/user/$(id -u)/biomeos/beardog.sock`
- `/run/user/$(id -u)/biomeos/songbird.sock`

**Success Criteria**:
- ✅ Both sockets created at standard paths
- ✅ Health checks return status within 500ms
- ✅ JSON-RPC communication works
- ✅ Security integration functional

---

### **Phase 2: Node Atomic (NEW TEST)** 🆕

**Primals**: Tower + Toadstool  
**Purpose**: Add GPU compute capabilities  
**Status**: READY FOR FIRST TEST

**Dependencies**:
- Tower Atomic must be operational
- Toadstool socket-standardized ✅ (Jan 30, 09:07 AM)

**Steps**:
1. Ensure Tower Atomic is running
2. Start Toadstool with primal identity
3. Verify Toadstool socket creation
4. Health check Toadstool
5. Test Toadstool discovery of Songbird
6. Test barraCUDA operations (if applicable)
7. Verify inter-primal communication

**Expected Sockets**:
- `/run/user/$(id -u)/biomeos/beardog.sock` (Tower)
- `/run/user/$(id -u)/biomeos/songbird.sock` (Tower)
- `/run/user/$(id -u)/biomeos/toadstool.sock` (NEW)

**Success Criteria**:
- ✅ Toadstool socket created at standard path
- ✅ Toadstool discovers Songbird successfully
- ✅ Health check returns compute capabilities
- ✅ No hardcoded paths used
- ✅ Runtime discovery works

**Key Validation**:
- Toadstool should discover Songbird at `/run/user/$UID/biomeos/songbird.sock`
- Should NOT attempt old paths like `/primal/songbird`

---

### **Phase 3: Nest Atomic (NEW TEST)** 🆕

**Primals**: Tower + NestGate + Squirrel  
**Purpose**: Add storage + AI orchestration  
**Status**: READY FOR FIRST TEST

**Dependencies**:
- Tower Atomic must be operational
- NestGate socket-only mode ✅ (Jan 30, 10:09 AM)
- Squirrel socket-standardized ✅ (Jan 30, 10:10 AM)

**Steps**:
1. Ensure Tower Atomic is running
2. Start NestGate with `--socket-only` flag
3. Verify NestGate socket creation
4. Health check NestGate
5. Start Squirrel with primal identity
6. Verify Squirrel socket creation
7. Health check Squirrel
8. Test discovery helpers (Squirrel innovation!)
9. Verify inter-primal communication

**Expected Sockets**:
- `/run/user/$(id -u)/biomeos/beardog.sock` (Tower)
- `/run/user/$(id -u)/biomeos/songbird.sock` (Tower)
- `/run/user/$(id -u)/biomeos/nestgate.sock` (NEW)
- `/run/user/$(id -u)/biomeos/squirrel.sock` (NEW)

**NestGate Configuration**:
```bash
# Required environment variables
FAMILY_ID=nat0
NODE_ID=nest1
NESTGATE_JWT_SECRET="$(openssl rand -base64 48)"
NESTGATE_SOCKET_ONLY=true  # OR use --socket-only flag

# Start command
nestgate daemon --socket-only
```

**Success Criteria**:
- ✅ NestGate socket created at standard path
- ✅ No HTTP port conflicts (port 8080 free)
- ✅ No external DB/Redis required in socket-only mode
- ✅ Squirrel socket created at standard path
- ✅ Squirrel discovery helpers work
- ✅ Health checks return capabilities
- ✅ Inter-primal discovery works

**Key Validation**:
- NestGate should start WITHOUT port 8080 binding
- Squirrel discovery helpers should find primals
- All runtime discovery (no hardcoding)

---

### **Phase 4: Full NUCLEUS (COMPLETE TEST)** 🚀

**Primals**: All 5 (BearDog + Songbird + Toadstool + NestGate + Squirrel)  
**Purpose**: Complete system integration  
**Status**: READY FOR FIRST FULL TEST

**Steps**:
1. Start all 5 primals in sequence
2. Verify all 5 sockets created
3. Health check all 5 primals
4. Test inter-primal discovery
5. Validate capability discovery
6. Test cross-primal operations
7. Monitor system stability

**Expected Sockets (ALL 5)**:
```
/run/user/$(id -u)/biomeos/beardog.sock   ✅
/run/user/$(id -u)/biomeos/songbird.sock  ✅
/run/user/$(id -u)/biomeos/toadstool.sock ✅
/run/user/$(id -u)/biomeos/nestgate.sock  ✅
/run/user/$(id -u)/biomeos/squirrel.sock  ✅
```

**Success Criteria**:
- ✅ All 5 sockets created at standard paths
- ✅ All health checks return within 1 second
- ✅ Runtime discovery works for all primals
- ✅ No hardcoded paths used anywhere
- ✅ No port conflicts
- ✅ All primals report "healthy" status
- ✅ System remains stable for 60+ seconds

---

## 📋 **Environment Configuration**

### **Common Variables (All Primals)**

```bash
export FAMILY_ID=nat0
export NODE_ID=nucleus1
export RUST_LOG=info
```

### **BearDog Specific**

```bash
# Primal identity (required)
export FAMILY_ID=nat0
export NODE_ID=nucleus1

# Logging
export RUST_LOG=beardog=info
```

### **Songbird Specific**

```bash
# Primal identity
export FAMILY_ID=nat0
export NODE_ID=nucleus1

# Security provider configuration
export SONGBIRD_SECURITY_PROVIDER=beardog
export BEARDOG_SOCKET=/run/user/$(id -u)/biomeos/beardog.sock

# Logging
export RUST_LOG=songbird=info
```

### **Toadstool Specific**

```bash
# Primal identity
export FAMILY_ID=nat0
export NODE_ID=nucleus1

# Logging
export RUST_LOG=toadstool=info
```

### **NestGate Specific**

```bash
# Primal identity
export FAMILY_ID=nat0
export NODE_ID=nucleus1

# Security (REQUIRED - must be unique!)
export NESTGATE_JWT_SECRET="$(openssl rand -base64 48)"

# Socket-only mode (RECOMMENDED for NUCLEUS testing)
export NESTGATE_SOCKET_ONLY=true
```

### **Squirrel Specific**

```bash
# Primal identity
export FAMILY_ID=nat0
export NODE_ID=nucleus1
```

---

## 🔧 **Startup Scripts**

### **Script 1: Tower Atomic**

```bash
#!/bin/bash
# Start Tower Atomic (BearDog + Songbird)

export FAMILY_ID=nat0
export NODE_ID=tower1

echo "🏗️ Starting Tower Atomic..."
echo ""

# 1. Start BearDog (Security Foundation)
echo "1️⃣ Starting BearDog..."
RUST_LOG=beardog=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    beardog server > /tmp/beardog_nucleus.log 2>&1 &
BEARDOG_PID=$!
echo "   PID: $BEARDOG_PID"

sleep 3

# 2. Start Songbird (Network + Discovery)
echo "2️⃣ Starting Songbird..."
RUST_LOG=songbird=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    SONGBIRD_SECURITY_PROVIDER=beardog \
    BEARDOG_SOCKET=/run/user/$(id -u)/biomeos/beardog.sock \
    songbird server > /tmp/songbird_nucleus.log 2>&1 &
SONGBIRD_PID=$!
echo "   PID: $SONGBIRD_PID"

sleep 3

# 3. Verify sockets
echo ""
echo "🔍 Verifying sockets..."
ls -lh /run/user/$(id -u)/biomeos/*.sock

# 4. Health checks
echo ""
echo "🏥 Health checks..."
echo "BearDog:"
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U /run/user/$(id -u)/biomeos/beardog.sock -w 2
echo ""
echo "Songbird:"
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U /run/user/$(id -u)/biomeos/songbird.sock -w 2

echo ""
echo "✅ Tower Atomic operational!"
echo "PIDs: BearDog=$BEARDOG_PID Songbird=$SONGBIRD_PID"
```

### **Script 2: Node Atomic**

```bash
#!/bin/bash
# Start Node Atomic (Tower + Toadstool)

export FAMILY_ID=nat0
export NODE_ID=node1

echo "🔬 Starting Node Atomic..."
echo ""

# 1. Ensure Tower Atomic is running
echo "✅ Assuming Tower Atomic is already running..."
echo ""

# 2. Start Toadstool (GPU Compute)
echo "3️⃣ Starting Toadstool..."
RUST_LOG=toadstool=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    toadstool server > /tmp/toadstool_nucleus.log 2>&1 &
TOADSTOOL_PID=$!
echo "   PID: $TOADSTOOL_PID"

sleep 5

# 3. Verify sockets
echo ""
echo "🔍 Verifying sockets..."
ls -lh /run/user/$(id -u)/biomeos/*.sock

# 4. Health check
echo ""
echo "🏥 Toadstool health check..."
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U /run/user/$(id -u)/biomeos/toadstool.sock -w 2

echo ""
echo "✅ Node Atomic operational!"
echo "PID: Toadstool=$TOADSTOOL_PID"
```

### **Script 3: Nest Atomic**

```bash
#!/bin/bash
# Start Nest Atomic (Tower + NestGate + Squirrel)

export FAMILY_ID=nat0
export NODE_ID=nest1
export JWT_SECRET="$(openssl rand -base64 48)"

echo "🏡 Starting Nest Atomic..."
echo ""

# 1. Ensure Tower Atomic is running
echo "✅ Assuming Tower Atomic is already running..."
echo ""

# 2. Start NestGate (Storage + Persistence)
echo "4️⃣ Starting NestGate (socket-only mode)..."
FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    NESTGATE_JWT_SECRET="$JWT_SECRET" \
    nestgate daemon --socket-only > /tmp/nestgate_nucleus.log 2>&1 &
NESTGATE_PID=$!
echo "   PID: $NESTGATE_PID"

sleep 3

# 3. Start Squirrel (AI Orchestration)
echo "5️⃣ Starting Squirrel..."
FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    squirrel server > /tmp/squirrel_nucleus.log 2>&1 &
SQUIRREL_PID=$!
echo "   PID: $SQUIRREL_PID"

sleep 3

# 4. Verify sockets
echo ""
echo "🔍 Verifying sockets..."
ls -lh /run/user/$(id -u)/biomeos/*.sock

# 5. Health checks
echo ""
echo "🏥 Health checks..."
echo "NestGate:"
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U /run/user/$(id -u)/biomeos/nestgate.sock -w 2
echo ""
echo "Squirrel:"
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U /run/user/$(id -u)/biomeos/squirrel.sock -w 2

echo ""
echo "✅ Nest Atomic operational!"
echo "PIDs: NestGate=$NESTGATE_PID Squirrel=$SQUIRREL_PID"
```

### **Script 4: Full NUCLEUS**

```bash
#!/bin/bash
# Start Full NUCLEUS Stack (All 5 Primals)

export FAMILY_ID=nat0
export NODE_ID=nucleus1
export JWT_SECRET="$(openssl rand -base64 48)"

echo "🎊 Starting FULL NUCLEUS Stack..."
echo "════════════════════════════════════════════"
echo ""

# 1. Start BearDog (Security Foundation)
echo "1️⃣ Starting BearDog (Security)..."
RUST_LOG=beardog=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    beardog server > /tmp/beardog_nucleus.log 2>&1 &
BEARDOG_PID=$!
echo "   PID: $BEARDOG_PID"
sleep 3

# 2. Start Songbird (Network + Discovery)
echo "2️⃣ Starting Songbird (Network)..."
RUST_LOG=songbird=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    SONGBIRD_SECURITY_PROVIDER=beardog \
    BEARDOG_SOCKET=/run/user/$(id -u)/biomeos/beardog.sock \
    songbird server > /tmp/songbird_nucleus.log 2>&1 &
SONGBIRD_PID=$!
echo "   PID: $SONGBIRD_PID"
sleep 3

# 3. Start Toadstool (GPU Compute)
echo "3️⃣ Starting Toadstool (Compute)..."
RUST_LOG=toadstool=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    toadstool server > /tmp/toadstool_nucleus.log 2>&1 &
TOADSTOOL_PID=$!
echo "   PID: $TOADSTOOL_PID"
sleep 3

# 4. Start NestGate (Storage + Persistence)
echo "4️⃣ Starting NestGate (Storage)..."
FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    NESTGATE_JWT_SECRET="$JWT_SECRET" \
    nestgate daemon --socket-only > /tmp/nestgate_nucleus.log 2>&1 &
NESTGATE_PID=$!
echo "   PID: $NESTGATE_PID"
sleep 3

# 5. Start Squirrel (AI Orchestration)
echo "5️⃣ Starting Squirrel (AI)..."
FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    squirrel server > /tmp/squirrel_nucleus.log 2>&1 &
SQUIRREL_PID=$!
echo "   PID: $SQUIRREL_PID"
sleep 5

# 6. Verify all sockets
echo ""
echo "🔍 Verifying ALL sockets..."
echo "════════════════════════════════════════════"
ls -lh /run/user/$(id -u)/biomeos/*.sock
echo ""

# Expected output:
# beardog.sock   ✅
# songbird.sock  ✅
# toadstool.sock ✅
# nestgate.sock  ✅
# squirrel.sock  ✅

# 7. Count sockets
SOCKET_COUNT=$(ls /run/user/$(id -u)/biomeos/*.sock 2>/dev/null | wc -l)
echo "Socket count: $SOCKET_COUNT/5"
echo ""

# 8. Health checks (all 5)
echo "🏥 Health Checks (All 5 Primals)..."
echo "════════════════════════════════════════════"
echo ""

for primal in beardog songbird toadstool nestgate squirrel; do
    echo "🔍 $primal:"
    echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
        nc -U /run/user/$(id -u)/biomeos/${primal}.sock -w 2 2>/dev/null || \
        echo "   ⚠️  No response"
    echo ""
done

# 9. Summary
echo "════════════════════════════════════════════"
echo "✅ FULL NUCLEUS STACK DEPLOYED!"
echo ""
echo "PIDs:"
echo "  BearDog:   $BEARDOG_PID"
echo "  Songbird:  $SONGBIRD_PID"
echo "  Toadstool: $TOADSTOOL_PID"
echo "  NestGate:  $NESTGATE_PID"
echo "  Squirrel:  $SQUIRREL_PID"
echo ""
echo "Logs:"
echo "  /tmp/beardog_nucleus.log"
echo "  /tmp/songbird_nucleus.log"
echo "  /tmp/toadstool_nucleus.log"
echo "  /tmp/nestgate_nucleus.log"
echo "  /tmp/squirrel_nucleus.log"
echo ""
echo "🎊 NUCLEUS is operational!"
```

---

## ✅ **Success Criteria Summary**

### **Tower Atomic**
- ✅ BearDog + Songbird sockets created
- ✅ Health checks pass
- ✅ Security integration works

### **Node Atomic**
- ✅ All Tower + Toadstool sockets
- ✅ Toadstool discovers Songbird (runtime)
- ✅ No hardcoded paths

### **Nest Atomic**
- ✅ All Tower + NestGate + Squirrel sockets
- ✅ NestGate socket-only mode (no port 8080)
- ✅ Squirrel discovery helpers work
- ✅ No external dependencies

### **Full NUCLEUS**
- ✅ All 5 sockets created
- ✅ All health checks pass (< 1 second each)
- ✅ Runtime discovery works
- ✅ No conflicts
- ✅ System stable for 60+ seconds

---

## 🚨 **Known Issues & Solutions**

### **Issue 1: Port 8080 Conflict (NestGate + Songbird)**

**Solution**: Use NestGate's `--socket-only` mode
```bash
nestgate daemon --socket-only
```

### **Issue 2: Missing Primal Identity**

**Solution**: Always set FAMILY_ID and NODE_ID
```bash
export FAMILY_ID=nat0
export NODE_ID=nucleus1
```

### **Issue 3: Insecure JWT Secret**

**Solution**: Generate secure secret
```bash
export NESTGATE_JWT_SECRET="$(openssl rand -base64 48)"
```

### **Issue 4: Old Hardcoded Paths**

**Solution**: All primals now use runtime discovery
- Standard path: `/run/user/$UID/biomeos/{primal}.sock`
- Environment override: `{PRIMAL}_SOCKET`
- No hardcoded paths remain

---

## 📊 **Validation Checklist**

### **Pre-Validation**
- [ ] All old processes killed
- [ ] All old sockets removed
- [ ] Temp logs cleared
- [ ] Environment variables set

### **Phase 1: Tower Atomic**
- [ ] BearDog socket created
- [ ] Songbird socket created
- [ ] BearDog health check passes
- [ ] Songbird health check passes
- [ ] Security integration works

### **Phase 2: Node Atomic**
- [ ] Toadstool socket created
- [ ] Toadstool health check passes
- [ ] Toadstool discovers Songbird
- [ ] No hardcoded paths used
- [ ] barraCUDA available (if tested)

### **Phase 3: Nest Atomic**
- [ ] NestGate socket created
- [ ] Squirrel socket created
- [ ] NestGate health check passes
- [ ] Squirrel health check passes
- [ ] No port 8080 binding
- [ ] Discovery helpers work

### **Phase 4: Full NUCLEUS**
- [ ] All 5 sockets created
- [ ] All 5 health checks pass
- [ ] Runtime discovery validated
- [ ] System stable 60+ seconds
- [ ] No conflicts observed

---

## 📚 **Reference Documents**

**Harvest Reports**:
- `TOADSTOOL_BEARDOG_EPIC_HARVEST_JAN30_2026.md`
- `NESTGATE_LEGENDARY_HARVEST_JAN30_2026.md`
- `SQUIRREL_EXCEPTIONAL_HARVEST_JAN30_2026.md`
- `FULL_NUCLEUS_ECOSYSTEM_COMPLETE_JAN30_2026.md`

**Handoff Documents**:
- `docs/handoffs/TOADSTOOL_SQUIRREL_SOCKET_STANDARDIZATION.md`
- `docs/handoffs/NESTGATE_CONFIGURATION_UNIX_SOCKET.md`

**Previous Validation**:
- `NUCLEUS_VALIDATION_RESULTS_JAN_30_2026.md` (Tower test)
- `FULL_SESSION_SUMMARY_JAN_30_2026.md`

**Primal Documentation**:
- BearDog: `/phase1/beardog/` - 11 docs
- Toadstool: `/phase1/toadstool/` - 39 docs
- NestGate: `/phase1/nestgate/` - 35+ docs
- Squirrel: `/phase1/squirrel/` - 27 docs

---

## 🎯 **Next Actions**

1. **Review this plan** with user
2. **Execute Phase 1** - Tower Atomic revalidation
3. **Execute Phase 2** - Node Atomic first test
4. **Execute Phase 3** - Nest Atomic first test
5. **Execute Phase 4** - Full NUCLEUS integration
6. **Document results** - Comprehensive validation report
7. **Production deployment** - If all phases pass

---

**Plan Date:** January 30, 2026  
**Status:** Ready for execution  
**Expected Duration:** 30-60 minutes (all phases)  
**Success Probability:** HIGH (all primals A+ quality)

🦀✨ **TRUE PRIMAL Architecture - Ready for Full Validation!** ✨🦀
