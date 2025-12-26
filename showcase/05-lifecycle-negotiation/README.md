# 🔄 Scenario 05: Lifecycle Negotiation

**Status**: 🚧 Building Now  
**Complexity**: Medium-High  
**Duration**: 10 minutes

---

## 🎯 What This Demonstrates

BiomeOS's **Cell Senescence Model** - requesting lifecycle transitions with respect for primal sovereignty:

1. **Request, Don't Command**: BiomeOS requests, primals decide
2. **Primal Autonomy**: Can accept, defer, or refuse requests
3. **Graceful Handling**: BiomeOS adapts to primal decisions
4. **Ecosystem Coordination**: Health-based lifecycle management

---

## 🏗️ Philosophy: Cell Senescence vs Overwatch

### ❌ Overwatch Model (What We Avoid)
```
BiomeOS: "Stop now!"
Primal: *forced to stop*
Result: Violation of sovereignty
```

### ✅ Cell Senescence Model (What We Build)
```
BiomeOS: "The ecosystem would benefit if you gracefully stopped"
Primal: "I have active connections. Can I have 30 seconds?"
BiomeOS: "Of course. I'll wait."
Primal: *gracefully completes work, then stops*
Result: Coordinated, consensual, respectful
```

---

## 🧬 Lifecycle Negotiation Protocol

```rust
// BiomeOS requests a transition
let request = LifecycleRequest::new(
    LifecycleTransition::GracefulStop,
    TransitionReason::EcosystemHealth,
);

// Primal decides
match primal.request_transition(request).await? {
    // Primal accepts
    LifecycleResponse::Accepted => {
        println!("✅ Primal agreed, proceeding with stop");
        // BiomeOS proceeds
    }
    
    // Primal needs time
    LifecycleResponse::Deferred { duration, reason } => {
        println!("⏳ Primal asked for {} seconds: {}", 
                 duration.as_secs(), reason);
        // BiomeOS waits, then retries
    }
    
    // Primal refuses
    LifecycleResponse::Refused { reason } => {
        println!("❌ Primal refused: {}", reason);
        // BiomeOS respects decision, adapts plan
    }
}
```

---

## 🚀 Running the Demo

### With Mock Primals
```bash
cd showcase/05-lifecycle-negotiation/
./demo.sh
```

### What Happens
1. **Start all primals** (using adapters)
2. **Request graceful stop** from each
3. **Show different responses**:
   - Squirrel: Accepts immediately
   - NestGate: Defers (active operations)
   - ToadStool: Refuses (critical task)
   - BearDog: Accepts after cleanup
   - Songbird: Defers (coordinating mesh)
4. **BiomeOS adapts** to each response
5. **Ecosystem remains healthy**

---

## 📊 Expected Output

```
=== Lifecycle Negotiation Demo ===

🚀 Starting all 5 primals...
✅ All primals started and healthy

🔄 Scenario 1: Request Graceful Stop
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📤 Requesting: Squirrel to gracefully stop
   Reason: Ecosystem maintenance
   Urgency: Normal

📥 Squirrel Response: Accepted
   ✅ Proceeding with graceful stop
   🛑 Squirrel stopped gracefully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📤 Requesting: NestGate to gracefully stop
   Reason: Ecosystem maintenance
   Urgency: Normal

📥 NestGate Response: Deferred (30 seconds)
   Reason: "Completing 3 active write operations"
   ⏳ BiomeOS waiting as requested...
   ⏱️  30 seconds elapsed
   
📤 Retrying: NestGate graceful stop

📥 NestGate Response: Accepted
   ✅ Operations complete, stopping now
   🛑 NestGate stopped gracefully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📤 Requesting: ToadStool to gracefully stop
   Reason: Ecosystem maintenance
   Urgency: Normal

📥 ToadStool Response: Refused
   Reason: "Running critical compute job, ETA 5 minutes"
   ❌ BiomeOS respects decision
   ℹ️  Will retry after job completes

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📤 Requesting: BearDog to gracefully stop
   Reason: Ecosystem maintenance
   Urgency: Normal

📥 BearDog Response: Accepted
   ℹ️  Flushing security logs...
   ✅ Security state saved
   🛑 BearDog stopped gracefully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📤 Requesting: Songbird to gracefully stop
   Reason: Ecosystem maintenance
   Urgency: Normal

📥 Songbird Response: Deferred (15 seconds)
   Reason: "Migrating active service mesh connections"
   ⏳ BiomeOS waiting as requested...
   ⏱️  15 seconds elapsed
   
📤 Retrying: Songbird graceful stop

📥 Songbird Response: Accepted
   ✅ Mesh handoff complete
   🛑 Songbird stopped gracefully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 Summary:
   ✅ Accepted immediately: 2 (Squirrel, BearDog)
   ⏳ Deferred then accepted: 2 (NestGate, Songbird)
   ❌ Refused (sovereignty preserved): 1 (ToadStool)
   
   Total stopped: 4/5
   ToadStool still running (critical work)

🌱 Key Principles Demonstrated:
   ✅ Request, not command
   ✅ Primals have full autonomy
   ✅ Deferred requests honored
   ✅ Refusals respected
   ✅ Ecosystem coordination without coercion
   ✅ Cell senescence > overwatch

🎉 Lifecycle Negotiation Complete!
```

---

## 🎭 Mock Primal Behaviors

### Enhanced Mocks with Lifecycle Logic

Each mock now supports lifecycle requests:

