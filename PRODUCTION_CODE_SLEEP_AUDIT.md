# Production Code sleep() Audit - Week 2 Priority

**Date**: January 15, 2026 (Evening)  
**Purpose**: Audit and classify sleep() usage in production code  
**Status**: Planning phase

---

## 📋 Files with sleep() in Production Code (27 files)

### **Category 1: Retry/Backoff Logic** (Legitimate - Optimize)

These use sleep() for retry/backoff logic and should be reviewed for exponential backoff:

1. `biomeos-core/src/retry.rs` - Retry logic
2. `biomeos-core/src/adaptive_client.rs` - Adaptive client
3. `biomeos-core/src/primal_orchestrator.rs` - Orchestrator
4. `biomeos-atomic-deploy/src/primal_discovery.rs` - Discovery
5. `biomeos-atomic-deploy/src/primal_launcher.rs` - Launcher
6. `biomeos-federation/src/modules/status.rs` - Status monitoring
7. `biomeos-cli/src/discovery.rs` - CLI discovery
8. `biomeos-cli/src/commands/health.rs` - Health checks
9. `biomeos-cli/src/commands/monitor.rs` - Monitoring

**Action**: Review for exponential backoff pattern (like we did in tests)

---

### **Category 2: Event Loops/Polling** (Review Required)

These have event loops that may need architectural review:

10. `biomeos-graph/src/executor.rs` - Graph executor
11. `biomeos-graph/src/events.rs` - Event handling
12. `biomeos-atomic-deploy/src/neural_executor.rs` - Neural executor
13. `biomeos-spore/src/neural_spore.rs` - Spore management

**Action**: Evaluate if event-driven architecture would be better

---

### **Category 3: System/Boot Level** (Legitimate - System Requirements)

These are system-level code where sleep() may be necessary:

14. `biomeos-boot/src/init_shell.rs` - Init shell
15. `biomeos-boot/src/rootfs.rs` - Root filesystem
16. `biomeos-boot/src/bin/init.rs` - Init binary
17. `biomeos-system/src/lib.rs` - System library
18. `biomeos-deploy/src/qemu.rs` - QEMU management
19. `biomeos-deploy/src/health.rs` - Health monitoring
20. `biomeos-deploy/src/verify.rs` - Verification

**Action**: Document rationale, ensure minimal delays

---

### **Category 4: Network/Transport** (Review Required)

Network operations that may benefit from async patterns:

21. `biomeos-core/src/clients/beardog/btsp.rs` - BTSP protocol
22. `biomeos-types/src/service/networking.rs` - Networking types

**Action**: Evaluate async/await improvements

---

### **Category 5: Integration/Adapters** (Review Required)

Adapter code that interfaces with external systems:

23. `biomeos-core/src/api_adapter/cli_adapter.rs` - CLI adapter
24. `biomeos-core/src/universal_biomeos_manager/core.rs` - Universal manager
25. `biomeos-core/src/vm_federation.rs` - VM federation

**Action**: Review for better synchronization primitives

---

### **Category 6: Test Infrastructure** (Already Reviewed)

Test support files (not production):

26. `biomeos-api/tests/websocket_integration.rs` - WebSocket tests
27. `biomeos-boot/tests/qemu_harness.rs` - QEMU test harness
28. `biomeos-atomic-deploy/tests/chaos_tests.rs` - Chaos tests (intentional)

**Action**: Already handled or intentional

---

## 🎯 Week 2 Priority Actions

### **Phase 1: High-Value Optimizations** (2-3 hours)

1. **Retry Logic Evolution** (Priority: HIGH)
   - File: `biomeos-core/src/retry.rs`
   - Change: Implement exponential backoff
   - Impact: Better performance under load
   - Estimated: 30 minutes

2. **Adaptive Client Optimization** (Priority: HIGH)
   - File: `biomeos-core/src/adaptive_client.rs`
   - Change: Review and optimize backoff
   - Impact: Faster client adaptation
   - Estimated: 30 minutes

3. **Discovery Optimization** (Priority: HIGH)
   - Files: `primal_discovery.rs`, `cli/discovery.rs`
   - Change: Exponential backoff pattern
   - Impact: 10x faster discovery in common cases
   - Estimated: 45 minutes

### **Phase 2: Architectural Reviews** (3-4 hours)

4. **Event Loop Architecture** (Priority: MEDIUM)
   - Files: `graph/executor.rs`, `graph/events.rs`
   - Review: Event-driven vs polling
   - Impact: May enable better concurrency
   - Estimated: 2 hours

5. **Network/Transport** (Priority: MEDIUM)
   - File: `clients/beardog/btsp.rs`
   - Review: Async improvements
   - Impact: Better throughput
   - Estimated: 1 hour

### **Phase 3: Documentation** (1 hour)

6. **System-Level Documentation** (Priority: LOW)
   - Files: Boot/system code
   - Action: Document sleep() rationale
   - Impact: Clarity for future developers
   - Estimated: 1 hour

---

## 📊 Expected Impact

| Category | Files | Estimated Effort | Impact |
|----------|-------|------------------|--------|
| Retry/Backoff | 9 | 3-4 hours | High (10x faster) |
| Event Loops | 4 | 2-3 hours | Medium (architecture) |
| System Level | 7 | 1 hour (docs) | Low (document only) |
| Network | 2 | 1-2 hours | Medium (throughput) |
| Adapters | 3 | 2 hours | Medium (reliability) |

**Total Estimated Effort**: 9-12 hours (Week 2 work)

---

## 🎓 Principles

**For Each File**:
1. Is sleep() necessary? (vs channel/event-driven)
2. If yes, is it optimized? (exponential backoff)
3. Is it documented? (rationale clear)
4. Can it be tested? (stress tests)

**Philosophy**: 
- Test patterns → Production patterns
- Deep solutions, not quick fixes
- Document the "why" when sleep() is necessary

---

## 🚀 Next Steps

### Immediate (This Session)
1. ✅ Complete concurrent test evolution
2. ✅ Create stress tests
3. ⏭️  **Start retry.rs evolution** (next task)

### Week 2 (Next Sessions)
1. Retry/backoff optimizations (high priority)
2. Discovery improvements
3. Event loop architectural review
4. Network/transport async improvements

---

**Status**: Audit complete, ready to proceed with Phase 1! 🚀


