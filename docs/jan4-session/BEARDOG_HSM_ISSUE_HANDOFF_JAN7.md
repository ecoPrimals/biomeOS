# 🔥 HANDOFF: BearDog HSM Provider Issue

**Date**: January 7, 2026  
**Status**: 🚨 **BLOCKER** - BearDog cannot initialize  
**Team**: BearDog  
**Priority**: CRITICAL

---

## 🎯 Issue Summary

BearDog v0.15.0 (fresh binary from Jan 7, 20:31) is failing to start with:

```
Error: Failed to initialize BTSP provider

Caused by:
    System error: Failed to generate BirdSong master key: Business error: No HSM providers available
```

---

## 📊 Context

### What We're Trying to Do
Deploy biomeOS with genetic lineage and port-free P2P federation using:
- BearDog for security and BTSP
- Songbird for federation
- Genetic family trust (nat0)

### What's Working
✅ biomeOS tower orchestration
✅ Genetic seed derivation (siblings not clones)
✅ Songbird v3.19.0 (port-free P2P ready)
✅ Configuration (family ID, node ID, socket paths)

### What's Broken
❌ BearDog fails to initialize BTSP provider
❌ Error: "No HSM providers available"

---

## 🔍 Error Details

### Full Error Log
```
2026-01-08T01:31:27.408905Z  INFO beardog-server.rs:107: 🐻 BearDog Server Starting
2026-01-08T01:31:27.408927Z  INFO beardog-server.rs:108:    Family: nat0
2026-01-08T01:31:27.408928Z  INFO beardog-server.rs:109:    Node: node-alpha
2026-01-08T01:31:27.408929Z  INFO beardog-server.rs:110:    Socket: /tmp/primals/beardog-node-alpha.sock
2026-01-08T01:31:27.408930Z  INFO beardog-server.rs:111:    HTTP Port: 0 (0 = disabled for port-free operation)
2026-01-08T01:31:27.408931Z  INFO beardog-server.rs:114: 🔐 Initializing HSM...
2026-01-08T01:31:27.408934Z  INFO beardog-server.rs:118: 🧬 Initializing genetic lineage engine...
2026-01-08T01:31:27.408935Z  INFO crates/beardog-genetics/src/ecosystem_evolution/engine.rs:55: Initializing ecosystem genetic engine
2026-01-08T01:31:27.408939Z  INFO beardog-server.rs:125: 🔒 Initializing BTSP provider...
2026-01-08T01:31:27.408940Z  INFO crates/beardog-tunnel/src/btsp_provider.rs:348: 🐻 Initializing BearDog BTSP Provider with BirdSong genetics
Error: Failed to initialize BTSP provider

Caused by:
    System error: Failed to generate BirdSong master key: Business error: No HSM providers available
```

### Error Chain
```
1. beardog-server.rs:125 - Initializing BTSP provider
2. btsp_provider.rs:348 - Initializing with BirdSong genetics
3. FAILURE: No HSM providers available
```

---

## 🧪 Environment

### Binary Info
```bash
$ ls -lh /media/eastgate/biomeOS1/biomeOS/primals/beardog-server
-rwxrwxr-x 1 eastgate eastgate 2.4M Jan  7 20:31 beardog-server

$ file beardog-server
ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked

$ ./beardog-server --version
2026-01-08T01:31:58.503881Z  INFO beardog-server.rs:107: 🐻 BearDog Server Starting
2026-01-08T01:31:58.503891Z  INFO beardog-server.rs:108:    Family: unknown
2026-01-08T01:31:58.503893Z  INFO beardog-server.rs:109:    Node: pop-os
2026-01-08T01:31:58.503895Z  INFO beardog-server.rs:110:    Socket: /tmp/primals/beardog-pop-os.sock
2026-01-08T01:31:58.503897Z  INFO beardog-server.rs:111:    HTTP Port: 0 (0 = disabled for port-free operation)
Aborted (core dumped)
```

### Configuration (tower.toml)
```toml
[primals.beardog]
id = "83cd1a59-9a02-4040-83b1-6230876e9dea"
name = "beardog"
binary = "./primals/beardog-server"
priority = 1
health_check_interval_secs = 30

[primals.beardog.env]
BEARDOG_FAMILY_SEED_FILE = "./.family.seed"
BEARDOG_NODE_ID = "node-alpha"
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_SOCKET_PATH = "/tmp/primals/beardog-node-alpha.sock"
BEARDOG_HTTP_PORT = "0"
RUST_LOG = "info"
```

