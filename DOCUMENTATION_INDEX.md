# biomeOS Documentation Index

**Last Updated**: January 28, 2026 (Final)  
**Status**: ✅ Production Ready | **Tests**: 400+ | **Crates**: 21 | **TODOs**: 3

---

## 📚 Quick Navigation

| Start Here | Description |
|------------|-------------|
| **[START_HERE.md](START_HERE.md)** ⭐ | Quick orientation & current status |
| **[README.md](README.md)** | Project overview |
| **[DOCUMENTATION_HUB.md](DOCUMENTATION_HUB.md)** | Central navigation hub |
| **[PROTOCOL_ESCALATION_ROADMAP.md](PROTOCOL_ESCALATION_ROADMAP.md)** ⭐ | JSON-RPC → tarpc evolution |

---

## 📁 Key Directories

| Directory | Contents | Focus |
|-----------|----------|-------|
| `crates/` | 21 Rust crates | Core implementation |
| `specs/` | 67 specifications | Technical details |
| `graphs/` | TOML graphs | Deployment definitions |
| `scripts/` | Deployment scripts ⭐ | `bootstrap_tower_atomic.sh` |
| `docs/handoffs/` | Team handoffs | Evolution guides |
| `archive/` | 900+ files | Historical records |

---

## 🚀 Quick Commands

```bash
# Build
cargo build --release --workspace

# Test
cargo test --workspace

# Deploy Tower Atomic (recommended)
./scripts/bootstrap_tower_atomic.sh

# Legacy deployment
./deploy_tower_atomic.sh
```

---

## 📊 Current Metrics

| Metric | Value |
|--------|-------|
| **TLS 1.3 Success** | 93% (81/87 sites) |
| **Web Compatibility** | 96% |
| **Pure Rust** | 100% |
| **Protocol Escalation** | Phase 1 Complete |
| **Tests Passing** | 400+ (153 in atomic-deploy) |
| **Crates** | 21 |
| **Lines of Code** | ~108k |
| **Clippy Errors** | 0 |
| **TODOs Remaining** | 3 (external handoffs) |

---

**See [DOCUMENTATION_HUB.md](DOCUMENTATION_HUB.md) for complete navigation**
