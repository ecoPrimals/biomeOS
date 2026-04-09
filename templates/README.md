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

See `niches/templates/` at repository root for the primary niche definitions.
The `templates/niches/` subdirectory here contains supplementary templates:

- `spring-template.yaml` — Spring niche base template
- `gaming.yaml` — Gaming niche configuration

## Test Configurations

| File | Purpose |
|------|---------|
| `test-biome.yaml` | Basic test manifest |

## Usage

```bash
# Deploy a biome manifest
biomeos deploy -m templates/biome.yaml

# Use as a niche template
biomeos niche deploy gaming-tournament

# Or from source
cargo run -p biomeos -- deploy -m templates/biome.yaml
```

## See Also

- `niches/templates/` - Primary location for niche definitions
- `chimeras/definitions/` - Chimera specifications
- `examples/` - Code examples

