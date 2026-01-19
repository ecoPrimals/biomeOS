# 🧠 biomeOS UniBin Evolution - Implementation Guide

**Date**: January 18, 2026  
**Goal**: Consolidate biomeOS binaries into single UniBin  
**Timeline**: ~1 week for UniBin, ~1 week for ecoBin  
**Status**: 🚀 **STARTING NOW!**

---

## 🎯 Objective

Transform biomeOS from **multiple separate binaries** to a **single UniBin** with mode-based execution, following the proven pattern of all 5 primals.

---

## 📊 Current State

### Multiple Binaries (NOT UniBin):
1. `biomeos` - CLI (40+ subcommands)
2. `neural-api-server` - Graph orchestrator
3. `neural-deploy` - Deployment executor  
4. `biomeos-api` - HTTP/WebSocket API
5. `verify-lineage` - Lineage verification

**Problem**: Can't deploy as single portable unit, violates UniBin standard.

---

## 🎯 Target State

### Single UniBin with Modes:

```bash
biomeos <mode> [args...]
```

**Modes**:
1. `cli` - CLI commands (default if no mode specified)
2. `neural-api` - Neural API server
3. `deploy` - Deployment executor
4. `api` - HTTP/WebSocket API server
5. `verify-lineage` - Lineage verification
6. `doctor` - Health diagnostics
7. `version` - Version information

---

## 🏗️ Architecture Design

### Entry Point Structure

```rust
// src/main.rs

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "biomeos")]
#[command(about = "🧠 biomeOS Universal Nucleus & Orchestrator")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// CLI mode - System management commands (default)
    Cli {
        #[command(subcommand)]
        command: CliCommand,
    },
    
    /// Neural API server mode - Graph-based orchestration
    NeuralApi {
        /// Graphs directory
        #[arg(long, default_value = "graphs")]
        graphs_dir: PathBuf,
        
        /// Family ID
        #[arg(long, default_value = "nat0")]
        family_id: String,
        
        /// Unix socket path
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    
    /// Deploy mode - Execute deployment graph
    Deploy {
        /// Graph file path
        graph: PathBuf,
        
        /// Validate only (don't execute)
        #[arg(short, long)]
        validate_only: bool,
        
        /// Dry run (show what would happen)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    
    /// API server mode - HTTP/WebSocket API
    Api {
        /// Port to bind (HTTP mode)
        #[arg(short, long)]
        port: Option<u16>,
        
        /// Unix socket path (Unix socket mode, preferred)
        #[arg(long)]
        socket: Option<PathBuf>,
        
        /// Disable HTTP, Unix socket only
        #[arg(long)]
        unix_only: bool,
    },
    
    /// Verify lineage - Validate genetic lineage
    VerifyLineage {
        /// Path to verify
        path: PathBuf,
        
        /// Detailed output
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Doctor mode - Health diagnostics
    Doctor {
        /// Detailed diagnostics
        #[arg(short, long)]
        detailed: bool,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Check specific subsystem
        #[arg(short, long)]
        subsystem: Option<String>,
    },
    
    /// Version information
    Version {
        /// Show detailed version info
        #[arg(short, long)]
        detailed: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging (configured per mode)
    init_logging(&cli.mode)?;
    
    // Dispatch to mode handler
    match cli.mode {
        Mode::Cli { command } => {
            cli_mode::run(command).await
        }
        Mode::NeuralApi { graphs_dir, family_id, socket } => {
            neural_api_mode::run(graphs_dir, family_id, socket).await
        }
        Mode::Deploy { graph, validate_only, dry_run } => {
            deploy_mode::run(graph, validate_only, dry_run).await
        }
        Mode::Api { port, socket, unix_only } => {
            api_mode::run(port, socket, unix_only).await
        }
        Mode::VerifyLineage { path, detailed } => {
            verify_lineage_mode::run(path, detailed).await
        }
        Mode::Doctor { detailed, format, subsystem } => {
            doctor_mode::run(detailed, format, subsystem).await
        }
        Mode::Version { detailed } => {
            version_mode::run(detailed).await
        }
    }
}
```

---

## 📁 Module Structure

### New Crate: `biomeos` (root binary)

```
crates/biomeos/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point (mode dispatch)
│   ├── modes/
│   │   ├── mod.rs       # Mode module exports
│   │   ├── cli.rs       # CLI mode (from biomeos-cli)
│   │   ├── neural_api.rs  # Neural API mode (from atomic-deploy)
│   │   ├── deploy.rs    # Deploy mode (from atomic-deploy)
│   │   ├── api.rs       # API mode (from biomeos-api)
│   │   ├── verify_lineage.rs  # Verify mode (from biomeos-cli)
│   │   ├── doctor.rs    # Doctor mode (new)
│   │   └── version.rs   # Version mode (new)
│   └── logging.rs       # Shared logging setup
```

