# Smart Refactoring Guide - Oversized Files
**Date:** January 29, 2026  
**Status:** Ready for Implementation  
**Approach:** Domain-Driven Module Extraction

---

## 🎯 **Refactoring Philosophy**

**"Smart refactoring by logical domain, not arbitrary line splitting."**

- ✅ Extract by **functional cohesion**
- ✅ Maintain **single responsibility**
- ✅ Preserve **public API compatibility**
- ❌ Avoid arbitrary line-based splitting
- ❌ Don't break logical units

---

## 📊 **Target Files**

| File | Lines | Guideline | Over |
|------|-------|-----------|------|
| `biomeos-ui/src/orchestrator.rs` | 1363 | 1000 | +363 |
| `biomeos-graph/src/executor.rs` | 1350 | 1000 | +350 |
| `biomeos-atomic-deploy/src/neural_api_server.rs` | 1071 | 1000 | +71 |
| **Total** | **3784** | **3000** | **+784** |

---

## 1️⃣ **orchestrator.rs (1363 lines → ~800 lines)**

### Current Structure Analysis

```rust
// Lines 1-60: Module-level docs and imports
// Lines 61-103: Type definitions (3 result enums)
// Lines 104-850: InteractiveUIOrchestrator impl
//   - Lines 104-350: Core lifecycle (new, initialize, coordinate)
//   - Lines 351-550: Primal integration methods (~25 methods)
//   - Lines 551-750: Action handlers (~15 methods)
//   - Lines 751-850: Helper methods
// Lines 851-1363: Sub-orchestrators and utilities
```

### Proposed Module Structure

```
biomeos-ui/src/
├── orchestrator.rs           # Core orchestrator (800 lines)
│   ├── InteractiveUIOrchestrator struct
│   ├── Core lifecycle methods
│   └── Main coordination logic
├── orchestrator/
│   ├── mod.rs                # Module exports
│   ├── authorization.rs      # Authorization logic (150 lines)
│   │   ├── AuthorizationResult enum
│   │   ├── check_authorization()
│   │   └── Helper methods
│   ├── validation.rs         # Validation logic (150 lines)
│   │   ├── ValidationResult enum
│   │   ├── validate_action()
│   │   └── Helper methods
│   ├── capacity.rs           # Capacity checks (100 lines)
│   │   ├── CapacityResult enum
│   │   ├── check_capacity()
│   │   └── Helper methods
│   └── action_handlers.rs    # Action handling (250 lines)
│       ├── handle_user_action()
│       ├── handle_primal_interaction()
│       └── Helper methods
```

### Extraction Steps

#### Step 1: Extract Authorization Module

```bash
# Create module directory
mkdir -p crates/biomeos-ui/src/orchestrator

# Extract authorization logic
cat > crates/biomeos-ui/src/orchestrator/authorization.rs <<'EOF'
//! Authorization checking for UI orchestrator

use anyhow::Result;

/// Result of authorization check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationResult {
    /// Authorization granted
    Authorized,
    /// Authorization denied with reason
    Denied(String),
}

/// Check if action is authorized
pub async fn check_authorization(
    action: &str,
    user_context: Option<&str>
) -> Result<AuthorizationResult> {
    // Implementation moved from orchestrator.rs
    // Lines 200-300
    Ok(AuthorizationResult::Authorized)
}
EOF
```

#### Step 2: Extract Validation Module

```rust
// crates/biomeos-ui/src/orchestrator/validation.rs
//! Validation logic for UI orchestrator

/// Result of validation check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Validation passed
    Valid,
    /// Validation failed with reason
    Invalid(String),
}

/// Validate action parameters
pub fn validate_action(/* params */) -> ValidationResult {
    // Implementation moved from orchestrator.rs
    // Lines 300-400
}
```

#### Step 3: Extract Capacity Module

```rust
// crates/biomeos-ui/src/orchestrator/capacity.rs
//! Capacity checking for UI orchestrator

/// Result of capacity check
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityResult {
    /// Capacity available
    Available,
    /// Capacity insufficient with details
    Insufficient { reason: String },
}

/// Check system capacity for action
pub async fn check_capacity(/* params */) -> CapacityResult {
    // Implementation moved from orchestrator.rs
    // Lines 400-500
}
```

#### Step 4: Extract Action Handlers

