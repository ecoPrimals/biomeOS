# `biomeOS` - Architecture Overview v1

**Status:** Draft | **Author:** The Architect & The Artisan AI | **Date:** July 2025

---

## 1. Preamble: From Organs to Organism

This document synthesizes the individual Primal components (`toadstool`, `nestgate`, `songbird`, `beardog`) into a single, coherent system architecture for a `biomeOS` instance. It treats each mature Primal as a "culture" and this specification as the act of "inoculating" a new, unified biome with their combined capabilities.

The core design is a layered stack, where each Primal provides a distinct, non-overlapping service, creating a resilient and sovereign compute environment from the bare metal up.

## 2. The `biomeOS` Stack: A Layered Architecture

The ecosystem is designed as a hierarchy of services, where higher layers depend on the foundational capabilities provided by the lower layers.

```mermaid
graph TD
    subgraph "User & Application Layer"
        A[Workloads / Applications]
        U[User / External APIs]
    end

    subgraph "Coordination & Security Layer"
        S[songbird]
        B[beardog]
    end

    subgraph "Runtime & Resource Layer"
        T[toadstool]
        N[nestgate]
    end

    subgraph "Physical Layer"
        H[Hardware: CPU, GPU, Disk, Network]
    end

    A --> S
    U --> S

    S --> B
    T --> B
    N --> B

    S -.-> T
    T --> N
    
    N --> H
    T --> H

    linkStyle 7,8,9,10 stroke-width:2px,stroke:red,stroke-dasharray: 5 5;
```

### **Layer 0: Physical Hardware**
- The foundation. Consists of the raw compute, storage, and networking hardware.

### **Layer 1: Runtime & Resource Primals**
- **`toadstool` (The Universal Runtime):** The first and most essential Primal. It runs directly on the hardware and is responsible for bootstrapping the entire `biomeOS`. It creates secure, sandboxed environments (Wasm, containers) for all other Primals and applications to run within. It abstracts the physical hardware for the layers above.
- **`nestgate` (The Sovereign Storage):** Manages the physical storage hardware. It provides an intelligent, tiered, and resilient storage layer (ZFS pools, datasets, snapshots) that `toadstool` can provision for any running workload.

### **Layer 2: Coordination & Security Primals**
- **`songbird` (The Service Mesh & API Gateway):** This is the nervous system. It handles all network communication, both within the `biomeOS` (service-to-service) and externally. It discovers services started by `toadstool`, manages load balancing, and exposes a unified API endpoint for the outside world.
- **`beardog` (The Immune System):** The security Primal. It is not a true layer but a cross-cutting concern that underpins the entire stack. `toadstool` enforces `beardog`'s access control policies at the runtime level. `nestgate` uses `beardog` for encrypting data at rest. `songbird` uses `beardog` to secure communications and authenticate API requests. It is the root of trust for the entire system.

### **Layer 3: Applications & Workloads**
- This is the user-facing layer. It consists of the actual applications, AI models, or services (e.g., an AlphaFold instance, a database, a web UI) that run within the `biomeOS`. These workloads are managed by `toadstool`, networked by `songbird`, stored on `nestgate`, and secured by `beardog`.

## 3. Primal Maturity and API Compatibility Assessment

Based on a thorough review of the four component projects:

- **Maturity:** All four Primals (`beardog`, `nestgate`, `songbird`, `toadstool`) exhibit a high degree of maturity. They contain extensive documentation, modular codebases, comprehensive test plans, and clear `Cargo.toml` definitions. They appear more than ready for initial integration testing.
- **API Compatibility:** The projects were clearly designed with each other in mind. The `Cargo.toml` files show explicit dependencies and integrations (e.g., `toadstool/crates/integration/nestgate`). The code itself reveals shared types and concepts (e.g., `songbird` re-exporting `beardog` security types). While minor mismatches may exist, the architectural alignment is extremely strong. Given the established development velocity, resolving any specific API incompatibilities would be a trivial task, likely taking less than a day per issue.

The foundational components are sound and ready to be combined. The next step is to define the declarative manifest that will instruct `toadstool` on how to assemble them into a living `biomeOS`.

## Core Philosophy

biomeOS is a biological computing platform that uses natural ecosystem metaphors to create sovereign, intelligent, and adaptive computing environments. The system is completely agnostic to specific implementations, allowing any system to participate as a "Primal" through standardized interfaces.

## MYCORRHIZA: Energy Flow Management

MYCORRHIZA is the universal energy flow management system that controls all external access to biomeOS ecosystems. Named after the underground fungal networks that protect and coordinate forest ecosystems, MYCORRHIZA manages three energy flow states:

### Energy Flow States

#### Closed System (Default)
- **Foundation locked** to external access
- **All Primals locked** to external APIs/services
- **Personal AI cat door**: llama.cpp + personal API keys for big AI
- **Internal freedom**: Primals communicate freely within the biome
- **Sovereignty maintained**: Zero external dependencies beyond personal AI

#### Private Open System (Trust-Based)
- **Selective external access** via crypto keys granted on good faith
- **Relationship sovereignty**: User controls who gets access
- **Research partnerships**: Trusted developers, academic collaborations
- **Still protected**: MYCORRHIZA monitors all external flows

#### Commercial Open System (Pay-to-Play)
- **Enterprise integrations** require commercial licensing
- **Cloud provider access**: AWS, GCP, Azure must pay for integration
- **Revenue model**: Funds biomeOS development and sovereignty
- **Market pressure**: Integrate with biological computing or be irrelevant

### Security Enforcement

MYCORRHIZA implements comprehensive protection:
- **Deep packet inspection** of all outbound traffic
- **API signature detection** for known external services
- **Behavioral analysis** for anomalous patterns
- **ML-based detection** for unknown external APIs
- **Autonomous response**: Block, preserve evidence, maintain sovereignty

## Core Components

### Universal Primal System

Every component in biomeOS implements the `Primal` trait, making the system completely extensible:

```rust
trait Primal {
    fn primal_type(&self) -> String;
    fn capabilities(&self) -> Vec<Capability>;
    fn health_status(&self) -> HealthStatus;
    fn resource_requirements(&self) -> ResourceRequirements;
    fn mycorrhiza_compliance(&self) -> ComplianceStatus;  // NEW: MYCORRHIZA integration
}
``` 