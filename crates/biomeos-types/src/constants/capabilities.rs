// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capability constants for discovery
//!
//! **DESIGN PRINCIPLE**: Query by capability, not by primal name.
//!
//! These constants are used for capability-based discovery through the
//! universal adapter. No primal should hardcode knowledge of
//! other primals by name.
//!
//! # Example
//! ```
//! use biomeos_types::constants::capabilities;
//!
//! assert_eq!(capabilities::COMPUTE, "compute");
//! assert_eq!(capabilities::STORAGE, "storage");
//! assert_eq!(capabilities::SECURITY, "security");
//! ```

/// Compute and execution capability (e.g., ToadStool)
pub const COMPUTE: &str = "compute";

/// Storage and persistence capability (e.g., NestGate)
pub const STORAGE: &str = "storage";

/// Security and cryptography capability (e.g., BearDog)
pub const SECURITY: &str = "security";

/// AI and intelligence capability (e.g., Squirrel)
pub const AI: &str = "ai";

/// Discovery and service mesh capability (e.g., Songbird)
pub const DISCOVERY: &str = "discovery";

/// Orchestration capability (e.g., BiomeOS, Songbird)
pub const ORCHESTRATION: &str = "orchestration";

/// UI and visualization capability (e.g., PetalTongue)
pub const VISUALIZATION: &str = "visualization";

/// Networking capability
pub const NETWORKING: &str = "networking";

/// Monitoring and observability capability
pub const MONITORING: &str = "monitoring";

/// Data processing capability
pub const DATA_PROCESSING: &str = "data-processing";

// =====================================================================
// Spring capability domains — registered by springs at runtime.
// =====================================================================

/// Ecology and agriculture capability (airSpring)
pub const ECOLOGY: &str = "ecology";

/// Life science, analytical chemistry, microbial ecology (wetSpring)
pub const SCIENCE: &str = "science";

/// Medical, PK/PD, microbiome, biosignal (healthSpring)
pub const MEDICAL: &str = "medical";

/// Game science, HCI, procedural content (ludoSpring)
pub const GAME: &str = "game";

/// Computational physics, nuclear EOS, GPU compute (hotSpring)
pub const PHYSICS: &str = "physics";

/// Measurement, signal processing, inverse problems (groundSpring)
pub const MEASUREMENT: &str = "measurement";

/// Machine learning, surrogates, isomorphic patterns (neuralSpring)
pub const LEARNING: &str = "learning";
