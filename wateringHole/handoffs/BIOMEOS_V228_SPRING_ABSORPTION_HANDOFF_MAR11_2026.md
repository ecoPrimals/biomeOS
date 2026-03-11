# biomeOS V2.28 — Spring Absorption Handoff — March 11, 2026

**From:** biomeOS v2.28
**To:** All springs, petalTongue, barraCuda, toadStool
**License:** AGPL-3.0-only
**Supersedes:** BIOMEOS_V227_CONTINUOUS_XR_SURGICAL_HANDOFF_MAR11_2026

---

## Executive Summary

biomeOS v2.28 absorbs capabilities from all 7 springs and petalTongue V1.6.1,
adding 25+ capability translations, 4 deploy graphs, 4 niche templates, and
expanded domain keywords. Total translations: 165+.

## What Changed

### 1. Capability Translation Absorption

| Source | Methods Added | Domain |
|--------|-------------|--------|
| **wetSpring V110** | `science.kinetics.gompertz`, `.first_order`, `.monod`, `.haldane`, `science.beta_diversity`, `science.rarefaction`, `science.nmf`, `science.monitoring`, `science.brain.observe`, `.attention`, `.urgency`, `science.metrics` | Kinetics, diversity, telemetry |
| **airSpring v0.7.5** | `ecology.spi_drought_index`, `ecology.autocorrelation`, `ecology.gamma_cdf`, `ecology.bootstrap_ci`, `ecology.jackknife_ci` | Drought, statistics |
| **petalTongue V1.6.1** | `interaction.sensor_stream.subscribe`, `.poll`, `.unsubscribe`, `interaction.poll`, `interaction.unsubscribe`, `visualization.render.stream`, `visualization.render.dashboard` | Sensor events, live viz |
| **healthSpring V20** | `medical.mm_pk_simulate`, `medical.scfa_production`, `medical.classify_beat`, `medical.assess_stress`, `medical.trt_pipeline` | PK/PD, biosignal, clinical |

### 2. Deploy Graphs

| Graph | Springs | Purpose |
|-------|---------|---------|
| `hotspring_deploy.toml` | hotSpring + Tower + ToadStool | Physics simulation primal |
| `groundspring_deploy.toml` | groundSpring + Tower + ToadStool | Measurement science primal |
| `healthspring_deploy.toml` | healthSpring + Tower + ToadStool | Medical science primal |
| `cross_spring_ecology.toml` | airSpring + wetSpring + neuralSpring | ET₀ → diversity → spectral pipeline |

### 3. Niche Templates

| Niche ID | Category | Springs |
|----------|----------|---------|
| `ecology-pipeline` | science | airSpring + wetSpring + neuralSpring |
| `hotspring` | science | hotSpring |
| `groundspring` | science | groundSpring |
| `healthspring` | medical | healthSpring |

### 4. Capability Domain Expansion

| Domain | New Keywords |
|--------|-------------|
| wetSpring (science) | `kinetics`, `monitoring` |
| airSpring (ecology) | `drought`, `statistics` |
| petalTongue (visualization) | `sensor_stream` |

## Cross-Spring Impact

### For hotSpring
- Deploy graph available: `niche.deploy { template_id: "hotspring" }`
- barraCuda sovereign dispatch routing recognized in compute domain

### For groundSpring
- Deploy graph available: `niche.deploy { template_id: "groundspring" }`
- Neural API methods already wired from V98 NUCLEUS integration

### For wetSpring
- 14 new translations route through wetSpring's IPC server
- Kinetics and brain methods now discoverable via `capability.call`
- Cross-spring ecology pipeline chains airSpring ET₀ into wetSpring diversity

### For airSpring
- 5 new translations for SPI, ACF, gamma, bootstrap, jackknife
- Drought monitoring capabilities discoverable ecosystem-wide
- Cross-spring ecology pipeline originates from airSpring ET₀

### For neuralSpring
- Cross-spring ecology pipeline terminates with neuralSpring spectral analysis
- Kokkos parity benchmarks tracked (12.4× gap noted from hotSpring)

### For healthSpring
- 5 new medical translations: MM PK, SCFA, beat classify, stress, TRT
- Deploy graph + niche template for standalone medical primal deployment

### For petalTongue
- 8 new translations: sensor stream + interaction events + live viz
- Self-registration path (`lifecycle.register` + 30s heartbeat) validated
- `interaction.sensor_stream.*` methods enable ludoSpring Fitts/Hick analysis

### For ludoSpring
- Can subscribe to petalTongue sensor stream via biomeOS capability routing
- 7 GameDataChannel types mapped through petalTongue (per V1.6.1 handoff)

## Metrics

| Metric | v2.27 | v2.28 |
|--------|-------|-------|
| Capability translations | 140+ | 165+ |
| Deploy graphs | 17 | 21 |
| Niche templates | 9 | 13 |
| Domain keywords | 58 | 63 |
| Tests | 3,670+ | 3,670+ |

## Spring Status Summary (Mar 11, 2026)

| Spring | Version | Tests | Key Capability |
|--------|---------|-------|----------------|
| hotSpring | v0.6.29 | 847+ | Precision Brain, sovereign dispatch, Kokkos parity |
| groundSpring | V100 | 936 | NUCLEUS integration, Neural API |
| neuralSpring | S144 | 1,112+ | Streaming bio parsers, BLAST pipeline |
| wetSpring | V110 | 1,611 | R industry parity (vegan/DADA2/phyloseq) |
| airSpring | v0.7.5 | 1,051+ | 35 capabilities, SPI drought |
| healthSpring | V20 | 395 | 5 tracks, clinical TRT |
| ludoSpring | V2 | 123 | 13 HCI models, first continuous niche |
| petalTongue | V1.6.1 | 3,245 | Self-registration, sensor stream |

## Remaining Gaps

| Gap | Owner | Priority |
|-----|-------|----------|
| Songbird `discover_capabilities` | Songbird | P0 (blocks pure runtime discovery) |
| NestGate inverted boolean | NestGate | P1 |
| GPU BLAST (Smith-Waterman) | barraCuda | P1 |
| Kokkos 12.4× performance gap | hotSpring/barraCuda | P2 |
| Cross-spring config centralization | All springs | P3 |

---

*This handoff is unidirectional: biomeOS → ecosystem. No response expected.*
