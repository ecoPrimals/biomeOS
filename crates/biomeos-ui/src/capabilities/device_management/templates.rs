// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Built-in Niche Templates for Device Management
//!
//! Defines standard deployment templates (tower, node) for orchestration.
//! These templates describe the primal composition and resource requirements
//! for common deployment patterns.
//!
//! # Architecture
//!
//! Templates are used by biomeOS to bootstrap coordinated primal deployments:
//! - **tower**: Minimal secure base (BearDog + Songbird)
//! - **node**: Compute-ready node (tower + ToadStool GPU)
//!
//! # Adding New Templates
//!
//! Templates should be added here rather than inline in provider.rs to:
//! 1. Keep provider logic focused on capability handling
//! 2. Allow templates to be loaded from config files in the future
//! 3. Enable template versioning and evolution

use super::types::{NicheTemplate, PrimalRole, ResourceRequirements};

/// Get all built-in niche templates
///
/// Returns the standard templates that come with biomeOS.
/// These can be extended by loading additional templates from config.
#[must_use]
pub fn builtin_templates() -> Vec<NicheTemplate> {
    vec![tower_template(), node_template()]
}

/// Tower template - minimal secure base
///
/// The tower template provides the foundational security and discovery
/// infrastructure needed for any biomeOS deployment.
///
/// # Required Primals
///
/// - **security**: BearDog for cryptography and identity
/// - **discovery**: Songbird for mesh networking and P2P
///
/// # Resources
///
/// Lightweight footprint suitable for edge devices:
/// - 2 CPU cores, 512MB RAM, 1GB storage
/// - No GPU required
#[must_use]
pub fn tower_template() -> NicheTemplate {
    NicheTemplate {
        id: "tower".to_string(),
        name: "Tower (Secure Base)".to_string(),
        description: "BearDog + Songbird atomic deployment".to_string(),
        required_primals: vec![
            PrimalRole {
                role: "security".to_string(),
                capabilities: vec!["crypto".to_string(), "identity".to_string()],
                min_health: 0.9,
                metadata: serde_json::json!({}),
            },
            PrimalRole {
                role: "discovery".to_string(),
                capabilities: vec!["mesh".to_string(), "p2p".to_string()],
                min_health: 0.8,
                metadata: serde_json::json!({}),
            },
        ],
        optional_primals: vec![],
        estimated_resources: ResourceRequirements {
            cpu_cores: 2,
            memory_mb: 512,
            storage_gb: 1,
            gpu_required: false,
            network_bandwidth_mbps: 10,
        },
        metadata: serde_json::json!({}),
    }
}

