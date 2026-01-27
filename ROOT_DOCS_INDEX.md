# 🧬 biomeOS - Root Documentation Index

**Last Updated**: January 27, 2026  
**Status**: ✅ **Production Ready** - 93% TLS Validation  
**Tower Atomic**: Pure Rust TLS 1.3 Validated  
**LiveSpore**: Genetic Lineage Federation Ready  
**Tests**: 1,071 passing | **Crates**: 21 | **TODOs**: 52 remaining

---

## 🚀 **START HERE**

### **For Everyone**
1. **[START_HERE.md](START_HERE.md)** ⭐ Quick orientation & status
2. **[README.md](README.md)** - Project overview
3. **[DOCUMENTATION_HUB.md](DOCUMENTATION_HUB.md)** - Complete navigation

---

## 🏆 **January 26, 2026 Achievements**

### Tower Atomic Production Ready
- **87 sites tested** across 11 categories
- **93% TLS 1.3 success** (Pure Rust)
- **96% web compatibility** with User-Agent
- **100% cipher suite support** (SHA-256, SHA-384)

### Key Commits
| Primal | Commit | Feature |
|--------|--------|---------|
| BearDog | `964babd25` | SHA-384 evolution complete |
| Songbird | `eaa1dda9d` | Adaptive HTTP + User-Agent |
| Songbird | `478a3e622` | cipher_suite to finished_verify_data |
| Songbird | `7c974f6f7` | Chunked encoding fix |

---

## 📚 **CORE DOCUMENTATION**

### Architecture (5 files)
| File | Description |
|------|-------------|
| `BIOMEOS_ATOMICS_ARCHITECTURE.md` | Atomic deployment system |
| `BIOMEOS_PRIMAL_INTEGRATION_SPEC.md` | Primal integration patterns |
| `TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md` | Zero coupling pattern |
| `GENOMEBIN_ARCHITECTURE_STANDARD.md` | UniBin/ecoBin standards |
| `ISOMORPHIC_EVOLUTION.md` | Evolution principles |

### Deployment (3 files)
| File | Description |
|------|-------------|
| `deploy_tower_atomic.sh` | Production deployment script |
| `DEPLOYMENT.md` | General deployment guide |
| `QUICK_START.md` | Quick deployment |

### Evolution (3 files)
| File | Description |
|------|-------------|
| `SONGBIRD_EVOLUTION_HANDOFF.md` | TLS 1.2, server mode roadmap |
| `INFRASTRUCTURE_EVOLUTION.md` | Terraria, Apoptosis |
| `SEMANTIC_EVOLUTION_STRATEGY.md` | Semantic naming |

### Navigation (4 files)
| File | Description |
|------|-------------|
| `START_HERE.md` | Quick orientation ⭐ |
| `README.md` | Project overview |
| `DOCUMENTATION_HUB.md` | Main navigation hub |
| `ROOT_DOCS_INDEX.md` | This file |

---

## 📁 **DIRECTORY STRUCTURE**

```
biomeOS/
├── *.md                    # Root documentation (16 files)
├── deploy_tower_atomic.sh  # Production deployment
├── scripts/                # LiveSpore & deployment scripts ⭐
│   ├── create_sibling_spore.sh
│   ├── verify_sibling_lineage.sh
│   └── test_federation.sh
├── crates/                 # Rust crates (21)
│   ├── biomeos/            # UniBin main
│   ├── biomeos-atomic-deploy/  # Neural API ⭐
│   ├── biomeos-spore/      # LiveSpore system ⭐
│   └── ...
├── graphs/                 # Graph definitions
│   ├── tower_atomic_bootstrap.toml ⭐
│   ├── livespore_create.toml ⭐
│   └── federation_verify_lineage.toml
├── specs/                  # Technical specifications (60+)
├── archive/                # Historical docs (900+)
│   └── legacy_http_patterns_jan_27_2026/  # Archived HTTP patterns
├── plasmidBin/             # Deployed binaries
│   └── primals/            # BearDog, Songbird
└── tests/                  # Test files
```

---

## 🗂️ **SPECS ORGANIZATION**

```
specs/
├── README.md               # Specs overview ⭐
├── ARCHITECTURE_OVERVIEW.md
├── lifecycle/              # Lifecycle specs
│   ├── BIOMEOS_BOOTSTRAP_MODE.md
│   └── PRIMAL_LIFECYCLE_*.md
├── architecture/           # Architecture specs
│   └── BTSP_EVOLUTION_*.md
├── examples/               # YAML examples
└── archive/                # Historical specs
```

See `specs/README.md` for complete spec listing.

---

## 🗄️ **ARCHIVE STRUCTURE**

### Recent Sessions
| Archive | Date | Focus |
|---------|------|-------|
| `session_jan_26_2026_tls_analysis/` | Jan 26 | TLS validation (87 sites) |
| `session_jan_26_2026_tower_atomic/` | Jan 26 | Tower Atomic integration |
| `session_jan_25_2026_complete/` | Jan 25 | capability.call |
| `session_jan_25_2026_deep_debt/` | Jan 25 | Deep debt resolution |

### Historical
- `docs-fossil-record/` - 198 files
- `specs-fossil-record/` - 11 files
- `sessions/` - 124 files
- Total: 900+ archived files

---

## 🎯 **QUICK REFERENCE**

### Deploy Tower Atomic
```bash
./deploy_tower_atomic.sh        # Start
./deploy_tower_atomic.sh status # Check
./deploy_tower_atomic.sh stop   # Stop
```

### LiveSpore USB Deployment
```bash
# Create sibling spore from parent
./scripts/create_sibling_spore.sh /media/parent/biomeOS /media/newusb node-beta

# Verify genetic lineage
./scripts/verify_sibling_lineage.sh /media/usb1/biomeOS /media/usb2/biomeOS

# Test federation
./scripts/test_federation.sh
```

### Test JSON-RPC (TRUE PRIMAL)
```bash
# Health check via Unix socket
echo '{"jsonrpc":"2.0","method":"health.check","id":1}' | nc -U /tmp/beardog-nat0.sock

# Federation verify
echo '{"jsonrpc":"2.0","method":"federation.verify_family_member","params":{
  "family_id":"nat0","node_id":"node-beta"},"id":1}' | nc -U /tmp/beardog-nat0-node-alpha.sock
```

### Build
```bash
cargo build --release --workspace
cargo test --workspace
```

---

## 📊 **CURRENT STATUS**

| Metric | Value | Status |
|--------|-------|--------|
| **TLS 1.3 Validation** | 93% (81/87) | ✅ Production |
| **Web Compatibility** | 96% | ✅ Production |
| **Cipher Suites** | 100% | ✅ All 3 mandatory |
| **Pure Rust** | 100% | ✅ ecoBin |
| **LiveSpore** | Genetic Federation | ✅ Ready |
| **Tests** | 1,071 passing | ✅ |
| **Crates** | 21 | ✅ |
| **Lines of Code** | ~103k | ✅ |
| **Clippy** | 0 errors | ✅ Clean |
| **Formatting** | Clean | ✅ `cargo fmt` |
| **TODOs Remaining** | 52 | 📋 Ongoing |

---

## 🔄 **DOCUMENT LIFECYCLE**

### Root Directory
- **Active**: Current, essential documents only
- **Clean**: ~15 files
- **Updated**: After each major milestone

### Archive Directory
- **Historical**: Organized by date/topic
- **Complete**: Nothing deleted
- **Searchable**: `grep -r "term" archive/`

---

**Status**: ✅ Production Ready | **TLS**: 93% | **Archive**: Complete

*Start with [START_HERE.md](START_HERE.md) for quick orientation*
