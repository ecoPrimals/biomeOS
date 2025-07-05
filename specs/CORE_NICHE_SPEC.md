# `biomeOS` - Core Sovereignty Niche Specification v1

**Status:** Draft | **Author:** The Architect & The Artisan AI | **Date:** July 2025

**Related Documents:** [MANIFEST_SPEC_V1.md](./MANIFEST_SPEC_V1.md)

---

## 1. Preamble: The First Seed

To provide immediate value and a clear, working example for the community, the default `biomeOS-installer.iso` must ship with a foundational Niche. The "Core Sovereignty Niche" is this first seed—a minimal, secure, and useful biome that provides essential services for a sovereign digital life.

It serves as the "Hello, World!" of `ecoPrimals`, demonstrating the power of the integrated stack and providing a well-documented starting point for all future Niches.

## 2. Niche Package Definition

The Niche will be structured as a standard Niche Package, ready to be processed by the `iso-forge`.

-   **`niche.yaml`**:
    ```yaml
    id: "org.ecoprimals.core-sovereignty"
    name: "Core Sovereignty"
    author: "The ecoPrimals Project"
    description: "A foundational biome for a sovereign digital life, providing secure storage, service orchestration, and file synchronization."
    default_manifest: "core.yaml"
    ```
-   **`icon.png`**: The official `ecoPrimals` project logo.
-   **`manifests/core.yaml`**: The core manifest file (defined below).
-   **`workloads/`**: Contains the WASM binary for the `rust-sync` workload.

## 3. The Core Manifest (`core.yaml`)

This `biome.yaml` manifest defines the minimal set of Primals and one high-value application needed for a useful sovereign environment.

```yaml
version: 1
biome:
  name: "my-sovereign-biome"
  description: "A biome running the Core Sovereignty Niche."

networks:
  - name: "primal-net"

volumes:
  - name: "nestgate-storage"
  - name: "sync-data"

workloads:
  # The Core Primals, providing the foundation
  - name: "toadstool"
    runtime: "native" # Toadstool runs itself
    # ... other native config
  
  - name: "beardog"
    runtime: "wasm"
    image: "org.ecoprimals/beardog:latest"
    networks: ["primal-net"]
    
  - name: "songbird"
    runtime: "wasm"
    image: "org.ecoprimals/songbird:latest"
    networks: ["primal-net"]
    ports:
      - "80:80" # Expose web gateway
      - "443:443"

  - name: "nestgate"
    runtime: "wasm"
    image: "org.ecoprimals/nestgate:latest"
    networks: ["primal-net"]
    volumes:
      - "nestgate-storage:/data" # Mount the main storage volume

  # A high-value application for immediate utility
  - name: "rust-sync"
    runtime: "wasm"
    image: "org.opensource/rust-sync:latest"
    description: "A Syncthing-like service for peer-to-peer file synchronization."
    networks: ["primal-net"]
    volumes:
      - "sync-data:/sync" # Mount the volume for synced files
``` 