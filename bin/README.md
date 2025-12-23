# BiomeOS Binary Directory

This directory stores compiled primal binaries for use by BiomeOS.

## Structure

```
bin/
├── primals/          # Standard primal binaries (pulled from parent repos)
│   ├── beardog       # Cryptographic identity & security
│   ├── songbird      # Orchestration & discovery
│   ├── toadstool     # Universal compute
│   ├── nestgate      # Storage & persistence
│   └── squirrel      # AI coordination
│
└── chimeras/         # Compiled chimera binaries (amalgams)
    ├── p2p-secure    # BearDog + Songbird P2P mesh
    ├── ml-pipeline   # ToadStool + NestGate + Squirrel
    └── ...           # Custom chimeras
```

## Pulling Primal Binaries

BiomeOS can pull and cache primal binaries from their parent repositories:

```bash
# Pull all primals
biomeos primal pull --all

# Pull specific primal
biomeos primal pull beardog

# Pull with specific version
biomeos primal pull songbird@2.0.0
```

## Building Chimeras

Chimeras are built from chimera definitions in `chimeras/definitions/`:

```bash
# Build a chimera from definition
biomeos chimera build p2p-secure

# Build all chimeras
biomeos chimera build --all
```

## .gitignore

Binaries are not committed to git. This directory is populated at build/deploy time.

