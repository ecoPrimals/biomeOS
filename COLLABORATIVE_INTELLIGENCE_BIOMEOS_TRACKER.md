# 🎯 Collaborative Intelligence - biomeOS Local Work Tracker

**Date**: January 11, 2026  
**Status**: In Progress  
**Handoff**: ✅ Complete (sent to primal teams)  
**Timeline**: 6-8 weeks for full integration

---

## 📊 **OVERVIEW**

This document tracks biomeOS's local work for Collaborative Intelligence while primal teams implement their components.

### **What We're Building**

**Collaborative Intelligence**: Human-AI collaborative graph orchestration where users can:
- View live graph execution
- Modify graphs in real-time
- Learn from AI suggestions
- Bootstrap new systems 10x faster

### **Our Role (biomeOS)**

- **Graph Execution Engine**: Execute user-modified graphs
- **Live Modification Handler**: Apply changes during execution
- **Integration Coordinator**: Wire together 10 cooperating intelligences
- **Event Streaming**: Broadcast graph events to UI

---

## ✅ **COMPLETED**

### **Phase 0: Specification & Handoff** ✅

- [x] Created vision and architecture (COLLABORATIVE_INTELLIGENCE_HANDOFF.md)
- [x] Wrote complete specification (specs/COLLABORATIVE_INTELLIGENCE_SPEC.md)
- [x] Documented evolution plan (COLLABORATIVE_INTELLIGENCE_EVOLUTION.md)
- [x] Handed off to primal teams (January 11, 2026)

**Status**: ✅ Complete - Teams have clear requirements

---

## 🔄 **IN PROGRESS - BIOMEOS LOCAL WORK**

### **Week 1-2: Foundation** (Current)

#### **Task 1: Graph Modification Handler**

**Status**: ⏳ Not Started  
**Priority**: P0 (Critical Path)  
**Dependencies**: None  
**Owner**: biomeOS team

**Requirements**:
```rust
// Add to crates/biomeos-graph/src/executor.rs
impl GraphExecutor {
    /// Apply modification to graph structure
    pub async fn apply_modification(
        &mut self,
        modification: GraphModification,
    ) -> Result<()> {
        match modification {
            GraphModification::AddNode { node, position } => {
                // Insert node at position
                // Update execution plan
                // Validate dependencies
            }
            GraphModification::RemoveNode { node_id } => {
                // Remove node
                // Revalidate graph
                // Update execution plan
            }
            GraphModification::ModifyNode { node_id, changes } => {
                // Update node properties
                // Revalidate
            }
            // ... other modification types
        }
    }
    
    /// Pause execution at safe checkpoint
    pub async fn pause(&mut self) -> Result<PausePoint> {
        // Find safe pause point (between nodes)
        // Pause execution
        // Return current state
    }
    
    /// Resume execution after pause
    pub async fn resume(&mut self) -> Result<()> {
        // Resume from pause point
        // Continue execution
    }
}
```

**Acceptance Criteria**:
- [ ] Can add node to running graph
- [ ] Can remove node from running graph
- [ ] Can modify node properties
- [ ] Can pause/resume execution safely
- [ ] All modifications validated before applying

**Tests Needed**:
- [ ] `test_add_node_to_running_graph`
- [ ] `test_remove_node_from_running_graph`
- [ ] `test_modify_node_properties`
- [ ] `test_pause_and_resume`
- [ ] `test_invalid_modification_rejected`

---

#### **Task 2: Event Streaming System**

**Status**: ⏳ Not Started  
**Priority**: P0 (Critical Path)  
**Dependencies**: None  
**Owner**: biomeOS team

