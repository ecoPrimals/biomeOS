# 🎯 Phase 3: Device Assignment Implementation Plan

**Date**: January 11, 2026  
**Phase**: 3 of 6 (Interaction)  
**Duration**: 3 weeks estimated  
**Status**: ⏳ **STARTING NOW**

---

## 📊 **Overview**

Phase 3 implements the core interaction: **device assignment to primals**.

This is where the network effect truly shines - a single user action (drag-and-drop) triggers coordination between 6 different primals!

---

## 🎯 **Goals**

1. ✅ User can assign a device to a primal
2. ✅ Assignment is authorized by BearDog
3. ✅ Assignment is validated by Songbird
4. ✅ Assignment is persisted by NestGate
5. ✅ Resources are checked by ToadStool
6. ✅ UI updates via petalTongue
7. ✅ User receives clear feedback

---

## 🤝 **Network Effect Coordination**

### **Assignment Flow** (6 Primals!)

```
User clicks "Assign GPU-0 to ToadStool"
         ↓
biomeOS orchestrator receives UserAction::AssignDevice
         ↓
    ┌────┴────┐
    │         │
Phase 1: Authorization (BearDog)
    │ → Is user authorized to assign this device?
    │ → Does primal accept this device type?
    │ ← Authorized: Yes/No
    │
Phase 2: Validation (Songbird)
    │ → Is device available?
    │ → Is primal running and healthy?
    │ → Are there conflicts?
    │ ← Valid: Yes/No
    │
Phase 3: Resource Check (ToadStool)
    │ → Does primal have capacity?
    │ → Will this cause resource contention?
    │ ← Capacity: Yes/No
    │
Phase 4: Register (Songbird)
    │ → Register device → primal assignment
    │ ← Registered: Assignment ID
    │
Phase 5: Persist (NestGate)
    │ → Store assignment for restart recovery
    │ ← Persisted: Success
    │
Phase 6: Update UI (petalTongue)
    │ → Push topology update
    │ → Show success feedback
    │ ← Rendered: Success
    │
    └────┬────┘
         ↓
User sees: "GPU-0 assigned to ToadStool ✓"
```

**6 primals coordinating = network effect!**

---

## 📋 **Implementation Tasks**

### **Task 1: BearDog Authorization** (Week 1, Day 1-2)

**Goal**: Verify user is authorized to assign device

**Implementation**:
```rust
async fn authorize_device_assignment(
    &self,
    user_id: &str,
    device_id: &str,
    primal_id: &str,
) -> Result<AuthorizationResult> {
    // Query BearDog if available
    if let Some(beardog) = &self.beardog {
        // Check user permissions
        let user_perms = beardog.check_permission(
            user_id,
            &format!("device.assign.{}", device_id)
        ).await?;
        
        // Check primal acceptance
        let primal_policy = beardog.get_device_policy(primal_id).await?;
        
        if user_perms.allowed && primal_policy.accepts_device(device_id) {
            Ok(AuthorizationResult::Authorized)
        } else {
            Ok(AuthorizationResult::Denied("Insufficient permissions"))
        }
    } else {
        // Graceful degradation: allow if no security primal
        warn!("No security primal available, skipping authorization");
        Ok(AuthorizationResult::Authorized)
    }
}
```

**Tests**:
- `test_authorize_device_assignment_success`
- `test_authorize_device_assignment_denied`
- `test_authorize_device_assignment_no_beardog`

**Deliverable**: Authorization method working with/without BearDog

---

### **Task 2: Songbird Validation** (Week 1, Day 3-4)

**Goal**: Verify device is available and primal is healthy

**Implementation**:
```rust
async fn validate_device_assignment(
    &self,
    device_id: &str,
    primal_id: &str,
) -> Result<ValidationResult> {
    if let Some(songbird) = &self.songbird {
        // Check device availability
        let device_status = songbird.get_device_status(device_id).await?;
        if device_status.assigned {
            return Ok(ValidationResult::Invalid("Device already assigned"));
        }
        
        // Check primal health
        let primal_health = songbird.get_service_health(primal_id).await?;
        if !primal_health.is_healthy() {
            return Ok(ValidationResult::Invalid("Primal unhealthy"));
        }
        
        // Check for conflicts
        let conflicts = songbird.check_device_conflicts(device_id, primal_id).await?;
        if !conflicts.is_empty() {
            return Ok(ValidationResult::Invalid(format!("Conflicts: {:?}", conflicts)));
        }
        
        Ok(ValidationResult::Valid)
    } else {
        warn!("No service registry available, skipping validation");
        Ok(ValidationResult::Valid)
    }
}
```