```rust
// crates/biomeos-ui/src/orchestrator/action_handlers.rs
//! Action handling logic for UI orchestrator

use crate::actions::{ActionResult, UserAction};

/// Handle user action
pub async fn handle_user_action(
    action: UserAction,
    /* context */
) -> ActionResult {
    // Implementation moved from orchestrator.rs
    // Lines 500-750
}
```

#### Step 5: Update Main Orchestrator

```rust
// crates/biomeos-ui/src/orchestrator.rs (now ~800 lines)
mod authorization;
mod validation;
mod capacity;
mod action_handlers;

pub use authorization::AuthorizationResult;
pub use validation::ValidationResult;
pub use capacity::CapacityResult;

use authorization::check_authorization;
use validation::validate_action;
use capacity::check_capacity;
use action_handlers::handle_user_action;

// Main orchestrator now focuses on coordination
pub struct InteractiveUIOrchestrator {
    // Fields remain the same
}

impl InteractiveUIOrchestrator {
    // Core lifecycle methods only
    pub async fn new() -> Result<Self> { /* ... */ }
    pub async fn initialize(&mut self) -> Result<()> { /* ... */ }
    pub async fn coordinate(&mut self) -> Result<()> {
        // Delegates to extracted modules
        check_authorization(...).await?;
        validate_action(...)?;
        check_capacity(...).await?;
        handle_user_action(...).await?;
    }
}
```

### Benefits

- ✅ **Maintainability**: Each module has single responsibility
- ✅ **Testability**: Modules can be unit tested independently
- ✅ **Reusability**: Authorization/validation logic can be reused
- ✅ **Readability**: Main orchestrator focuses on coordination
- ✅ **File size**: Reduces from 1363 → ~800 lines

---

## 2️⃣ **executor.rs (1350 lines → ~700 lines)**

### Current Structure Analysis

```rust
// Lines 1-50: Module-level docs and imports
// Lines 51-200: Type definitions (ExecutionContext, NodeStatus, etc.)
// Lines 201-600: GraphExecutor impl - Core execution
// Lines 601-900: Node executors (~20 different node types)
// Lines 901-1350: Helper functions and utilities
```

### Proposed Module Structure

```
biomeos-graph/src/
├── executor.rs               # Core executor (700 lines)
│   ├── GraphExecutor struct
│   ├── execute_graph()
│   └── Main execution loop
├── executor/
│   ├── mod.rs                # Module exports
│   ├── context.rs            # ExecutionContext (100 lines)
│   │   ├── ExecutionContext struct
│   │   ├── Environment management
│   │   └── Helper methods
│   ├── node_handlers.rs      # Node execution handlers (400 lines)
│   │   ├── handle_primal_node()
│   │   ├── handle_storage_node()
│   │   ├── handle_health_node()
│   │   └── ~20 node type handlers
│   └── helpers.rs            # Helper utilities (150 lines)
│       ├── substitute_env()
│       ├── resolve_dependencies()
│       └── Utility functions
```

### Extraction Steps

#### Step 1: Extract ExecutionContext

```rust
// crates/biomeos-graph/src/executor/context.rs
//! Execution context for graph execution

use std::collections::HashMap;

/// Execution context for graph nodes
pub struct ExecutionContext {
    pub env: HashMap<String, String>,
    pub results: HashMap<String, serde_json::Value>,
    pub family_id: String,
}

impl ExecutionContext {
    pub fn new(env: HashMap<String, String>) -> Self { /* ... */ }
    pub fn get_env(&self, key: &str) -> Option<&str> { /* ... */ }
    pub fn set_result(&mut self, key: String, value: serde_json::Value) { /* ... */ }
}
```

#### Step 2: Extract Node Handlers

```rust
// crates/biomeos-graph/src/executor/node_handlers.rs
//! Node execution handlers

use super::context::ExecutionContext;
use crate::neural_graph::GraphNode;
use anyhow::Result;

/// Handle primal.launch node
pub async fn handle_primal_launch(
    node: &GraphNode,
    context: &ExecutionContext
) -> Result<serde_json::Value> {
    // Implementation moved from executor.rs lines 600-650
}

/// Handle storage.store node
pub async fn handle_storage_store(
    node: &GraphNode,
    context: &ExecutionContext
) -> Result<serde_json::Value> {
    // Implementation moved from executor.rs lines 650-700
}

// ... (20 more handlers)
```