### Dependencies:

```toml
[dependencies]
# Mode implementations (as libraries)
biomeos-cli = { path = "../biomeos-cli" }
biomeos-atomic-deploy = { path = "../biomeos-atomic-deploy" }
biomeos-api = { path = "../biomeos-api" }

# CLI framework
clap = { version = "4.0", features = ["derive"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## 🔄 Migration Plan

### Phase 1: Create Unified Binary (Days 1-2)

**Step 1**: Create new `crates/biomeos/` crate
```bash
mkdir -p crates/biomeos/src/modes
```

**Step 2**: Create `Cargo.toml`
```toml
[package]
name = "biomeos"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "biomeos"
path = "src/main.rs"

[dependencies]
# ... (see above)
```

**Step 3**: Implement mode dispatch (see architecture above)

**Step 4**: Create mode adapters:

```rust
// src/modes/cli.rs
use biomeos_cli::commands;
use anyhow::Result;

pub async fn run(command: commands::CliCommand) -> Result<()> {
    // Delegate to existing biomeos-cli implementation
    commands::execute(command).await
}
```

```rust
// src/modes/neural_api.rs
use biomeos_atomic_deploy::neural_api_server::NeuralApiServer;
use anyhow::Result;
use std::path::PathBuf;

pub async fn run(
    graphs_dir: PathBuf,
    family_id: String,
    socket: Option<PathBuf>,
) -> Result<()> {
    let socket_path = socket.unwrap_or_else(|| {
        PathBuf::from(format!("/tmp/neural-api-{}.sock", family_id))
    });
    
    let server = NeuralApiServer::new(graphs_dir, family_id, socket_path);
    server.serve().await
}
```

**Step 5**: Add to workspace:
```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/biomeos",  # NEW!
    "crates/biomeos-cli",
    # ... rest
]
```

---

### Phase 2: Convert Libraries (Day 2)

**Update existing crates to be libraries**:

**biomeos-cli/Cargo.toml**:
```toml
[package]
name = "biomeos-cli"
version = "0.1.0"

[lib]
name = "biomeos_cli"
path = "src/lib.rs"

# Remove [[bin]] entries (or mark as examples)
```

**biomeos-atomic-deploy/Cargo.toml**:
```toml
# Already a library, just ensure no bin targets
```

**biomeos-api/Cargo.toml**:
```toml
# Already a library, just ensure proper exports
```

---

### Phase 3: Testing (Day 3)

**Test all modes**:

```bash
# Build unified binary
cargo build --release --bin biomeos

# Test each mode
./target/release/biomeos cli discover
./target/release/biomeos neural-api --help
./target/release/biomeos deploy graphs/test.toml --validate-only
./target/release/biomeos api --unix-only
./target/release/biomeos verify-lineage /path/to/spore
./target/release/biomeos doctor --detailed
./target/release/biomeos version --detailed
```

---

### Phase 4: Documentation (Day 4)

**Update docs**:
1. Update `README.md` with new usage
2. Create `BIOMEOS_UNIBIN_MIGRATION_GUIDE.md`
3. Update deployment graphs
4. Update scripts (bin/pull-primals.sh, etc.)

---

### Phase 5: Integration (Day 5)

**Deploy testing**:
1. Update graphs to use `biomeos neural-api` instead of `neural-api-server`
2. Update systemd/supervisor configs
3. Test Neural API graph execution
4. Validate all modes in production-like environment

---

## 🎯 Doctor Mode Implementation

### New Feature: Health Diagnostics

```rust
// src/modes/doctor.rs

pub async fn run(
    detailed: bool,
    format: String,
    subsystem: Option<String>,
) -> Result<()> {
    let diagnostics = if let Some(subsys) = subsystem {
        check_subsystem(&subsys, detailed).await?
    } else {
        check_all_subsystems(detailed).await?
    };
    
    match format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&diagnostics)?),
        _ => print_diagnostics(&diagnostics),
    }
    
    Ok(())
}