**Tests**:
- `test_validate_device_available`
- `test_validate_device_already_assigned`
- `test_validate_primal_unhealthy`
- `test_validate_conflicts_detected`
- `test_validate_no_songbird`

**Deliverable**: Validation method working with/without Songbird

---

### **Task 3: ToadStool Resource Check** (Week 1, Day 5)

**Goal**: Verify primal has capacity for device

**Implementation**:
```rust
async fn check_primal_capacity(
    &self,
    device_id: &str,
    primal_id: &str,
) -> Result<CapacityResult> {
    if let Some(toadstool) = &self.toadstool {
        // Get current resource usage
        let usage = toadstool.get_resource_usage(primal_id).await?;
        
        // Get device resource requirements
        let device_reqs = self.get_device_requirements(device_id).await?;
        
        // Check if primal can handle it
        if usage.can_accommodate(&device_reqs) {
            Ok(CapacityResult::Available {
                current: usage,
                after: usage.with_device(&device_reqs),
            })
        } else {
            Ok(CapacityResult::Insufficient {
                required: device_reqs,
                available: usage.available_capacity(),
            })
        }
    } else {
        warn!("No compute primal available, skipping capacity check");
        Ok(CapacityResult::Available {
            current: ResourceUsage::unknown(),
            after: ResourceUsage::unknown(),
        })
    }
}
```

**Tests**:
- `test_check_capacity_available`
- `test_check_capacity_insufficient`
- `test_check_capacity_no_toadstool`

**Deliverable**: Capacity checking working with/without ToadStool

---

### **Task 4: Register Assignment (Songbird)** (Week 2, Day 1-2)

**Goal**: Register device→primal assignment in service registry

**Implementation**:
```rust
async fn register_assignment(
    &self,
    device_id: &str,
    primal_id: &str,
    assignment: &Assignment,
) -> Result<String> {
    if let Some(songbird) = &self.songbird {
        // Register the assignment
        let assignment_id = songbird.register_device_assignment(
            device_id,
            primal_id,
            &assignment.metadata,
        ).await?;
        
        info!("Assignment registered: {}", assignment_id);
        
        // Update local state
        let mut state = self.state.write().await;
        state.assignments.insert(device_id.to_string(), assignment.clone());
        
        Ok(assignment_id)
    } else {
        // Fallback: generate local assignment ID
        let assignment_id = format!("local-{}", uuid::Uuid::new_v4());
        
        let mut state = self.state.write().await;
        state.assignments.insert(device_id.to_string(), assignment.clone());
        
        Ok(assignment_id)
    }
}
```

**Tests**:
- `test_register_assignment_with_songbird`
- `test_register_assignment_without_songbird`
- `test_register_assignment_duplicate`

**Deliverable**: Assignment registration working

---

### **Task 5: Persist Assignment (NestGate)** (Week 2, Day 3)

**Goal**: Persist assignment for recovery after restart

**Implementation**:
```rust
async fn persist_assignment(
    &self,
    assignment_id: &str,
    assignment: &Assignment,
) -> Result<()> {
    if let Some(nestgate) = &self.nestgate {
        // Serialize assignment
        let data = serde_json::to_value(assignment)?;
        
        // Store in NestGate
        let key = format!("assignment:{}", assignment_id);
        nestgate.store(&key, &data, &self.family_id).await?;
        
        info!("Assignment persisted: {}", assignment_id);
        
        Ok(())
    } else {
        warn!("No storage primal available, assignment not persisted");
        Ok(())
    }
}
```

**Tests**:
- `test_persist_assignment_success`
- `test_persist_assignment_no_nestgate`
- `test_load_persisted_assignments`

