# 🔧 executor.rs Smart Refactoring Plan

**File:** `crates/biomeos-graph/src/executor.rs`  
**Current Size:** 1,350 lines  
**Status:** READY FOR REFACTORING  
**Approach:** Responsibility-based splitting (not just size)

---

## 📊 **Current Structure Analysis**

### **Identified Responsibilities**

| Responsibility | Line Range | Size | Description |
|---------------|------------|------|-------------|
| **Types & Context** | 1-115 | ~115 lines | NodeStatus, RollbackAction, ExecutionContext |
| **Core Execution** | 118-787 | ~670 lines | GraphExecutor, execute(), execute_phase() |
| **Dependency Resolution** | 788-855 | ~68 lines | topological_sort() |
| **Rollback Management** | 856-1059 | ~204 lines | rollback(), rollback_*() methods |
| **Reporting** | 1060-1350 | ~290 lines | ExecutionReport, PhaseResult |

**Total:** 1,350 lines across 5 major responsibilities

---

## 🎯 **Refactoring Strategy**

### **Principle: Responsibility-Based Splitting**

Split by **what the code does**, not by size. Each module should have a single, clear purpose.

### **Target Structure**

```
crates/biomeos-graph/src/
├── executor.rs (NEW - ~50 lines, re-exports only)
├── executor/
│   ├── mod.rs (~50 lines, module organization)
│   ├── types.rs (~120 lines, data types)
│   ├── context.rs (~100 lines, ExecutionContext)
│   ├── graph_executor.rs (~400 lines, core execution)
│   ├── dependency_resolver.rs (~80 lines, topological sort)
│   ├── rollback.rs (~250 lines, rollback management)
│   └── reporting.rs (~350 lines, reports & metrics)
```

**Result:** 1 large file → 7 focused modules  
**Average size:** ~193 lines per module  
**Max size:** ~400 lines (graph_executor - acceptable)

---

## 📋 **Detailed Refactoring Plan**

### **Step 1: Create Module Directory**

```bash
mkdir -p crates/biomeos-graph/src/executor
```

### **Step 2: Extract Types Module**

**File:** `executor/types.rs` (~120 lines)

**Contents:**
- `NodeStatus` enum
- `RollbackAction` enum  
- `PrimalOperationExecutor` trait

**Rationale:** Core data types used throughout executor

**Code:**
```rust
//! Execution types for graph deployment

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Execution status for a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed(serde_json::Value),
    Failed(String),
    Skipped,
}

/// Rollback action recorded during execution
#[derive(Debug, Clone)]
pub enum RollbackAction {
    StopProcess { primal: String, pid: u32, socket: String },
    RemoveFile { path: PathBuf },
    RemoveDir { path: PathBuf },
    JsonRpc { socket: String, method: String, params: serde_json::Value },
}

/// Trait for executing operations on primals
#[async_trait::async_trait]
pub trait PrimalOperationExecutor: Send + Sync {
    async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
    ) -> Result<serde_json::Value>;
}
```

---

### **Step 3: Extract Context Module**

**File:** `executor/context.rs` (~100 lines)

**Contents:**
- `ExecutionContext` struct
- `ExecutionContext` implementation
- Context management methods

**Rationale:** Shared execution state management

**Code:**
```rust
//! Execution context for graph deployment

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::{NodeStatus, RollbackAction};

/// Execution context shared across nodes
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub env: HashMap<String, String>,
    pub outputs: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    pub status: Arc<Mutex<HashMap<String, NodeStatus>>>,
    pub checkpoint_dir: Option<PathBuf>,
    pub rollback_actions: Arc<Mutex<Vec<(String, RollbackAction)>>>,
}

impl ExecutionContext {
    pub fn new(env: HashMap<String, String>) -> Self {
        // ... implementation
    }

    pub async fn record_rollback(&self, node_id: &str, action: RollbackAction) {
        // ... implementation
    }

    // ... other methods
}
```

---

### **Step 4: Extract Dependency Resolver**

**File:** `executor/dependency_resolver.rs` (~80 lines)

**Contents:**
- `topological_sort()` function
- Dependency resolution logic

**Rationale:** Single responsibility - resolve dependencies

**Code:**
```rust
//! Dependency resolution for graph execution

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use tracing::debug;

use crate::graph::{Graph, GraphNode};

/// Perform topological sort to determine execution phases
pub fn topological_sort(graph: &Graph) -> Result<Vec<Vec<String>>> {
    // Build dependency graph
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

    // ... implementation
    
    Ok(phases)
}
```

