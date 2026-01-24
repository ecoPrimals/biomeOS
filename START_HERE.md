# START HERE - biomeOS Quick Orientation

**Last Updated**: January 25, 2026

## 🎯 Current State

**Major Win**: ✅ **100% Pure Rust HTTPS Complete** (Songbird v5.12.6+)
- HTTP 200 OK from real servers
- Zero C dependencies
- Tower Atomic validated

**Current Work**: Deep debt resolution (4/9 tasks complete)
- ✅ 1,080+ tests passing
- ✅ reqwest removed from production
- ✅ Mocks isolated to tests
- ⏳ Large file refactoring in progress

**See Full Status**: [`STATUS_JAN_25_2026.md`](./STATUS_JAN_25_2026.md)

---

## 📋 Quick Navigation

### For New Developers
1. **Read This**: [`README.md`](./README.md) - Project overview
2. **Architecture**: See "Core Principles" section in README
3. **Integration**: [`BIOMEOS_PRIMAL_INTEGRATION_SPEC.md`](./BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)
4. **Standards**: [`../wateringHole/`](../../wateringHole/) - UniBin, ecoBin specs

### For Contributors
1. **Current Plan**: [`MASTER_EXECUTION_PLAN_JAN_24_2026.md`](./MASTER_EXECUTION_PLAN_JAN_24_2026.md)
2. **Deep Debt**: [`DEEP_DEBT_EXECUTION_BIOMEOS_JAN_24_2026.md`](./DEEP_DEBT_EXECUTION_BIOMEOS_JAN_24_2026.md)
3. **Test & Build**: See "Building" section below
4. **Code Style**: Follow Deep Debt Principles (see deep debt doc)

### For Primal Teams
1. **Songbird Team**: [`SONGBIRD_IPC_EVOLUTION_REQUIRED_JAN_25_2026.md`](./SONGBIRD_IPC_EVOLUTION_REQUIRED_JAN_25_2026.md)
2. **BearDog Team**: No current blockers (stable)
3. **Integration Spec**: [`BIOMEOS_PRIMAL_INTEGRATION_SPEC.md`](./BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)
4. **UniBin Standard**: [`../wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md`](../../wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md)

### Documentation Index
**Comprehensive index**: [`DOCS_INDEX.md`](./DOCS_INDEX.md)

---

## 🏗️ Building & Testing

### Quick Build
```bash
# Check compilation
cargo check --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint (pedantic)
cargo clippy --workspace -- -D warnings
```

### Current Status
- ✅ **Compilation**: Clean (0 errors)
- ✅ **Tests**: 1,080+ passing
- ⚠️ **Warnings**: ~30 minor linter warnings
- ✅ **Unsafe**: 0 blocks (100% safe Rust)

---

## 🎯 What to Work On

### High Priority (This Week)
1. **Large file refactoring**: Split `neural_executor.rs` (1,577 lines)
2. **Error handling**: Replace 50 panic! calls with Result<T,E>
3. **Unwrap reduction**: Reduce 517 unwrap() calls in production
4. **Test coverage**: Achieve 90% coverage (currently ~60%)

### Blocked (External)
- **Songbird IPC**: Waiting for Unix socket JSON-RPC interface
- **Neural API Deployment**: Blocked on Songbird IPC

### Ready (When Unblocked)
- **Tower Atomic Deployment**: Deploy BearDog + Songbird via Neural API
- **HTTPS Integration Tests**: End-to-end validation

---

## 📚 Key Concepts

### TRUE PRIMAL Architecture
- **Self-Knowledge Only**: Primals don't hardcode other primals' names/ports
- **Runtime Discovery**: Find services via capability (e.g., "security", "http")
- **Unix Socket First**: JSON-RPC 2.0 for inter-primal communication
- **Pure Rust**: Zero C dependencies for core functionality

### Tower Atomic Stack
```
Applications (Squirrel, etc.)
         ↓
    Songbird (TLS/HTTP)
         ↓
    BearDog (Crypto)
```
**Security Boundary**: All protocol translation happens here

### Neural API
- **Capability Mesh**: Route requests by capability, not hardcoded names
- **Semantic Translation**: Stable semantic names → provider methods
- **Evolution Engine**: Primals can evolve without breaking clients
- **Optional Orchestration**: Primals work independently too

---

## 🚀 Quick Start

### 1. Clone and Build
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo build --workspace --release
```

### 2. Run Tests
```bash
cargo test --workspace
```

### 3. Deploy a Primal (Example: BearDog)
```bash
# Via neural-api (when available)
target/release/biomeos neural-api &
target/release/biomeos deploy graphs/tower_atomic_bootstrap.toml

# Direct (for testing)
../beardog/target/release/beardog server --socket /tmp/beardog-nat0.sock
```

### 4. Make HTTP Request via Songbird (When IPC Ready)
```bash
# Via Unix socket JSON-RPC
echo '{"jsonrpc":"2.0","id":1,"method":"http.request","params":{"url":"https://example.com"}}' \
  | nc -U /tmp/songbird-nat0.sock
```

---

## 📖 Documentation Structure

### By Priority
1. **Start**: `STATUS_JAN_25_2026.md`, `README.md` (this file)
2. **Execute**: `MASTER_EXECUTION_PLAN_JAN_24_2026.md`
3. **Integrate**: `BIOMEOS_PRIMAL_INTEGRATION_SPEC.md`
4. **Deep Dive**: `DOCS_INDEX.md` → full documentation

### By Role
- **New Developer**: README → DOCS_INDEX → Integration Spec
- **Contributor**: Current Plan → Deep Debt Doc → Test
- **Primal Team**: Integration Spec → UniBin Standard → IPC Protocol
- **Architect**: Architecture docs in `archive/` folder

---

## ❓ Common Questions

### Q: Why is Songbird deployment blocked?
**A**: Songbird's HTTPS client works at the library level but isn't exposed via Unix socket JSON-RPC yet. The Songbird team is implementing `http.request` RPC method. See `SONGBIRD_IPC_EVOLUTION_REQUIRED_JAN_25_2026.md`.

### Q: How do primals communicate?
**A**: JSON-RPC 2.0 over Unix sockets. No HTTP, no ports, no hardcoded addresses. Environment variables or capability-based discovery resolve socket paths.

### Q: What's the difference between ecoBin and UniBin?
**A**: **UniBin** = single binary with subcommands. **ecoBin** = UniBin + Pure Rust (zero C deps). See `../wateringHole/` specs.

### Q: Can I use HTTP for primal communication?
**A**: Not in production. HTTP transport is deprecated and feature-gated. Use Unix sockets for IPC, delegate external HTTP to Songbird.

### Q: How do I add a new primal?
**A**: Follow the integration spec (`BIOMEOS_PRIMAL_INTEGRATION_SPEC.md`). Implement JSON-RPC server, declare capabilities, register with Neural API.

---

## 🔗 Important Links

- **Status**: [`STATUS_JAN_25_2026.md`](./STATUS_JAN_25_2026.md)
- **README**: [`README.md`](./README.md)
- **Docs Index**: [`DOCS_INDEX.md`](./DOCS_INDEX.md)
- **Integration Spec**: [`BIOMEOS_PRIMAL_INTEGRATION_SPEC.md`](./BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)
- **Master Plan**: [`MASTER_EXECUTION_PLAN_JAN_24_2026.md`](./MASTER_EXECUTION_PLAN_JAN_24_2026.md)

---

*Need help? Check the docs or ask in the team channel.*