**Deliverable**: Assignment persistence working

---

### **Task 6: UI Feedback (petalTongue)** (Week 2, Day 4-5)

**Goal**: Push topology update to UI and show success feedback

**Implementation**:
```rust
async fn update_ui_topology(
    &self,
    event: UIEvent,
) -> Result<()> {
    if let Some(petaltongue) = &self.petaltongue {
        // Get current topology
        let state = self.state.read().await;
        let topology = &state.topology;
        
        // Push update to petalTongue
        petaltongue.push_topology_update(topology).await?;
        
        // Optionally show notification
        if let UIEvent::AssignmentCreated(assignment) = event {
            petaltongue.show_notification(&format!(
                "Device {} assigned to primal {}",
                assignment.device_id,
                assignment.primal_id
            )).await?;
        }
        
        info!("UI updated");
        
        Ok(())
    } else {
        debug!("No visualization primal available, skipping UI update");
        Ok(())
    }
}
```

**Tests**:
- `test_update_ui_topology_success`
- `test_update_ui_topology_no_petaltongue`
- `test_ui_notification_shown`

**Deliverable**: UI updates working

---

### **Task 7: Comprehensive Error Handling** (Week 3, Day 1-2)

**Goal**: Handle all error cases gracefully with clear user feedback

**Error Cases**:
1. Authorization denied → "You don't have permission to assign this device"
2. Device already assigned → "Device is already assigned to X"
3. Primal unhealthy → "Cannot assign to unhealthy primal"
4. Capacity insufficient → "Primal doesn't have capacity for this device"
5. Registration failed → "Failed to register assignment, try again"
6. Persistence failed → "Assignment succeeded but not persisted (will be lost on restart)"

**Implementation**:
```rust
async fn handle_assign_device(&self, device_id: &str, primal_id: &str) -> Result<ActionResult> {
    // Phase 1: Authorization
    let auth_result = self.authorize_device_assignment(
        "current_user", // TODO: Get from session
        device_id,
        primal_id,
    ).await;
    
    match auth_result {
        Ok(AuthorizationResult::Authorized) => {}
        Ok(AuthorizationResult::Denied(reason)) => {
            return Ok(ActionResult::error(format!(
                "Authorization denied: {}",
                reason
            )));
        }
        Err(e) => {
            return Ok(ActionResult::error(format!(
                "Authorization check failed: {}",
                e
            )));
        }
    }
    
    // Phase 2: Validation
    let validation_result = self.validate_device_assignment(device_id, primal_id).await;
    
    match validation_result {
        Ok(ValidationResult::Valid) => {}
        Ok(ValidationResult::Invalid(reason)) => {
            return Ok(ActionResult::error(format!(
                "Validation failed: {}",
                reason
            )));
        }
        Err(e) => {
            return Ok(ActionResult::error(format!(
                "Validation check failed: {}",
                e
            )));
        }
    }
    
    // Phase 3: Capacity
    let capacity_result = self.check_primal_capacity(device_id, primal_id).await;
    
    match capacity_result {
        Ok(CapacityResult::Available { .. }) => {}
        Ok(CapacityResult::Insufficient { required, available }) => {
            return Ok(ActionResult::error(format!(
                "Insufficient capacity: requires {:?}, available {:?}",
                required, available
            )));
        }
        Err(e) => {
            warn!("Capacity check failed: {}, proceeding anyway", e);
        }
    }
    
    // Create assignment
    let assignment = Assignment {
        device_id: device_id.to_string(),
        primal_id: primal_id.to_string(),
        assigned_at: chrono::Utc::now(),
        assigned_by: "current_user".to_string(), // TODO
        metadata: serde_json::json!({}),
    };
    
    // Phase 4: Register
    let assignment_id = self.register_assignment(device_id, primal_id, &assignment).await?;
    
    // Phase 5: Persist (non-critical)
    if let Err(e) = self.persist_assignment(&assignment_id, &assignment).await {
        warn!("Failed to persist assignment: {}, continuing", e);
    }
    
    // Phase 6: Update UI (non-critical)
    if let Err(e) = self.update_ui_topology(UIEvent::AssignmentCreated(assignment)).await {
        warn!("Failed to update UI: {}, continuing", e);
    }
    
    Ok(ActionResult::success(format!(
        "Device {} assigned to primal {}",
        device_id, primal_id
    )))
}
```

