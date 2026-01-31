# 📐 executor.rs Smart Refactoring Plan - TRUE ecoBin v2.0

**Document Version:** 1.0  
**Date:** January 30, 2026  
**Current File Size:** 1273 lines  
**Target:** <300 lines per module  
**Priority:** HIGH - Maintainability improvement

---

## 🎯 **Refactoring Philosophy**

**Smart Refactoring Principles:**
1. **Domain-Driven:** Organize by business logic, not arbitrary splits
2. **Cohesion:** Keep related functionality together
3. **Reusability:** Extract common patterns
4. **Testability:** Each module should be independently testable
5. **Discoverability:** Clear naming and structure

**NOT Just Splitting:** We're improving architecture, not just reducing line counts.

---

## 📊 **Current State Analysis**

### **File Structure (executor.rs - 1273 lines)**

```
Lines 1-108:    Main executor logic (GraphExecutor, execute())
Lines 109-171:  Phase execution (execute_phase())
Lines 172-198:  Node dispatcher (execute_node())
Lines 200-236:  Node: filesystem.check_exists
Lines 237-384:  Node: crypto.derive_child_seed (BearDog delegation)
Lines 385-500:  Node: primal.launch (service management)
Lines 501-609:  Node: health.check_atomic
Lines 610-679:  Node: lineage.verify_siblings
Lines 680-750:  Node: report.deployment_success
Lines 751+:     Helper functions (discover_beardog_socket, substitute_env, etc.)
```

### **Existing Modules (executor/ directory)**

✅ **Already Created:**
- `context.rs` (4.7KB) - ExecutionContext, NodeStatus
- `monitoring.rs` (3.5KB) - ExecutionReport, PhaseResult
- `rollback.rs` (8.8KB) - RollbackManager, RollbackAction
- `topological.rs` (7.3KB) - TopologicalSorter
- `mod.rs` (1.7KB) - Module exports

**Total Existing:** ~26KB in 5 modules

---

## 🏗️ **Target Architecture**

### **New Module Structure**

```
crates/biomeos-graph/src/
├── executor.rs                    (150 lines) - Public API only
└── executor/
    ├── mod.rs                     (100 lines) - Module exports
    ├── context.rs                 (150 lines) ✅ EXISTS
    ├── monitoring.rs              (120 lines) ✅ EXISTS
    ├── rollback.rs                (250 lines) ✅ EXISTS
    ├── topological.rs             (200 lines) ✅ EXISTS
    │
    ├── core.rs                    (250 lines) 🆕 - Main execution
    │   • GraphExecutor struct
    │   • execute() method
    │   • execute_phase() method
    │   • execute_node() dispatcher
    │
    ├── helpers.rs                 (150 lines) 🆕 - Utilities
    │   • substitute_env()
    │   • discover_beardog_socket()
    │   • discover_service_socket()
    │   • parse_node_config()
    │
    └── nodes/                     🆕 - Node executors by domain
        ├── mod.rs                 (50 lines) - Node exports
        ├── filesystem.rs          (100 lines) - File operations
        ├── crypto.rs              (180 lines) - Crypto (BearDog delegation)
        ├── primal.rs              (150 lines) - Primal launch/management
        ├── health.rs              (130 lines) - Health checks
        ├── lineage.rs             (100 lines) - Lineage verification
        └── report.rs              (80 lines) - Deployment reports
```

**Total:** ~2000 lines organized into 15 focused modules

---

## 📝 **Detailed Refactoring Steps**

### **Phase 1: Create Core Module** (1 hour)

**Goal:** Extract main execution logic

**Create `executor/core.rs`:**