**Requirements**:
```rust
// Add to crates/biomeos-graph/src/events.rs (new file)

/// Event emitted during graph execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEvent {
    /// Graph execution started
    GraphStarted {
        graph_id: String,
        total_nodes: usize,
        estimated_duration: Duration,
    },
    
    /// Node started executing
    NodeStarted {
        node_id: String,
        primal: String,
        operation: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Node completed successfully
    NodeCompleted {
        node_id: String,
        duration: Duration,
        result: Value,
    },
    
    /// Node failed
    NodeFailed {
        node_id: String,
        error: String,
        retry_count: usize,
    },
    
    /// AI decision made
    DecisionMade {
        decision: String,
        reasoning: Vec<String>,
        confidence: f64,
    },
    
    /// Graph execution completed
    GraphCompleted {
        graph_id: String,
        duration: Duration,
        nodes_executed: usize,
        success_rate: f64,
    },
}

/// Event broadcaster for graph execution
pub struct GraphEventBroadcaster {
    subscribers: Vec<Sender<GraphEvent>>,
}

impl GraphEventBroadcaster {
    pub fn new() -> Self { ... }
    
    pub fn subscribe(&mut self) -> Receiver<GraphEvent> { ... }
    
    pub async fn broadcast(&self, event: GraphEvent) { ... }
}
```

**Integration Points**:
- GraphExecutor emits events during execution
- petalTongue subscribes via WebSocket
- Events streamed in real-time

**Acceptance Criteria**:
- [ ] Events emitted for all execution stages
- [ ] Multiple subscribers supported
- [ ] Events include all relevant data
- [ ] No performance impact on execution

**Tests Needed**:
- [ ] `test_event_emission`
- [ ] `test_multiple_subscribers`
- [ ] `test_event_ordering`
- [ ] `test_subscriber_disconnect_handling`

---

#### **Task 3: Graph Validation Enhanced**

**Status**: ⏳ Not Started  
**Priority**: P1 (Important)  
**Dependencies**: Songbird validation API  
**Owner**: biomeOS team

**Requirements**:
```rust
// Add to crates/biomeos-graph/src/validator.rs

pub struct GraphValidator {
    songbird_client: Option<SongbirdClient>,
    beardog_client: Option<BearDogClient>,
}

impl GraphValidator {
    /// Validate graph before execution
    pub async fn validate(&self, graph: &Graph) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Structure validation (local)
        self.validate_structure(graph, &mut report)?;
        
        // Primal availability (via Songbird)
        if let Some(songbird) = &self.songbird_client {
            self.validate_primal_availability(graph, songbird, &mut report).await?;
        }
        
        // Security validation (via BearDog)
        if let Some(beardog) = &self.beardog_client {
            self.validate_security(graph, beardog, &mut report).await?;
        }
        
        Ok(report)
    }
    
    fn validate_structure(&self, graph: &Graph, report: &mut ValidationReport) -> Result<()> {
        // Check for cycles
        // Validate dependencies
        // Check for orphaned nodes
        // Validate coordination patterns
    }
}

pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<ValidationSuggestion>,
}
```

**Acceptance Criteria**:
- [ ] Detects cycles in graph
- [ ] Validates all dependencies exist
- [ ] Checks primal availability (if Songbird available)
- [ ] Security validation (if BearDog available)
- [ ] Provides helpful error messages

**Tests Needed**:
- [ ] `test_detect_cycles`
- [ ] `test_missing_dependencies`
- [ ] `test_invalid_coordination_pattern`
- [ ] `test_primal_availability_check`

---

### **Week 3-4: AI Integration**

#### **Task 4: Squirrel Integration for Learning**

**Status**: ⏳ Not Started  
**Priority**: P0 (Critical Path)  
**Dependencies**: Squirrel learning API  
**Owner**: biomeOS team

