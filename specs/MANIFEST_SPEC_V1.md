# `biomeOS` - Manifest Specification v1

**Status:** Draft | **Author:** The Architect & The Artisan AI | **Date:** July 2025

**Related Documents:** [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md)

---

## 1. Preamble

This document defines the structure of the `biome.yaml` manifest file. This file is the "genome" of a `biomeOS` instance, providing a declarative blueprint that `toadstool`, the universal runtime, uses to bootstrap and manage the entire ecosystem.

The goal is a manifest that is human-readable, version-controllable, and powerful enough to describe complex interactions between Primals and other workloads.

## 2. Manifest Schema v1

The manifest is composed of several root keys:

-   `version`: (Required) The version of the manifest schema.
-   `biome`: (Required) Metadata about the biome itself.
-   `sources`: (Optional) A map of named sources for fetching workloads, promoting reuse.
-   `volumes`: (Optional) Defines storage volumes to be provisioned by `nestgate`.
-   `networks`: (Optional) Defines virtual networks to be managed by `songbird`.
-   `services`: (Required) A map of all services (Primals and applications) to be run by `toadstool`.

## 3. Conceptual `biome.yaml` Example

This example demonstrates how the four core Primals could be provisioned for a base `biomeOS` instance.

```yaml
# The genome for a standard, secure biomeOS instance.
version: "1.0"

biome:
  name: "ecoprimals-core-v1"
  description: "Core sovereign compute environment"

sources:
  # Defines a reusable source for our blessed Primal images
  primal_registry:
    type: "oci"
    url: "oci.ecoprimals.io/primal-foundry"

volumes:
  # Provisioned by nestgate, attached by toadstool
  beardog_db:
    driver: "nestgate-zfs"
    options:
      # These options would be passed to the nestgate driver
      recordsize: "128k"
      compression: "zstd"
      quota: "10G"
  nestgate_metadata:
    driver: "nestgate-zfs"
    options:
      recordsize: "1M"
      quota: "50G"

networks:
  # Provisioned by toadstool, configured/discovered by songbird
  primal_net:
    driver: "songbird-bridge"
    subnet: "10.42.0.0/16"

services:
  # The services are started in dependency order.
  # ToadStool would build a dependency graph and start in this order:
  # 1. beardog
  # 2. nestgate & songbird (in parallel)
  # 3. other apps...

  beardog:
    source: "primal_registry:beardog-v0.2.0"
    runtime: "wasm" # Prioritize Wasm for security
    networks:
      - "primal_net"
    volumes:
      - "beardog_db:/var/lib/beardog"
    # Capabilities and policies would be defined here
    # In a real system, beardog would have the highest privilege.
    security_context:
      privileged: true 

  nestgate:
    source: "primal_registry:nestgate-v0.1.5"
    runtime: "container" # Requires host access for ZFS, so use container
    networks:
      - "primal_net"
    volumes:
      - "nestgate_metadata:/var/lib/nestgate"
      - "/dev/zfs:/dev/zfs" # Example of privileged host path mount
    depends_on:
      - "beardog" # Must wait for the security provider

  songbird:
    source: "primal_registry:songbird-v0.3.1"
    runtime: "wasm"
    networks:
      - "primal_net"
    ports:
      - "80:8080" # Expose songbird's API to the host
      - "443:8443"
    depends_on:
      - "beardog" # Must wait for the security provider

  # --- Example of a user application ---
  # alphafold_worker:
  #   source: "ghcr.io/deepmind/alphafold:latest"
  #   runtime: "container"
  #   runtime_options:
  #     gpu_request: "1"
  #     gpu_type: "nvidia"
  #   networks:
  #     - "primal_net"
  #   depends_on:
  #     - "nestgate"
  #     - "songbird"
  #     - "beardog"

```

## 4. Inoculum Analysis

This manifest acts as a "code inoculum," carrying the genetic instructions for the whole system.

-   **`toadstool`'s Role:** It is the "cell." It reads this genome and begins the process of division and differentiation, creating the sandboxes for each service. It uses `runtime: wasm` vs `runtime: container` to provide different levels of isolation based on need.
-   **`nestgate`'s Role:** It functions as the "nutrient store," using the `volumes` section to create persistent, specialized storage areas (`beardog_db`) before they are needed.
-   **`songbird`'s Role:** It is the "nervous system," using the `networks` definition to create the communication pathways that will connect all the services once they are running.
-   **`beardog`'s Role:** It is the "immune system," started first because it must be present to validate the identity and policies of all subsequent services. The `depends_on` key establishes this critical startup order.

This manifest provides a clear, declarative plan for assembling the Primals into a functional, integrated `biomeOS`. It is the blueprint for the first living specimen. 