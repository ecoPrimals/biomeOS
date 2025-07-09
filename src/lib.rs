//! # biomeOS - Universal Biological Computing Platform
//!
//! biomeOS provides a unified platform for orchestrating the five Primals
//! (Songbird, NestGate, Toadstool, BearDog, Squirrel) into a cohesive
//! biological computing environment.

pub use biomeos_core::*;
pub mod universal_adapter;

/// Re-export ecosystem integration for external use
pub mod ecosystem {
    pub use biomeos_core::ecosystem_integration::*;
}

/// Re-export universal adapter for federation coordination
pub mod federation {
    pub use crate::universal_adapter::{
        BiomeOSUniversalAdapter, PrimalCoordination, CoordinationResult,
        CoordinationStatus, UniversalFederationCoordination
    };
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!(
    "biomeOS v", env!("CARGO_PKG_VERSION")
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_build_info() {
        assert!(BUILD_INFO.contains("biomeOS"));
    }
} 