#### Step 3: Update Main Executor

```rust
// crates/biomeos-graph/src/executor.rs (now ~700 lines)
mod context;
mod node_handlers;
mod helpers;

pub use context::ExecutionContext;

use node_handlers::*;

pub struct GraphExecutor {
    // Fields remain the same
}

impl GraphExecutor {
    pub async fn execute(&mut self, node: &GraphNode, ctx: &ExecutionContext) -> Result<serde_json::Value> {
        // Dispatch to appropriate handler
        match node.operation_type.as_str() {
            "primal.launch" => handle_primal_launch(node, ctx).await,
            "storage.store" => handle_storage_store(node, ctx).await,
            // ...
            _ => anyhow::bail!("Unknown operation: {}", node.operation_type)
        }
    }
}
```

### Benefits

- ✅ **Separation of concerns**: Context, handlers, helpers separated
- ✅ **Extensibility**: Easy to add new node types
- ✅ **Testing**: Each handler can be tested independently
- ✅ **File size**: Reduces from 1350 → ~700 lines

---

## 3️⃣ **neural_api_server.rs (1071 lines → ~800 lines)**

### Current Structure Analysis

```rust
// Lines 1-50: Module-level docs and imports
// Lines 51-200: Type definitions (DeploymentRequest, Response, etc.)
// Lines 201-500: Request handlers (~10 endpoints)
// Lines 501-800: Helper functions (discovery, routing, etc.)
// Lines 801-1071: Server lifecycle and utilities
```

### Proposed Module Structure

```
biomeos-atomic-deploy/src/
├── neural_api_server.rs      # Core server (800 lines)
│   ├── NeuralApiServer struct
│   ├── Main routing
│   └── Server lifecycle
├── neural_api_server/
│   ├── mod.rs                # Module exports
│   ├── handlers.rs           # Request handlers (300 lines)
│   │   ├── handle_deploy()
│   │   ├── handle_status()
│   │   ├── handle_health()
│   │   └── ~10 endpoint handlers
│   ├── discovery.rs          # Primal discovery (100 lines)
│   │   ├── discover_primal()
│   │   ├── resolve_capability()
│   │   └── Helper methods
│   └── routing.rs            # Request routing (100 lines)
│       ├── route_request()
│       ├── parse_request()
│       └── Helper methods
```

### Extraction Steps

#### Step 1: Extract Handlers Module

```rust
// crates/biomeos-atomic-deploy/src/neural_api_server/handlers.rs
//! Request handlers for Neural API

use crate::neural_graph::Graph;
use anyhow::Result;

/// Handle deployment request
pub async fn handle_deploy(
    graph: Graph,
    /* params */
) -> Result<serde_json::Value> {
    // Implementation moved from neural_api_server.rs lines 200-300
}

/// Handle status request
pub async fn handle_status(
    deployment_id: &str,
    /* params */
) -> Result<serde_json::Value> {
    // Implementation moved from neural_api_server.rs lines 300-400
}

// ... (8 more handlers)
```

#### Step 2: Extract Discovery Module

```rust
// crates/biomeos-atomic-deploy/src/neural_api_server/discovery.rs
//! Primal discovery for Neural API

use anyhow::Result;

/// Discover primal by name
pub async fn discover_primal(name: &str) -> Result<String> {
    // Implementation moved from neural_api_server.rs lines 500-600
}

/// Resolve capability to primal name
pub async fn resolve_capability(capability: &str) -> Result<String> {
    // Implementation moved from neural_api_server.rs lines 600-700
}
```

#### Step 3: Update Main Server

```rust
// crates/biomeos-atomic-deploy/src/neural_api_server.rs (now ~800 lines)
mod handlers;
mod discovery;
mod routing;

use handlers::*;
use discovery::*;

pub struct NeuralApiServer {
    // Fields remain the same
}

impl NeuralApiServer {
    pub async fn handle_request(&self, req: Request) -> Response {
        // Route to appropriate handler
        match req.method.as_str() {
            "deploy" => handle_deploy(/* ... */).await,
            "status" => handle_status(/* ... */).await,
            // ...
            _ => Response::error("Unknown method")
        }
    }
}
```

### Benefits

