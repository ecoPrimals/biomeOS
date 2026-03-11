# Genome Distribution Architecture

**Version**: 1.0.0  
**Date**: February 12, 2026  
**Status**: PROPOSED → IMPLEMENTATION READY

---

## 🧬 Spore Types

The ecosystem uses two types of USB spores:

### LiveSpore (Bootable)

A **LiveSpore** is a fully bootable USB that can:
- Boot any x86_64/aarch64 computer directly from BIOS
- Contains minimal Linux (Alpine) + biomeOS genomes
- Auto-starts NUCLEUS on boot
- Self-contained deployment system (~200MB)
- No host OS required

Use case: Deploy NUCLEUS to bare metal, rescue/recovery, air-gapped systems

### ColdSpore (Non-Bootable / Data-Only)

A **ColdSpore** is a data-only USB that:
- Contains genome binaries and configuration only
- Requires existing OS to deploy
- Lighter weight (~50MB)
- Uses `deploy_cross_arch.sh` to install

Use case: Update existing systems, portable genome storage, development

### Temporal Siblings

Spores created from the same parent at different times are **temporal siblings**:
- Same generation (Gen 1)
- Same mito beacon (family recognition via BirdSong)
- Different lineage seeds (individual identity)
- "Younger" siblings created later are still Gen 1, not Gen 2

```
Tower (Parent, Gen 0)
     │
     ├── ColdSpore-A (Gen 1, born 2026-02-12 14:00)
     ├── ColdSpore-B (Gen 1, born 2026-02-12 14:00)
     │
     └── [SAME GENERATION, YOUNGER]
         ├── LiveSpore-A (Gen 1, born 2026-02-12 16:00) ← temporal sibling
         └── LiveSpore-B (Gen 1, born 2026-02-12 16:00) ← temporal sibling
```

All 4 are siblings because they derive from the same parent's genetics.

---

## 🎯 Problem Statement

The current genome storage (`plasmidBin/` in biomeOS) works for local testing and spore creation, but doesn't address:

1. **Remote Setup**: How do new NUCLEUS nodes fetch genomes?
2. **Updates**: How do running nodes receive updated primals?
3. **Verification**: How do we ensure genome integrity and lineage?
4. **Cross-Arch**: How do we serve x86_64, aarch64, etc. dynamically?
5. **Public vs Family**: How do we differentiate public access from family-verified access?

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        GENOME DISTRIBUTION LAYER                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────────┐       ┌──────────────────┐     ┌───────────────┐ │
│  │  wateringHole/   │       │   nestgate.io    │     │    GitHub     │ │
│  │   genomeBin/     │──────▶│  (Genome API)    │────▶│   Releases    │ │
│  │ (source of truth)│       │ +Dark Forest Gate│     │  (fallback)   │ │
│  └──────────────────┘       └────────┬─────────┘     └───────────────┘ │
│                                      │                                  │
│                          ┌───────────┴───────────┐                      │
│                          │    Cloudflare Tunnel  │                      │
│                          │   (api.nestgate.io)   │                      │
│                          └───────────┬───────────┘                      │
│                                      │                                  │
├──────────────────────────────────────┼──────────────────────────────────┤
│                                      ▼                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                 │
│  │   Tower     │    │    Pixel    │    │  New Node   │                 │
│  │ (USB Spore) │    │ (Mobile)    │    │ (Remote)    │                 │
│  │             │    │             │    │             │                 │
│  │ plasmidBin/ │    │ /data/local │    │ fetch from  │                 │
│  │ (full cross │    │ /tmp/biome  │    │ nestgate.io │                 │
│  │  arch set)  │    │ (minimal)   │    │             │                 │
│  └─────────────┘    └─────────────┘    └─────────────┘                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 📁 Directory Structure Evolution

### Current: `biomeOS/plasmidBin/`

