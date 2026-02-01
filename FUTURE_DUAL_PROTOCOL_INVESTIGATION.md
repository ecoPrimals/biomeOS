# 🚧 FUTURE INVESTIGATION: Dual-Protocol Integration

**Date**: February 1, 2026  
**Priority**: Medium (after STUN federation testing)  
**Estimated Time**: 2-4 hours

═══════════════════════════════════════════════════════════════════

## 📋 **INVESTIGATION NEEDED**

### **Verify Dual-Protocol Integration** (1-2 hours)

**nestgate**:
- Check if tarpc server starts in daemon mode
- service.rs line 182 says "planned for v0.2.0"
- Verify both JSON-RPC + tarpc operational

**squirrel**:
- Has tarpc code but main.rs only starts JSON-RPC
- Find where tarpc server should start
- Verify dual-protocol in production

**songbird**:
- Has tarpc_server.rs infrastructure
- Check server mode integration
- Verify both protocols available

**beardog**:
- JSON-RPC operational (Unix sockets)
- Investigate if tarpc needed for crypto ops
- May not need it (already fast)

---

## 🎯 **GOAL**

All primals should support:
- **JSON-RPC**: neuralAPI discovery & coordination (flexible)
- **tarpc**: Performance escalation after established (fast)

**Pattern**: neuralAPI uses JSON-RPC → discovers services → escalates to tarpc for performance

---

## 📚 **REFERENCE**

**Analysis Document**: `PRIMAL_ROLES_DUAL_PROTOCOL_ANALYSIS.md`

**Reference Implementation**: toadstool (both protocols operational)

---

## ⏳ **DEFERRED**

**Reason**: STUN federation testing has higher priority  
**Return After**: Cross-device STUN handshake validated

═══════════════════════════════════════════════════════════════════

**Status**: 🚧 **MARKED FOR FUTURE INVESTIGATION**
