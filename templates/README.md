# BiomeOS Templates

Example biome manifests and configuration templates.

## Biome Manifests

These YAML files define complete biome configurations:

| File | Description |
|------|-------------|
| `biome.yaml` | Universal template (primal-agnostic) |
| `ai-research.yaml` | AI/ML research environment |
| `ai-team.biome.yaml` | AI team collaboration |
| `ai-training.biome.yaml` | Model training workloads |
| `gaming-team.biome.yaml` | Gaming development |
| `tournament.biome.yaml` | Gaming tournament setup |
| `frontend-team.biome.yaml` | Frontend development |
| `frontend-webapp.biome.yaml` | Web application deployment |
| `federation-demo.yaml` | Federation showcase |
| `federation-showcase.yaml` | Multi-node federation |
| `songbird-coordination.yaml` | Songbird mesh config |
| `biome-with-crypto-locks.yaml` | BearDog security example |

## Niche Templates

The `niches/` subdirectory contains reusable niche definitions:

```
niches/
├── ai-research.yaml
├── custom-generic.yaml
├── federation-aware.yaml
├── gaming-tournament.yaml
├── research-lab.yaml
└── web-development.yaml
```

## Test Configurations

| File | Purpose |
|------|---------|
| `test-biome.yaml` | Basic test manifest |

## Usage

```bash
# Deploy a biome manifest
cargo run -p biomeos-cli --bin biomeos -- deploy -m templates/biome.yaml

# Use as a niche template
cargo run -p biomeos-cli --bin biomeos -- niche deploy gaming-tournament
```

## See Also

- `niches/templates/` - Primary location for niche definitions
- `chimeras/definitions/` - Chimera specifications
- `examples/` - Code examples

