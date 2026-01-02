# 00 - Local BiomeOS Capabilities

**Purpose:** Demonstrate BiomeOS core capabilities without any primals  
**Duration:** 10-15 minutes  
**Philosophy:** Show BiomeOS value before primal integration

---

## 🎯 What This Demonstrates

BiomeOS provides valuable orchestration capabilities even before connecting to any primals:

1. **Manifest Parsing** - Parse and validate biome.yaml files
2. **Capability Matching** - Match requirements to potential primals
3. **Configuration Management** - Manage system configuration
4. **Sovereignty Guardian** - Privacy and consent protections
5. **Client Registry** - Initialize primal client system

---

## 🚀 Quick Start

Run all demos:
```bash
./run-all-local-demos.sh
```

Or run individual demos:
```bash
./01-manifest-parsing.sh
./02-capability-matching.sh
./03-config-management.sh
./04-sovereignty-guardian.sh
./05-client-registry.sh
```

---

## 📋 Demo Details

### 01 - Manifest Parsing
**File:** `01-manifest-parsing.sh`  
**Shows:** Parse biome.yaml, validate structure, display parsed content

**What You'll See:**
- Load example biome.yaml files
- Validate manifest structure
- Display parsed configuration
- Show validation errors for invalid manifests

**Key BiomeOS Features:**
- Manifest validation engine
- Schema enforcement
- Clear error messages

---

### 02 - Capability Matching
**File:** `02-capability-matching.sh`  
**Shows:** Match biome requirements to primal capabilities

**What You'll See:**
- Load biome requirements (compute, storage, security)
- Show available capability types
- Match requirements to capability patterns
- Display matching results

**Key BiomeOS Features:**
- Capability-based discovery
- No hardcoded primal knowledge
- Flexible matching engine

---

### 03 - Configuration Management
**File:** `03-config-management.sh`  
**Shows:** BiomeOS configuration system

**What You'll See:**
- Load BiomeOS configuration
- Show default values
- Override with environment variables
- Display final configuration

**Key BiomeOS Features:**
- Layered configuration (defaults → env → file)
- Clear precedence rules
- Validation and type checking

---

### 04 - Sovereignty Guardian
**File:** `04-sovereignty-guardian.sh`  
**Shows:** Privacy and human dignity protections

**What You'll See:**
- Initialize sovereignty guardian
- Check data access policies
- Evaluate consent requirements
- Show audit trail generation
- Block privacy violations

**Key BiomeOS Features:**
- Comprehensive privacy policies
- Human dignity protections
- Audit trail for accountability
- Consent management

---

### 05 - Client Registry
**File:** `05-client-registry.sh`  
**Shows:** Primal client initialization and management

**What You'll See:**
- Initialize client registry
- Register client types (without connecting)
- Show available client adapters
- Display client capabilities

**Key BiomeOS Features:**
- Dynamic client registration
- Type-safe client management
- Ready for live primal connection

---

## 🔍 What We're Looking For

As we run these demos, we're looking for:

### Gaps to Document:
- ❓ Configuration edge cases
- ❓ Manifest validation gaps
- ❓ Capability matching limitations
- ❓ Sovereignty policy gaps
- ❓ Client registry issues

### Questions to Answer:
- Can we parse all real-world biome.yaml files?
- Does capability matching handle complex requirements?
- Are sovereignty protections comprehensive?
- Is the client registry ready for real primals?

---

## 📊 Expected Outcomes

After running these demos, you should understand:

1. **BiomeOS adds value** even before connecting primals
2. **Manifest system** is robust and well-validated
3. **Capability matching** works without hardcoded knowledge
4. **Sovereignty guardian** provides real protections
5. **Client registry** is ready for live integration

---

## 🚧 Known Gaps (Will Update as We Find Them)

### Manifest Parsing:
- [ ] TO BE DISCOVERED

### Capability Matching:
- [ ] TO BE DISCOVERED

### Configuration:
- [ ] TO BE DISCOVERED

### Sovereignty:
- [ ] TO BE DISCOVERED

### Client Registry:
- [ ] TO BE DISCOVERED

---

## 🎓 Learning Path

**For New Users:**
1. Start with 01-manifest-parsing (understand biome.yaml)
2. Move to 02-capability-matching (understand discovery)
3. Try 04-sovereignty-guardian (understand protections)

**For Developers:**
1. Study all demos to understand architecture
2. Review gap documentation
3. Consider improvements

---

## 📝 Notes

- **No Mocks**: All demos use real BiomeOS code
- **No Primals Needed**: These demos work standalone
- **Real Output**: Actual BiomeOS functionality, not simulated
- **Gap Discovery**: We document real issues as we find them

---

**Next:** After completing these demos, move to `01-single-primal/` to connect with real primals.
