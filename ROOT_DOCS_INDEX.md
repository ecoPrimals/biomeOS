# 📚 biomeOS Root Documentation Index

**Last Updated**: January 24, 2026  
**biomeOS Version**: 0.1.0 (TRUE ecoBin #5)

---

## 🎯 Start Here

| Document | Description | Audience |
|----------|-------------|----------|
| **[README.md](README.md)** | Project overview, quick start, architecture | Everyone |
| **[START_HERE.md](START_HERE.md)** | Orientation guide for new contributors | New developers |
| **[QUICK_START.md](QUICK_START.md)** | 5-minute deployment guide | Operators |

---

## 🏗️ Core Architecture

### Primal Integration
| Document | Description |
|----------|-------------|
| **[BIOMEOS_PRIMAL_INTEGRATION_SPEC.md](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)** | How primals interact with biomeOS |
| **[TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)** | Unix socket IPC architecture |

### Neural API & Deployment
| Document | Description |
|----------|-------------|
| **[BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md](BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md)** | Graph-based deployment system |
| **[NEURAL_API_IMPLEMENTATION_TRACKER.md](NEURAL_API_IMPLEMENTATION_TRACKER.md)** | Implementation status tracker |
| **[BIOMEOS_ATOMICS_ARCHITECTURE.md](BIOMEOS_ATOMICS_ARCHITECTURE.md)** | Tower Atomic patterns |

### Standards & Compliance
| Document | Description |
|----------|-------------|
| **[BIOMEOS_ECOBIN_CERTIFICATION_JAN_24_2026.md](BIOMEOS_ECOBIN_CERTIFICATION_JAN_24_2026.md)** | TRUE ecoBin #5 certification |
| **[GENOMEBIN_ARCHITECTURE_STANDARD.md](GENOMEBIN_ARCHITECTURE_STANDARD.md)** | genomeBin evolution standard |
| **[TOWER_ATOMIC_VALIDATION_AND_EVOLUTION.md](TOWER_ATOMIC_VALIDATION_AND_EVOLUTION.md)** | Pure Rust TLS validation |

---

## 📖 Operational Guides

### Deployment
| Document | Description |
|----------|-------------|
| **[DEPLOYMENT.md](DEPLOYMENT.md)** | Comprehensive deployment guide |
| **[QUICK_START_TOWER_DEPLOYMENT.md](QUICK_START_TOWER_DEPLOYMENT.md)** | Tower-specific quick start |

### Testing & Quality
| Document | Description |
|----------|-------------|
| **[TEST_COVERAGE_MILESTONE_JAN_24_2026.md](TEST_COVERAGE_MILESTONE_JAN_24_2026.md)** | Test coverage achievements |
| **[INTEGRATION_TEST_RESULTS.md](INTEGRATION_TEST_RESULTS.md)** | Integration test results |
| **[MULTI_DEVICE_BONDING_TESTS.md](MULTI_DEVICE_BONDING_TESTS.md)** | Multi-device testing |

### Code Quality
| Document | Description |
|----------|-------------|
| **[PRODUCTION_CODE_SLEEP_AUDIT.md](PRODUCTION_CODE_SLEEP_AUDIT.md)** | Production code audit |

---

## 🤝 Integration & Handoffs

| Document | Description |
|----------|-------------|
| **[HANDOFF_SONGBIRD_INTEGRATION_TESTING.md](HANDOFF_SONGBIRD_INTEGRATION_TESTING.md)** | Songbird integration handoff |
| **[HANDOFF_SQUIRREL_TOWER_INTEGRATION.md](HANDOFF_SQUIRREL_TOWER_INTEGRATION.md)** | Squirrel/Tower integration handoff |

---

## 📂 Directory Organization

### Primary Documentation Locations

```
biomeOS/
├── README.md                    # Main project README
├── ROOT_DOCS_INDEX.md          # This file
├── START_HERE.md               # New contributor guide
│
├── docs/                       # Comprehensive documentation
│   ├── architecture/           # Architecture docs
│   ├── api/                    # API documentation
│   ├── guides/                 # User guides
│   └── development/            # Development docs
│
├── specs/                      # Technical specifications
│   ├── primal-ipc/            # IPC protocol specs
│   ├── neural-api/            # Neural API specs
│   └── deployment/            # Deployment specs
│
├── graphs/                     # Deployment graphs (examples)
│   ├── nucleus_simple.toml
│   ├── nucleus_ecosystem.toml
│   └── README.md
│
├── examples/                   # Example code
│   ├── basic_deployment/
│   ├── custom_primal/
│   └── federation/
│
├── templates/                  # Primal templates
│   └── primal_template.yaml
│
└── archive/                    # Historical documentation
    ├── sessions/               # Development session logs
    │   └── 2026-01-24/        # Today's session docs
    └── docs-fossil-record/    # Old documentation
```

---

## 🔍 Finding What You Need

### By Role

#### **I'm an Operator/User**
1. Start with [README.md](README.md)
2. Follow [QUICK_START.md](QUICK_START.md)
3. Read [DEPLOYMENT.md](DEPLOYMENT.md)
4. Explore `graphs/` for deployment examples

#### **I'm a Developer**
1. Read [START_HERE.md](START_HERE.md)
2. Understand [BIOMEOS_PRIMAL_INTEGRATION_SPEC.md](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)
3. Study `docs/development/`
4. Check `examples/` for code samples

#### **I'm Writing a Primal**
1. Read [BIOMEOS_PRIMAL_INTEGRATION_SPEC.md](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)
2. Study [TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)
3. Use `templates/primal_template.yaml`
4. Reference `specs/primal-ipc/`

#### **I'm an Architect**
1. Review [BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md](BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md)
2. Study [BIOMEOS_ATOMICS_ARCHITECTURE.md](BIOMEOS_ATOMICS_ARCHITECTURE.md)
3. Understand [GENOMEBIN_ARCHITECTURE_STANDARD.md](GENOMEBIN_ARCHITECTURE_STANDARD.md)
4. Explore `specs/` directory

### By Topic

#### **Deployment**
- Quick: [QUICK_START.md](QUICK_START.md)
- Detailed: [DEPLOYMENT.md](DEPLOYMENT.md)
- Tower: [QUICK_START_TOWER_DEPLOYMENT.md](QUICK_START_TOWER_DEPLOYMENT.md)
- Graphs: `graphs/README.md`

#### **Architecture**
- Overview: [README.md](README.md) → Architecture section
- Primal Integration: [BIOMEOS_PRIMAL_INTEGRATION_SPEC.md](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)
- IPC: [TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)
- Neural API: [BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md](BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md)

#### **Standards & Compliance**
- ecoBin: [BIOMEOS_ECOBIN_CERTIFICATION_JAN_24_2026.md](BIOMEOS_ECOBIN_CERTIFICATION_JAN_24_2026.md)
- genomeBin: [GENOMEBIN_ARCHITECTURE_STANDARD.md](GENOMEBIN_ARCHITECTURE_STANDARD.md)
- IPC Protocol: `specs/primal-ipc/`

#### **Testing & Quality**
- Coverage: [TEST_COVERAGE_MILESTONE_JAN_24_2026.md](TEST_COVERAGE_MILESTONE_JAN_24_2026.md)
- Integration: [INTEGRATION_TEST_RESULTS.md](INTEGRATION_TEST_RESULTS.md)
- Multi-device: [MULTI_DEVICE_BONDING_TESTS.md](MULTI_DEVICE_BONDING_TESTS.md)

---

## 📊 Document Categories

### 🎯 Essential (Read First)
- README.md
- QUICK_START.md
- BIOMEOS_PRIMAL_INTEGRATION_SPEC.md

### 🏗️ Architecture (Understanding the System)
- BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md
- TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md
- BIOMEOS_ATOMICS_ARCHITECTURE.md

### 📜 Standards (Compliance & Certification)
- BIOMEOS_ECOBIN_CERTIFICATION_JAN_24_2026.md
- GENOMEBIN_ARCHITECTURE_STANDARD.md
- TOWER_ATOMIC_VALIDATION_AND_EVOLUTION.md

### 🔧 Operations (Running & Deploying)
- DEPLOYMENT.md
- QUICK_START_TOWER_DEPLOYMENT.md

### 🧪 Testing (Quality Assurance)
- TEST_COVERAGE_MILESTONE_JAN_24_2026.md
- INTEGRATION_TEST_RESULTS.md
- MULTI_DEVICE_BONDING_TESTS.md

### 🤝 Integration (Primal Handoffs)
- HANDOFF_SONGBIRD_INTEGRATION_TESTING.md
- HANDOFF_SQUIRREL_TOWER_INTEGRATION.md

### 📈 Tracking (Status & Progress)
- NEURAL_API_IMPLEMENTATION_TRACKER.md
- PRODUCTION_CODE_SLEEP_AUDIT.md

---

## 🗄️ Archive Organization

Session documentation and historical records are archived in:

```
archive/
├── sessions/
│   └── 2026-01-24/                    # Today's session
│       ├── COMPREHENSIVE_CODEBASE_AUDIT_JAN_24_2026.md
│       ├── DEEP_DEBT_EXECUTION_*.md
│       ├── TEST_COVERAGE_*.md
│       └── SESSION_COMPLETE_*.md
│
├── docs-fossil-record/                # Historical documentation
├── sessions-jan*/                     # Previous session archives
└── *_versions_jan_*/                  # Version-specific archives
```

---

## 🔄 Document Maintenance

### Document Lifecycle
1. **Active**: Lives in root (`*.md`)
2. **Completed**: Moved to `archive/sessions/YYYY-MM-DD/`
3. **Historical**: Kept in `archive/docs-fossil-record/`

### When to Archive
- ✅ Session-specific documents (after session complete)
- ✅ Time-stamped status reports
- ✅ Completed implementation plans
- ❌ Core architecture documents (keep in root)
- ❌ Active specifications (keep in root)
- ❌ Current guides (keep in root)

---

## 🆕 Recent Updates (Jan 24, 2026)

### New Documents
- ✅ Updated README.md with comprehensive overview
- ✅ Updated this ROOT_DOCS_INDEX.md
- ✅ Created TEST_COVERAGE_MILESTONE_JAN_24_2026.md

### Archived Today
- Moved 20+ session documents to `archive/sessions/2026-01-24/`
- Includes: audits, execution plans, status reports
- All accessible for historical reference

### Active Documents Refreshed
- README.md: Complete rewrite with modern structure
- BIOMEOS_ECOBIN_CERTIFICATION_JAN_24_2026.md: Updated certification
- TEST_COVERAGE_MILESTONE_JAN_24_2026.md: Latest coverage stats

---

## 📝 Contributing to Documentation

### Guidelines
1. **Keep root clean**: Only active, essential docs in root
2. **Use clear names**: Descriptive, not generic
3. **Include dates**: For time-sensitive docs (YYYY-MM-DD)
4. **Link properly**: Use relative links
5. **Archive promptly**: Move completed session docs

### Document Template
```markdown
# Title

**Last Updated**: YYYY-MM-DD
**Status**: Active/Archived
**Audience**: Who should read this

## Overview
Brief description

## Content
...

## Related Documents
- [Link](file.md)
```

---

## 🔗 External Resources

### Ecosystem Standards
Located in `../wateringHole/`:
- `UNIBIN_ARCHITECTURE_STANDARD.md`
- `ECOBIN_ARCHITECTURE_STANDARD.md`
- `PRIMAL_IPC_PROTOCOL.md`
- `GENOMEBIN_ARCHITECTURE_STANDARD.md`

### Related Projects
- BearDog: Security primal
- Songbird: Discovery primal
- NestGate: Storage primal
- sourDough: Scaffolding tool
- Tower: AI/Neural primal

---

## 💡 Tips for Navigation

### Quick Find
```bash
# Find all architecture docs
ls *ARCHITECTURE*.md

# Find all test docs
ls *TEST*.md

# Find certification docs
ls *CERTIFICATION*.md

# List archived sessions
ls archive/sessions/
```

### Search Content
```bash
# Search for specific topics
grep -r "Neural API" *.md

# Find mentions of a primal
grep -r "Songbird" docs/

# Search specifications
grep -r "JSON-RPC" specs/
```

---

**Status**: ✅ Up to date  
**Maintained By**: biomeOS Core Team  
**Last Audit**: January 24, 2026