---

### **Step 5: Extract Rollback Management**

**File:** `executor/rollback.rs` (~250 lines)

**Contents:**
- `RollbackManager` struct
- `rollback()` method
- `rollback_stop_process()` method
- `rollback_remove_file()` method
- `rollback_remove_dir()` method
- `rollback_jsonrpc()` method

**Rationale:** All rollback logic in one place

**Code:**
```rust
//! Rollback management for failed deployments

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn, error};

use super::context::ExecutionContext;
use super::types::RollbackAction;

/// Manages rollback of failed deployments
pub struct RollbackManager<'a> {
    context: &'a ExecutionContext,
}

impl<'a> RollbackManager<'a> {
    pub fn new(context: &'a ExecutionContext) -> Self {
        Self { context }
    }

    pub async fn execute_rollback(&self) -> Result<()> {
        let actions = self.context.get_rollback_actions().await;
        
        info!("🔄 Rolling back {} actions", actions.len());
        
        for (node_id, action) in actions {
            // ... implementation
        }
        
        Ok(())
    }

    async fn rollback_stop_process(&self, primal: &str, pid: u32, socket: &str) -> Result<()> {
        // ... implementation
    }

    // ... other rollback methods
}
```

---

### **Step 6: Extract Reporting**

**File:** `executor/reporting.rs` (~350 lines)

**Contents:**
- `ExecutionReport` struct
- `PhaseResult` struct
- Report generation and formatting

**Rationale:** Reporting is a separate concern from execution

**Code:**
```rust
//! Execution reporting and metrics

use serde::{Deserialize, Serialize};

/// Execution report for the entire graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub graph_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub phase_results: Vec<PhaseResult>,
    pub error: Option<String>,
}

impl ExecutionReport {
    pub fn new(graph_id: String) -> Self {
        // ... implementation
    }
}

/// Result of a single execution phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub nodes_executed: usize,
    pub nodes_succeeded: usize,
    pub nodes_failed: usize,
    pub duration_ms: u64,
}

impl PhaseResult {
    pub fn new(nodes: usize) -> Self {
        // ... implementation
    }
}
```

---

### **Step 7: Refactor GraphExecutor**

**File:** `executor/graph_executor.rs` (~400 lines)

**Contents:**
- `GraphExecutor` struct
- `execute()` method
- `execute_phase()` method
- Node execution logic

**Rationale:** Core execution logic, uses all other modules

**Code:**
```rust
//! Core graph execution orchestration

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, error, warn};

use crate::graph::Graph;
use super::{
    types::{NodeStatus, PrimalOperationExecutor},
    context::ExecutionContext,
    dependency_resolver,
    rollback::RollbackManager,
    reporting::{ExecutionReport, PhaseResult},
};

/// Graph executor
pub struct GraphExecutor {
    graph: Graph,
    context: ExecutionContext,
    max_parallelism: usize,
}

impl GraphExecutor {
    pub fn new(graph: Graph, env: HashMap<String, String>) -> Self {
        Self {
            graph,
            context: ExecutionContext::new(env),
            max_parallelism: 3,
        }
    }

    pub async fn execute(&mut self) -> Result<ExecutionReport> {
        info!("🚀 Starting graph execution: {}", self.graph.id);

        let start_time = std::time::Instant::now();
        let mut report = ExecutionReport::new(self.graph.id.clone());

        // Use dependency resolver
        let phases = dependency_resolver::topological_sort(&self.graph)?;

        // Execute each phase
        for (phase_num, phase_nodes) in phases.iter().enumerate() {
            match self.execute_phase(phase_nodes).await {
                Ok(phase_results) => {
                    report.phase_results.push(phase_results);
                }
                Err(e) => {
                    error!("❌ Phase {} failed: {}", phase_num + 1, e);
                    report.success = false;
                    report.error = Some(e.to_string());

                    // Use rollback manager
                    if self.graph.config.rollback_on_failure {
                        let rollback_mgr = RollbackManager::new(&self.context);
                        rollback_mgr.execute_rollback().await?;
                    }

                    break;
                }
            }
        }

        report.duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(report)
    }

    async fn execute_phase(&mut self, nodes: &[String]) -> Result<PhaseResult> {
        // ... implementation
    }
}
```

---

### **Step 8: Create Module Organization**

