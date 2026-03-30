// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capability domain constants for discovery
//!
//! **`WateringHole` standard**: No hardcoded primal names, capability-based discovery.
//! A primal should only know about itself and discover others at runtime via capabilities.
//!
//! Use these constants when querying for primals by what they do, not by name.
//! Primal names (beardog, songbird, etc.) are implementation details discovered at runtime.
//!
//! # Example
//! ```
//! use biomeos_types::constants::capability;
//!
//! assert_eq!(capability::CRYPTO, "crypto");
//! assert_eq!(capability::STORAGE, "storage");
//! assert_eq!(capability::MESH_NETWORKING, "mesh_networking");
//! ```

/// Crypto/security capability (family seed, signing, encryption)
pub const CRYPTO: &str = "crypto";

/// Family seed and lineage operations
pub const FAMILY_SEED: &str = "crypto.family_seed";

/// Mesh networking / P2P relay
pub const MESH_NETWORKING: &str = "mesh_networking";

/// TLS and secure transport
pub const TLS: &str = "tls";

/// Storage and data persistence
pub const STORAGE: &str = "storage";

/// Gateway / NAT traversal
pub const GATEWAY: &str = "gateway";

/// NAT traversal (hole punch, STUN, relay)
pub const NAT_TRAVERSAL: &str = "nat_traversal";

/// Caching capability
pub const CACHING: &str = "caching";

/// Visualization / UI rendering
pub const VISUALIZATION: &str = "visualization";

/// Graph database
pub const GRAPH_DATABASE: &str = "graph_database";

/// Persistence / durable storage
pub const PERSISTENCE: &str = "persistence";

/// GPU compute acceleration
pub const GPU_COMPUTE: &str = "gpu_compute";

/// Cryptographic signing
pub const SIGNING: &str = "crypto.sign";

/// Encryption/decryption
pub const ENCRYPTION: &str = "crypto.encrypt";