**Tests**:
- `test_assign_device_full_flow_success`
- `test_assign_device_authorization_denied`
- `test_assign_device_validation_failed`
- `test_assign_device_capacity_insufficient`
- `test_assign_device_all_primals_available`
- `test_assign_device_no_primals_available`
- `test_assign_device_partial_primals`

**Deliverable**: Comprehensive error handling with clear user feedback

---

### **Task 8: Integration Testing** (Week 3, Day 3-5)

**Goal**: End-to-end integration tests with live primals

**Test Scenarios**:

1. **Happy Path**: All primals available, assignment succeeds
2. **Authorization Failure**: BearDog denies, clear error message
3. **Validation Failure**: Device already assigned, clear error message
4. **Capacity Failure**: ToadStool reports insufficient capacity
5. **Partial Primals**: Some primals missing, graceful degradation
6. **No Primals**: All primals missing, local state only
7. **Concurrent Assignments**: Multiple users assigning simultaneously
8. **Assignment Recovery**: Load persisted assignments after restart

**Implementation**:
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_device_assignment_flow() {
        // Start orchestrator
        let mut orchestrator = InteractiveUIOrchestrator::new("test-family").await.unwrap();
        orchestrator.start().await.unwrap();
        
        // Attempt assignment
        let result = orchestrator.handle_user_action(UserAction::AssignDevice {
            device_id: "gpu-0".to_string(),
            primal_id: "toadstool-1".to_string(),
        }).await.unwrap();
        
        // Verify success
        assert!(result.is_success());
        
        // Verify state updated
        let state = orchestrator.state().read().await;
        assert!(state.assignments.contains_key("gpu-0"));
    }
    
    // ... more integration tests
}
```

**Deliverable**: Comprehensive integration test suite

---

## 📊 **Success Criteria**

Phase 3 is complete when:

1. ✅ User can assign device via UI
2. ✅ Assignment goes through 6-phase validation
3. ✅ All error cases handled gracefully
4. ✅ Works with any combination of available primals
5. ✅ Works with zero primals available (local state)
6. ✅ Assignments persisted and recovered
7. ✅ UI provides clear feedback
8. ✅ Integration tests passing

---

## 🎯 **Timeline**

| Week | Days | Focus |
|------|------|-------|
| Week 1 | 1-2 | BearDog authorization |
| Week 1 | 3-4 | Songbird validation |
| Week 1 | 5 | ToadStool capacity check |
| Week 2 | 1-2 | Register assignment (Songbird) |
| Week 2 | 3 | Persist assignment (NestGate) |
| Week 2 | 4-5 | UI feedback (petalTongue) |
| Week 3 | 1-2 | Comprehensive error handling |
| Week 3 | 3-5 | Integration testing |

**Total**: 3 weeks (15 working days)

---

## 🚀 **Next Session Starting Point**

When implementing:

1. **Start here**: Read this plan
2. **Review orchestrator**: `crates/biomeos-ui/src/orchestrator.rs`
3. **Review spec**: `specs/INTERACTIVE_UI_SPEC.md`
4. **Implement Task 1**: BearDog authorization
5. **Write tests**: Test-driven development
6. **Iterate**: One task at a time

**Quick Start**:
```bash
# Review the orchestrator
cat crates/biomeos-ui/src/orchestrator.rs

# Start implementing authorization
code crates/biomeos-ui/src/orchestrator.rs

# Add authorization method after handle_assign_device()
```

---

## 💡 **Key Principles**

1. **Network Effect**: Every assignment coordinates 6 primals
2. **Graceful Degradation**: Works with 0-6 primals available
3. **TRUE PRIMAL**: Zero hardcoding, runtime discovery
4. **Clear Feedback**: User always knows what's happening
5. **Test-Driven**: Write tests first, then implementation

---

**Status**: ⏳ **READY TO START PHASE 3!**

🚀 **Let's implement multi-primal device assignment!** 🚀