async fn check_all_subsystems(detailed: bool) -> Result<Diagnostics> {
    let mut diag = Diagnostics::new();
    
    // 1. Binary Health
    diag.add_check("Binary", check_binary_health().await?);
    
    // 2. Configuration
    diag.add_check("Configuration", check_configuration().await?);
    
    // 3. Graphs Directory
    diag.add_check("Graphs", check_graphs_dir().await?);
    
    // 4. Primal Discovery
    diag.add_check("Primal Discovery", check_primal_discovery().await?);
    
    // 5. Neural API Socket
    diag.add_check("Neural API", check_neural_api_socket().await?);
    
    // 6. plasmidBin
    diag.add_check("PlasmidBin", check_plasmid_bin().await?);
    
    // 7. System Resources
    diag.add_check("System", check_system_resources().await?);
    
    if detailed {
        // Additional checks
        diag.add_check("Dependencies", check_dependencies().await?);
        diag.add_check("Permissions", check_permissions().await?);
    }
    
    Ok(diag)
}
```

**Output Example**:
```
🧠 biomeOS Doctor v0.1.0

Health Diagnostics:
═══════════════════════════════════════════════════════════════

✅ Binary Health
   • Binary: /usr/local/bin/biomeos (21M)
   • Version: 0.1.0
   • Modes: 7/7 available
   • Pure Rust: 100%

✅ Configuration
   • Config file: ~/.config/biomeos/config.toml
   • Valid: Yes
   • Family ID: nat0

✅ Graphs Directory
   • Path: ./graphs/
   • Graphs found: 5
   • Valid: 5/5

⚠️  Primal Discovery
   • BearDog: ✅ Found (/tmp/beardog.sock)
   • Songbird: ✅ Found (/tmp/songbird.sock)
   • Squirrel: ❌ Not found
   • NestGate: ✅ Found (/tmp/nestgate.sock)
   • ToadStool: ✅ Found (/tmp/toadstool.sock)

✅ Neural API
   • Socket: /tmp/neural-api-nat0.sock
   • Status: Ready

✅ PlasmidBin
   • Path: ./plasmidBin/primals/
   • Binaries: 5/5 present
   • Total size: 76M

✅ System Resources
   • Memory: 16GB (8GB available)
   • Disk: 500GB (350GB available)
   • Load: 1.5 (4 cores)

═══════════════════════════════════════════════════════════════
Overall Health: ⚠️  HEALTHY (1 warning)

Recommendations:
  • Start Squirrel primal for full functionality
```

---

## 📊 Success Criteria

### UniBin Compliance Checklist:

- [ ] Single `biomeos` binary
- [ ] 7+ modes (cli, neural-api, deploy, api, verify-lineage, doctor, version)
- [ ] Mode-based execution (no separate binaries)
- [ ] Comprehensive help system (`--help` for each mode)
- [ ] Professional CLI experience
- [ ] Version information (`biomeos version`)
- [ ] Doctor mode (health diagnostics)
- [ ] Backward compatibility (CLI commands unchanged)
- [ ] All tests passing
- [ ] Documentation updated

---

## 🎯 Timeline

### Week 1: UniBin Implementation

| Day | Tasks | Output |
|-----|-------|--------|
| **Mon** | Design & create crate structure | `crates/biomeos/` scaffolding |
| **Tue** | Implement mode dispatch & adapters | Basic modes working |
| **Wed** | Add doctor mode, testing | All modes functional |
| **Thu** | Documentation, deployment updates | Docs complete |
| **Fri** | Integration testing, polish | Production-ready |

**Result**: biomeOS UniBin v0.1.0 🎉

---

## 📚 Reference Implementations

### Follow Songbird Pattern:

**Songbird** (reference UniBin):
```bash
songbird server     # Server mode
songbird doctor     # Health diagnostics
songbird config     # Configuration
```

**biomeOS** (should be):
```bash
biomeos cli <cmd>       # CLI mode
biomeos neural-api      # Orchestrator mode
biomeos deploy <graph>  # Deploy mode
biomeos api             # API server mode
biomeos doctor          # Health diagnostics
```

---

## 🚀 Next Steps

### Immediate (Today):

1. **Create `crates/biomeos/` structure**
2. **Implement basic mode dispatch**
3. **Test CLI mode integration**

### Tomorrow:

4. **Implement all mode adapters**
5. **Add doctor mode**
6. **Integration testing**

### This Week:

7. **Documentation updates**
8. **Deployment graph updates**
9. **Production validation**
10. **Harvest to plasmidBin**

---

## 🏆 Bottom Line

**Goal**: Transform biomeOS from multi-bin to UniBin

**Timeline**: ~1 week (5 days)

**Complexity**: Medium (proven pattern available)

**Confidence**: High (all primals already UniBin)

**Result**: biomeOS as TRUE UniBin orchestrator!

---

**Status**: 🚀 **READY TO START!**  
**Next**: Create `crates/biomeos/` and implement mode dispatch!

🧠🦀✨ **biomeOS UniBin Evolution Begins!** ✨🦀🧠

