# ✅ Graph Orchestration Implementation Checklist

**Week-by-week action items**

---

## Week 1: Core Data Structures

### **Day 1-2: PrimalGraph & Node Types**

- [ ] Create `biomeos-core/src/graph_executor.rs`
- [ ] Define `PrimalGraph` struct
- [ ] Define `PrimalNode` struct
- [ ] Define `PrimalEdge` struct
- [ ] Define `CoordinationPattern` enum
- [ ] Define `EdgeCondition` enum
- [ ] Define `PrimalAction` enum
- [ ] Add serde derives for TOML parsing
- [ ] Write unit tests for parsing

**Test**: Can parse a simple graph from TOML

---

### **Day 3-4: DAG Builder**

- [ ] Define `DAG` struct
- [ ] Define `DAGNode` struct
- [ ] Define `NodeStatus` enum
- [ ] Implement `DAG::from_graph()`
- [ ] Implement cycle detection
- [ ] Implement `ready_nodes()` logic
- [ ] Implement `mark_complete()` logic
- [ ] Write unit tests for DAG operations

**Test**: Can build DAG from graph, detects cycles

---

### **Day 5: Integration & Testing**

- [ ] Add to `biomeos-core/src/lib.rs`
- [ ] Export public API
- [ ] Write integration tests
- [ ] Document API usage
- [ ] Review & refine

**Milestone**: Core data structures complete

---

## Week 2: Graph Executor

### **Day 1-2: Basic Executor**

- [ ] Define `GraphExecutor` struct
- [ ] Implement `execute()` method
- [ ] Implement `execute_sequential()`
- [ ] Implement `execute_parallel()`
- [ ] Write tests for sequential/parallel

**Test**: Can execute simple 2-node graphs

---

### **Day 3-4: DAG Execution**

- [ ] Implement `execute_dag()` method
- [ ] Add concurrent task spawning
- [ ] Add ready node detection loop
- [ ] Add completion tracking
- [ ] Handle node failures
- [ ] Write tests for DAG execution

**Test**: Can execute complex multi-node DAG

---

### **Day 5: TOML Configuration**

- [ ] Update `TowerConfig` enum
- [ ] Add `GraphConfig` variant
- [ ] Implement legacy → graph conversion
- [ ] Update parser
- [ ] Write parsing tests

**Test**: Both old and new formats parse correctly

**Milestone**: Graph executor complete

---

## Week 3: USB Spore Deployment

### **Day 1-2: Update USB Config**

- [ ] Update `tower.toml` on USB to graph format
- [ ] Update `activate-tower.sh` script
- [ ] Add example graph configs
- [ ] Test locally (not on USB)

**Test**: Graph config works in development

---

### **Day 3: USB Testing**

- [ ] Update USB spore with new binaries
- [ ] Update USB spore with graph config
- [ ] Test local deployment from USB
- [ ] Measure startup time
- [ ] Verify faster than waves

**Test**: USB deployment works, faster startup

---

### **Day 4: Multi-Spore Testing**

- [ ] Update both USB spores
- [ ] Test dual spore deployment
- [ ] Verify independent startup
- [ ] Verify federation works
- [ ] Measure total deployment time

**Test**: Both spores deploy successfully

---

### **Day 5: Documentation & Cleanup**

- [ ] Update deployment docs
- [ ] Update USB README
- [ ] Add troubleshooting guide
- [ ] Clean up old wave code (mark deprecated)
- [ ] Final review

**Milestone**: Production-ready graph orchestration!

---

## Success Metrics

### **Performance**
- [ ] Startup time: < 5 seconds (vs 8+ with waves)
- [ ] Graph execution overhead: < 5%
- [ ] Concurrent startup: confirmed

### **Robustness**
- [ ] No defunct processes
- [ ] Graceful failure handling
- [ ] Restart recovery works

### **Usability**
- [ ] TOML config clear and documented
- [ ] Error messages helpful
- [ ] Backwards compatible (legacy format works)

---

## Next Steps After Week 3

1. ✅ Phase 1 (Graph Execution) complete
2. 🟡 Write Metrics Collection spec
3. 🟡 Begin Phase 2 implementation
4. 🟡 Iterate based on usage

---

✅ **3 weeks to production-ready graph orchestration!**
