# `biomeOS` - Headless Management Tool Specification v1

**Status:** Draft | **Author:** The Architect & The Artisan AI | **Date:** July 2025

**Related Documents:** [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md)

---

## 1. Preamble: The Nervous System

Once a biome is installed, it requires a robust, scriptable, and AI-friendly way to be managed. The `biome-ctl` (Biome Control) is this "nervous system"—a headless, command-line interface that serves as the primary tool for all "Day 2" operations.

It is designed explicitly to be used by both human operators and autonomous AI agents, ensuring that the management of a biome adheres to the same API-first principles as its installation.

## 2. Architecture: A Pure-Rust Client for `toadstool`

The `biome-ctl` is a pure-Rust application that acts as a client to the management API exposed by the `toadstool` Primal. `toadstool`, as the universal runtime, is the ultimate source of truth for the state of all running workloads. `biome-ctl` is simply a powerful and ergonomic interface to that truth.

```mermaid
graph TD
    subgraph "Management Clients"
        A["Human Operator"] --> C["`biome-ctl`"];
        B["AI Management Agent"] --> C;
    end

    subgraph "Running Biome"
        D["`toadstool` Primal<br>(Manages all workloads)"]
    end
    
    C -- "sends API commands" --> D;

    style D fill:#f9f
```

## 3. Core Commands

The `biome-ctl` will provide a set of intuitive commands for managing the lifecycle and observing the state of the biome.

-   **`biome-ctl status`**: Displays a high-level overview of the running biome, including the health and status of each Primal and workload.
    -   *Example Output:* `nestgate (running), songbird (running), beardog (unhealthy)`

-   **`biome-ctl list`**: Lists all running workloads (services, jobs) with their IDs and resource consumption.

-   **`biome-ctl logs <workload_id>`**: Streams the real-time logs from a specified workload. Supports following (`-f`).
    -   *Example:* `biome-ctl logs songbird -f`

-   **`biome-ctl exec <workload_id> -- <command>`**: Executes a command inside the sandboxed environment of a running workload.
    -   *Example:* `biome-ctl exec nestgate -- zfs list`

-   **`biome-ctl stop <workload_id>`**: Gracefully stops a running workload.

-   **`biome-ctl start <workload_id>`**: Starts a previously stopped workload.

-   **`biome-ctl update <workload_id> --image <new_image>`**: Triggers a safe, rolling update of a workload to a new version, managed by `toadstool`.

-   **`biome-ctl manifest apply -f <new_manifest.yaml>`**: Applies a new manifest to the running biome, allowing for declarative updates to the entire system state. `toadstool` will compute the diff and apply the necessary changes.

## 4. Sovereignty and Security

All commands executed by `biome-ctl` are subject to the security policies enforced by `bearDog`. An AI agent cannot simply execute `biome-ctl stop beardog`; the request would be intercepted by `toadstool`, validated against `bearDog`'s policy engine, and rejected. This ensures that even the powerful management interface cannot be used to compromise the biome's integrity. 