### Family Seed
```bash
$ ls -lh /media/eastgate/biomeOS1/biomeOS/.family.seed
-rw------- 1 eastgate eastgate 32 Jan  7 16:13 .family.seed

$ file .family.seed
.family.seed: data
```

---

## 🤔 Analysis

### Possible Causes

#### 1. HSM Provider Not Configured
BearDog expects an HSM provider to be available for key generation, but none is configured or detected.

**Questions**:
- Does BearDog require a hardware HSM?
- Is there a software HSM fallback?
- Should we configure `BEARDOG_HSM_PROVIDER` env var?

#### 2. Missing Dependencies
The BTSP provider initialization might require additional system libraries or services.

**Questions**:
- Are there missing system dependencies?
- Does it need a specific crypto library?

#### 3. Configuration Missing
The HSM provider might need explicit configuration that we're not providing.

**Questions**:
- Should we set `BEARDOG_HSM_TYPE=software`?
- Is there a config file for HSM setup?

---

## 🎯 What We Need from BearDog Team

### Question 1: HSM Requirements
**Q**: What HSM provider does BearDog require?  
**Options**:
- Hardware HSM (YubiHSM, etc.)
- Software HSM (SoftHSM, etc.)
- Built-in fallback (memory-based)

### Question 2: Configuration
**Q**: How do we configure the HSM provider?  
**Needed**:
- Environment variables
- Config file format
- Default/fallback behavior

### Question 3: Dependencies
**Q**: Are there system dependencies we're missing?  
**Check**:
- Crypto libraries (OpenSSL, etc.)
- HSM client libraries
- System services

### Question 4: Workaround
**Q**: Is there a way to run BearDog without HSM for dev/testing?  
**Options**:
- `BEARDOG_HSM_MODE=memory`
- `BEARDOG_SKIP_HSM=true`
- Software fallback

---

## 🚀 Recommended Actions

### For BearDog Team

#### Option 1: Add Software HSM Fallback (Preferred)
```rust
// In HSM initialization
let hsm = if let Some(provider) = env::var("BEARDOG_HSM_PROVIDER").ok() {
    HsmProvider::from_config(&provider)?
} else {
    warn!("No HSM provider configured, using software fallback");
    HsmProvider::software_fallback()
};
```

#### Option 2: Document HSM Requirements
Add to README:
```markdown
## HSM Configuration

BearDog requires an HSM provider for key generation.

### Software HSM (Development)
```bash
export BEARDOG_HSM_PROVIDER=software
```

### Hardware HSM (Production)
```bash
export BEARDOG_HSM_PROVIDER=yubihsm
export BEARDOG_HSM_URL=http://localhost:12345
```
```

#### Option 3: Make HSM Optional for Testing
```rust
#[cfg(feature = "hsm")]
fn initialize_hsm() -> Result<Hsm> {
    // Real HSM
}

#[cfg(not(feature = "hsm"))]
fn initialize_hsm() -> Result<Hsm> {
    // Memory-based fallback
}
```

---

## 📊 Impact

### Blocked Work
- ❌ biomeOS local federation testing
- ❌ Genetic trust verification
- ❌ Port-free P2P deployment
- ❌ USB spore self-propagation testing

### Workaround
None currently available. BearDog is required for:
- Identity and family ID
- BTSP tunnels
- Encryption/decryption
- Trust evaluation

---

## 🎊 What's Ready (Waiting on BearDog)

### biomeOS ✅
- ✅ Genetic sibling derivation (not clones)
- ✅ USB spore system (Live and Cold)
- ✅ Tower orchestration
- ✅ Primal health monitoring
- ✅ Zombie process reaping

### Songbird ✅
- ✅ Port-free P2P (v3.19.0)
- ✅ BTSP client (OnceCell pattern)
- ✅ UDP multicast discovery
- ✅ Genetic family tag broadcasting

### Configuration ✅
- ✅ Family seed files (32 bytes)
- ✅ Node IDs (node-alpha, node-beta)
- ✅ Socket paths (dynamic)
- ✅ Environment variables

**Everything is ready except BearDog HSM initialization!**

---

## 📝 Next Steps

1. **BearDog Team**: Investigate HSM provider requirements
2. **BearDog Team**: Add software fallback or document config
3. **biomeOS**: Wait for fix or workaround
4. **biomeOS**: Test federation once BearDog starts

---

**Handoff Date**: January 7, 2026, 20:35  
**Handed Off By**: biomeOS Team  
**Handed Off To**: BearDog Team  
**Priority**: 🚨 CRITICAL BLOCKER

