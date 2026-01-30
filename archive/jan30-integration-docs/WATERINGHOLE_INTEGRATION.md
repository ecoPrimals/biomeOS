# 🌊 wateringHole Integration

**Inter-Primal Standards Repository**

The [wateringHole](https://github.com/ecoPrimals/wateringHole) is the official repository for inter-primal standards, protocols, and architectural guidelines that enable primals to communicate and evolve together.

---

## 📚 Official Standards

### Core Protocols
- **[PRIMAL_IPC_PROTOCOL.md](https://github.com/ecoPrimals/wateringHole/blob/master/PRIMAL_IPC_PROTOCOL.md)**
  - JSON-RPC 2.0 over Unix sockets
  - Standard message format
  - Error handling conventions

- **[SEMANTIC_METHOD_NAMING_STANDARD.md](https://github.com/ecoPrimals/wateringHole/blob/master/SEMANTIC_METHOD_NAMING_STANDARD.md)**
  - Method naming conventions (v2.0)
  - `domain.operation` format
  - Evolution patterns

### Architecture Standards
- **[UNIBIN_ARCHITECTURE_STANDARD.md](https://github.com/ecoPrimals/wateringHole/blob/master/UNIBIN_ARCHITECTURE_STANDARD.md)**
  - Single binary standard
  - Subcommand structure
  - CLI interface requirements

- **[ECOBIN_ARCHITECTURE_STANDARD.md](https://github.com/ecoPrimals/wateringHole/blob/master/ECOBIN_ARCHITECTURE_STANDARD.md)**
  - Pure Rust requirement
  - Cross-compilation matrix
  - Zero C dependencies standard

---

## 🔗 How biomeOS Uses wateringHole

### Compliance
biomeOS is:
- ✅ **UniBin Compliant** - Single binary with subcommands
- ✅ **ecoBin Compliant** - 100% Pure Rust, zero C deps
- ✅ **IPC Protocol** - JSON-RPC 2.0 over Unix sockets
- ✅ **Semantic Methods** - domain.operation naming

### Implementation
```rust
// biomeOS follows semantic method naming
neural_api.call_capability("crypto.generate_keypair", params).await?;
// ↓ translates to provider-specific
// "x25519_generate_ephemeral" for BearDog

// All via JSON-RPC 2.0 over Unix sockets
// Following PRIMAL_IPC_PROTOCOL.md
```

---

## 🎯 Reference in Your Code

### Standards Location
```bash
# Clone wateringHole standards
git clone git@github.com:ecoPrimals/wateringHole.git

# Or view online
https://github.com/ecoPrimals/wateringHole
```

### In Documentation
When referencing standards, link to the official repo:
```markdown
See [PRIMAL_IPC_PROTOCOL.md](https://github.com/ecoPrimals/wateringHole/blob/master/PRIMAL_IPC_PROTOCOL.md)
```

---

## 🌟 Contributing to Standards

Standards are community-driven:

1. **Propose**: Open an issue in wateringHole
2. **Discuss**: Community feedback
3. **Implement**: Create PR with proposal
4. **Adopt**: Cross-primal adoption

**Repository**: https://github.com/ecoPrimals/wateringHole

---

## ✅ biomeOS Compliance Status

| Standard | Version | Status | Notes |
|----------|---------|--------|-------|
| UniBin | 1.0 | ✅ Compliant | Single binary, subcommands |
| ecoBin | 1.0 | ✅ Certified | Pure Rust, 0 C deps |
| IPC Protocol | 2.0 | ✅ Compliant | JSON-RPC over Unix sockets |
| Semantic Methods | 2.0 | ✅ Compliant | domain.operation format |

---

**wateringHole**: The place where primals come together to establish common ground.

🌊 **Official Repository**: https://github.com/ecoPrimals/wateringHole


---

## 🌍 TRUE ecoBin Evolution - January 30, 2026

### **Platform-Agnostic IPC Standard**

**New Standards Added:**
- `ECOBIN_TRUE_PRIMAL_STANDARD.md` - ecoBin v2.0 with platform-agnostic IPC
- `docs/deep-debt/PLATFORM_AGNOSTIC_IPC_EVOLUTION.md` - Implementation guide

**Key Evolution:**
- **ecoBin v1.0:** Cross-architecture (x86_64, ARM64, RISC-V)
- **ecoBin v2.0:** Cross-architecture + Cross-platform (Linux, Android, Windows, macOS, iOS, WASM, embedded)

**Philosophy:**
> "If it can't run on the arch/platform, it's not a true ecoBin"

**Implementation:**
- `biomeos-ipc` crate for platform-agnostic transport
- Runtime discovery (Unix sockets, abstract sockets, TCP, named pipes, etc.)
- Graceful fallback (prefer native, fall back to TCP)
- Zero platform assumptions

**Adoption Target:** Q1 2026 (migrate all primals to TRUE ecoBin v2.0)

**References:**
- Platform-agnostic IPC: `docs/deep-debt/PLATFORM_AGNOSTIC_IPC_EVOLUTION.md`
- TRUE ecoBin standard: `ECOBIN_TRUE_PRIMAL_STANDARD.md`
- genomeBin updates: `GENOMEBIN_ARCHITECTURE_STANDARD.md`