**Requirements**:
```rust
// Add to crates/biomeos-graph/src/learning.rs (new file)

pub struct GraphLearningCoordinator {
    squirrel_client: Option<SquirrelClient>,
}

impl GraphLearningCoordinator {
    /// Send modification to Squirrel for learning
    pub async fn learn_from_modification(
        &self,
        original_graph: &Graph,
        modified_graph: &Graph,
        outcome: ExecutionOutcome,
    ) -> Result<()> {
        if let Some(squirrel) = &self.squirrel_client {
            squirrel.learn_from_modification(
                original_graph,
                modified_graph,
                outcome,
            ).await?;
        }
        Ok(())
    }
    
    /// Get AI suggestions for graph
    pub async fn get_suggestions(
        &self,
        graph: &Graph,
        user_id: &str,
    ) -> Result<Vec<Suggestion>> {
        if let Some(squirrel) = &self.squirrel_client {
            squirrel.suggest_improvements(graph, user_id).await
        } else {
            Ok(vec![])
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Send modifications to Squirrel after execution
- [ ] Include execution outcome (success/failure)
- [ ] Request suggestions before execution
- [ ] Display suggestions to user (via petalTongue)

**Tests Needed**:
- [ ] `test_send_modification_to_squirrel`
- [ ] `test_get_suggestions`
- [ ] `test_no_squirrel_graceful_degradation`

---

### **Week 5-6: Real-Time Execution**

#### **Task 5: WebSocket Server for Events**

**Status**: ⏳ Not Started  
**Priority**: P0 (Critical Path)  
**Dependencies**: petalTongue WebSocket client  
**Owner**: biomeOS team

**Requirements**:
```rust
// Add to crates/biomeos-api/src/websocket.rs (new file)

use tokio_tungstenite::WebSocketStream;

pub struct GraphEventWebSocket {
    connections: HashMap<String, WebSocketStream>,
    event_broadcaster: GraphEventBroadcaster,
}

impl GraphEventWebSocket {
    pub async fn start(addr: SocketAddr) -> Result<Self> {
        // Start WebSocket server
        // Accept connections
        // Subscribe to graph events
        // Broadcast to all connected clients
    }
    
    pub async fn handle_connection(
        &mut self,
        stream: TcpStream,
    ) -> Result<()> {
        // Upgrade to WebSocket
        // Subscribe to events
        // Stream events to client
    }
}
```

**Acceptance Criteria**:
- [ ] WebSocket server starts on configurable port
- [ ] Accepts multiple client connections
- [ ] Broadcasts graph events to all clients
- [ ] Handles client disconnections gracefully

**Tests Needed**:
- [ ] `test_websocket_server_start`
- [ ] `test_multiple_clients`
- [ ] `test_event_broadcast`
- [ ] `test_client_disconnect`

---

### **Week 7-8: Polish & Integration**

#### **Task 6: Template Integration with NestGate**

**Status**: ⏳ Not Started  
**Priority**: P1 (Important)  
**Dependencies**: NestGate template API  
**Owner**: biomeOS team

**Requirements**:
```rust
// Add to crates/biomeos-graph/src/templates.rs (new file)

pub struct GraphTemplateManager {
    nestgate_client: Option<NestGateClient>,
}

impl GraphTemplateManager {
    /// Save graph as template
    pub async fn save_template(
        &self,
        graph: &Graph,
        name: String,
        user_id: String,
    ) -> Result<String> {
        if let Some(nestgate) = &self.nestgate_client {
            let template = GraphTemplate {
                name,
                graph: graph.clone(),
                created_by: user_id,
                created_at: Utc::now(),
            };
            
            nestgate.templates_store(&template).await
        } else {
            Err(anyhow!("No storage primal available"))
        }
    }
    
    /// Load template
    pub async fn load_template(
        &self,
        template_id: &str,
    ) -> Result<Graph> {
        if let Some(nestgate) = &self.nestgate_client {
            let template: GraphTemplate = nestgate
                .templates_retrieve(template_id)
                .await?;
            
            Ok(template.graph)
        } else {
            Err(anyhow!("No storage primal available"))
        }
    }
    
