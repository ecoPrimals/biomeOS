# biomeOS bin/

**Updated**: April 28, 2026

## Contents

- **`tower`** — Symlink / wrapper for the main biomeOS binary
- **`primals/`** — Compiled primal binaries (gitignored; populated by `scripts/build_primals_for_testing.sh` or `biomeos-harvest`)
- **`chimeras/`** — Chimera composition crate (p2p-secure, gaming-mesh)

## Building Primals

```bash
# From workspace root
scripts/build_primals_for_testing.sh

# Or via harvest tool (from tools/harvest/ — excluded from root workspace)
cd tools/harvest && cargo run -- local --all
```

Primal binaries are **not committed** — they are built from source or harvested
from plasmidBin / GitHub releases.

## Related

- [`scripts/`](../scripts/) — Build and deployment scripts
- [`tools/harvest/`](../tools/harvest/) — Primal binary harvesting system