/// Node template - compute-ready node
///
/// The node template extends tower with GPU compute capabilities
/// via ToadStool, suitable for AI/ML workloads.
///
/// # Required Primals
///
/// - **security**: BearDog for cryptography
/// - **discovery**: Songbird for mesh networking
/// - **compute**: ToadStool for GPU and CPU workloads
///
/// # Resources
///
/// Heavier footprint for compute workloads:
/// - 4 CPU cores, 2GB RAM, 10GB storage
/// - GPU required
#[must_use]
pub fn node_template() -> NicheTemplate {
    NicheTemplate {
        id: "node".to_string(),
        name: "Node (Compute Ready)".to_string(),
        description: "Tower + Toadstool for compute workloads".to_string(),
        required_primals: vec![
            PrimalRole {
                role: "security".to_string(),
                capabilities: vec!["crypto".to_string()],
                min_health: 0.9,
                metadata: serde_json::json!({}),
            },
            PrimalRole {
                role: "discovery".to_string(),
                capabilities: vec!["mesh".to_string()],
                min_health: 0.8,
                metadata: serde_json::json!({}),
            },
            PrimalRole {
                role: "compute".to_string(),
                capabilities: vec!["gpu".to_string(), "cpu".to_string()],
                min_health: 0.8,
                metadata: serde_json::json!({}),
            },
        ],
        optional_primals: vec![],
        estimated_resources: ResourceRequirements {
            cpu_cores: 4,
            memory_mb: 2048,
            storage_gb: 10,
            gpu_required: true,
            network_bandwidth_mbps: 100,
        },
        metadata: serde_json::json!({}),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates_count() {
        let templates = builtin_templates();
        assert_eq!(
            templates.len(),
            2,
            "Should have exactly 2 built-in templates"
        );
    }

    #[test]
    fn test_tower_template_structure() {
        let tower = tower_template();
        assert_eq!(tower.id, "tower");
        assert!(!tower.required_primals.is_empty());
        assert_eq!(tower.required_primals.len(), 2);

        // Check security role
        let security = tower.required_primals.iter().find(|r| r.role == "security");
        assert!(security.is_some(), "Tower should have security role");
        assert!(
            security
                .unwrap()
                .capabilities
                .contains(&"crypto".to_string())
        );

        // Check discovery role
        let discovery = tower
            .required_primals
            .iter()
            .find(|r| r.role == "discovery");
        assert!(discovery.is_some(), "Tower should have discovery role");
    }

    #[test]
    fn test_node_template_requires_gpu() {
        let node = node_template();
        assert_eq!(node.id, "node");
        assert!(node.estimated_resources.gpu_required);
        assert_eq!(node.required_primals.len(), 3);

        // Check compute role
        let compute = node.required_primals.iter().find(|r| r.role == "compute");
        assert!(compute.is_some(), "Node should have compute role");
        assert!(compute.unwrap().capabilities.contains(&"gpu".to_string()));
    }

    #[test]
    fn test_tower_resources_lightweight() {
        let tower = tower_template();
        assert!(!tower.estimated_resources.gpu_required);
        assert!(tower.estimated_resources.cpu_cores <= 2);
        assert!(tower.estimated_resources.memory_mb <= 1024);
    }

    #[test]
    fn test_node_resources_heavier() {
        let node = node_template();
        let tower = tower_template();

        // Node should require more resources than tower
        assert!(node.estimated_resources.cpu_cores > tower.estimated_resources.cpu_cores);
        assert!(node.estimated_resources.memory_mb > tower.estimated_resources.memory_mb);
    }

    #[test]
    fn test_templates_have_descriptions() {
        for template in builtin_templates() {
            assert!(
                !template.description.is_empty(),
                "Template {} should have description",
                template.id
            );
            assert!(
                !template.name.is_empty(),
                "Template {} should have name",
                template.id
            );
        }
    }

    #[test]
    fn test_primal_roles_have_min_health() {
        for template in builtin_templates() {
            for role in &template.required_primals {
                assert!(
                    role.min_health > 0.0,
                    "Role {} should have min_health > 0",
                    role.role
                );
                assert!(
                    role.min_health <= 1.0,
                    "Role {} should have min_health <= 1",
                    role.role
                );
            }
        }
    }

    #[test]
    fn test_niche_template_serde_roundtrip() {
        let tower = tower_template();
        let json = serde_json::to_string(&tower).unwrap();
        let parsed: NicheTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, tower.id);
        assert_eq!(parsed.name, tower.name);
        assert_eq!(parsed.required_primals.len(), tower.required_primals.len());
    }

    #[test]
    fn test_primal_role_serde_roundtrip() {
        let role = PrimalRole {
            role: "security".to_string(),
            capabilities: vec!["crypto".to_string()],
            min_health: 0.9,
            metadata: serde_json::json!({}),
        };
        let json = serde_json::to_string(&role).unwrap();
        let parsed: PrimalRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, role.role);
        assert!((parsed.min_health - role.min_health).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resource_requirements_serde_roundtrip() {
        let reqs = ResourceRequirements {
            cpu_cores: 4,
            memory_mb: 2048,
            storage_gb: 10,
            gpu_required: true,
            network_bandwidth_mbps: 100,
        };
        let json = serde_json::to_string(&reqs).unwrap();
        let parsed: ResourceRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.cpu_cores, reqs.cpu_cores);
        assert_eq!(parsed.gpu_required, reqs.gpu_required);
    }
}
