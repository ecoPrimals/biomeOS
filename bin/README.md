# biomeOS bin/

**Updated**: May 27, 2026

## Contents

- **`primals/`** — Compiled primal binaries (gitignored; populated by `scripts/build_primals_for_testing.sh` or `biomeos-harvest`)

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