```
plasmidBin/
├── primals/           # Active binaries for local use
├── stable/            # Versioned stable releases
│   ├── x86_64/
│   └── aarch64/
├── optimized/         # Optimized builds
└── archive/           # Old versions
```

### Proposed: `wateringHole/genomeBin/`

```
wateringHole/
├── genomeBin/                          # Centralized genome distribution
│   ├── manifest.toml                   # Master manifest (all primals, versions, arches)
│   ├── checksums.toml                  # SHA256 + BLAKE3 checksums
│   ├── signatures/                     # GPG signatures (optional)
│   │   ├── beardog-0.9.0-x86_64.sig
│   │   └── ...
│   ├── primals/
│   │   ├── beardog/
│   │   │   ├── v0.9.0/
│   │   │   │   ├── beardog-x86_64-linux-musl
│   │   │   │   ├── beardog-aarch64-linux-musl
│   │   │   │   └── metadata.toml
│   │   │   └── latest -> v0.9.0
│   │   ├── songbird/
│   │   │   ├── v0.2.1/
│   │   │   └── latest -> v0.2.1
│   │   ├── nestgate/
│   │   ├── squirrel/
│   │   └── toadstool/
│   └── atomics/                        # Pre-built atomic combinations
│       ├── tower/                      # BearDog + Songbird
│       │   └── v1.0.0/
│       ├── node/                       # Tower + Toadstool
│       └── full/                       # All primals
├── GENOMEBIN_ARCHITECTURE_STANDARD.md  # Already exists
└── ...
```

### LiveSpore: Self-Contained Cross-Arch

```
livespore-usb/
├── primals/                    # Cross-arch genome binaries
│   ├── x86_64/
│   │   ├── beardog
│   │   ├── songbird
│   │   ├── nestgate
│   │   ├── squirrel
│   │   └── toadstool
│   ├── aarch64/
│   │   ├── beardog
│   │   ├── songbird
│   │   ├── nestgate
│   │   ├── squirrel
│   │   └── toadstool
│   └── riscv64/                # Future
├── manifest.toml               # Local manifest
├── checksums.toml              # Integrity verification
├── deploy.sh                   # Auto-detect arch and deploy
├── graphs/                     # NUCLEUS deployment graphs
│   ├── tower_atomic.toml
│   ├── node_atomic.toml
│   └── full_atomic.toml
└── .family.seed                # Family lineage
```

---

## 🌐 API Design: `api.nestgate.io/genome/`

### Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| `GET` | `/genome/manifest` | List all available genomes | None |
| `GET` | `/genome/{primal}/latest` | Get latest version info | None |
| `GET` | `/genome/{primal}/{version}/{arch}` | Download binary | Dark Forest |
| `GET` | `/genome/checksum/{primal}/{version}/{arch}` | Get checksum | None |
| `POST` | `/genome/verify` | Verify local binary against registry | Dark Forest |
| `GET` | `/genome/atomics/{atomic}/latest` | Get atomic bundle info | None |
| `GET` | `/genome/atomics/{atomic}/{version}/{arch}` | Download atomic | Dark Forest |

### Example: Manifest Response

```json
{
  "primals": {
    "beardog": {
      "latest": "0.9.0",
      "versions": ["0.9.0", "0.8.5", "0.8.0"],
      "architectures": ["x86_64-linux-musl", "aarch64-linux-musl"]
    },
    "songbird": {
      "latest": "0.2.1",
      "versions": ["0.2.1", "0.2.0", "0.1.0"],
      "architectures": ["x86_64-linux-musl", "aarch64-linux-musl"]
    }
  },
  "atomics": {
    "tower": {"primals": ["beardog", "songbird"], "latest": "1.0.0"},
    "node": {"primals": ["beardog", "songbird", "toadstool"], "latest": "1.0.0"},
    "full": {"primals": ["beardog", "songbird", "nestgate", "squirrel", "toadstool"], "latest": "1.0.0"}
  }
}
```

### Example: Download Flow

