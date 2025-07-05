# `biomeOS` - API & Struct Inoculum v1

**Status:** Draft | **Author:** The Architect & The Artisan AI | **Date:** July 2025

**Related Documents:** [MANIFEST_SPEC_V1.md](./MANIFEST_SPEC_V1.md)

---

## 1. Preamble

This document serves as a "code inoculum" for the `biomeOS`. It contains key Rust data structures, copy-pasted from the mature Primal projects, that define the concrete shape of the `biome.yaml` manifest and the primary APIs for inter-Primal communication.

This provides a definitive, code-level starting point for the `toadstool` orchestrator and standardizes the "language" that Primals will use to speak to each other.

## 2. Manifest Deserialization Structs

These are the core Rust structs that the `toadstool` manifest parser will use. They are derived from the `config` and `workload` modules of the existing Primals and directly correspond to the `biome.yaml` schema.

```rust
// Inoculated from toadstool/crates/core/config/src/manifest.rs

use serde::Deserialize;
use std::collections::HashMap;

/// The root of the biome.yaml manifest file. This is the genome.
#[derive(Debug, Clone, Deserialize)]
pub struct BiomeManifest {
    pub version: String,
    pub biome: BiomeMeta,
    #[serde(default)]
    pub sources: HashMap<String, SourceDefinition>,
    #[serde(default)]
    pub volumes: HashMap<String, VolumeDefinition>,
    #[serde(default)]
    pub networks: HashMap<String, NetworkDefinition>,
    pub services: HashMap<String, ServiceDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BiomeMeta {
    pub name: String,
    pub description: String,
}

/// Defines a source for fetching workloads (e.g., an OCI registry).
#[derive(Debug, Clone, Deserialize)]
pub struct SourceDefinition {
    #[serde(rename = "type")]
    pub source_type: String, // e.g., "oci", "git", "ipfs"
    pub url: String,
}

/// Defines a storage volume to be provisioned by nestgate.
#[derive(Debug, Clone, Deserialize)]
pub struct VolumeDefinition {
    pub driver: String, // e.g., "nestgate-zfs"
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// Defines a virtual network to be managed by songbird.
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkDefinition {
    pub driver: String, // e.g., "songbird-bridge"
    pub subnet: String,
}

/// Defines a single service, Primal, or application to be run by toadstool.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceDefinition {
    /// A reference to a source from the `sources` map, plus the image name/tag.
    /// Example: "primal_registry:beardog-v0.2.0"
    pub source: String,

    /// The requested runtime environment.
    /// Inoculated from toadstool/crates/runtime/wasm/src/lib.rs
    #[serde(default = "default_runtime")]
    pub runtime: RuntimeType,

    /// Networks to attach to this service.
    #[serde(default)]
    pub networks: Vec<String>,
    
    /// Volumes to mount into the workload's sandbox.
    /// Format: "volume_name:/path/in/workload"
    #[serde(default)]
    pub volumes: Vec<String>,

    /// Ports to expose from the workload to the host.
    /// Format: "host_port:workload_port"
    #[serde(default)]
    pub ports: Vec<String>,
    
    /// A list of services that must be healthy before this one starts.
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Security context and permissions for this workload.
    /// This would be passed to `beardog` for policy enforcement.
    #[serde(default)]
    pub security_context: SecurityContext,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    Wasm,
    Container,
    Native,
}

fn default_runtime() -> RuntimeType {
    RuntimeType::Wasm
}
```

## 3. Security Context API

This struct, inoculated from `beardog`, defines the permissions and capabilities a service will be granted. It is parsed by `toadstool` from the manifest and enforced in collaboration with the `beardog` Primal.

```rust
// Inoculated from beardog/src/core.rs

use serde::Deserialize;

/// Defines the security posture and permissions for a running service.
/// This is the core of the Principle of Least Privilege in a biomeOS.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SecurityContext {
    /// Whether the workload can access host-level resources.
    /// This should be extremely rare and is required for Primals like `nestgate`.
    #[serde(default)]
    pub privileged: bool,

    /// A list of specific capabilities to grant the workload (e.g., "NET_RAW").
    #[serde(default)]
    pub capabilities_add: Vec<String>,

    /// A list of capabilities to explicitly deny.
    #[serde(default)]
    pub capabilities_drop: Vec<String>,

    /// Filesystem paths the workload can access, with read/write permissions.
    /// Example: `"/data:ro"`
    #[serde(default)]
    pub allowed_paths: Vec<String>,
}
```

## 4. Service Registration API (Songbird Pattern)

This struct defines the "birth announcement" that each Primal will send to `songbird` upon starting. It provides `songbird` with all the information it needs to add the service to its registry and start routing traffic to it.

```rust
// Inoculated from songbird/src/registry/types.rs (conceptual)

use serde::Serialize;
use std::collections::HashMap;

/// The message a service sends to songbird to register itself in the service mesh.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceRegistration {
    /// The unique name of the service, from the manifest.
    pub service_name: String,

    /// The runtime type, for songbird's awareness.
    pub runtime: RuntimeType, 

    /// The internal IP address and port where the service's API is listening.
    /// This is on the private `primal_net`.
    pub internal_address: String, // e.g., "10.42.0.5:8000"

    /// The public-facing paths that should be routed to this service.
    /// Example: `"/api/v1/storage"`
    pub public_routes: Vec<String>,

    /// A URL for songbird to poll for health checks.
    pub health_check_url: String,

    /// Any additional metadata the service wants to provide.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}
```

This inoculum provides the concrete, code-level foundation for the `biomeOS`. It ensures that the concepts defined in the architecture and manifest specifications have a direct, verifiable implementation path rooted in the mature code of the existing Primals. 