# `biomeOS` - Service Discovery Specification v1

**Status:** Draft | **Author:** The Architect & The Artisan AI | **Date:** July 2025

**Related Documents:** [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md)

---

## 1. Preamble: The Mycelial Network

For a biome to function as a resilient organism, its components must be able to find and communicate with each other without hard-coded dependencies. This specification defines the Service Discovery mechanism for `biomeOS`—a "mycelial network" that allows workloads to connect dynamically by name, not by fragile IP addresses.

This crucial function is orchestrated by `songbird`, the Primal responsible for all inter-service communication.

## 2. Architecture: `songbird` as an Internal DNS Provider

The architecture is simple, robust, and leverages existing standards. `songbird` will run an internal, private DNS server that is only accessible to workloads within the biome's virtual network.

`toadstool`, the runtime, and `songbird`, the network, work together to manage this system.

```mermaid
graph TD
    subgraph "Phase 1: Registration"
        A["`toadstool` receives<br>`biome.yaml`"] --> B["`toadstool` launches<br>workload: `nestgate`"];
        B --> C{"`toadstool` sends API call to `songbird`<br>Register('nestgate', '10.0.1.5')"};
        C --> D["`songbird`'s internal DNS adds A-record:<br>`nestgate.biome.local -> 10.0.1.5`"];
    end

    subgraph "Phase 2: Discovery"
        E["`beardog` workload needs<br>to connect to `nestgate`"] --> F["`beardog` performs DNS query:<br>resolve(`nestgate.biome.local`)"];
        F --> G["`songbird` DNS responds:<br>`10.0.1.5`"];
        G --> H["`beardog` connects to `nestgate`<br>at `tcp://10.0.1.5:8080`"];
    end

    style D fill:#f9f
    style H fill:#cfc
```

## 3. The Process

1.  **Launch & Registration:** When `toadstool` starts a new workload as defined in a manifest (e.g., `nestgate`), it assigns the workload an IP address within the biome's private network. Immediately after, `toadstool` makes an API call to `songbird`, registering the workload's name (`nestgate`) and its assigned IP address.
2.  **DNS Record Creation:** `songbird` creates a DNS `A` record, mapping the service name to the IP address under a private Top-Level Domain (TLD), `.biome.local`. For example: `nestgate.biome.local`.
3.  **Discovery:** When another workload (e.g., `beardog`) needs to communicate with `nestgate`, it simply uses the well-known DNS name `nestgate.biome.local`. The biome's internal network resolver (managed by `songbird`) correctly resolves this to the current IP address.
4.  **Resilience:** If `toadstool` needs to restart `nestgate` for any reason, it may receive a new IP address. `toadstool` simply repeats the registration process, and `songbird` updates the DNS record. All other services can continue to use the same DNS name without ever knowing the underlying IP address changed.

## 4. Security

The internal DNS server managed by `songbird` is not exposed to the outside world. It is only accessible on the private virtual network created for the biome. This prevents information leakage about the biome's internal structure and ensures that only authorized workloads can discover and communicate with each other. All such communication is, of course, still subject to the policies enforced by `bearDog`. 