**File:** `executor/mod.rs` (~50 lines)

**Contents:**
- Module declarations
- Re-exports

**Code:**
```rust
//! Graph executor module - Smart refactored for maintainability
//!
//! Organized by responsibility:
//! - `types` - Core data types
//! - `context` - Execution context management
//! - `dependency_resolver` - Topological sorting
//! - `rollback` - Rollback management
//! - `reporting` - Execution reports
//! - `graph_executor` - Core execution orchestration

pub mod types;
pub mod context;
pub mod dependency_resolver;
pub mod rollback;
pub mod reporting;
pub mod graph_executor;

// Re-export main types
pub use types::{NodeStatus, RollbackAction, PrimalOperationExecutor};
pub use context::ExecutionContext;
pub use reporting::{ExecutionReport, PhaseResult};
pub use graph_executor::GraphExecutor;
```

---

### **Step 9: Update Parent executor.rs**

**File:** `executor.rs` (~50 lines)

**Contents:**
- Re-exports from executor module

**Code:**
```rust
//! Graph executor for deterministic deployment orchestration
//!
//! **EVOLVED:** Smart-refactored into focused modules by responsibility.
//!
//! This module executes Neural API graphs with:
//! - Topological sorting for dependency resolution
//! - Parallel execution within phases
//! - Checkpoint/rollback support
//! - Live monitoring and metrics
//!
//! ## Module Organization
//!
//! - `executor::types` - Core data types (NodeStatus, RollbackAction)
//! - `executor::context` - Execution context management
//! - `executor::dependency_resolver` - Dependency resolution
//! - `executor::rollback` - Rollback management
//! - `executor::reporting` - Execution reports and metrics
//! - `executor::graph_executor` - Core execution orchestration

// Re-export all public types from the executor module
pub use self::executor::*;

mod executor;
```

---

## ✅ **Validation Steps**

### **1. Compilation**
```bash
cargo check --package biomeos-graph
```

### **2. Tests**
```bash
cargo test --package biomeos-graph executor
```

### **3. Line Count Verification**
```bash
find crates/biomeos-graph/src/executor -name "*.rs" -exec wc -l {} +
```

**Expected Output:**
```
  120 executor/types.rs
  100 executor/context.rs
   80 executor/dependency_resolver.rs
  250 executor/rollback.rs
  350 executor/reporting.rs
  400 executor/graph_executor.rs
   50 executor/mod.rs
 1350 total
```

### **4. Public API Unchanged**
```bash
# Verify all public exports still work
grep -r "use.*executor" crates/biomeos-graph/src/*.rs
```

---

## 📊 **Benefits**

### **Before Refactoring**

| Metric | Value |
|--------|-------|
| File size | 1,350 lines |
| Responsibilities | 5 mixed |
| Navigability | Poor |
| Testability | Difficult |
| Maintainability | Low |

### **After Refactoring**

| Metric | Value |
|--------|-------|
| Largest module | ~400 lines |
| Responsibilities | 1 per module |
| Navigability | Excellent |
| Testability | Easy (unit test each module) |
| Maintainability | High |

---

## 🎯 **Success Criteria**

- ✅ No file over 500 lines
- ✅ Single responsibility per module
- ✅ All tests pass
- ✅ Public API unchanged
- ✅ Clear module organization
- ✅ Improved maintainability

---

## 📝 **Implementation Notes**

### **Ordering**

1. Create directory structure
2. Extract types (no dependencies)
3. Extract context (depends on types)
4. Extract dependency_resolver (minimal dependencies)
5. Extract rollback (depends on context, types)
6. Extract reporting (minimal dependencies)
7. Refactor graph_executor (uses all modules)
8. Create mod.rs
9. Update parent executor.rs
10. Run tests and validation

### **Testing Strategy**

- Unit test each module independently
- Integration tests for GraphExecutor
- Preserve all existing test coverage
- Add module-specific tests if gaps found

### **Migration Safety**

- Create new modules first
- Keep old executor.rs until complete
- Verify compilation at each step
- Run tests after each major step
- Only delete old file when 100% complete

---

**Created:** January 30, 2026 (Evening)  
**Status:** READY FOR EXECUTION  
**Estimated Time:** 2-3 hours for careful implementation  
**Complexity:** Medium (clear responsibilities, good separation)

🔧🦀✨ **Smart Refactoring - Responsibility Over Size!** ✨🦀🔧