```bash
# 1. Get manifest
curl https://api.nestgate.io/genome/manifest

# 2. Get checksum (public)
curl https://api.nestgate.io/genome/checksum/beardog/0.9.0/x86_64-linux-musl
# Returns: {"sha256": "abc123...", "blake3": "def456...", "size": 5242880}

# 3. Download binary (requires Dark Forest Token)
curl -H "X-Dark-Forest-Token: $(birdsong.generate_token)" \
     https://api.nestgate.io/genome/beardog/0.9.0/x86_64-linux-musl \
     -o beardog

# 4. Verify locally
sha256sum beardog  # Compare with checksum
```

---

## 🔐 Security Model

### Family-Verified Access (Dark Forest Gate)

```
┌─────────────────────────────────────────────────────────────────┐
│                    GENOME DOWNLOAD FLOW                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Requestor (Pixel/New Node)                                     │
│       │                                                         │
│       │ 1. Generate encrypted beacon with BirdSong              │
│       ▼                                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ X-Dark-Forest-Token: birdsong.generate_encrypted_beacon │    │
│  │ {                                                        │    │
│  │   "node_id": "pixel8a",                                  │    │
│  │   "family_id": "8ff3b864a4bc589a",                       │    │
│  │   "capabilities_requested": ["genome.download"],         │    │
│  │   "genome": "beardog",                                   │    │
│  │   "version": "0.9.0",                                    │    │
│  │   "arch": "aarch64-linux-musl"                           │    │
│  │ }                                                        │    │
│  └─────────────────────────────────────────────────────────┘    │
│       │                                                         │
│       │ 2. Request to api.nestgate.io                           │
│       ▼                                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              api.nestgate.io (Cloudflare)                │    │
│  │                        │                                 │    │
│  │  3. Forward to Tower biomeos-api                         │    │
│  │                        ▼                                 │    │
│  │  ┌────────────────────────────────────────────────────┐  │    │
│  │  │            Dark Forest Gate Middleware             │  │    │
│  │  │                                                    │  │    │
│  │  │  4. Decrypt beacon via BearDog birdsong.decrypt    │  │    │
│  │  │  5. Verify family_id matches                       │  │    │
│  │  │  6. Check requested capability is allowed          │  │    │
│  │  │  7. Rate limit check                               │  │    │
│  │  │                                                    │  │    │
│  │  │  If PASS → Serve genome binary                     │  │    │
│  │  │  If FAIL → 403 Forbidden                           │  │    │
│  │  └────────────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Public vs Family Access

| Access Level | What's Available | Authentication |
|--------------|------------------|----------------|
| **Public** | Manifest, version info, checksums | None |
| **Family** | Binary downloads, atomic bundles | Dark Forest Token |
| **GitHub** | All binaries (fallback) | GitHub token (optional) |

---

## 📦 Implementation Plan

### Phase 1: Repository Structure (Day 1)

1. **Create `wateringHole/genomeBin/`**
   ```bash
   mkdir -p wateringHole/genomeBin/{primals,atomics,signatures}
   ```

2. **Migrate stable binaries from `biomeOS/plasmidBin/stable/`**
   ```bash
   cp -r biomeOS/plasmidBin/stable/* wateringHole/genomeBin/primals/
   ```

3. **Create manifest.toml**
   ```toml
   [manifest]
   version = "1.0.0"
   generated = "2026-02-12T20:00:00Z"
   
   [primals.beardog]
   latest = "0.9.0"
   versions = ["0.9.0"]
   architectures = ["x86_64-linux-musl", "aarch64-linux-musl"]
   
   [primals.songbird]
   latest = "0.2.1"
   versions = ["0.2.1"]
   architectures = ["x86_64-linux-musl", "aarch64-linux-musl"]
   
   # ... more primals
   
   [atomics.tower]
   primals = ["beardog", "songbird"]
   latest = "1.0.0"
   
   [atomics.node]
   primals = ["beardog", "songbird", "toadstool"]
   latest = "1.0.0"
   ```

4. **Generate checksums**
   ```bash
   for f in primals/*/*/*; do
     echo "$(sha256sum $f | cut -d' ' -f1)  $f" >> checksums.toml
   done
   ```

### Phase 2: API Implementation (Day 2-3)

1. **Add genome routes to `biomeos-api`**
   ```rust
   // crates/biomeos-api/src/genome.rs
   
   pub fn genome_routes() -> Router {
       Router::new()
           .route("/genome/manifest", get(get_manifest))
           .route("/genome/:primal/latest", get(get_latest))
           .route("/genome/:primal/:version/:arch", get(download_genome))
           .route("/genome/checksum/:primal/:version/:arch", get(get_checksum))
           .route("/genome/verify", post(verify_genome))
   }
   ```

2. **Implement Dark Forest Gate for downloads**
   ```rust
   async fn download_genome(
       State(state): State<AppState>,
       Path((primal, version, arch)): Path<(String, String, String)>,
       headers: HeaderMap,
   ) -> Result<impl IntoResponse, ApiError> {
       // Verify Dark Forest Token
       let token = headers.get("X-Dark-Forest-Token")
           .ok_or(ApiError::Unauthorized)?;
       
       verify_dark_forest_token(&state, token).await?;
       
       // Serve binary from genomeBin
       let binary_path = format!("genomeBin/primals/{}/{}/{}-{}", 
           primal, version, primal, arch);
       
       serve_binary(&binary_path).await
   }
   ```

3. **GitHub fallback for public access**
   ```rust
   async fn download_genome_public(primal: &str, version: &str, arch: &str) -> Result<Bytes> {
       let url = format!(
           "https://github.com/ecoPrimals/releases/download/v{}/{}-{}",
           version, primal, arch
       );
       // Redirect or proxy from GitHub
   }
   ```

### Phase 3: Client Integration (Day 4)

1. **Update `PrimalRegistry` with remote support**
   ```rust
   impl PrimalRegistry {
       /// Fetch genome from nestgate.io with Dark Forest verification
       pub async fn fetch_genome(
           &self,
           primal: &str,
           version: Option<&str>,
           arch: &str,
       ) -> Result<PathBuf> {
           let version = version.unwrap_or("latest");
           let url = format!("{}/genome/{}/{}/{}", 
               self.api_endpoint, primal, version, arch);
           
           // Generate Dark Forest token
           let token = self.beardog.birdsong_generate_beacon()?;
           
           // Download
           let response = self.http_client
               .get(&url)
               .header("X-Dark-Forest-Token", token)
               .send().await?;
           
           // Verify checksum
           let checksum_url = format!("{}/genome/checksum/{}/{}/{}", 
               self.api_endpoint, primal, version, arch);
           let expected = self.http_client.get(&checksum_url).send().await?.json()?;
           
           verify_checksum(&response, &expected)?;
           
           // Save to local genomeBin
           let local_path = self.genome_dir.join(primal).join(version).join(arch);
           tokio::fs::write(&local_path, response.bytes().await?).await?;
           
           Ok(local_path)
       }
   }
   ```

2. **LiveSpore auto-update capability**
   ```rust
   /// Update all primals from registry
   pub async fn update_all_primals(&self) -> Result<Vec<UpdateResult>> {
       let manifest = self.fetch_manifest().await?;
       let arch = detect_arch();
       
       let mut results = vec![];
       for (primal, info) in &manifest.primals {
           if self.needs_update(primal, &info.latest)? {
               let result = self.fetch_genome(primal, Some(&info.latest), &arch).await;
               results.push(UpdateResult { primal: primal.clone(), result });
           }
       }
       
       Ok(results)
   }
   ```

### Phase 4: LiveSpore Enhancement (Day 5)

1. **Full cross-arch embedding**
   ```bash
   # Build script for LiveSpore
   #!/bin/bash
   ARCHES=("x86_64-unknown-linux-musl" "aarch64-unknown-linux-musl")
   PRIMALS=("beardog" "songbird" "nestgate" "squirrel" "toadstool")
   
   for arch in "${ARCHES[@]}"; do
       mkdir -p livespore/primals/${arch%%%-*}
       for primal in "${PRIMALS[@]}"; do
           cp genomeBin/primals/$primal/latest/$primal-$arch \
              livespore/primals/${arch%%%-*}/$primal
       done
   done
   ```

2. **Auto-detect and deploy script**
   ```bash
   #!/bin/bash
   # deploy.sh - Auto-detect arch and deploy appropriate binaries
   
   ARCH=$(uname -m)
   case $ARCH in
       x86_64)  ARCH_DIR="x86_64" ;;
       aarch64) ARCH_DIR="aarch64" ;;
       *)       echo "Unsupported arch: $ARCH"; exit 1 ;;
   esac
   
   echo "Detected architecture: $ARCH_DIR"
   echo "Deploying primals..."
   
   for primal in primals/$ARCH_DIR/*; do
       cp "$primal" /usr/local/bin/
       echo "  Deployed: $(basename $primal)"
   done
   ```

---

## 🔄 Update Flow

### Scenario: New Primal Release

```
1. Developer releases new BearDog v0.10.0
   └─▶ Build for x86_64 + aarch64
   └─▶ Copy to wateringHole/genomeBin/primals/beardog/v0.10.0/
   └─▶ Update manifest.toml (latest = "0.10.0")
   └─▶ Generate checksums
   └─▶ Push to git

2. api.nestgate.io serves new manifest
   └─▶ Nodes poll /genome/manifest periodically
   └─▶ Detect new version available

3. Node initiates update
   └─▶ Generate Dark Forest token
   └─▶ Download new binary
   └─▶ Verify checksum
   └─▶ Atomic replace (backup old, install new)
   └─▶ Restart primal service
```

### Scenario: New Node Bootstrap

```
1. User boots LiveSpore USB on new computer
   └─▶ deploy.sh auto-detects arch (x86_64)
   └─▶ Copies primals from USB to /usr/local/bin/
   └─▶ Starts Tower atomic

2. New node joins family
   └─▶ Exchanges beacons with existing nodes
   └─▶ Verifies family lineage via Dark Forest
   └─▶ Registers in address book

3. Node checks for updates
   └─▶ Queries api.nestgate.io/genome/manifest
   └─▶ Downloads any newer versions
   └─▶ Self-updates primals
```

---

## 📊 Migration Checklist

- [ ] Create `wateringHole/genomeBin/` structure
- [ ] Generate `manifest.toml` from current binaries
- [ ] Generate `checksums.toml`
- [ ] Add `/genome/*` routes to `biomeos-api`
- [ ] Implement Dark Forest verification for downloads
- [ ] Update `PrimalRegistry` with `fetch_genome`
- [ ] Update `PrimalRegistry` with `update_all_primals`
- [ ] Update LiveSpore with cross-arch structure
- [ ] Update LiveSpore `deploy.sh` for auto-detect
- [ ] Test: Download genome from nestgate.io
- [ ] Test: Update primal on running node
- [ ] Test: Bootstrap new node from LiveSpore
- [ ] Document in `wateringHole/GENOME_DISTRIBUTION_STANDARD.md`

---

## 🎯 Benefits

1. **Centralized Truth**: `wateringHole/genomeBin/` is the authoritative source
2. **Secure Distribution**: Dark Forest Gate ensures only family members can download
3. **Public Fallback**: GitHub releases for public adoption
4. **Cross-Arch**: One manifest serves all architectures
5. **Self-Updating**: Nodes can update themselves
6. **Offline-First**: LiveSpore contains everything needed
7. **Verification**: Checksums and signatures ensure integrity

---

**Next Steps**: Implement Phase 1 (repository structure) and Phase 2 (API routes)

