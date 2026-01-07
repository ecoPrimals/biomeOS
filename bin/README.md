# biomeOS bin/ - Utility Scripts & Executables

**Purpose**: Collection of utility scripts and compiled binaries for biomeOS operation.

**Updated**: January 7, 2026

---

## 📋 Contents

### Executables
- **`tower`** - Main biomeOS orchestrator binary (compiled from `crates/biomeos-cli/`)

### Utility Scripts
- **`pull-primals.sh`** - Build primal binaries from local parent directory
- **`live-demo.sh`** - End-to-end validation demo with verifiable output
- **`nestgate-auth-showcase.sh`** - NestGate authentication evolution showcase
- **`showcase-runner.sh`** - Discover and execute showcase demos

### Directories
- **`primals/`** - Compiled primal binaries (beardog, songbird, toadstool, etc.)
- **`chimeras/`** - Chimera composition scripts and manifests

---

## 🚀 Usage

### Building Primals
```bash
./bin/pull-primals.sh
# Builds all primal binaries from ../phase1/{beardog,songbird,toadstool}
# Output: bin/primals/{beardog,songbird,toadstool}
```

### Running Demo
```bash
./bin/live-demo.sh
# Runs end-to-end biomeOS validation
# Shows primal startup, capability registry, health checks
```

### Running Showcases
```bash
./bin/showcase-runner.sh
# Discovers and lists available showcases
# Interactive menu for running specific demos
```

---

## 📁 Directory Structure

```
bin/
├── README.md (this file)
├── tower (executable)
├── pull-primals.sh
├── live-demo.sh
├── nestgate-auth-showcase.sh
├── showcase-runner.sh
├── primals/ (compiled binaries)
│   ├── beardog
│   ├── songbird
│   ├── toadstool
│   ├── nestgate
│   └── ...
└── chimeras/ (composition tools)
```

---

## 🔧 Script Details

### `pull-primals.sh`
**Purpose**: Build all primal binaries from source  
**Dependencies**: Requires `../phase1/` structure with primal repos  
**Output**: Copies binaries to `bin/primals/`  
**When to Use**: After primal teams update their code

### `live-demo.sh`
**Purpose**: Demonstrate biomeOS capabilities  
**Features**: 
- Primal startup validation
- Capability registry test
- Health check verification
- Receipt generation

### `nestgate-auth-showcase.sh`
**Purpose**: Showcase NestGate authentication evolution  
**Features**:
- Phase 1 primal integration
- Pluggable auth demonstration
- BearDog sovereignty example

### `showcase-runner.sh`
**Purpose**: Unified showcase discovery and execution  
**Features**:
- Discovers showcases from parent ecoPrimals
- Interactive menu
- Organized by category

---

## 📝 Notes

### Script Maintenance
- All scripts have headers with purpose/description
- Scripts are self-documenting (check `--help` if available)
- For major changes, update this README

### Binary Updates
- Primal binaries built from `../phase1/{primal}/`
- Use `pull-primals.sh` to rebuild after primal updates
- Binaries are NOT in git (too large, frequently updated)

### Showcase Scripts
- Showcase scripts are for demonstration purposes
- Not required for production deployment
- Useful for onboarding and validation

---

## 🎯 Related Documentation

- [Main README](../README.md) - Project overview
- [Scripts Directory](../scripts/) - Deployment/utility scripts
- [Examples Directory](../examples/) - Code examples

---

**Last Updated**: January 7, 2026  
**Maintainer**: biomeOS core team