```rust
//! Core graph execution logic
//!
//! This module contains the main GraphExecutor and orchestration logic.

use super::*;
use crate::graph::{Graph, GraphNode};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info};

/// Main graph executor
pub struct GraphExecutor {
    graph: Graph,
    context: ExecutionContext,
    max_parallelism: usize,
}

impl GraphExecutor {
    /// Create new graph executor
    pub fn new(graph: Graph, env: HashMap<String, String>) -> Self {
        Self {
            graph,
            context: ExecutionContext::new(env),
            max_parallelism: 3,
        }
    }

    /// Execute the entire graph
    pub async fn execute(&mut self) -> Result<ExecutionReport> {
        info!("🚀 Starting graph execution: {}", self.graph.id);
        
        let start_time = std::time::Instant::now();
        let mut report = ExecutionReport::new(self.graph.id.clone());

        // Topological sort
        let phases = TopologicalSorter::sort(&self.graph)?;
        info!("   Execution plan: {} phases", phases.len());

        // Execute each phase
        for (phase_num, phase_nodes) in phases.iter().enumerate() {
            info!("📍 Phase {}/{}: {} nodes", phase_num + 1, phases.len(), phase_nodes.len());

            match self.execute_phase(phase_nodes).await {
                Ok(phase_results) => {
                    report.phase_results.push(phase_results);
                }
                Err(e) => {
                    error!("❌ Phase {} failed: {}", phase_num + 1, e);
                    report.success = false;
                    report.error = Some(e.to_string());

                    // Rollback if enabled
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

    /// Execute a single phase (parallel execution)
    async fn execute_phase(&mut self, nodes: &[String]) -> Result<PhaseResult> {
        let phase_start = std::time::Instant::now();
        let mut phase_result = PhaseResult::new(nodes.len());

        // Semaphore for max parallelism
        let semaphore = Arc::new(Semaphore::new(self.max_parallelism));

        // Execute nodes in parallel
        let mut handles = Vec::new();

        for node_id in nodes {
            let node = self.graph.nodes.iter()
                .find(|n| &n.id == node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node_id))?
                .clone();

            let context = self.context.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            let handle = tokio::spawn(async move {
                let result = execute_node(&node, &context).await;
                drop(permit);
                (node.id.clone(), result)
            });

            handles.push(handle);
        }

        // Wait for all nodes
        for handle in handles {
            let (node_id, result) = handle.await?;
            // ... (status handling)
        }

        phase_result.duration_ms = phase_start.elapsed().as_millis() as u64;
        
        if phase_result.failed > 0 {
            anyhow::bail!("Phase failed: {} nodes failed", phase_result.failed);
        }

        Ok(phase_result)
    }
}

/// Execute a single node (dispatcher)
pub async fn execute_node(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    debug!("   Executing node: {}", node.id);

    // Mark as running
    context.set_status(&node.id, NodeStatus::Running).await;

    // Dispatch to node-specific executor
    use crate::executor::nodes;
    
    let result = match node.node_type.as_str() {
        "filesystem.check_exists" => nodes::filesystem::check_exists(node, context).await,
        "crypto.derive_child_seed" => nodes::crypto::derive_child_seed(node, context).await,
        "primal.launch" => nodes::primal::launch(node, context).await,
        "health.check_atomic" => nodes::health::check_atomic(node, context).await,
        "lineage.verify_siblings" => nodes::lineage::verify_siblings(node, context).await,
        "report.deployment_success" => nodes::report::deployment_success(node, context).await,
        _ => {
            warn!("Unknown node type: {}, skipping", node.node_type);
            Ok(serde_json::json!({"skipped": true}))
        }
    };

    result.context(format!("Node execution failed: {}", node.id))
}
```

**Result:** Core logic is now in a focused 250-line module! ✅

---

### **Phase 2: Create Helpers Module** (30 min)

**Goal:** Extract common utility functions

**Create `executor/helpers.rs`:**

```rust
//! Helper functions for graph execution
//!
//! Common utilities used across node executors.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

/// Substitute environment variables in a string
///
/// Replaces `${VAR_NAME}` with values from the environment map.
pub fn substitute_env(input: &str, env: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    
    for (key, value) in env {
        let pattern = format!("${{{}}}", key);
        result = result.replace(&pattern, value);
    }
    
    result
}

/// Discover BearDog socket path
///
/// Uses capability-based discovery (no hardcoding).
pub fn discover_beardog_socket(env: &HashMap<String, String>) -> Result<String> {
    // Try environment variable first
    if let Some(socket) = env.get("BEARDOG_SOCKET") {
        return Ok(socket.clone());
    }
    
    // Try XDG runtime directory
    if let Some(xdg_runtime) = env.get("XDG_RUNTIME_DIR") {
        let socket = format!("{}/biomeos/beardog.sock", xdg_runtime);
        return Ok(socket);
    }
    
    // Fallback to discovery (NOT hardcoded localhost!)
    anyhow::bail!("Could not discover BearDog socket. Set BEARDOG_SOCKET or XDG_RUNTIME_DIR.");
}

/// Discover any primal socket by name
pub fn discover_primal_socket(
    primal_name: &str,
    env: &HashMap<String, String>,
) -> Result<String> {
    // Try primal-specific env var
    let env_var = format!("{}_SOCKET", primal_name.to_uppercase());
    if let Some(socket) = env.get(&env_var) {
        return Ok(socket.clone());
    }
    
    // Try XDG runtime directory
    if let Some(xdg_runtime) = env.get("XDG_RUNTIME_DIR") {
        let socket = format!("{}/biomeos/{}.sock", xdg_runtime, primal_name);
        return Ok(socket);
    }
    
    anyhow::bail!(
        "Could not discover {} socket. Set {} or XDG_RUNTIME_DIR.",
        primal_name,
        env_var
    );
}

/// Parse a node config value as a specific type
pub fn parse_config<T>(
    node_config: &serde_json::Value,
    key: &str,
) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    node_config
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("Missing config key: {}", key))?
        .clone()
        .try_into()
        .context(format!("Failed to parse config key: {}", key))
}
```