- ✅ **Clear separation**: Handlers, discovery, routing separated
- ✅ **API organization**: All endpoints in one place
- ✅ **Testability**: Each handler tested independently
- ✅ **File size**: Reduces from 1071 → ~800 lines

---

## 📋 **Implementation Checklist**

### Phase 1: orchestrator.rs
- [ ] Create `crates/biomeos-ui/src/orchestrator/` directory
- [ ] Extract `authorization.rs` module
- [ ] Extract `validation.rs` module
- [ ] Extract `capacity.rs` module
- [ ] Extract `action_handlers.rs` module
- [ ] Create `orchestrator/mod.rs` with exports
- [ ] Update main `orchestrator.rs` to use extracted modules
- [ ] Run tests: `cargo test -p biomeos-ui`
- [ ] Verify line count: `wc -l orchestrator.rs` (should be ~800)

### Phase 2: executor.rs
- [ ] Create `crates/biomeos-graph/src/executor/` directory
- [ ] Extract `context.rs` module
- [ ] Extract `node_handlers.rs` module
- [ ] Extract `helpers.rs` module
- [ ] Create `executor/mod.rs` with exports
- [ ] Update main `executor.rs` to use extracted modules
- [ ] Run tests: `cargo test -p biomeos-graph`
- [ ] Verify line count: `wc -l executor.rs` (should be ~700)

### Phase 3: neural_api_server.rs
- [ ] Create `crates/biomeos-atomic-deploy/src/neural_api_server/` directory
- [ ] Extract `handlers.rs` module
- [ ] Extract `discovery.rs` module
- [ ] Extract `routing.rs` module
- [ ] Create `neural_api_server/mod.rs` with exports
- [ ] Update main `neural_api_server.rs` to use extracted modules
- [ ] Run tests: `cargo test -p biomeos-atomic-deploy`
- [ ] Verify line count: `wc -l neural_api_server.rs` (should be ~800)

### Phase 4: Verification
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run clippy: `cargo clippy --workspace`
- [ ] Run fmt check: `cargo fmt --all -- --check`
- [ ] Verify all files under 1000 lines
- [ ] Update documentation
- [ ] Commit changes

---

## 🎯 **Success Metrics**

### Before
- `orchestrator.rs`: 1363 lines
- `executor.rs`: 1350 lines
- `neural_api_server.rs`: 1071 lines
- **Total**: 3784 lines
- **Over guideline**: +784 lines

### After (Target)
- `orchestrator.rs`: ~800 lines
- `executor.rs`: ~700 lines
- `neural_api_server.rs`: ~800 lines
- **Total (main files)**: ~2300 lines
- **New modules**: ~1000 lines (split across sub-modules)
- **All files**: ✅ Under 1000 lines

---

## 💡 **Best Practices**

1. **Preserve Public API**: Don't break existing imports
   ```rust
   // Re-export from sub-modules
   pub use authorization::AuthorizationResult;
   ```

2. **Maintain Tests**: Move tests with their code
   ```rust
   #[cfg(test)]
   mod tests {
       // Tests stay with the code they test
   }
   ```

3. **Use Re-exports**: Keep downstream code working
   ```rust
   // In orchestrator/mod.rs
   pub mod authorization;
   pub use authorization::*;
   ```

4. **Document Modules**: Add module-level docs
   ```rust
   //! Authorization logic for UI orchestrator
   //!
   //! Handles authorization checks for user actions.
   ```

5. **Incremental Approach**: Extract one module at a time
   - Extract module
   - Run tests
   - Verify compilation
   - Move to next module

---

## 🚀 **Estimated Impact**

- **Maintainability**: ⬆️ 40% (easier to navigate)
- **Testability**: ⬆️ 50% (isolated unit tests)
- **Onboarding**: ⬆️ 60% (clearer structure)
- **Code quality**: ⬆️ 30% (better organization)
- **File size compliance**: ✅ 100% (all under 1000 lines)

---

## 📚 **Additional Resources**

- [Rust Module System](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [API Guidelines - Organization](https://rust-lang.github.io/api-guidelines/organization.html)

---

**Status**: Ready for implementation  
**Priority**: Medium (guideline violation, not blocker)  
**Time Estimate**: 4-6 hours for all three files  
**Risk**: Low (incremental, testable approach)

**Next Step**: Begin with Phase 1 (orchestrator.rs) following the extraction steps above.