#### Squirrel: Immediate Acceptance
```python
# Always accepts gracefully
def handle_lifecycle_request(request):
    if request.transition == "GracefulStop":
        return {"status": "accepted"}
    return {"status": "not_supported"}
```

#### NestGate: Conditional Deferral
```python
def handle_lifecycle_request(request):
    if has_active_operations():
        return {
            "status": "deferred",
            "duration_secs": 30,
            "reason": "Completing active write operations"
        }
    return {"status": "accepted"}
```

#### ToadStool: Can Refuse
```python
def handle_lifecycle_request(request):
    if running_critical_job():
        return {
            "status": "refused",
            "reason": "Running critical compute job, ETA 5 minutes"
        }
    return {"status": "accepted"}
```

#### BearDog: Cleanup Before Accept
```python
def handle_lifecycle_request(request):
    flush_security_logs()
    save_state()
    return {"status": "accepted"}
```

#### Songbird: Mesh Migration
```python
def handle_lifecycle_request(request):
    if has_active_mesh_connections():
        return {
            "status": "deferred",
            "duration_secs": 15,
            "reason": "Migrating active service mesh connections"
        }
    return {"status": "accepted"}
```

---

## 🧬 How It Works

### 1. Lifecycle Request
```rust
pub struct LifecycleRequest {
    transition: LifecycleTransition,  // What we're requesting
    reason: TransitionReason,         // Why we're requesting
    urgency: Urgency,                 // How urgent
    requestor: String,                // Who's requesting
}
```

### 2. Primal Decision
```rust
pub enum LifecycleResponse {
    Accepted,
    Deferred { duration: Duration, reason: String },
    Refused { reason: String },
    NotSupported,
}
```

### 3. BiomeOS Adaptation
```rust
async fn handle_response(response: LifecycleResponse) {
    match response {
        Accepted => execute_transition(),
        Deferred { duration, .. } => {
            wait(duration).await;
            retry_request().await;
        }
        Refused { reason } => {
            log_refusal(reason);
            adapt_ecosystem_plan();
        }
    }
}
```

---

## 🌱 Key Principles

### 1. Sovereignty First
- Primals always have final say
- No forced lifecycle changes
- Refusals are legitimate and respected
- BiomeOS adapts, doesn't override

### 2. Negotiation Over Commands
- "Would you consider?" vs "You must"
- Provide reason and context
- Accept primal's decision
- Coordinate through consent

### 3. Graceful Coordination
- Ecosystem health through cooperation
- Individual autonomy preserved
- Collective benefit emerges
- No single point of control

### 4. Cell Senescence Model
Like biological cells:
- Receive signals (not commands)
- Make autonomous decisions
- Coordinate with neighbors
- Die gracefully when beneficial to organism

---

## 🎯 Transition Types

```rust
pub enum LifecycleTransition {
    Start,              // Request to start
    GracefulStop,       // Request graceful shutdown
    EmergencyStop,      // Request immediate stop (higher urgency)
    Restart,            // Request restart
    ScaleDown,          // Request resource reduction
}
```

---

## 🔍 Reasons for Transitions

```rust
pub enum TransitionReason {
    EcosystemHealth,    // Overall ecosystem needs
    UserRequest,        // User explicitly requested
    ResourcePressure,   // System resource constraints
    FailureDetected,    // Primal appears unhealthy
    Maintenance,        // Routine maintenance
    Other(String),      // Custom reason
}
```

---

## ⚡ Urgency Levels

```rust
pub enum Urgency {
    Low,      // Can wait indefinitely
    Normal,   // Should happen soon
    High,     // Important, but not critical
    Critical, // Immediate attention needed
}
```

Higher urgency → less likely primal will defer/refuse
But primal always has final decision!

---

## 📚 Real-World Scenarios

### Scenario A: Ecosystem Maintenance
```
BiomeOS: All primals graceful stop for update
- 3 accept immediately
- 2 defer (active work)
- BiomeOS waits for deferrals
- Update proceeds when ready
```

### Scenario B: Resource Pressure
```
BiomeOS: ToadStool please scale down (high memory usage)
ToadStool: Refused (critical computation)
BiomeOS: Adapts by scaling down other services instead
```

### Scenario C: Health Issue Detected
```
BiomeOS: NestGate appears unhealthy, restart?
NestGate: Deferred (syncing data)
BiomeOS: Waits, then restarts after sync
```

### Scenario D: Emergency
```
BiomeOS: Security issue, all primals stop NOW (Critical urgency)
All primals: Even with active work, accept due to urgency
Ecosystem: Secured quickly
```

---

## 🔄 What's Next

### Scenario 06: Songbird Port Manager
- Dynamic port allocation
- Service mesh registration
- Zero hardcoded endpoints
- Connection routing

### Scenario 07: Data Sovereignty
- NestGate integration
- Sovereign data storage
- Privacy policies
- User data control

---

## 📖 References

- [Lifecycle Negotiation Protocol](../../crates/biomeos-core/src/primal_adapter/lifecycle.rs)
- [Primal Integration Architecture](../../docs/PRIMAL_INTEGRATION_ARCHITECTURE.md)
- [Cell Senescence Philosophy](../../docs/PRIMAL_INTEGRATION_ARCHITECTURE.md#cell-senescence-model)

---

**Status**: 🚧 Building now  
**Philosophy**: Cell senescence, not overwatch  
**Key**: Sovereignty through negotiation

*"The ecosystem asks, primals decide."* 🔄🌱✨