**Result:** Reusable helpers in a clean 150-line module! ✅

---

### **Phase 3: Create Node Executor Modules** (2 hours)

**Goal:** Extract node executors by domain

#### **3.1: `executor/nodes/filesystem.rs`**

```rust
//! Filesystem node executors

use super::*;
use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;

/// Execute: filesystem.check_exists
pub async fn check_exists(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let path = parse_config::<String>(&node.config, "path")?;
    let path = substitute_env(&path, &context.env);
    let path = PathBuf::from(path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Check size if specified
    if let Some(expected_size) = node.config.get("expected_size").and_then(|v| v.as_u64()) {
        let metadata = std::fs::metadata(&path)?;
        if metadata.len() != expected_size {
            anyhow::bail!(
                "File size mismatch: expected {}, got {}",
                expected_size,
                metadata.len()
            );
        }
    }

    Ok(serde_json::json!({
        "exists": true,
        "path": path.to_string_lossy()
    }))
}
```

#### **3.2: `executor/nodes/crypto.rs`**

```rust
//! Crypto node executors (delegates to BearDog)

use super::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Execute: crypto.derive_child_seed
///
/// Delegates to BearDog primal via JSON-RPC over Unix socket.
pub async fn derive_child_seed(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // Extract parameters
    let parent_seed = parse_config::<String>(&node.config, "parent_seed")?;
    let parent_seed = substitute_env(&parent_seed, &context.env);
    
    let node_id = parse_config::<String>(&node.config, "node_id")?;
    let output_path = parse_config::<String>(&node.config, "output_path")?;
    let output_path = substitute_env(&output_path, &context.env);

    // Discover BearDog (capability-based)
    let beardog_socket = discover_beardog_socket(&context.env)?;

    debug!("Calling BearDog for seed derivation: node_id={}", node_id);

    // Connect to BearDog
    let stream = UnixStream::connect(&beardog_socket).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Prepare JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.derive_child_seed",
        "params": {
            "parent_seed": parent_seed,
            "node_id": node_id,
            "output_path": output_path,
        },
        "id": 1
    });

    // Send request
    let request_str = serde_json::to_string(&request)?;
    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // Read response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;
    
    let response: serde_json::Value = serde_json::from_str(&response_line)?;
    
    // Extract result
    response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("BearDog returned no result"))
}
```

#### **3.3: `executor/nodes/primal.rs`**

```rust
//! Primal lifecycle node executors

use super::*;
use std::process::Stdio;
use tokio::process::Command;

/// Execute: primal.launch
pub async fn launch(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let primal_name = parse_config::<String>(&node.config, "primal")?;
    let primal_bin = parse_config::<String>(&node.config, "binary_path")?;
    let primal_bin = substitute_env(&primal_bin, &context.env);

    info!("🚀 Launching primal: {}", primal_name);

    // Prepare environment
    let mut cmd = Command::new(&primal_bin);
    cmd.arg("server"); // UniBin mode
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Add environment variables
    for (key, value) in &context.env {
        cmd.env(key, value);
    }

    // Launch
    let child = cmd.spawn()?;
    let pid = child.id().ok_or_else(|| anyhow::anyhow!("Failed to get PID"))?;

    info!("   Primal {} started (PID: {})", primal_name, pid);

    // Wait for ready (check socket)
    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok(serde_json::json!({
        "primal": primal_name,
        "pid": pid,
        "status": "running"
    }))
}
```

#### **3.4-3.6: Other Node Modules**

Similar structure for:
- `health.rs` - Health check executors
- `lineage.rs` - Lineage verification
- `report.rs` - Deployment reporting

---

### **Phase 4: Update Main executor.rs** (30 min)

**Goal:** Reduce to public API only

**New `executor.rs` (150 lines):**