    /// List user templates
    pub async fn list_templates(
        &self,
        user_id: &str,
    ) -> Result<Vec<GraphTemplateSummary>> {
        if let Some(nestgate) = &self.nestgate_client {
            nestgate.templates_list(user_id, None).await
        } else {
            Ok(vec![])
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Can save graph as template
- [ ] Can load template by ID
- [ ] Can list user's templates
- [ ] Can list community templates (via NestGate)

**Tests Needed**:
- [ ] `test_save_template`
- [ ] `test_load_template`
- [ ] `test_list_templates`

---

#### **Task 7: End-to-End Integration Testing**

**Status**: ⏳ Not Started  
**Priority**: P0 (Critical)  
**Dependencies**: All primal APIs ready  
**Owner**: biomeOS team

**Requirements**:
```rust
// Add to tests/collaborative_intelligence_integration_test.rs

#[tokio::test]
async fn test_full_collaborative_flow() {
    // 1. User loads template
    let template = template_manager
        .load_template("compute-node-base")
        .await
        .unwrap();
    
    // 2. User modifies graph
    let mut graph = template.graph;
    graph.add_node(new_monitoring_node);
    
    // 3. AI suggests improvement
    let suggestions = learning_coordinator
        .get_suggestions(&graph, "user_123")
        .await
        .unwrap();
    
    assert!(!suggestions.is_empty());
    
    // 4. User accepts suggestion
    graph.apply_modification(suggestions[0].modification.clone());
    
    // 5. Validate graph
    let validation = validator.validate(&graph).await.unwrap();
    assert!(validation.valid);
    
    // 6. Execute graph
    let executor = GraphExecutor::new(graph);
    let mut event_rx = executor.subscribe_to_events();
    
    executor.execute().await.unwrap();
    
    // 7. Verify events received
    let events = collect_events(&mut event_rx).await;
    assert!(events.iter().any(|e| matches!(e, GraphEvent::GraphCompleted { .. })));
    
    // 8. AI learns from execution
    learning_coordinator
        .learn_from_modification(&template.graph, &graph, ExecutionOutcome::Success)
        .await
        .unwrap();
    
    // 9. Save modified graph as new template
    let template_id = template_manager
        .save_template(&graph, "compute-node-monitoring", "user_123")
        .await
        .unwrap();
    
    assert!(!template_id.is_empty());
}
```

**Acceptance Criteria**:
- [ ] Full flow works end-to-end
- [ ] All primals coordinate correctly
- [ ] Events stream in real-time
- [ ] Learning loop functional

---

## 📋 **TASK SUMMARY**

| Week | Task | Priority | Status |
|------|------|----------|--------|
| 1-2 | Graph Modification Handler | P0 | ⏳ Not Started |
| 1-2 | Event Streaming System | P0 | ⏳ Not Started |
| 1-2 | Graph Validation Enhanced | P1 | ⏳ Not Started |
| 3-4 | Squirrel Integration | P0 | ⏳ Not Started |
| 5-6 | WebSocket Server | P0 | ⏳ Not Started |
| 7-8 | Template Integration | P1 | ⏳ Not Started |
| 7-8 | End-to-End Testing | P0 | ⏳ Not Started |

**Total Tasks**: 7  
**Completed**: 0  
**In Progress**: 0  
**Not Started**: 7

---

## 🔗 **INTEGRATION DEPENDENCIES**

### **Week 1-2: Foundation**
- **Depends on**: None (can start immediately)
- **Blocks**: Weeks 3-4 (AI Integration)

### **Week 3-4: AI Integration**
- **Depends on**: Squirrel learning API
- **Blocks**: Week 5-6 (Real-time)
- **Coordination**: Weekly sync with Squirrel team

### **Week 5-6: Real-Time**
- **Depends on**: petalTongue WebSocket client
- **Blocks**: Week 7-8 (Polish)
- **Coordination**: Weekly sync with petalTongue team

### **Week 7-8: Polish**
- **Depends on**: NestGate template API
- **Blocks**: Nothing (final phase)
- **Coordination**: Integration testing with all teams

---

## 📊 **PROGRESS TRACKING**

### **Week 1** (Current Week)
- [ ] Task 1: Graph Modification Handler (5 days)
- [ ] Task 2: Event Streaming System (3 days)
- [ ] Task 3: Graph Validation Enhanced (2 days)

### **Week 2**
- [ ] Complete any remaining foundation work
- [ ] Begin Squirrel integration
- [ ] Write integration tests for foundation

### **Week 3-4**
- [ ] Complete Squirrel integration
- [ ] Test learning loop
- [ ] Begin WebSocket server

### **Week 5-6**
- [ ] Complete WebSocket server
- [ ] Real-time event streaming working
- [ ] Integration testing with petalTongue

### **Week 7-8**
- [ ] Template integration
- [ ] End-to-end testing
- [ ] Performance optimization
- [ ] Documentation updates

---

## 🎯 **SUCCESS CRITERIA**

### **Minimum Viable (Week 4)**
- [ ] Can modify graphs before execution
- [ ] Can validate graphs
- [ ] Can receive AI suggestions
- [ ] Basic event streaming

### **Full Feature (Week 6)**
- [ ] Can modify graphs during execution
- [ ] Real-time event streaming via WebSocket
- [ ] AI learning loop functional
- [ ] Template save/load working

### **Production Ready (Week 8)**
- [ ] All integration tests passing
- [ ] Performance optimized (<100ms event latency)
- [ ] Documentation complete
- [ ] Security validation working

---

## 📚 **REFERENCE DOCUMENTS**

### **External (Handed Off)**
- **COLLABORATIVE_INTELLIGENCE_HANDOFF.md** - Primal team requirements
- **specs/COLLABORATIVE_INTELLIGENCE_SPEC.md** - Complete specification
- **COLLABORATIVE_INTELLIGENCE_EVOLUTION.md** - Evolution roadmap

### **Internal (This Doc)**
- **COLLABORATIVE_INTELLIGENCE_BIOMEOS_TRACKER.md** - Local work tracking (this file)

---

## 🚀 **GETTING STARTED**

### **Start with Task 1: Graph Modification Handler**

```bash
# Create new branch
git checkout -b collaborative-intelligence-foundation

# Create new files
touch crates/biomeos-graph/src/modification.rs
touch crates/biomeos-graph/src/events.rs

# Start implementation
code crates/biomeos-graph/src/modification.rs
```

### **Run Tests**
```bash
cargo test -p biomeos-graph
cargo test collaborative_intelligence
```

---

## 💡 **NOTES**

### **Design Decisions**
- Using event-driven architecture for real-time updates
- GraphModification enum for type-safe modifications
- Graceful degradation if primals unavailable
- WebSocket for real-time UI updates (not polling)

### **Open Questions**
- [ ] Should we support forking graphs (parallel what-if scenarios)?
- [ ] How to handle conflicting modifications from multiple users?
- [ ] Should we cache templates locally or always fetch from NestGate?

### **Risks**
- WebSocket connection stability (mitigation: auto-reconnect)
- Event ordering guarantees (mitigation: sequence numbers)
- Performance impact of event streaming (mitigation: buffering)

---

## 📅 **WEEKLY SYNC**

### **Schedule**
- **When**: Wednesdays, 2pm UTC
- **Who**: All primal teams + biomeOS
- **Format**: Status update, blockers, next week's plan

### **Agenda Template**
1. Progress update (what was completed)
2. Blockers (what's blocking progress)
3. Integration points (coordination needed)
4. Next week's plan (what will be done)

---

## 🎊 **STATUS**

**Overall Progress**: 0% (0/7 tasks complete)  
**On Track**: ✅ Yes (just handed off)  
**Blockers**: None  
**Next Action**: Start Task 1 (Graph Modification Handler)

**Last Updated**: January 11, 2026  
**Next Review**: January 18, 2026 (Week 1 complete)

---

**Ready to start building!** 🚀