```rust
//! Graph executor for deterministic deployment orchestration
//!
//! **TRUE ecoBin v2.0 EVOLVED:** Smart refactoring into domain modules.
//!
//! This module provides the public API for graph execution.
//! Implementation details are in focused submodules.

use anyhow::Result;
use std::collections::HashMap;

// Re-export executor modules
mod executor;
pub use executor::{
    core::GraphExecutor,
    context::{ExecutionContext, NodeStatus},
    monitoring::{ExecutionReport, PhaseResult},
    rollback::{RollbackManager, RollbackAction},
    topological::TopologicalSorter,
};

/// Trait for executing operations on primals
#[async_trait::async_trait]
pub trait PrimalOperationExecutor: Send + Sync {
    async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
    ) -> Result<serde_json::Value>;
}

// Public API re-exports
pub use crate::graph::{Graph, GraphNode, Operation};

/// Execute a graph
pub async fn execute_graph(
    graph: Graph,
    env: HashMap<String, String>,
) -> Result<ExecutionReport> {
    let mut executor = GraphExecutor::new(graph, env);
    executor.execute().await
}
```

**Result:** Public API is now a clean 150-line file! ✅

---

## ✅ **Success Criteria**

- [ ] No module >300 lines
- [ ] All node executors extracted to `nodes/` by domain
- [ ] Helper functions in `helpers.rs`
- [ ] Main execution logic in `core.rs`
- [ ] Public API in `executor.rs` (thin wrapper)
- [ ] All tests pass
- [ ] `cargo check` succeeds
- [ ] No functionality changes (refactor only)

---

## 📈 **Before & After**

### **Before (Current)**
```
executor.rs: 1273 lines (monolithic)
executor/:
  - context.rs: 150 lines ✅
  - monitoring.rs: 120 lines ✅
  - rollback.rs: 250 lines ✅
  - topological.rs: 200 lines ✅
  - mod.rs: 50 lines ✅

Total: 2043 lines in 6 files
Largest file: 1273 lines ❌
```

### **After (Target)**
```
executor.rs: 150 lines (public API)
executor/:
  - core.rs: 250 lines (main execution)
  - helpers.rs: 150 lines (utilities)
  - context.rs: 150 lines ✅
  - monitoring.rs: 120 lines ✅
  - rollback.rs: 250 lines ✅
  - topological.rs: 200 lines ✅
  - mod.rs: 100 lines (exports)
  - nodes/
    - filesystem.rs: 100 lines
    - crypto.rs: 180 lines
    - primal.rs: 150 lines
    - health.rs: 130 lines
    - lineage.rs: 100 lines
    - report.rs: 80 lines
    - mod.rs: 50 lines

Total: 2010 lines in 15 files
Largest file: 250 lines ✅
```

---

## 🚀 **Implementation Timeline**

### **Session 1: Core & Helpers** (1.5 hours)
- [ ] Create `executor/core.rs`
- [ ] Create `executor/helpers.rs`
- [ ] Update `executor/mod.rs` exports
- [ ] Test compilation

### **Session 2: Node Modules** (2 hours)
- [ ] Create `executor/nodes/` directory
- [ ] Extract `filesystem.rs`
- [ ] Extract `crypto.rs`
- [ ] Extract `primal.rs`
- [ ] Test compilation

### **Session 3: Remaining Nodes** (1 hour)
- [ ] Extract `health.rs`
- [ ] Extract `lineage.rs`
- [ ] Extract `report.rs`
- [ ] Update `nodes/mod.rs`
- [ ] Test compilation

### **Session 4: Integration & Testing** (1 hour)
- [ ] Update main `executor.rs`
- [ ] Run all tests
- [ ] Fix any issues
- [ ] Update documentation

**Total Estimated Time:** 5.5 hours

---

## 💡 **Key Insights**

### **Why This is "Smart" Refactoring**

1. **Domain-Driven Organization:**
   - Node executors grouped by domain (filesystem, crypto, primal, health)
   - Clear separation of concerns
   - Easy to find related code

2. **Improved Testability:**
   - Each node executor can be tested independently
   - Helpers can be unit tested in isolation
   - Core logic is focused and testable

3. **Better Maintainability:**
   - New node types go in appropriate domain module
   - Helper functions are discoverable
   - Public API is stable

4. **Enhanced Reusability:**
   - `helpers.rs` functions can be used across modules
   - Node executors follow consistent patterns
   - Easy to extract common code

### **What We're NOT Doing**

❌ Arbitrarily splitting at line 500  
❌ Creating single-function modules  
❌ Breaking logical cohesion  
❌ Changing functionality

### **What We ARE Doing**

✅ Organizing by business domain  
✅ Extracting reusable utilities  
✅ Improving code discoverability  
✅ Maintaining logical cohesion

---

**📐 Smart refactoring creates architecture, not just smaller files!**

**Next Step:** Implement Phase 1 (Core & Helpers) in next session.
